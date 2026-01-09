import * as THREE from 'three';

export interface User {
    id: string;
    username: string;
    display_name: string;
    avatar_url: string | null;
}

export interface Position {
    x: number;
    y: number;
    z: number;
}

export interface PlayerData {
    id: string;
    username: string;
    position: Position;
    rotation: number;
    is_moving?: boolean;
}

interface Player {
    id: string;
    username: string;
    mesh: THREE.Group;
    leftLeg: THREE.Mesh;
    rightLeg: THREE.Mesh;
    leftArm: THREE.Mesh;
    rightArm: THREE.Mesh;
    position: Position;
    rotation: number;
    walkCycle: number;
    isMoving: boolean;
    lastPosition: Position | null;
}

interface CameraRotation {
    theta: number;
    phi: number;
}

interface MousePosition {
    x: number;
    y: number;
}

type WebSocketMessage =
    | { type: 'Join'; player: PlayerData }
    | { type: 'Leave'; player_id: string }
    | { type: 'Move'; player_id: string; position: Position; rotation: number; is_moving: boolean }
    | { type: 'WorldState'; players: PlayerData[] };

type RandomMoveAction = 'idle' | 'forward' | 'left' | 'right';

export class GameClient {
    private user: User;
    private scene: THREE.Scene | null = null;
    private camera: THREE.PerspectiveCamera | null = null;
    private renderer: THREE.WebGLRenderer | null = null;
    private ws: WebSocket | null = null;
    private players: Map<string, Player> = new Map();
    private myPlayer: Player | null = null;
    private keys: Record<string, boolean> = {};
    private moveSpeed = 10;
    private rotationSpeed = 0.02;

    // Camera rotation state
    private cameraRotation: CameraRotation = { theta: 0, phi: 0.5 };
    private cameraDistance = 1000;
    private cameraAnchored = true;
    private cameraFocusPoint: THREE.Vector3 | null = null;
    private isRightMouseDown = false;
    private lastMousePos: MousePosition = { x: 0, y: 0 };

    // Random movement state
    private randomMoveTimer = 0;
    private randomMoveAction: RandomMoveAction = 'idle';

    // Lighting
    private ambientLight: THREE.AmbientLight | null = null;
    private sunLight: THREE.DirectionalLight | null = null;
    private sunMesh: THREE.Mesh | null = null;
    private moonMesh: THREE.Mesh | null = null;

    constructor(user: User) {
        this.user = user;
    }

    init(): void {
        this.setupScene();
        this.setupControls();
        this.connectWebSocket();
        this.animate();
    }

    private setupScene(): void {
        // Scene
        this.scene = new THREE.Scene();
        this.scene.background = new THREE.Color(0x87ceeb); // Sky blue

        // Camera
        this.camera = new THREE.PerspectiveCamera(
            75,
            window.innerWidth / window.innerHeight,
            10,
            100000
        );
        this.camera.position.set(0, 500, 1000);
        this.camera.lookAt(0, 0, 0);

        // Renderer
        this.renderer = new THREE.WebGLRenderer({ antialias: true });
        this.renderer.setSize(window.innerWidth, window.innerHeight);
        this.renderer.shadowMap.enabled = true;
        document.getElementById('game-container')?.appendChild(this.renderer.domElement);

        // Lighting
        this.ambientLight = new THREE.AmbientLight(0xffffff, 0.4);
        this.scene.add(this.ambientLight);

        this.sunLight = new THREE.DirectionalLight(0xffffff, 1.0);
        this.sunLight.position.set(0, 10000, 0);
        this.sunLight.castShadow = true;

        // Optimize shadows for sun
        this.sunLight.shadow.camera.left = -5000;
        this.sunLight.shadow.camera.right = 5000;
        this.sunLight.shadow.camera.top = 5000;
        this.sunLight.shadow.camera.bottom = -5000;
        this.sunLight.shadow.mapSize.width = 2048;
        this.sunLight.shadow.mapSize.height = 2048;

        this.scene.add(this.sunLight);

        // Sun Mesh
        const sunGeometry = new THREE.SphereGeometry(200, 32, 32);
        const sunMaterial = new THREE.MeshBasicMaterial({ color: 0xffff00 });
        this.sunMesh = new THREE.Mesh(sunGeometry, sunMaterial);
        this.scene.add(this.sunMesh);

        // Moon Mesh
        const moonGeometry = new THREE.SphereGeometry(150, 32, 32);
        const moonMaterial = new THREE.MeshBasicMaterial({ color: 0xcccccc });
        this.moonMesh = new THREE.Mesh(moonGeometry, moonMaterial);
        this.scene.add(this.moonMesh);

        // Ground
        const groundGeometry = new THREE.PlaneGeometry(10000, 10000);
        const groundMaterial = new THREE.MeshStandardMaterial({ color: 0x90ee90 });
        const ground = new THREE.Mesh(groundGeometry, groundMaterial);
        ground.rotation.x = -Math.PI / 2;
        ground.receiveShadow = true;
        this.scene.add(ground);

        // Create my player
        this.myPlayer = this.createPlayer(this.user.id, this.user.username);
        this.scene.add(this.myPlayer.mesh);
        this.players.set(this.user.id, this.myPlayer);

        // Camera follows player
        this.camera.position.set(
            this.myPlayer.mesh.position.x,
            this.myPlayer.mesh.position.y + 500,
            this.myPlayer.mesh.position.z + 1000
        );

        // Add a tree
        const tree = this.createTree(500, -500);
        this.scene.add(tree);

        // Add a two-story house
        const house = this.createHouse(-600, -400);
        this.scene.add(house);

        // Handle window resize
        window.addEventListener('resize', () => {
            if (this.camera && this.renderer) {
                this.camera.aspect = window.innerWidth / window.innerHeight;
                this.camera.updateProjectionMatrix();
                this.renderer.setSize(window.innerWidth, window.innerHeight);
            }
        });
    }

