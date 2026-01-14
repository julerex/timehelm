/**
 * World object factory module.
 * 
 * Provides factory methods for creating world objects like:
 * - Ground plane
 * - Trees (procedural and simple)
 * - Houses
 * - Poles
 * - Furniture (loaded from GLB files)
 */

import * as THREE from 'three';
import { GLTFLoader } from 'three/examples/jsm/loaders/GLTFLoader.js';
import { Tree, TreePreset } from '@dgreenheck/ez-tree';
import { HouseBuilder } from './HouseBuilder';

/**
 * Available tree presets from ez-tree library.
 */
export type TreePresetName = keyof typeof TreePreset;

/**
 * Factory class for creating world objects.
 * 
 * All objects use meters as units (1 unit = 1 m).
 */
export class WorldObjectFactory {
    // --- Public Static Methods ---

    public static createGround(size: number = 100): THREE.Group {
        const group = new THREE.Group();

        // Base ground plane
        const geometry = new THREE.PlaneGeometry(size, size);
        const material = new THREE.MeshStandardMaterial({ color: 0x90ee90 });
        const ground = new THREE.Mesh(geometry, material);
        ground.rotation.x = -Math.PI / 2;
        ground.receiveShadow = true;
        group.add(ground);

        // Add grey gridlines ~1 meter apart
        const divisions = Math.max(1, Math.round(size));
        const gridHelper = new THREE.GridHelper(size, divisions, 0x808080, 0x808080);
        group.add(gridHelper);

        return group;
    }

    /**
     * Creates a procedurally generated tree using ez-tree
     * @param x - X position
     * @param z - Z position
     * @param seed - Optional seed for reproducible tree generation
     * @param preset - Optional preset type (defaults to 'Oak')
     */
    public static createTree(
        x: number,
        z: number,
        seed?: number,
        preset: TreePresetName = 'Oak Medium'
    ): THREE.Group {
        const tree = new Tree();

        // Apply preset for base tree type
        tree.loadPreset(TreePreset[preset]);

        // Set seed for reproducibility (use position-based seed if not provided)
        tree.options.seed = seed ?? Math.abs(x * 10000 + z);

        // Generate the tree geometry
        tree.generate();

        // Enable shadows
        tree.traverse((child) => {
            if (child instanceof THREE.Mesh) {
                child.castShadow = true;
                child.receiveShadow = true;
            }
        });

        tree.position.set(x, 0, z);
        return tree;
    }

    /**
     * Creates a simple box-based tree (fallback/low-poly version)
     */
    public static createSimpleTree(x: number, z: number): THREE.Group {
        const group = new THREE.Group();

        // Trunk (brown)
        const trunkGeometry = new THREE.BoxGeometry(0.4, 1.5, 0.4);
        const trunkMaterial = new THREE.MeshStandardMaterial({ color: 0x8b4513 });
        const trunk = new THREE.Mesh(trunkGeometry, trunkMaterial);
        trunk.position.y = 0.75;
        trunk.castShadow = true;
        trunk.receiveShadow = true;
        group.add(trunk);

        // Foliage (green)
        const leavesGeometry = new THREE.BoxGeometry(2.0, 2.5, 2.0);
        const leavesMaterial = new THREE.MeshStandardMaterial({ color: 0x228b22 });
        const leaves = new THREE.Mesh(leavesGeometry, leavesMaterial);
        leaves.position.y = 1.5 + 1.25;
        leaves.castShadow = true;
        leaves.receiveShadow = true;
        group.add(leaves);

        group.position.set(x, 0, z);
        return group;
    }

    public static createHouse(x: number, z: number): THREE.Group {
        const builder = new HouseBuilder();
        const house = builder.build();
        house.position.set(x, 0, z);
        return house;
    }

