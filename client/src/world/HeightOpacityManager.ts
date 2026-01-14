import * as THREE from 'three';

const HIDDEN_OPACITY = 0.35;
const VISIBLE_OPACITY = 1.0;

type OpacityMaterial = THREE.Material & {
    opacity: number;
    transparent: boolean;
    needsUpdate: boolean;
    userData: Record<string, unknown>;
};

/**
 * Manages height-based opacity for world objects.
 * Objects above a threshold become semi-transparent.
 *
 * Implementation detail:
 * - We treat "above the cutoff" as: the mesh's world-space bounding box extends above the cutoff.
 *   This gives a better "cutaway" feel than using the mesh origin, while keeping doors (short)
 *   opaque when the cutoff is above them.
 */
export class HeightOpacityManager {
    private readonly worldObjects: THREE.Object3D[];
    private readonly tmpBox = new THREE.Box3();

    constructor(worldObjects: THREE.Object3D[]) {
        this.worldObjects = worldObjects;
    }

    /**
     * Sets opacity for objects based on height threshold.
     * Objects above the threshold become semi-transparent.
     * @param heightThreshold - Height in meters above which objects become transparent. null = fully opaque.
     */
    public setHeightOpacity(heightThreshold: number | null): void {
        for (const obj of this.worldObjects) {
            obj.traverse((child) => {
                if (!(child instanceof THREE.Mesh)) return;

                const materials = Array.isArray(child.material) ? child.material : [child.material];
                if (materials.length === 0) return;

                // Determine target opacity (per mesh) based on world-space height.
                let applyHidden = false;
                if (heightThreshold !== null) {
                    // Ensure matrixWorld is current before we transform the local bbox.
                    child.updateWorldMatrix(true, false);

                    const geom = child.geometry;
                    if (geom?.boundingBox === null) {
                        geom.computeBoundingBox();
                    }
                    if (geom?.boundingBox) {
                        this.tmpBox.copy(geom.boundingBox).applyMatrix4(child.matrixWorld);
                        applyHidden = this.tmpBox.max.y > heightThreshold;
                    }
                }

                for (const material of materials) {
                    if (!material) continue;
                    const mat = material as Partial<OpacityMaterial>;
                    if (typeof mat.opacity !== 'number' || typeof mat.transparent !== 'boolean') continue;

                    // Cache original state once so we can restore on reset.
                    const userData = (material.userData ??= {});
                    if (userData.__heightOpacityOriginalOpacity === undefined) {
                        userData.__heightOpacityOriginalOpacity = mat.opacity;
                    }
                    if (userData.__heightOpacityOriginalTransparent === undefined) {
                        userData.__heightOpacityOriginalTransparent = mat.transparent;
                    }

                    const originalOpacity = userData.__heightOpacityOriginalOpacity as number;
                    const originalTransparent = userData.__heightOpacityOriginalTransparent as boolean;

                    if (heightThreshold === null) {
                        mat.opacity = originalOpacity;
                        mat.transparent = originalTransparent || originalOpacity < VISIBLE_OPACITY;
                    } else if (applyHidden) {
                        mat.opacity = Math.min(originalOpacity, HIDDEN_OPACITY);
                        mat.transparent = true;
                    } else {
                        mat.opacity = originalOpacity;
                        mat.transparent = originalTransparent || originalOpacity < VISIBLE_OPACITY;
                    }

                    mat.needsUpdate = true;
                }
            });
        }
    }
}




