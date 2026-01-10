/**
 * Main game client orchestrator.
 * 
 * Manages Three.js scene, players, entities, networking, camera, input,
 * day/night cycle, and world objects. Coordinates all game systems.
 */

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

/**
 * User information interface.
 */
export interface User {
    /** Unique user identifier */
    id: string;
    /** Username */
    username: string;
    /** Display name */
    display_name: string;
    /** Optional avatar URL */
    avatar_url: string | null;
}

/**
 * Main game client class.
 * 
 * Orchestrates all game systems including:
 * - Three.js scene and rendering
 * - Player and entity management
 * - Network communication
 * - Camera control
 * - Input handling
 * - Day/night cycle
 * - World objects
 */
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

    /**
     * Create a new game client.
     * 
     * @param user - User information for the local player
     */
    constructor(user: User) {
        this.user = user;
    }

    // --- Public Methods ---

    /**
     * Initialize the game client.
     * 
     * Sets up scene, input, camera, network, and starts the animation loop.
     */
    public init(): void {
        this.setupScene();
        this.setupInput();
        this.setupCamera();
        this.setupNetwork();
        this.animate();
    }

    /**
     * Clean up resources and disconnect from server.
     * 
     * Should be called when the game client is being destroyed.
     */
    public dispose(): void {
        this.inputManager?.dispose();
        this.networkManager?.disconnect();
    }

    // --- Setup Methods ---

    /**
     * Set up the Three.js scene, renderer, and initial world objects.
     * 
     * Creates:
     * - Scene with sky blue background
     * - WebGL renderer with shadows
     * - Day/night cycle lighting
     * - Ground plane
     * - Local player
     * - World objects (house, pole)
     * - Random movement controller
     * - Height opacity manager
     */
    private setupScene(): void {
        // Create Three.js scene with sky blue background
        this.scene = new THREE.Scene();
        this.scene.background = new THREE.Color(0x87ceeb);

        // Create WebGL renderer with antialiasing and shadows
        this.renderer = new THREE.WebGLRenderer({ antialias: true });
        this.renderer.setSize(window.innerWidth, window.innerHeight);
        this.renderer.shadowMap.enabled = true;
        document.getElementById('game-container')?.appendChild(this.renderer.domElement);

        // Initialize day/night cycle (handles lighting and sky color)
        this.dayNightCycle = new DayNightCycle(this.scene);

        // Create ground plane
        const ground = WorldObjectFactory.createGround();
        this.scene.add(ground);

        // Create local player
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

        // Create world objects
        const house = WorldObjectFactory.createHouse(-600, -400);
        this.scene.add(house);
        this.worldObjects.push(house);

        // Pole next to house (100 units to the right)
        const pole = WorldObjectFactory.createPole(-500, -400);
        this.scene.add(pole);
        this.worldObjects.push(pole);

        // Initialize height opacity manager for floor visibility controls
        this.heightOpacityManager = new HeightOpacityManager(this.worldObjects);

        // Handle window resize events
        window.addEventListener('resize', this.handleResize);
    }

    /**
     * Set up input handling.
     * 
     * Registers keyboard, mouse, and wheel event handlers for:
     * - Camera controls (toggle anchor, rotation, zoom)
     * - Height-based opacity controls (1-9 keys)
     */
    private setupInput(): void {
        this.inputManager = new InputManager();

        // Camera toggle and height visibility controls
        this.inputManager.onKeyPress((key) => {
            // Toggle camera anchor mode (C key)
            if (key === 'c') {
                this.cameraController?.toggleAnchor();
                this.updateCameraStatusDisplay();
            }

            // Height-based opacity: 1-9 hide objects above (n * 3m), 0 resets
            if (key >= '1' && key <= '9') {
                const level = parseInt(key, 10);
                const heightThreshold = level * 300; // 300cm = 3m per level
                this.heightOpacityManager?.setHeightOpacity(heightThreshold);
            } else if (key === '0') {
                this.heightOpacityManager?.setHeightOpacity(null); // Reset to fully opaque
            }
        });

        // Camera rotation via right mouse drag
        this.inputManager.onMouseDrag((deltaX, deltaY) => {
            this.cameraController?.adjustRotation(deltaX * 0.01, deltaY * 0.01);
        });

        // Camera zoom via mouse wheel
        this.inputManager.onWheel((delta) => {
            this.cameraController?.adjustZoom(delta);
        });
    }

    /**
     * Set up camera controller.
     * 
     * Creates perspective camera and sets initial position based on player.
     */
    private setupCamera(): void {
        this.cameraController = new CameraController(window.innerWidth / window.innerHeight);

        if (this.myPlayer) {
            this.cameraController.setInitialPosition(this.myPlayer);
        }
    }

    /**
     * Set up network connection and message handlers.
     * 
     * Connects to WebSocket server and registers handlers for:
     * - World state updates
     * - Player join/leave events
     * - Player movement updates
     * - Activity changes
     * - Time synchronization
     */
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

        // Send initial join message after a brief delay to ensure connection is established
        setTimeout(() => {
            if (this.myPlayer && this.networkManager?.isConnected()) {
                this.networkManager.sendJoin(this.myPlayer.toData());
            }
        }, 100);
    }

    // --- Network Event Handlers ---

    /**
     * Handle world state update from server.
     * 
     * Updates all players and entities, and removes entities that are no longer in the world.
     * 
     * @param players - Array of all player data
     * @param entities - Array of all entity data
     */
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

    /**
     * Handle player join event.
     * 
     * @param playerData - Data for the joining player
     */
    private handlePlayerJoin(playerData: PlayerData): void {
        if (playerData.id !== this.user.id) {
            this.addRemotePlayer(playerData);
            this.updatePlayersList();
        }
    }

    /**
     * Handle player leave event.
     * 
     * @param playerId - ID of the player who left
     */
    private handlePlayerLeave(playerId: string): void {
        this.removePlayer(playerId);
        this.updatePlayersList();
    }

    /**
     * Handle player movement update.
     * 
     * @param playerId - ID of the moving player
     * @param position - New position
     * @param rotation - New rotation
     * @param isMoving - Whether the player is moving
     */
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

    /**
     * Handle player activity change.
     * 
     * @param playerId - ID of the player
     * @param activity - New activity
     */
    private handleActivityChanged(playerId: string, activity: Activity): void {
        const player = this.players.get(playerId);
        if (player) {
            player.activity = activity;
        }
    }

    /**
     * Handle game time synchronization from server.
     * 
     * @param gameTimeMinutes - Current game time in minutes
     */
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

    /**
     * Update player movement based on random movement controller.
     * 
     * Sends movement updates to the server when the player moves.
     */
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

    /**
     * Update animations for all players.
     * 
     * Detects movement for remote players and updates their animations.
     */
    private updateAnimations(): void {
        for (const player of this.players.values()) {
            // Detect movement for remote players based on position changes
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

    /**
     * Main animation loop.
     * 
     * Called every frame to update:
     * - Day/night cycle
     * - Player movement
     * - Animations
     * - Camera
     * - Rendering
     */
    private animate = (): void => {
        requestAnimationFrame(this.animate);

        // Update day/night cycle (lighting, sky color, celestial bodies)
        this.dayNightCycle?.update();
        // Update player movement
        this.updateMovement();
        // Update player animations
        this.updateAnimations();

        // Update camera based on player and input
        if (this.cameraController && this.inputManager) {
            this.cameraController.update(this.myPlayer, this.inputManager);
        }

        // Render the scene
        if (this.renderer && this.scene && this.cameraController) {
            this.renderer.render(this.scene, this.cameraController.getCamera());
        }
    };
}