    private createPlayer(id: string, username: string): Player {
        // Improved low-poly humanoid character
        const group = new THREE.Group();

        // Body (torso)
        const bodyGeometry = new THREE.BoxGeometry(60, 80, 40);
        const bodyMaterial = new THREE.MeshStandardMaterial({ color: 0x4a90e2 });
        const body = new THREE.Mesh(bodyGeometry, bodyMaterial);
        body.position.y = 110; // Centered at torso height
        body.castShadow = true;
        group.add(body);

        // Head
        const headGeometry = new THREE.BoxGeometry(40, 40, 40);
        const headMaterial = new THREE.MeshStandardMaterial({ color: 0xffdbac });
        const head = new THREE.Mesh(headGeometry, headMaterial);
        head.position.y = 170;
        head.castShadow = true;
        group.add(head);

        // Legs
        const legGeometry = new THREE.BoxGeometry(20, 70, 20);
        const legMaterial = new THREE.MeshStandardMaterial({ color: 0x333333 });

        const leftLeg = new THREE.Mesh(legGeometry, legMaterial);
        leftLeg.position.set(-15, 35, 0);
        leftLeg.castShadow = true;
        group.add(leftLeg);

        const rightLeg = new THREE.Mesh(legGeometry, legMaterial);
        rightLeg.position.set(15, 35, 0);
        rightLeg.castShadow = true;
        group.add(rightLeg);

        // Arms
        const armGeometry = new THREE.BoxGeometry(20, 70, 20);
        const armMaterial = new THREE.MeshStandardMaterial({ color: 0x4a90e2 });

        const leftArm = new THREE.Mesh(armGeometry, armMaterial);
        leftArm.position.set(-40, 110, 0);
        leftArm.castShadow = true;
        group.add(leftArm);

        const rightArm = new THREE.Mesh(armGeometry, armMaterial);
        rightArm.position.set(40, 110, 0);
        rightArm.castShadow = true;
        group.add(rightArm);

        // Name label
        const canvas = document.createElement('canvas');
        const context = canvas.getContext('2d');
        canvas.width = 256;
        canvas.height = 64;
        if (context) {
            context.fillStyle = 'rgba(0, 0, 0, 0.8)';
            context.fillRect(0, 0, canvas.width, canvas.height);
            context.fillStyle = 'white';
            context.font = '24px Arial';
            context.textAlign = 'center';
            context.fillText(username, canvas.width / 2, canvas.height / 2 + 8);
        }

        const texture = new THREE.CanvasTexture(canvas);
        const spriteMaterial = new THREE.SpriteMaterial({ map: texture });
        const sprite = new THREE.Sprite(spriteMaterial);
        sprite.position.y = 250;
        sprite.scale.set(200, 50, 100);
        group.add(sprite);

        return {
            id,
            username,
            mesh: group,
            leftLeg,
            rightLeg,
            leftArm,
            rightArm,
            position: { x: 0, y: 0, z: 0 },
            rotation: 0,
            walkCycle: 0,
            isMoving: false,
            lastPosition: null
        };
    }

    private createTree(x: number, z: number): THREE.Group {
        const group = new THREE.Group();

        // Trunk (brown)
        const trunkGeometry = new THREE.BoxGeometry(40, 150, 40);
        const trunkMaterial = new THREE.MeshStandardMaterial({ color: 0x8b4513 });
        const trunk = new THREE.Mesh(trunkGeometry, trunkMaterial);
        trunk.position.y = 75; // Half of height to stand on ground
        trunk.castShadow = true;
        trunk.receiveShadow = true;
        group.add(trunk);

        // Foliage (green)
        const leavesGeometry = new THREE.BoxGeometry(200, 250, 200);
        const leavesMaterial = new THREE.MeshStandardMaterial({ color: 0x228b22 });
        const leaves = new THREE.Mesh(leavesGeometry, leavesMaterial);
        leaves.position.y = 150 + 125; // trunk height + half of leaves height
        leaves.castShadow = true;
        leaves.receiveShadow = true;
        group.add(leaves);

        group.position.set(x, 0, z);
        return group;
    }

