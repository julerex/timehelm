import * as THREE from 'three';

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
    private _walkCycle: number = 0;
    private _isMoving: boolean = false;
    private _lastPosition: Position | null = null;

    constructor(id: string, username: string) {
        this.id = id;
        this.username = username;

        // Create mesh group and body parts
        const { group, leftLeg, rightLeg, leftArm, rightArm } = this.createMesh(username);
        this.mesh = group;
        this.leftLeg = leftLeg;
        this.rightLeg = rightLeg;
        this.leftArm = leftArm;
        this.rightArm = rightArm;
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

    // --- Public Methods ---

    public static fromData(data: PlayerData): Player {
        const player = new Player(data.id, data.username);
        player.position = data.position;
        player.rotation = data.rotation;
        player.isMoving = data.is_moving || false;
        return player;
    }

    public toData(): PlayerData {
        return {
            id: this.id,
            username: this.username,
            position: this.position,
            rotation: this.rotation,
            is_moving: this.isMoving
        };
    }

    public moveForward(speed: number): void {
        this.mesh.translateZ(-speed);
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

    public updateAnimation(deltaTime: number = 1): void {
        if (this._isMoving) {
            // Walk cycle speed scaled for 60x game time (1.5 = 0.15 * 10)
            this._walkCycle += 1.5 * deltaTime;
            const swing = Math.sin(this._walkCycle) * 0.6;

            // Animate legs
            this.leftLeg.rotation.x = swing;
            this.rightLeg.rotation.x = -swing;

            // Animate arms (opposite to legs)
            this.leftArm.rotation.x = -swing;
            this.rightArm.rotation.x = swing;

            // Slight body bob
            const torso = this.mesh.children[0] as THREE.Mesh;
            if (torso) {
                torso.position.y = 110 + Math.abs(Math.cos(this._walkCycle)) * 5;
            }
        } else {
            // Reset to idle pose
            this._walkCycle = 0;
            this.leftLeg.rotation.x = THREE.MathUtils.lerp(this.leftLeg.rotation.x, 0, 0.2);
            this.rightLeg.rotation.x = THREE.MathUtils.lerp(this.rightLeg.rotation.x, 0, 0.2);
            this.leftArm.rotation.x = THREE.MathUtils.lerp(this.leftArm.rotation.x, 0, 0.2);
            this.rightArm.rotation.x = THREE.MathUtils.lerp(this.rightArm.rotation.x, 0, 0.2);

            const torso = this.mesh.children[0] as THREE.Mesh;
            if (torso) {
                torso.position.y = THREE.MathUtils.lerp(torso.position.y, 110, 0.2);
            }
        }
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

        return { group, leftLeg, rightLeg, leftArm, rightArm };
    }
}
