import * as THREE from 'three';
import type { Player } from '../entities/Player';
import type { InputManager } from '../input/InputManager';

interface CameraRotation {
    theta: number;
    phi: number;
}

export class CameraController {
    private readonly camera: THREE.PerspectiveCamera;
    private rotation: CameraRotation = { theta: 0, phi: 0.5 };
    private distance: number = 1000;
    private anchored: boolean = true;
    private focusPoint: THREE.Vector3 | null = null;

    // Configuration
    private readonly minDistance: number = 200;
    private readonly maxDistance: number = 5000;
    private readonly rotationSpeed: number = 0.02;
    private readonly freeMoveSpeed: number = 50;

    constructor(aspectRatio: number) {
        this.camera = new THREE.PerspectiveCamera(75, aspectRatio, 10, 100000);
        this.camera.position.set(0, 500, 1000);
        this.camera.lookAt(0, 0, 0);
    }

    // --- Getters ---

    public getCamera(): THREE.PerspectiveCamera {
        return this.camera;
    }

    public isAnchored(): boolean {
        return this.anchored;
    }

    // --- Public Methods ---

    public toggleAnchor(): void {
        this.anchored = !this.anchored;
        if (this.anchored) {
            this.focusPoint = null;
        }
    }

    public setAnchor(anchored: boolean): void {
        this.anchored = anchored;
        if (anchored) {
            this.focusPoint = null;
        }
    }

    public adjustZoom(delta: number): void {
        this.distance += delta;
        this.distance = Math.max(this.minDistance, Math.min(this.maxDistance, this.distance));
    }

    public adjustRotation(deltaTheta: number, deltaPhi: number): void {
        this.rotation.theta -= deltaTheta;
        this.rotation.phi += deltaPhi;

        // Clamp vertical rotation
        this.rotation.phi = Math.max(0.1, Math.min(Math.PI / 2 - 0.1, this.rotation.phi));
    }

    public update(player: Player | null, input: InputManager): void {
        if (!player) return;

        // Handle arrow keys for horizontal camera rotation
        if (input.isKeyPressed('arrowleft')) {
            this.rotation.theta += this.rotationSpeed;
        }
        if (input.isKeyPressed('arrowright')) {
            this.rotation.theta -= this.rotationSpeed;
        }

        let targetX: number, targetY: number, targetZ: number;
        let baseRotation = 0;

        if (this.anchored) {
            const pos = player.mesh.position;
            targetX = pos.x;
            targetY = pos.y;
            targetZ = pos.z;
            baseRotation = player.rotation;
            this.focusPoint = null;
        } else {
            if (!this.focusPoint) {
                const pos = player.mesh.position;
                this.focusPoint = new THREE.Vector3(pos.x, pos.y, pos.z);
            }

            // Move focus point with WASD when unanchored
            if (input.isKeyPressed('w')) {
                this.focusPoint.z -= Math.cos(this.rotation.theta) * this.freeMoveSpeed;
                this.focusPoint.x -= Math.sin(this.rotation.theta) * this.freeMoveSpeed;
            }
            if (input.isKeyPressed('s')) {
                this.focusPoint.z += Math.cos(this.rotation.theta) * this.freeMoveSpeed;
                this.focusPoint.x += Math.sin(this.rotation.theta) * this.freeMoveSpeed;
            }
            if (input.isKeyPressed('a')) {
                this.focusPoint.x -= Math.cos(this.rotation.theta) * this.freeMoveSpeed;
                this.focusPoint.z += Math.sin(this.rotation.theta) * this.freeMoveSpeed;
            }
            if (input.isKeyPressed('d')) {
                this.focusPoint.x += Math.cos(this.rotation.theta) * this.freeMoveSpeed;
                this.focusPoint.z -= Math.sin(this.rotation.theta) * this.freeMoveSpeed;
            }

            targetX = this.focusPoint.x;
            targetY = this.focusPoint.y;
            targetZ = this.focusPoint.z;
            baseRotation = 0;
        }

        const relativeTheta = baseRotation + this.rotation.theta;

        const x = Math.sin(relativeTheta) * Math.cos(this.rotation.phi) * this.distance;
        const y = Math.sin(this.rotation.phi) * this.distance;
        const z = Math.cos(relativeTheta) * Math.cos(this.rotation.phi) * this.distance;

        this.camera.position.set(targetX + x, targetY + y, targetZ + z);

        // Look at the target point
        const target = new THREE.Vector3(targetX, targetY + 150, targetZ);
        this.camera.lookAt(target);
    }

    public handleResize(aspectRatio: number): void {
        this.camera.aspect = aspectRatio;
        this.camera.updateProjectionMatrix();
    }

    public setInitialPosition(player: Player): void {
        const pos = player.mesh.position;
        this.camera.position.set(pos.x, pos.y + 500, pos.z + 1000);
    }
}