    private createHouse(x: number, z: number): THREE.Group {
        const group = new THREE.Group();

        // House dimensions
        const houseWidth = 400;
        const houseDepth = 300;
        const floorHeight = 200;
        const wallThickness = 20;

        // Materials
        const wallMaterial = new THREE.MeshStandardMaterial({ color: 0xf5deb3 }); // Wheat/beige walls
        const roofMaterial = new THREE.MeshStandardMaterial({ color: 0x8b0000 }); // Dark red roof
        const doorMaterial = new THREE.MeshStandardMaterial({ color: 0x8b4513 }); // Brown door
        const windowMaterial = new THREE.MeshStandardMaterial({ color: 0x87ceeb, transparent: true, opacity: 0.7 }); // Light blue windows
        const frameMaterial = new THREE.MeshStandardMaterial({ color: 0xffffff }); // White window frames
        const foundationMaterial = new THREE.MeshStandardMaterial({ color: 0x696969 }); // Gray foundation
        const chimneyMaterial = new THREE.MeshStandardMaterial({ color: 0xa52a2a }); // Brick red chimney

        // Foundation
        const foundationGeometry = new THREE.BoxGeometry(houseWidth + 40, 30, houseDepth + 40);
        const foundation = new THREE.Mesh(foundationGeometry, foundationMaterial);
        foundation.position.y = 15;
        foundation.castShadow = true;
        foundation.receiveShadow = true;
        group.add(foundation);

        // First floor walls
        // Front wall (with door and windows)
        const frontWallGeometry = new THREE.BoxGeometry(houseWidth, floorHeight, wallThickness);
        const frontWall = new THREE.Mesh(frontWallGeometry, wallMaterial);
        frontWall.position.set(0, 30 + floorHeight / 2, houseDepth / 2);
        frontWall.castShadow = true;
        frontWall.receiveShadow = true;
        group.add(frontWall);

        // Back wall
        const backWall = new THREE.Mesh(frontWallGeometry, wallMaterial);
        backWall.position.set(0, 30 + floorHeight / 2, -houseDepth / 2);
        backWall.castShadow = true;
        backWall.receiveShadow = true;
        group.add(backWall);

        // Side walls
        const sideWallGeometry = new THREE.BoxGeometry(wallThickness, floorHeight, houseDepth);
        const leftWall = new THREE.Mesh(sideWallGeometry, wallMaterial);
        leftWall.position.set(-houseWidth / 2, 30 + floorHeight / 2, 0);
        leftWall.castShadow = true;
        leftWall.receiveShadow = true;
        group.add(leftWall);

        const rightWall = new THREE.Mesh(sideWallGeometry, wallMaterial);
        rightWall.position.set(houseWidth / 2, 30 + floorHeight / 2, 0);
        rightWall.castShadow = true;
        rightWall.receiveShadow = true;
        group.add(rightWall);

        // Second floor walls
        const secondFloorY = 30 + floorHeight;

        const frontWall2 = new THREE.Mesh(frontWallGeometry, wallMaterial);
        frontWall2.position.set(0, secondFloorY + floorHeight / 2, houseDepth / 2);
        frontWall2.castShadow = true;
        frontWall2.receiveShadow = true;
        group.add(frontWall2);

        const backWall2 = new THREE.Mesh(frontWallGeometry, wallMaterial);
        backWall2.position.set(0, secondFloorY + floorHeight / 2, -houseDepth / 2);
        backWall2.castShadow = true;
        backWall2.receiveShadow = true;
        group.add(backWall2);

        const leftWall2 = new THREE.Mesh(sideWallGeometry, wallMaterial);
        leftWall2.position.set(-houseWidth / 2, secondFloorY + floorHeight / 2, 0);
        leftWall2.castShadow = true;
        leftWall2.receiveShadow = true;
        group.add(leftWall2);

        const rightWall2 = new THREE.Mesh(sideWallGeometry, wallMaterial);
        rightWall2.position.set(houseWidth / 2, secondFloorY + floorHeight / 2, 0);
        rightWall2.castShadow = true;
        rightWall2.receiveShadow = true;
        group.add(rightWall2);

        // Floor between stories
        const floorGeometry = new THREE.BoxGeometry(houseWidth - wallThickness, 15, houseDepth - wallThickness);
        const floorMaterial = new THREE.MeshStandardMaterial({ color: 0xdeb887 }); // Burlywood
        const floor = new THREE.Mesh(floorGeometry, floorMaterial);
        floor.position.y = secondFloorY;
        floor.castShadow = true;
        floor.receiveShadow = true;
        group.add(floor);

        // Door (front wall, first floor)
        const doorGeometry = new THREE.BoxGeometry(60, 120, wallThickness + 5);
        const door = new THREE.Mesh(doorGeometry, doorMaterial);
        door.position.set(0, 30 + 60, houseDepth / 2);
        door.castShadow = true;
        group.add(door);

        // Door frame
        const doorFrameGeometry = new THREE.BoxGeometry(70, 130, wallThickness + 2);
        const doorFrame = new THREE.Mesh(doorFrameGeometry, frameMaterial);
        doorFrame.position.set(0, 30 + 65, houseDepth / 2 - 2);
        group.add(doorFrame);

        // Door handle
        const handleGeometry = new THREE.BoxGeometry(8, 8, 10);
        const handleMaterial = new THREE.MeshStandardMaterial({ color: 0xffd700 }); // Gold
        const handle = new THREE.Mesh(handleGeometry, handleMaterial);
        handle.position.set(20, 30 + 60, houseDepth / 2 + 8);
        group.add(handle);

        // First floor windows (front)
        const windowGeometry = new THREE.BoxGeometry(50, 60, wallThickness + 5);
        const windowFrameGeometry = new THREE.BoxGeometry(60, 70, wallThickness + 2);

        // Left window (first floor front)
        const window1Frame = new THREE.Mesh(windowFrameGeometry, frameMaterial);
        window1Frame.position.set(-120, 30 + 100, houseDepth / 2 - 2);
        group.add(window1Frame);

        const window1 = new THREE.Mesh(windowGeometry, windowMaterial);
        window1.position.set(-120, 30 + 100, houseDepth / 2);
        group.add(window1);

        // Right window (first floor front)
        const window2Frame = new THREE.Mesh(windowFrameGeometry, frameMaterial);
        window2Frame.position.set(120, 30 + 100, houseDepth / 2 - 2);
        group.add(window2Frame);

        const window2 = new THREE.Mesh(windowGeometry, windowMaterial);
        window2.position.set(120, 30 + 100, houseDepth / 2);
        group.add(window2);

        // Second floor windows (front) - 3 windows
        const window3Frame = new THREE.Mesh(windowFrameGeometry, frameMaterial);
        window3Frame.position.set(-120, secondFloorY + 100, houseDepth / 2 - 2);
        group.add(window3Frame);

        const window3 = new THREE.Mesh(windowGeometry, windowMaterial);
        window3.position.set(-120, secondFloorY + 100, houseDepth / 2);
        group.add(window3);

        const window4Frame = new THREE.Mesh(windowFrameGeometry, frameMaterial);
        window4Frame.position.set(0, secondFloorY + 100, houseDepth / 2 - 2);
        group.add(window4Frame);

        const window4 = new THREE.Mesh(windowGeometry, windowMaterial);
        window4.position.set(0, secondFloorY + 100, houseDepth / 2);
        group.add(window4);

        const window5Frame = new THREE.Mesh(windowFrameGeometry, frameMaterial);
        window5Frame.position.set(120, secondFloorY + 100, houseDepth / 2 - 2);
        group.add(window5Frame);

        const window5 = new THREE.Mesh(windowGeometry, windowMaterial);
        window5.position.set(120, secondFloorY + 100, houseDepth / 2);
        group.add(window5);

        // Side windows (both floors, both sides)
        const sideWindowGeometry = new THREE.BoxGeometry(wallThickness + 5, 60, 50);
        const sideWindowFrameGeometry = new THREE.BoxGeometry(wallThickness + 2, 70, 60);

        // Left side windows
        const windowL1Frame = new THREE.Mesh(sideWindowFrameGeometry, frameMaterial);
        windowL1Frame.position.set(-houseWidth / 2 + 2, 30 + 100, 0);
        group.add(windowL1Frame);

        const windowL1 = new THREE.Mesh(sideWindowGeometry, windowMaterial);
        windowL1.position.set(-houseWidth / 2, 30 + 100, 0);
        group.add(windowL1);

        const windowL2Frame = new THREE.Mesh(sideWindowFrameGeometry, frameMaterial);
        windowL2Frame.position.set(-houseWidth / 2 + 2, secondFloorY + 100, 0);
        group.add(windowL2Frame);

        const windowL2 = new THREE.Mesh(sideWindowGeometry, windowMaterial);
        windowL2.position.set(-houseWidth / 2, secondFloorY + 100, 0);
        group.add(windowL2);

        // Right side windows
        const windowR1Frame = new THREE.Mesh(sideWindowFrameGeometry, frameMaterial);
        windowR1Frame.position.set(houseWidth / 2 - 2, 30 + 100, 0);
        group.add(windowR1Frame);

        const windowR1 = new THREE.Mesh(sideWindowGeometry, windowMaterial);
        windowR1.position.set(houseWidth / 2, 30 + 100, 0);
        group.add(windowR1);

        const windowR2Frame = new THREE.Mesh(sideWindowFrameGeometry, frameMaterial);
        windowR2Frame.position.set(houseWidth / 2 - 2, secondFloorY + 100, 0);
        group.add(windowR2Frame);

        const windowR2 = new THREE.Mesh(sideWindowGeometry, windowMaterial);
        windowR2.position.set(houseWidth / 2, secondFloorY + 100, 0);
        group.add(windowR2);

        // Roof (gabled roof using two planes)
        const roofHeight = 120;
        const roofOverhang = 40;

        // Create roof using box geometry angled
        const roofLength = houseDepth + roofOverhang * 2;
        const roofWidth = Math.sqrt(Math.pow(houseWidth / 2 + roofOverhang, 2) + Math.pow(roofHeight, 2));

        const roofGeometry = new THREE.BoxGeometry(roofWidth, 15, roofLength);

        // Left roof slope
        const roofLeft = new THREE.Mesh(roofGeometry, roofMaterial);
        const roofAngle = Math.atan2(roofHeight, houseWidth / 2 + roofOverhang);
        roofLeft.rotation.z = roofAngle;
        roofLeft.position.set(
            -(houseWidth / 4 + roofOverhang / 2) * Math.cos(roofAngle),
            secondFloorY + floorHeight + roofHeight / 2 + 15,
            0
        );
        roofLeft.castShadow = true;
        roofLeft.receiveShadow = true;
        group.add(roofLeft);

        // Right roof slope
        const roofRight = new THREE.Mesh(roofGeometry, roofMaterial);
        roofRight.rotation.z = -roofAngle;
        roofRight.position.set(
            (houseWidth / 4 + roofOverhang / 2) * Math.cos(roofAngle),
            secondFloorY + floorHeight + roofHeight / 2 + 15,
            0
        );
        roofRight.castShadow = true;
        roofRight.receiveShadow = true;
        group.add(roofRight);

        // Roof gable ends (triangular walls)
        const gableShape = new THREE.Shape();
        gableShape.moveTo(-houseWidth / 2, 0);
        gableShape.lineTo(0, roofHeight);
        gableShape.lineTo(houseWidth / 2, 0);
        gableShape.lineTo(-houseWidth / 2, 0);

        const gableGeometry = new THREE.ExtrudeGeometry(gableShape, { depth: wallThickness, bevelEnabled: false });

        const gableFront = new THREE.Mesh(gableGeometry, wallMaterial);
        gableFront.position.set(0, secondFloorY + floorHeight, houseDepth / 2 - wallThickness / 2);
        gableFront.castShadow = true;
        gableFront.receiveShadow = true;
        group.add(gableFront);

        const gableBack = new THREE.Mesh(gableGeometry, wallMaterial);
        gableBack.position.set(0, secondFloorY + floorHeight, -houseDepth / 2 - wallThickness / 2);
        gableBack.castShadow = true;
        gableBack.receiveShadow = true;
        group.add(gableBack);

        // Chimney
        const chimneyGeometry = new THREE.BoxGeometry(40, 150, 40);
        const chimney = new THREE.Mesh(chimneyGeometry, chimneyMaterial);
        chimney.position.set(houseWidth / 4, secondFloorY + floorHeight + roofHeight + 30, 0);
        chimney.castShadow = true;
        chimney.receiveShadow = true;
        group.add(chimney);

        // Chimney top
        const chimneyTopGeometry = new THREE.BoxGeometry(50, 15, 50);
        const chimneyTop = new THREE.Mesh(chimneyTopGeometry, chimneyMaterial);
        chimneyTop.position.set(houseWidth / 4, secondFloorY + floorHeight + roofHeight + 112, 0);
        chimneyTop.castShadow = true;
        group.add(chimneyTop);

        // Porch / Steps
        const stepGeometry = new THREE.BoxGeometry(100, 15, 40);
        const stepMaterial = new THREE.MeshStandardMaterial({ color: 0x808080 }); // Gray steps

        const step1 = new THREE.Mesh(stepGeometry, stepMaterial);
        step1.position.set(0, 7.5, houseDepth / 2 + 30);
        step1.castShadow = true;
        step1.receiveShadow = true;
        group.add(step1);

        const step2 = new THREE.Mesh(stepGeometry, stepMaterial);
        step2.position.set(0, 22.5, houseDepth / 2 + 60);
        step2.castShadow = true;
        step2.receiveShadow = true;
        group.add(step2);

        group.position.set(x, 0, z);
        return group;
    }

