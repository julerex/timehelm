import * as THREE from 'three';

// Floor height constants (matching WorldObjectFactory/HouseBuilder)
const FOUNDATION_HEIGHT = 40;
const FLOOR_HEIGHT = 270;
const FLOOR1_TOP = FOUNDATION_HEIGHT + FLOOR_HEIGHT;       // 310cm
const FLOOR2_TOP = FOUNDATION_HEIGHT + FLOOR_HEIGHT * 2;   // 580cm

const HIDDEN_OPACITY = 0.15;
const VISIBLE_OPACITY = 1.0;

/**
 * Manages height-based opacity for world objects.
 * Objects above a threshold become semi-transparent.
 * Doors and roof elements are made transparent along with walls at the same floor level.
 */
export class HeightOpacityManager {
    private readonly worldObjects: THREE.Object3D[];

    constructor(worldObjects: THREE.Object3D[]) {
        this.worldObjects = worldObjects;
    }

    /**
     * Sets opacity for objects based on height threshold.
     * Objects above the threshold become semi-transparent.
     * Doors and roof elements are made transparent along with walls at the same floor level.
     * @param heightThreshold - Height in cm above which objects become transparent. null = fully opaque.
     */
    public setHeightOpacity(heightThreshold: number | null): void {
        for (const obj of this.worldObjects) {
            obj.traverse((child) => {
                if (child instanceof THREE.Mesh && child.material instanceof THREE.MeshStandardMaterial) {
                    // Get world position of the mesh
                    const worldPos = new THREE.Vector3();
                    child.getWorldPosition(worldPos);

                    // Determine target opacity based on type
                    let targetOpacity: number;
                    if (heightThreshold === null) {
                        targetOpacity = VISIBLE_OPACITY;
                    } else {
                        const occlusionType = child.userData.occlusionType as string | undefined;
                        const floorLevel = child.userData.floorLevel as number | undefined;

                        if (occlusionType === 'door' && floorLevel !== undefined) {
                            // Doors become transparent when their floor's walls would be transparent
                            // Floor 1 doors: transparent when threshold < floor1Top (310cm)
                            // Floor 2 doors: transparent when threshold < floor2Top (580cm)
                            const floorTop = floorLevel === 1 ? FLOOR1_TOP : FLOOR2_TOP;
                            targetOpacity = heightThreshold < floorTop ? HIDDEN_OPACITY : VISIBLE_OPACITY;
                        } else if (occlusionType === 'roof') {
                            // Roof becomes transparent when threshold < floor2Top (when looking at 2nd floor)
                            targetOpacity = heightThreshold < FLOOR2_TOP ? HIDDEN_OPACITY : VISIBLE_OPACITY;
                        } else {
                            // Default height-based check for walls and other objects
                            targetOpacity = worldPos.y > heightThreshold ? HIDDEN_OPACITY : VISIBLE_OPACITY;
                        }
                    }

                    // Update material opacity
                    child.material.transparent = true;
                    child.material.opacity = targetOpacity;
                    child.material.needsUpdate = true;
                }
            });
        }
    }
}