    /**
     * Creates a pole with red marks every 1 meter height.
     * @param x - X position
     * @param z - Z position
     * @param height - Total height of the pole (default: 10 meters)
     */
    public static createPole(x: number, z: number, height: number = 10): THREE.Group {
        const group = new THREE.Group();

        // Main pole (grey/white)
        const poleGeometry = new THREE.CylinderGeometry(0.1, 0.1, height, 8);
        const poleMaterial = new THREE.MeshStandardMaterial({ color: 0xcccccc });
        const pole = new THREE.Mesh(poleGeometry, poleMaterial);
        pole.position.y = height / 2;
        pole.castShadow = true;
        pole.receiveShadow = true;
        group.add(pole);

        // Add red marks every 1 meter
        const markHeight = 0.3; // Height of each mark
        const markWidth = 0.4; // Width of each mark (extends outward from pole)
        const markGeometry = new THREE.BoxGeometry(markWidth, markHeight, markWidth);
        const markMaterial = new THREE.MeshStandardMaterial({ color: 0xff0000 });
        
        for (let y = 1; y < height; y += 1) {
            const mark = new THREE.Mesh(markGeometry, markMaterial);
            mark.position.y = y;
            mark.position.x = 0.3; // Offset from center of pole (pole radius 0.1 + mark width/2)
            mark.castShadow = true;
            mark.receiveShadow = true;
            group.add(mark);
        }

        group.position.set(x, 0, z);
        return group;
    }

    /**
     * Loads a bed model from GLB file and places it at the specified position.
     * @param x - X position
     * @param z - Z position
     * @param y - Optional Y position (default: 0)
     * @returns Promise that resolves to the loaded bed group
     */
    public static async loadBed(x: number, z: number, y: number = 0): Promise<THREE.Group> {
        const loader = new GLTFLoader();
        
        return new Promise((resolve, reject) => {
            loader.load(
                '/assets/bedDouble.glb',
                (gltf) => {
                    const bed = gltf.scene;
                    
                    // Optional additional normalization: the current bedDouble.glb asset is undersized
                    // (native GLB units are typically meters).
                    // To keep world scale consistent with the house and player, we auto-scale the bed
                    // so its *largest horizontal dimension* is ~2.0 meters.
                    const targetMaxHorizontalM = 2.0;
                    const bboxAfterUnitScale = new THREE.Box3().setFromObject(bed);
                    const sizeAfterUnitScale = new THREE.Vector3();
                    bboxAfterUnitScale.getSize(sizeAfterUnitScale);
                    const maxHorizontal = Math.max(sizeAfterUnitScale.x, sizeAfterUnitScale.z);
                    if (Number.isFinite(maxHorizontal) && maxHorizontal > 0) {
                        const correctionScale = targetMaxHorizontalM / maxHorizontal;
                        // Avoid extreme scaling if an asset is accidentally authored at a very different scale.
                        if (correctionScale > 0.2 && correctionScale < 5) {
                            bed.scale.multiplyScalar(correctionScale);
                        }
                    }
                    
                    // Enable shadows on all meshes
                    bed.traverse((child) => {
                        if (child instanceof THREE.Mesh) {
                            child.castShadow = true;
                            child.receiveShadow = true;
                        }
                    });

                    // Normalize so the model "rests" on y=0 (move it up by its bounding box min Y)
                    // This prevents models whose origin is centered from being half-buried.
                    const bbox = new THREE.Box3().setFromObject(bed);
                    if (Number.isFinite(bbox.min.y) && bbox.min.y !== 0) {
                        bed.position.y += -bbox.min.y;
                    }
                    
                    // Position the bed
                    bed.position.x += x;
                    bed.position.y += y;
                    bed.position.z += z;
                    
                    console.log('[WorldObjectFactory] Bed loaded at position:', bed.position, 'scale:', bed.scale);
                    
                    resolve(bed);
                },
                (progress) => {
                    console.log('[WorldObjectFactory] Loading bed progress:', progress);
                },
                (error) => {
                    console.error('[WorldObjectFactory] Failed to load bed:', error);
                    reject(error);
                }
            );
        });
    }
}