    private setupControls(): void {
        document.addEventListener('keydown', (e: KeyboardEvent) => {
            const key = e.key.toLowerCase();
            this.keys[key] = true;

            // Toggle camera anchor
            if (key === 'c') {
                this.cameraAnchored = !this.cameraAnchored;
                console.log('Camera anchored:', this.cameraAnchored);
                const cameraStatus = document.getElementById('camera-status');
                if (cameraStatus) {
                    cameraStatus.textContent = `Camera: ${this.cameraAnchored ? 'Anchored' : 'Free'} (Press 'C' to toggle)`;
                }
            }
        });

        document.addEventListener('keyup', (e: KeyboardEvent) => {
            this.keys[e.key.toLowerCase()] = false;
        });

        // Mouse controls for camera
        document.addEventListener('mousedown', (e: MouseEvent) => {
            if (e.button === 2) {
                // Right click
                this.isRightMouseDown = true;
                this.lastMousePos = { x: e.clientX, y: e.clientY };
            }
        });

        document.addEventListener('mousemove', (e: MouseEvent) => {
            if (this.isRightMouseDown) {
                const deltaX = e.clientX - this.lastMousePos.x;
                const deltaY = e.clientY - this.lastMousePos.y;

                this.cameraRotation.theta -= deltaX * 0.01;
                this.cameraRotation.phi += deltaY * 0.01;

                // Clamp vertical rotation
                this.cameraRotation.phi = Math.max(0.1, Math.min(Math.PI / 2 - 0.1, this.cameraRotation.phi));

                this.lastMousePos = { x: e.clientX, y: e.clientY };
            }
        });

        document.addEventListener('mouseup', (e: MouseEvent) => {
            if (e.button === 2) {
                this.isRightMouseDown = false;
            }
        });

        // Zoom control
        document.addEventListener('wheel', (e: WheelEvent) => {
            this.cameraDistance += e.deltaY;
            this.cameraDistance = Math.max(200, Math.min(5000, this.cameraDistance));
        });

        // Prevent context menu on right click
        document.addEventListener('contextmenu', (e: Event) => {
            e.preventDefault();
        });
    }

