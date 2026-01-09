import * as THREE from 'three';

export interface Position {
    x: number;
    y: number;
    z: number;
}

/// Daily routine activities that characters can be engaged in
export type Activity =
    | 'idle'
    | 'sleeping'
    | 'eating'
    | 'cooking'
    | 'working'
    | 'exercising'
    | 'socializing'
    | 'shopping'
    | 'cleaning'
    | 'bathing'
    | 'reading'
    | 'watching_tv'
    | 'gaming'
    | 'commuting';

export const ACTIVITY_DISPLAY_TEXT: Record<Activity, string> = {
    idle: 'Idle',
    sleeping: 'Sleeping 💤',
    eating: 'Eating 🍽️',
    cooking: 'Cooking 🍳',
    working: 'Working 💼',
    exercising: 'Exercising 🏃',
    socializing: 'Socializing 💬',
    shopping: 'Shopping 🛒',
    cleaning: 'Cleaning 🧹',
    bathing: 'Bathing 🛁',
    reading: 'Reading 📖',
    watching_tv: 'Watching TV 📺',
    gaming: 'Gaming 🎮',
    commuting: 'Commuting 🚶',
};

export interface PlayerData {
    id: string;
    username: string;
    position: Position;
    rotation: number;
    is_moving?: boolean;
    activity?: Activity;
}

export class Player {
    public readonly id: string;
    public readonly username: string;
    public readonly mesh: THREE.Group;
    public readonly leftLeg: THREE.Mesh;
    public readonly rightLeg: THREE.Mesh;
    public readonly leftArm: THREE.Mesh;
    public readonly rightArm: THREE.Mesh;

    private _position: Position = { x: 0, y: 0, z: 0 };
    private _rotation: number = 0;
    private _isMoving: boolean = false;
    private _lastPosition: Position | null = null;
    private _activity: Activity = 'idle';
    private _activityCanvas: HTMLCanvasElement;
    private _activityTexture: THREE.CanvasTexture;

    constructor(id: string, username: string) {
        this.id = id;
        this.username = username;

        // Create mesh group and body parts
        const { group, leftLeg, rightLeg, leftArm, rightArm, activityCanvas, activityTexture } = this.createMesh(username);
        this.mesh = group;
        this.leftLeg = leftLeg;
        this.rightLeg = rightLeg;
        this.leftArm = leftArm;
        this.rightArm = rightArm;
        this._activityCanvas = activityCanvas;
        this._activityTexture = activityTexture;
    }

    // --- Getters and Setters ---

    get position(): Position {
        return { ...this._position };
    }

    set position(pos: Position) {
        this._position = { ...pos };
        this.mesh.position.set(pos.x, pos.y, pos.z);
    }

    get rotation(): number {
        return this._rotation;
    }

    set rotation(rot: number) {
        this._rotation = rot;
        this.mesh.rotation.y = rot;
    }

    get isMoving(): boolean {
        return this._isMoving;
    }

    set isMoving(moving: boolean) {
        this._isMoving = moving;
    }

    get activity(): Activity {
        return this._activity;
    }

    set activity(activity: Activity) {
        if (this._activity !== activity) {
            this._activity = activity;
            this.updateActivityDisplay();
        }
    }

    // --- Public Methods ---

    public static fromData(data: PlayerData): Player {
        const player = new Player(data.id, data.username);
        player.position = data.position;
        player.rotation = data.rotation;
        player.isMoving = data.is_moving || false;
        player.activity = data.activity || 'idle';
        return player;
    }

    public toData(): PlayerData {
        return {
            id: this.id,
            username: this.username,
            position: this.position,
            rotation: this.rotation,
            is_moving: this.isMoving,
            activity: this.activity
        };
    }

