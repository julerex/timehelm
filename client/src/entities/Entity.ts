import * as THREE from 'three';
import type { Position } from './Player';

export interface EntityData {
    id: string;
    entity_type: 'human' | 'ball';
    position: Position;
    rotation: { x: number; y: number; z: number };
}

export class Entity {
    public readonly id: string;
    public readonly entityType: 'human' | 'ball';
    public readonly mesh: THREE.Object3D;

    private _position: Position = { x: 0, y: 0, z: 0 };
    private _rotation: { x: number; y: number; z: number } = { x: 0, y: 0, z: 0 };

    constructor(id: string, entityType: 'human' | 'ball') {
        this.id = id;
        this.entityType = entityType;
        this.mesh = this.createMesh();
    }

    // --- Getters and Setters ---

    get position(): Position {
        return { ...this._position };
    }

    set position(pos: Position) {
        this._position = { ...pos };
        this.mesh.position.set(pos.x, pos.y, pos.z);
    }

    get rotation(): { x: number; y: number; z: number } {
        return { ...this._rotation };
    }

    set rotation(rot: { x: number; y: number; z: number }) {
        this._rotation = { ...rot };
        this.mesh.rotation.set(rot.x, rot.y, rot.z);
    }

    // --- Public Methods ---

    public static fromData(data: EntityData): Entity {
        const entity = new Entity(data.id, data.entity_type);
        entity.position = data.position;
        entity.rotation = data.rotation;
        return entity;
    }

    // --- Private Methods ---

    private createMesh(): THREE.Object3D {
        if (this.entityType === 'ball') {
            return this.createBallMesh();
        } else {
            // Human entities are handled as players, but we can create a placeholder if needed
            return new THREE.Group();
        }
    }

    private createBallMesh(): THREE.Mesh {
        // Create a bouncy ball - a sphere with a bright color
        const geometry = new THREE.SphereGeometry(50, 32, 32); // 50 unit radius (0.5m in game units)
        const material = new THREE.MeshStandardMaterial({
            color: 0xff0000, // Bright red
            roughness: 0.3,
            metalness: 0.1
        });
        const ball = new THREE.Mesh(geometry, material);
        ball.castShadow = true;
        ball.receiveShadow = true;
        return ball;
    }
}

