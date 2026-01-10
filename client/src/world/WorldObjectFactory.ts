import * as THREE from 'three';
import { Tree, TreePreset } from '@dgreenheck/ez-tree';
import { HouseBuilder } from './HouseBuilder';

// Available tree presets from ez-tree
export type TreePresetName = keyof typeof TreePreset;

export class WorldObjectFactory {
    // --- Public Static Methods ---

    public static createGround(size: number = 10000): THREE.Group {
        const group = new THREE.Group();

        // Base ground plane
        const geometry = new THREE.PlaneGeometry(size, size);
        const material = new THREE.MeshStandardMaterial({ color: 0x90ee90 });
        const ground = new THREE.Mesh(geometry, material);
        ground.rotation.x = -Math.PI / 2;
        ground.receiveShadow = true;
        group.add(ground);

        // Add grey gridlines 100 units apart
        const gridHelper = new THREE.GridHelper(size, size / 100, 0x808080, 0x808080);
        gridHelper.rotation.x = -Math.PI / 2;
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

        // Scale the tree to match game units (1 unit = 1 cm)
        // Default ez-tree is in meters, so multiply by 100 to convert to cm
        const scaleFactor = 100;

        // Generate the tree geometry
        tree.generate();

        // Apply scaling and enable shadows
        tree.scale.set(scaleFactor, scaleFactor, scaleFactor);
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
        const trunkGeometry = new THREE.BoxGeometry(40, 150, 40);
        const trunkMaterial = new THREE.MeshStandardMaterial({ color: 0x8b4513 });
        const trunk = new THREE.Mesh(trunkGeometry, trunkMaterial);
        trunk.position.y = 75;
        trunk.castShadow = true;
        trunk.receiveShadow = true;
        group.add(trunk);

        // Foliage (green)
        const leavesGeometry = new THREE.BoxGeometry(200, 250, 200);
        const leavesMaterial = new THREE.MeshStandardMaterial({ color: 0x228b22 });
        const leaves = new THREE.Mesh(leavesGeometry, leavesMaterial);
        leaves.position.y = 150 + 125;
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
     * Creates a pole with red marks every 100 units height
     * @param x - X position
     * @param z - Z position
     * @param height - Total height of the pole (default: 1000 units = 10 meters)
     */
    public static createPole(x: number, z: number, height: number = 1000): THREE.Group {
        const group = new THREE.Group();

        // Main pole (grey/white)
        const poleGeometry = new THREE.CylinderGeometry(10, 10, height, 8);
        const poleMaterial = new THREE.MeshStandardMaterial({ color: 0xcccccc });
        const pole = new THREE.Mesh(poleGeometry, poleMaterial);
        pole.position.y = height / 2;
        pole.castShadow = true;
        pole.receiveShadow = true;
        group.add(pole);

        // Add red marks every 100 units
        const markHeight = 30; // Height of each mark
        const markWidth = 40; // Width of each mark (extends outward from pole)
        const markGeometry = new THREE.BoxGeometry(markWidth, markHeight, markWidth);
        const markMaterial = new THREE.MeshStandardMaterial({ color: 0xff0000 });
        
        for (let y = 100; y < height; y += 100) {
            const mark = new THREE.Mesh(markGeometry, markMaterial);
            mark.position.y = y;
            mark.position.x = 20; // Offset from center of pole (pole radius 10 + mark width/2)
            mark.castShadow = true;
            mark.receiveShadow = true;
            group.add(mark);
        }

        group.position.set(x, 0, z);
        return group;
    }
}