    private connectWebSocket(): void {
        const wsProtocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        const wsUrl = `${wsProtocol}//${window.location.host}/ws`;

        this.ws = new WebSocket(wsUrl);

        this.ws.onopen = () => {
            console.log('WebSocket connected');
            // Send join message
            this.sendMessage({
                type: 'Join',
                player: {
                    id: this.user.id,
                    username: this.user.username,
                    position: {
                        x: this.myPlayer!.mesh.position.x,
                        y: this.myPlayer!.mesh.position.y,
                        z: this.myPlayer!.mesh.position.z
                    },
                    rotation: this.myPlayer!.rotation
                }
            });
        };

        this.ws.onmessage = (event: MessageEvent) => {
            try {
                const message = JSON.parse(event.data) as WebSocketMessage;
                this.handleMessage(message);
            } catch (error) {
                console.error('Failed to parse message:', error);
            }
        };

        this.ws.onerror = (error: Event) => {
            console.error('WebSocket error:', error);
        };

        this.ws.onclose = () => {
            console.log('WebSocket disconnected');
            setTimeout(() => this.connectWebSocket(), 3000);
        };
    }

    private handleMessage(message: WebSocketMessage): void {
        switch (message.type) {
            case 'WorldState':
                this.handleWorldState(message.players);
                break;
            case 'Move':
                this.handlePlayerMove(message.player_id, message.position, message.rotation, message.is_moving);
                break;
            case 'Join':
                if (message.player.id !== this.user.id) {
                    this.addPlayer(message.player);
                }
                break;
            case 'Leave':
                this.removePlayer(message.player_id);
                break;
        }
    }

