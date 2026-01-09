import * as THREE from 'three';
import { Player, Position, PlayerData, Activity } from './entities/Player';
import { CameraController } from './camera/CameraController';
import { InputManager } from './input/InputManager';
import { NetworkManager, NetworkEventHandlers } from './network/NetworkManager';
import { DayNightCycle } from './environment/DayNightCycle';
import { WorldObjectFactory } from './world/WorldObjectFactory';

// Re-export types for external use
export { Player } from './entities/Player';
export type { Position, PlayerData, Activity } from './entities/Player';

export interface User {
    id: string;
    username: string;
    display_name: string;
    avatar_url: string | null;
}

type RandomMoveAction = 'idle' | 'forward' | 'left' | 'right';

export class GameClient {
    private readonly user: User;

    // Core components
    private scene: THREE.Scene | null = null;
    private renderer: THREE.WebGLRenderer | null = null;
    private cameraController: CameraController | null = null;
    private inputManager: InputManager | null = null;
    private networkManager: NetworkManager | null = null;
    private dayNightCycle: DayNightCycle | null = null;

    // Player management
    private players: Map<string, Player> = new Map();
    private myPlayer: Player | null = null;

    // World objects (for height-based opacity filtering)
    private worldObjects: THREE.Object3D[] = [];

    // Movement configuration (scaled for 60x game time)
    // 6000 units/game minute = 100 units/frame at 60 FPS
    private readonly moveSpeed = 100;
    private readonly rotationSpeed = 0.2;

    // Lawn boundaries (ground size 10000, so half = 5000, minus margin for player body)
    private readonly lawnBounds = {
        minX: -4970,
        maxX: 4970,
        minZ: -4970,
        maxZ: 4970
    };

    // Random movement state
    private randomMoveTimer = 0;
    private randomMoveAction: RandomMoveAction = 'idle';

    constructor(user: User) {
        this.user = user;
    }

    // --- Public Methods ---

    public init(): void {
        this.setupScene();
        this.setupInput();
        this.setupCamera();
        this.setupNetwork();
        this.animate();
    }

    public dispose(): void {
        this.inputManager?.dispose();
        this.networkManager?.disconnect();
    }

    // --- Setup Methods ---

    private setupScene(): void {
        // Scene
        this.scene = new THREE.Scene();
        this.scene.background = new THREE.Color(0x87ceeb);

        // Renderer
        this.renderer = new THREE.WebGLRenderer({ antialias: true });
        this.renderer.setSize(window.innerWidth, window.innerHeight);
        this.renderer.shadowMap.enabled = true;
        document.getElementById('game-container')?.appendChild(this.renderer.domElement);

        // Day/Night cycle (handles lighting)
        this.dayNightCycle = new DayNightCycle(this.scene);

        // Ground
        const ground = WorldObjectFactory.createGround();
        this.scene.add(ground);

        // Create my player
        this.myPlayer = new Player(this.user.id, this.user.username);
        this.scene.add(this.myPlayer.mesh);
        this.players.set(this.user.id, this.myPlayer);

        // World objects
        const tree = WorldObjectFactory.createTree(500, -500);
        this.scene.add(tree);
        this.worldObjects.push(tree);

        const house = WorldObjectFactory.createHouse(-600, -400);
        this.scene.add(house);
        this.worldObjects.push(house);

        // Handle window resize
        window.addEventListener('resize', this.handleResize);
    }

    private setupInput(): void {
        this.inputManager = new InputManager();

        // Camera toggle and height visibility
        this.inputManager.onKeyPress((key) => {
            if (key === 'c') {
                this.cameraController?.toggleAnchor();
                this.updateCameraStatusDisplay();
            }

            // Height-based opacity: 1-9 hide above (n * 3m), 0 resets
            if (key >= '1' && key <= '9') {
                const level = parseInt(key, 10);
                const heightThreshold = level * 300; // 300cm = 3m per level
                this.setHeightOpacity(heightThreshold);
            } else if (key === '0') {
                this.setHeightOpacity(null); // Reset to fully opaque
            }
        });

        // Camera rotation via mouse drag
        this.inputManager.onMouseDrag((deltaX, deltaY) => {
            this.cameraController?.adjustRotation(deltaX * 0.01, deltaY * 0.01);
        });

        // Camera zoom
        this.inputManager.onWheel((delta) => {
            this.cameraController?.adjustZoom(delta);
        });
    }

    private setupCamera(): void {
        this.cameraController = new CameraController(window.innerWidth / window.innerHeight);

        if (this.myPlayer) {
            this.cameraController.setInitialPosition(this.myPlayer);
        }
    }

    private setupNetwork(): void {
        const handlers: NetworkEventHandlers = {
            onWorldState: this.handleWorldState.bind(this),
            onPlayerJoin: this.handlePlayerJoin.bind(this),
            onPlayerLeave: this.handlePlayerLeave.bind(this),
            onPlayerMove: this.handlePlayerMove.bind(this),
            onActivityChanged: this.handleActivityChanged.bind(this),
            onTimeSync: this.handleTimeSync.bind(this)
        };

        this.networkManager = new NetworkManager(handlers);
        this.networkManager.connect();

        // Send initial join after a brief delay to ensure connection is established
        setTimeout(() => {
            if (this.myPlayer && this.networkManager?.isConnected()) {
                this.networkManager.sendJoin(this.myPlayer.toData());
            }
        }, 100);
    }

    // --- Network Event Handlers ---