    public moveForward(speed: number, bounds?: { minX: number; maxX: number; minZ: number; maxZ: number }): void {
        this.mesh.translateZ(-speed);
        
        // Clamp to bounds if provided
        if (bounds) {
            this.mesh.position.x = Math.max(bounds.minX, Math.min(bounds.maxX, this.mesh.position.x));
            this.mesh.position.z = Math.max(bounds.minZ, Math.min(bounds.maxZ, this.mesh.position.z));
        }
        
        this._position = {
            x: this.mesh.position.x,
            y: this.mesh.position.y,
            z: this.mesh.position.z
        };
    }

    public rotate(delta: number): void {
        this._rotation += delta;
        this.mesh.rotation.y = this._rotation;
    }

    public updateAnimation(_deltaTime: number = 1): void {
        // No walking animation - character stays in static pose
    }

    public detectMovementFromPosition(): void {
        const pos = this.mesh.position;
        if (this._lastPosition) {
            const dist = Math.sqrt(
                Math.pow(pos.x - this._lastPosition.x, 2) +
                Math.pow(pos.z - this._lastPosition.z, 2)
            );
            this._isMoving = dist > 0.1;
        }
        this._lastPosition = { x: pos.x, y: pos.y, z: pos.z };
    }

    // --- Private Methods ---

    private createMesh(username: string): {
        group: THREE.Group;
        leftLeg: THREE.Mesh;
        rightLeg: THREE.Mesh;
        leftArm: THREE.Mesh;
        rightArm: THREE.Mesh;
        activityCanvas: HTMLCanvasElement;
        activityTexture: THREE.CanvasTexture;
    } {
        const group = new THREE.Group();

        // Body (torso)
        const bodyGeometry = new THREE.BoxGeometry(60, 80, 40);
        const bodyMaterial = new THREE.MeshStandardMaterial({ color: 0x4a90e2 });
        const body = new THREE.Mesh(bodyGeometry, bodyMaterial);
        body.position.y = 110;
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

        // Activity info box (above name label)
        const activityCanvas = document.createElement('canvas');
        const activityContext = activityCanvas.getContext('2d');
        activityCanvas.width = 320;
        activityCanvas.height = 48;
        if (activityContext) {
            this.renderActivityCanvas(activityContext, activityCanvas.width, activityCanvas.height, ACTIVITY_DISPLAY_TEXT['idle']);
        }

        const activityTexture = new THREE.CanvasTexture(activityCanvas);
        const activitySpriteMaterial = new THREE.SpriteMaterial({ map: activityTexture });
        const activitySprite = new THREE.Sprite(activitySpriteMaterial);
        activitySprite.position.y = 310;
        activitySprite.scale.set(240, 36, 100);
        group.add(activitySprite);

        return { group, leftLeg, rightLeg, leftArm, rightArm, activityCanvas, activityTexture };
    }

    private renderActivityCanvas(context: CanvasRenderingContext2D, width: number, height: number, text: string): void {
        // Clear canvas
        context.clearRect(0, 0, width, height);

        // Draw rounded rectangle background
        const radius = 12;
        const padding = 4;
        context.fillStyle = 'rgba(255, 255, 255, 0.95)';
        context.beginPath();
        context.roundRect(padding, padding, width - padding * 2, height - padding * 2, radius);
        context.fill();

        // Draw border
        context.strokeStyle = 'rgba(74, 144, 226, 0.8)';
        context.lineWidth = 2;
        context.stroke();

        // Draw text
        context.fillStyle = '#333333';
        context.font = 'bold 20px Arial';
        context.textAlign = 'center';
        context.textBaseline = 'middle';
        context.fillText(text, width / 2, height / 2);
    }

    private updateActivityDisplay(): void {
        const context = this._activityCanvas.getContext('2d');
        if (context) {
            const displayText = ACTIVITY_DISPLAY_TEXT[this._activity] || 'Idle';
            this.renderActivityCanvas(context, this._activityCanvas.width, this._activityCanvas.height, displayText);
            this._activityTexture.needsUpdate = true;
        }
    }
}