    private handleWorldState(players: PlayerData[]): void {
        players.forEach((playerData) => {
            if (playerData.id !== this.user.id) {
                if (!this.players.has(playerData.id)) {
                    this.addPlayer(playerData);
                } else {
                    const player = this.players.get(playerData.id)!;
                    player.mesh.position.set(playerData.position.x, playerData.position.y, playerData.position.z);
                    player.mesh.rotation.y = playerData.rotation;
                    player.isMoving = playerData.is_moving || false;
                }
            }
        });
        this.updatePlayersList();
    }

    private handlePlayerMove(playerId: string, position: Position, rotation: number, isMoving: boolean): void {
        if (playerId === this.user.id) return;

        const player = this.players.get(playerId);
        if (player) {
            player.mesh.position.set(position.x, position.y, position.z);
            player.mesh.rotation.y = rotation;
            player.isMoving = isMoving || false;
        }
    }

    private addPlayer(playerData: PlayerData): void {
        const player = this.createPlayer(playerData.id, playerData.username);
        player.mesh.position.set(playerData.position.x, playerData.position.y, playerData.position.z);
        player.mesh.rotation.y = playerData.rotation;
        player.isMoving = playerData.is_moving || false;
        this.scene?.add(player.mesh);
        this.players.set(playerData.id, player);
        this.updatePlayersList();
    }