    private handleWorldState(players: PlayerData[]): void {
        for (const playerData of players) {
            if (playerData.id !== this.user.id) {
                const existingPlayer = this.players.get(playerData.id);
                if (existingPlayer) {
                    existingPlayer.position = playerData.position;
                    existingPlayer.rotation = playerData.rotation;
                    existingPlayer.isMoving = playerData.is_moving || false;
                    existingPlayer.activity = playerData.activity || 'idle';
                } else {
                    this.addRemotePlayer(playerData);
                }
            }
        }
        this.updatePlayersList();
    }

    private handlePlayerJoin(playerData: PlayerData): void {
        if (playerData.id !== this.user.id) {
            this.addRemotePlayer(playerData);
            this.updatePlayersList();
        }
    }

    private handlePlayerLeave(playerId: string): void {
        this.removePlayer(playerId);
        this.updatePlayersList();
    }

    private handlePlayerMove(
        playerId: string,
        position: Position,
        rotation: number,
        isMoving: boolean
    ): void {
        if (playerId === this.user.id) return;

        const player = this.players.get(playerId);
        if (player) {
            player.position = position;
            player.rotation = rotation;
            player.isMoving = isMoving;
        }
    }

    private handleActivityChanged(playerId: string, activity: Activity): void {
        const player = this.players.get(playerId);
        if (player) {
            player.activity = activity;
        }
    }

    private handleTimeSync(gameTimeMinutes: number): void {
        this.dayNightCycle?.syncTime(gameTimeMinutes);
    }

    // --- Player Management ---

    private addRemotePlayer(playerData: PlayerData): void {
        const player = Player.fromData(playerData);
        this.scene?.add(player.mesh);
        this.players.set(playerData.id, player);
    }

    private removePlayer(playerId: string): void {
        const player = this.players.get(playerId);
        if (player) {
            this.scene?.remove(player.mesh);
            this.players.delete(playerId);
        }
    }

    // --- Update Methods ---

    private updateMovement(): void {
        if (!this.myPlayer) return;

        let moved = false;

        // Random movement logic
        this.randomMoveTimer -= 1;
        if (this.randomMoveTimer <= 0) {
            const actions: RandomMoveAction[] = ['idle', 'forward', 'left', 'right'];
            this.randomMoveAction = actions[Math.floor(Math.random() * actions.length)];
            this.randomMoveTimer = Math.floor(Math.random() * 60) + 30;
        }

        if (this.randomMoveAction === 'forward') {
            this.myPlayer.moveForward(this.moveSpeed, this.lawnBounds);
            moved = true;
        } else if (this.randomMoveAction === 'left') {
            this.myPlayer.rotate(this.rotationSpeed);
            moved = true;
        } else if (this.randomMoveAction === 'right') {
            this.myPlayer.rotate(-this.rotationSpeed);
            moved = true;
        }

        this.myPlayer.isMoving = moved;

        if (moved) {
            this.networkManager?.sendMove(
                this.user.id,
                this.myPlayer.position,
                this.myPlayer.rotation,
                this.myPlayer.isMoving
            );
        }
    }

    private updateAnimations(): void {
        for (const player of this.players.values()) {
            // Detect movement for remote players
            if (player.id !== this.user.id) {
                player.detectMovementFromPosition();
            }

            player.updateAnimation();
        }
    }

    // --- UI Updates ---

    private updatePlayersList(): void {
        const list = document.getElementById('players-list');
        if (!list) return;

        list.innerHTML = '';
        for (const player of this.players.values()) {
            const li = document.createElement('li');
            li.textContent = player.username;
            list.appendChild(li);
        }
    }

    private updateCameraStatusDisplay(): void {
        const cameraStatus = document.getElementById('camera-status');
        if (cameraStatus && this.cameraController) {
            const status = this.cameraController.isAnchored() ? 'Anchored' : 'Free';
            cameraStatus.textContent = `Camera: ${status} (Press 'C' to toggle)`;
        }
    }

    // --- Height-Based Visibility ---

    /**
     * Sets opacity for objects based on height threshold.
     * Objects above the threshold become semi-transparent.
     * @param heightThreshold - Height in cm above which objects become transparent. null = fully opaque.
     */
    private setHeightOpacity(heightThreshold: number | null): void {
        const hiddenOpacity = 0.15;
        const visibleOpacity = 1.0;

        for (const obj of this.worldObjects) {
            obj.traverse((child) => {
                if (child instanceof THREE.Mesh && child.material instanceof THREE.MeshStandardMaterial) {
                    // Get world position of the mesh
                    const worldPos = new THREE.Vector3();
                    child.getWorldPosition(worldPos);

                    // Determine target opacity
                    let targetOpacity: number;
                    if (heightThreshold === null) {
                        targetOpacity = visibleOpacity;
                    } else {
                        targetOpacity = worldPos.y > heightThreshold ? hiddenOpacity : visibleOpacity;
                    }

                    // Update material opacity
                    child.material.transparent = true;
                    child.material.opacity = targetOpacity;
                    child.material.needsUpdate = true;
                }
            });
        }
    }

    // --- Event Handlers ---

    private handleResize = (): void => {
        const aspectRatio = window.innerWidth / window.innerHeight;
        this.cameraController?.handleResize(aspectRatio);
        this.renderer?.setSize(window.innerWidth, window.innerHeight);
    };

    // --- Animation Loop ---

    private animate = (): void => {
        requestAnimationFrame(this.animate);

        this.dayNightCycle?.update();
        this.updateMovement();
        this.updateAnimations();

        if (this.cameraController && this.inputManager) {
            this.cameraController.update(this.myPlayer, this.inputManager);
        }

        if (this.renderer && this.scene && this.cameraController) {
            this.renderer.render(this.scene, this.cameraController.getCamera());
        }
    };
}
