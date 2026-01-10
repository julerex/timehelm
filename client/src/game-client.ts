import * as THREE from 'three';
import { Player, Position, PlayerData, Activity } from './entities/Player';
import { Entity, EntityData } from './entities/Entity';
import { CameraController } from './camera/CameraController';
import { InputManager } from './input/InputManager';
import { NetworkManager, NetworkEventHandlers } from './network/NetworkManager';
import { DayNightCycle } from './environment/DayNightCycle';
import { WorldObjectFactory } from './world/WorldObjectFactory';
import { RandomMovementController } from './ai/RandomMovementController';
import { HeightOpacityManager } from './world/HeightOpacityManager';

// Re-export types for external use
export { Player } from './entities/Player';
export type { Position, PlayerData, Activity } from './entities/Player';

export interface User {
    id: string;
    username: string;
    display_name: string;
    avatar_url: string | null;
}

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

    // Entity management
    private entities: Map<string, Entity> = new Map();

    // World objects (for height-based opacity filtering)
    private worldObjects: THREE.Object3D[] = [];
    private heightOpacityManager: HeightOpacityManager | null = null;

    // Random movement controller
    private randomMovement: RandomMovementController | null = null;

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

        // Initialize random movement controller
        // Movement configuration (scaled for 60x game time)
        // 6000 units/game minute = 100 units/frame at 60 FPS
        // Lawn boundaries (ground size 10000, so half = 5000, minus margin for player body)
        this.randomMovement = new RandomMovementController({
            moveSpeed: 100,
            rotationSpeed: 0.2,
            bounds: {
                minX: -4970,
                maxX: 4970,
                minZ: -4970,
                maxZ: 4970
            }
        });

        // World objects
        const house = WorldObjectFactory.createHouse(-600, -400);
        this.scene.add(house);
        this.worldObjects.push(house);

        // Pole next to house (100 units to the right)
        const pole = WorldObjectFactory.createPole(-500, -400);
        this.scene.add(pole);
        this.worldObjects.push(pole);

        // Initialize height opacity manager
        this.heightOpacityManager = new HeightOpacityManager(this.worldObjects);

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
                this.heightOpacityManager?.setHeightOpacity(heightThreshold);
            } else if (key === '0') {
                this.heightOpacityManager?.setHeightOpacity(null); // Reset to fully opaque
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
            onWorldState: (players: PlayerData[], entities: EntityData[]) => {
                this.handleWorldState(players, entities);
            },
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

    private handleWorldState(players: PlayerData[], entities: EntityData[]): void {
        // Update players
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

        // Update entities
        for (const entityData of entities) {
            const existingEntity = this.entities.get(entityData.id);
            if (existingEntity) {
                existingEntity.position = entityData.position;
                existingEntity.rotation = entityData.rotation;
            } else {
                this.addEntity(entityData);
            }
        }

        // Remove entities that are no longer in the world state
        const entityIds = new Set(entities.map(e => e.id));
        for (const [entityId] of this.entities.entries()) {
            if (!entityIds.has(entityId)) {
                this.removeEntity(entityId);
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

    // --- Entity Management ---

    private addEntity(entityData: EntityData): void {
        const entity = Entity.fromData(entityData);
        this.scene?.add(entity.mesh);
        this.entities.set(entityData.id, entity);
    }

    private removeEntity(entityId: string): void {
        const entity = this.entities.get(entityId);
        if (entity) {
            this.scene?.remove(entity.mesh);
            this.entities.delete(entityId);
        }
    }

    // --- Update Methods ---

    private updateMovement(): void {
        if (!this.myPlayer || !this.randomMovement) return;

        const config = this.randomMovement.getConfig();
        const action = this.randomMovement.update();
        let moved = false;

        if (action === 'forward') {
            this.myPlayer.moveForward(config.moveSpeed, config.bounds);
            moved = true;
        } else if (action === 'left') {
            this.myPlayer.rotate(config.rotationSpeed);
            moved = true;
        } else if (action === 'right') {
            this.myPlayer.rotate(-config.rotationSpeed);
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