    private removePlayer(playerId: string): void {
        const player = this.players.get(playerId);
        if (player) {
            this.scene?.remove(player.mesh);
            this.players.delete(playerId);
            this.updatePlayersList();
        }
    }

    private updatePlayersList(): void {
        const list = document.getElementById('players-list');
        if (!list) return;
        list.innerHTML = '';
        this.players.forEach((player) => {
            const li = document.createElement('li');
            li.textContent = player.username;
            list.appendChild(li);
        });
    }

    private sendMessage(message: WebSocketMessage): void {
        if (this.ws && this.ws.readyState === WebSocket.OPEN) {
            this.ws.send(JSON.stringify(message));
        }
    }

    private updateMovement(): void {
        if (!this.myPlayer) return;

        let moved = false;
        let newRotation = this.myPlayer.rotation;

        // Random movement logic
        this.randomMoveTimer -= 1;
        if (this.randomMoveTimer <= 0) {
            // Pick a new random action
            const actions: RandomMoveAction[] = ['idle', 'forward', 'left', 'right'];
            this.randomMoveAction = actions[Math.floor(Math.random() * actions.length)];
            this.randomMoveTimer = Math.floor(Math.random() * 60) + 30; // 0.5 to 1.5 seconds at 60fps
        }

        if (this.randomMoveAction === 'forward') {
            this.myPlayer.mesh.translateZ(-this.moveSpeed);
            moved = true;
        } else if (this.randomMoveAction === 'left') {
            newRotation += this.rotationSpeed;
            moved = true;
        } else if (this.randomMoveAction === 'right') {
            newRotation -= this.rotationSpeed;
            moved = true;
        }

        this.myPlayer.isMoving = moved;

        if (moved) {
            this.myPlayer.rotation = newRotation;
            this.myPlayer.mesh.rotation.y = newRotation;

            // Send movement update
            this.sendMessage({
                type: 'Move',
                player_id: this.user.id,
                position: {
                    x: this.myPlayer.mesh.position.x,
                    y: this.myPlayer.mesh.position.y,
                    z: this.myPlayer.mesh.position.z
                },
                rotation: this.myPlayer.rotation,
                is_moving: this.myPlayer.isMoving
            });
        }
    }

    private updateAnimations(): void {
        this.players.forEach((player) => {
            // For other players, detect movement by position change
            if (player.id !== this.user.id) {
                const pos = player.mesh.position;
                if (player.lastPosition) {
                    const dist = Math.sqrt(
                        Math.pow(pos.x - player.lastPosition.x, 2) + Math.pow(pos.z - player.lastPosition.z, 2)
                    );
                    player.isMoving = dist > 0.1;
                }
                player.lastPosition = { x: pos.x, y: pos.y, z: pos.z };
            }

            if (player.isMoving) {
                player.walkCycle += 0.15;
                const swing = Math.sin(player.walkCycle) * 0.6;

                // Animate legs
                if (player.leftLeg) player.leftLeg.rotation.x = swing;
                if (player.rightLeg) player.rightLeg.rotation.x = -swing;

                // Animate arms (opposite to legs)
                if (player.leftArm) player.leftArm.rotation.x = -swing;
                if (player.rightArm) player.rightArm.rotation.x = swing;

                // Slight body bob
                const torso = player.mesh.children[0] as THREE.Mesh;
                if (torso) torso.position.y = 110 + Math.abs(Math.cos(player.walkCycle)) * 5;
            } else {
                // Reset to idle pose
                player.walkCycle = 0;
                if (player.leftLeg)
                    player.leftLeg.rotation.x = THREE.MathUtils.lerp(player.leftLeg.rotation.x, 0, 0.2);
                if (player.rightLeg)
                    player.rightLeg.rotation.x = THREE.MathUtils.lerp(player.rightLeg.rotation.x, 0, 0.2);
                if (player.leftArm)
                    player.leftArm.rotation.x = THREE.MathUtils.lerp(player.leftArm.rotation.x, 0, 0.2);
                if (player.rightArm)
                    player.rightArm.rotation.x = THREE.MathUtils.lerp(player.rightArm.rotation.x, 0, 0.2);

                const torso = player.mesh.children[0] as THREE.Mesh;
                if (torso) torso.position.y = THREE.MathUtils.lerp(torso.position.y, 110, 0.2);
            }
        });
    }

    private animate(): void {
        requestAnimationFrame(() => this.animate());
        this.updateTime();
        this.updateMovement();
        this.updateAnimations();
        this.updateCamera();
        this.renderer?.render(this.scene!, this.camera!);
    }

    private updateCamera(): void {
        if (!this.myPlayer || !this.camera) return;

        // Handle arrow keys for horizontal camera rotation
        if (this.keys['arrowleft']) {
            this.cameraRotation.theta += 0.02;
        }
        if (this.keys['arrowright']) {
            this.cameraRotation.theta -= 0.02;
        }

        let targetX: number, targetY: number, targetZ: number;
        let baseRotation = 0;

        if (this.cameraAnchored) {
            targetX = this.myPlayer.mesh.position.x;
            targetY = this.myPlayer.mesh.position.y;
            targetZ = this.myPlayer.mesh.position.z;
            baseRotation = this.myPlayer.rotation;
            this.cameraFocusPoint = null; // Reset when anchored
        } else {
            if (!this.cameraFocusPoint) {
                this.cameraFocusPoint = new THREE.Vector3(
                    this.myPlayer.mesh.position.x,
                    this.myPlayer.mesh.position.y,
                    this.myPlayer.mesh.position.z
                );
            }

            // Move focus point with WASD when unanchored
            const moveSpeed = 50;
            if (this.keys['w']) {
                this.cameraFocusPoint.z -= Math.cos(this.cameraRotation.theta) * moveSpeed;
                this.cameraFocusPoint.x -= Math.sin(this.cameraRotation.theta) * moveSpeed;
            }
            if (this.keys['s']) {
                this.cameraFocusPoint.z += Math.cos(this.cameraRotation.theta) * moveSpeed;
                this.cameraFocusPoint.x += Math.sin(this.cameraRotation.theta) * moveSpeed;
            }
            if (this.keys['a']) {
                this.cameraFocusPoint.x -= Math.cos(this.cameraRotation.theta) * moveSpeed;
                this.cameraFocusPoint.z += Math.sin(this.cameraRotation.theta) * moveSpeed;
            }
            if (this.keys['d']) {
                this.cameraFocusPoint.x += Math.cos(this.cameraRotation.theta) * moveSpeed;
                this.cameraFocusPoint.z -= Math.sin(this.cameraRotation.theta) * moveSpeed;
            }

            targetX = this.cameraFocusPoint.x;
            targetY = this.cameraFocusPoint.y;
            targetZ = this.cameraFocusPoint.z;
            baseRotation = 0; // Don't use player rotation when unanchored
        }

        const relativeTheta = baseRotation + this.cameraRotation.theta;

        const x = Math.sin(relativeTheta) * Math.cos(this.cameraRotation.phi) * this.cameraDistance;
        const y = Math.sin(this.cameraRotation.phi) * this.cameraDistance;
        const z = Math.cos(relativeTheta) * Math.cos(this.cameraRotation.phi) * this.cameraDistance;

        this.camera.position.set(targetX + x, targetY + y, targetZ + z);

        // Look at the target point
        const target = new THREE.Vector3(targetX, targetY + 150, targetZ);
        this.camera.lookAt(target);
    }

    private updateTime(): void {
        // 24 hours game time = 24 minutes real time
        // 1 hour game time = 1 minute real time
        const now = new Date();
        const seconds = now.getSeconds();
        const ms = now.getMilliseconds();

        // This will give us 0-23.999... cycle every 24 minutes
        const totalRealMinutes = now.getHours() * 60 + now.getMinutes();
        const gameTime = (totalRealMinutes % 24) + seconds / 60 + ms / 60000;

        // Update time display
        const displayHours = Math.floor(gameTime);
        const displayMinutes = Math.floor((gameTime % 1) * 60);
        const timeString = `${displayHours.toString().padStart(2, '0')}:${displayMinutes.toString().padStart(2, '0')}`;
        const timeDisplay = document.getElementById('game-time-display');
        if (timeDisplay) {
            timeDisplay.textContent = timeString;
        }

        // Calculate sun position (rotation around X axis)
        // 0h: midnight, 6h: sunrise, 12h: noon, 18h: sunset
        const angle = (gameTime / 24) * Math.PI * 2 - Math.PI / 2;
        const radius = 5000;

        const sunX = 0;
        const sunY = Math.sin(angle) * radius;
        const sunZ = Math.cos(angle) * radius;

        this.sunMesh?.position.set(sunX, sunY, sunZ);
        this.sunLight?.position.set(sunX, sunY, sunZ);

        // Update moon position (opposite to sun)
        this.moonMesh?.position.set(-sunX, -sunY, -sunZ);

        // Adjust intensity and background color
        const sunUp = sunY > 0;
        const intensity = sunUp ? Math.max(0, Math.sin(angle)) : 0;

        if (this.sunLight) this.sunLight.intensity = intensity;
        if (this.ambientLight) this.ambientLight.intensity = 0.1 + intensity * 0.3;

        // Sky color
        if (this.scene) {
            if (sunUp) {
                // Day sky: light blue
                const dayColor = new THREE.Color(0x87ceeb);
                const sunsetColor = new THREE.Color(0xff4500);
                const skyColor = dayColor.clone().lerp(sunsetColor, 1 - intensity);
                this.scene.background = skyColor;
            } else {
                // Night sky: dark blue/black
                const nightColor = new THREE.Color(0x000022);
                this.scene.background = nightColor;
            }
        }
    }
}
