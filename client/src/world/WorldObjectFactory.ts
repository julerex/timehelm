import * as THREE from 'three';

export class WorldObjectFactory {
    // --- Public Static Methods ---

    public static createGround(size: number = 10000): THREE.Mesh {
        const geometry = new THREE.PlaneGeometry(size, size);
        const material = new THREE.MeshStandardMaterial({ color: 0x90ee90 });
        const ground = new THREE.Mesh(geometry, material);
        ground.rotation.x = -Math.PI / 2;
        ground.receiveShadow = true;
        return ground;
    }

    public static createTree(x: number, z: number): THREE.Group {
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
}

// --- Private House Builder Class ---

class HouseBuilder {
    private readonly group: THREE.Group;

    // Dimensions
    private readonly width = 400;
    private readonly depth = 300;
    private readonly floorHeight = 200;
    private readonly wallThickness = 20;

    // Materials
    private readonly wallMaterial: THREE.MeshStandardMaterial;
    private readonly roofMaterial: THREE.MeshStandardMaterial;
    private readonly doorMaterial: THREE.MeshStandardMaterial;
    private readonly windowMaterial: THREE.MeshStandardMaterial;
    private readonly frameMaterial: THREE.MeshStandardMaterial;
    private readonly foundationMaterial: THREE.MeshStandardMaterial;
    private readonly chimneyMaterial: THREE.MeshStandardMaterial;

    constructor() {
        this.group = new THREE.Group();

        // Initialize materials
        this.wallMaterial = new THREE.MeshStandardMaterial({ color: 0xf5deb3 });
        this.roofMaterial = new THREE.MeshStandardMaterial({ color: 0x8b0000 });
        this.doorMaterial = new THREE.MeshStandardMaterial({ color: 0x8b4513 });
        this.windowMaterial = new THREE.MeshStandardMaterial({ color: 0x87ceeb, transparent: true, opacity: 0.7 });
        this.frameMaterial = new THREE.MeshStandardMaterial({ color: 0xffffff });
        this.foundationMaterial = new THREE.MeshStandardMaterial({ color: 0x696969 });
        this.chimneyMaterial = new THREE.MeshStandardMaterial({ color: 0xa52a2a });
    }

    public build(): THREE.Group {
        this.addFoundation();
        this.addFirstFloorWalls();
        this.addSecondFloorWalls();
        this.addFloorBetweenStories();
        this.addDoor();
        this.addFirstFloorWindows();
        this.addSecondFloorWindows();
        this.addSideWindows();
        this.addRoof();
        this.addChimney();
        this.addPorch();

        return this.group;
    }

    private addFoundation(): void {
        const geometry = new THREE.BoxGeometry(this.width + 40, 30, this.depth + 40);
        const foundation = new THREE.Mesh(geometry, this.foundationMaterial);
        foundation.position.y = 15;
        foundation.castShadow = true;
        foundation.receiveShadow = true;
        this.group.add(foundation);
    }

    private addFirstFloorWalls(): void {
        const baseY = 30 + this.floorHeight / 2;

        // Front wall
        const frontGeometry = new THREE.BoxGeometry(this.width, this.floorHeight, this.wallThickness);
        const frontWall = new THREE.Mesh(frontGeometry, this.wallMaterial);
        frontWall.position.set(0, baseY, this.depth / 2);
        frontWall.castShadow = true;
        frontWall.receiveShadow = true;
        this.group.add(frontWall);

        // Back wall
        const backWall = new THREE.Mesh(frontGeometry, this.wallMaterial);
        backWall.position.set(0, baseY, -this.depth / 2);
        backWall.castShadow = true;
        backWall.receiveShadow = true;
        this.group.add(backWall);

        // Side walls
        const sideGeometry = new THREE.BoxGeometry(this.wallThickness, this.floorHeight, this.depth);

        const leftWall = new THREE.Mesh(sideGeometry, this.wallMaterial);
        leftWall.position.set(-this.width / 2, baseY, 0);
        leftWall.castShadow = true;
        leftWall.receiveShadow = true;
        this.group.add(leftWall);

        const rightWall = new THREE.Mesh(sideGeometry, this.wallMaterial);
        rightWall.position.set(this.width / 2, baseY, 0);
        rightWall.castShadow = true;
        rightWall.receiveShadow = true;
        this.group.add(rightWall);
    }

    private addSecondFloorWalls(): void {
        const secondFloorY = 30 + this.floorHeight;
        const baseY = secondFloorY + this.floorHeight / 2;

        // Front wall
        const frontGeometry = new THREE.BoxGeometry(this.width, this.floorHeight, this.wallThickness);
        const frontWall = new THREE.Mesh(frontGeometry, this.wallMaterial);
        frontWall.position.set(0, baseY, this.depth / 2);
        frontWall.castShadow = true;
        frontWall.receiveShadow = true;
        this.group.add(frontWall);

        // Back wall
        const backWall = new THREE.Mesh(frontGeometry, this.wallMaterial);
        backWall.position.set(0, baseY, -this.depth / 2);
        backWall.castShadow = true;
        backWall.receiveShadow = true;
        this.group.add(backWall);

        // Side walls
        const sideGeometry = new THREE.BoxGeometry(this.wallThickness, this.floorHeight, this.depth);

        const leftWall = new THREE.Mesh(sideGeometry, this.wallMaterial);
        leftWall.position.set(-this.width / 2, baseY, 0);
        leftWall.castShadow = true;
        leftWall.receiveShadow = true;
        this.group.add(leftWall);

        const rightWall = new THREE.Mesh(sideGeometry, this.wallMaterial);
        rightWall.position.set(this.width / 2, baseY, 0);
        rightWall.castShadow = true;
        rightWall.receiveShadow = true;
        this.group.add(rightWall);
    }

    private addFloorBetweenStories(): void {
        const secondFloorY = 30 + this.floorHeight;
        const geometry = new THREE.BoxGeometry(
            this.width - this.wallThickness,
            15,
            this.depth - this.wallThickness
        );
        const floorMaterial = new THREE.MeshStandardMaterial({ color: 0xdeb887 });
        const floor = new THREE.Mesh(geometry, floorMaterial);
        floor.position.y = secondFloorY;
        floor.castShadow = true;
        floor.receiveShadow = true;
        this.group.add(floor);
    }

    private addDoor(): void {
        // Door
        const doorGeometry = new THREE.BoxGeometry(60, 120, this.wallThickness + 5);
        const door = new THREE.Mesh(doorGeometry, this.doorMaterial);
        door.position.set(0, 30 + 60, this.depth / 2);
        door.castShadow = true;
        this.group.add(door);

        // Door frame
        const frameGeometry = new THREE.BoxGeometry(70, 130, this.wallThickness + 2);
        const doorFrame = new THREE.Mesh(frameGeometry, this.frameMaterial);
        doorFrame.position.set(0, 30 + 65, this.depth / 2 - 2);
        this.group.add(doorFrame);

        // Door handle
        const handleGeometry = new THREE.BoxGeometry(8, 8, 10);
        const handleMaterial = new THREE.MeshStandardMaterial({ color: 0xffd700 });
        const handle = new THREE.Mesh(handleGeometry, handleMaterial);
        handle.position.set(20, 30 + 60, this.depth / 2 + 8);
        this.group.add(handle);
    }

    private addFirstFloorWindows(): void {
        const windowGeometry = new THREE.BoxGeometry(50, 60, this.wallThickness + 5);
        const windowFrameGeometry = new THREE.BoxGeometry(60, 70, this.wallThickness + 2);

        // Left window
        const window1Frame = new THREE.Mesh(windowFrameGeometry, this.frameMaterial);
        window1Frame.position.set(-120, 30 + 100, this.depth / 2 - 2);
        this.group.add(window1Frame);

        const window1 = new THREE.Mesh(windowGeometry, this.windowMaterial);
        window1.position.set(-120, 30 + 100, this.depth / 2);
        this.group.add(window1);

        // Right window
        const window2Frame = new THREE.Mesh(windowFrameGeometry, this.frameMaterial);
        window2Frame.position.set(120, 30 + 100, this.depth / 2 - 2);
        this.group.add(window2Frame);

        const window2 = new THREE.Mesh(windowGeometry, this.windowMaterial);
        window2.position.set(120, 30 + 100, this.depth / 2);
        this.group.add(window2);
    }

    private addSecondFloorWindows(): void {
        const secondFloorY = 30 + this.floorHeight;
        const windowGeometry = new THREE.BoxGeometry(50, 60, this.wallThickness + 5);
        const windowFrameGeometry = new THREE.BoxGeometry(60, 70, this.wallThickness + 2);

        const positions = [-120, 0, 120];

        for (const xPos of positions) {
            const frame = new THREE.Mesh(windowFrameGeometry, this.frameMaterial);
            frame.position.set(xPos, secondFloorY + 100, this.depth / 2 - 2);
            this.group.add(frame);

            const window = new THREE.Mesh(windowGeometry, this.windowMaterial);
            window.position.set(xPos, secondFloorY + 100, this.depth / 2);
            this.group.add(window);
        }
    }

    private addSideWindows(): void {
        const secondFloorY = 30 + this.floorHeight;
        const sideWindowGeometry = new THREE.BoxGeometry(this.wallThickness + 5, 60, 50);
        const sideWindowFrameGeometry = new THREE.BoxGeometry(this.wallThickness + 2, 70, 60);

        // Left side windows (both floors)
        for (const yOffset of [30 + 100, secondFloorY + 100]) {
            const frame = new THREE.Mesh(sideWindowFrameGeometry, this.frameMaterial);
            frame.position.set(-this.width / 2 + 2, yOffset, 0);
            this.group.add(frame);

            const window = new THREE.Mesh(sideWindowGeometry, this.windowMaterial);
            window.position.set(-this.width / 2, yOffset, 0);
            this.group.add(window);
        }

        // Right side windows (both floors)
        for (const yOffset of [30 + 100, secondFloorY + 100]) {
            const frame = new THREE.Mesh(sideWindowFrameGeometry, this.frameMaterial);
            frame.position.set(this.width / 2 - 2, yOffset, 0);
            this.group.add(frame);

            const window = new THREE.Mesh(sideWindowGeometry, this.windowMaterial);
            window.position.set(this.width / 2, yOffset, 0);
            this.group.add(window);
        }
    }

    private addRoof(): void {
        const secondFloorY = 30 + this.floorHeight;
        const roofHeight = 120;
        const roofOverhang = 40;

        const roofLength = this.depth + roofOverhang * 2;
        const roofWidth = Math.sqrt(
            Math.pow(this.width / 2 + roofOverhang, 2) + Math.pow(roofHeight, 2)
        );

        const roofGeometry = new THREE.BoxGeometry(roofWidth, 15, roofLength);
        const roofAngle = Math.atan2(roofHeight, this.width / 2 + roofOverhang);

        // Left roof slope
        const roofLeft = new THREE.Mesh(roofGeometry, this.roofMaterial);
        roofLeft.rotation.z = roofAngle;
        roofLeft.position.set(
            -(this.width / 4 + roofOverhang / 2) * Math.cos(roofAngle),
            secondFloorY + this.floorHeight + roofHeight / 2 + 15,
            0
        );
        roofLeft.castShadow = true;
        roofLeft.receiveShadow = true;
        this.group.add(roofLeft);

        // Right roof slope
        const roofRight = new THREE.Mesh(roofGeometry, this.roofMaterial);
        roofRight.rotation.z = -roofAngle;
        roofRight.position.set(
            (this.width / 4 + roofOverhang / 2) * Math.cos(roofAngle),
            secondFloorY + this.floorHeight + roofHeight / 2 + 15,
            0
        );
        roofRight.castShadow = true;
        roofRight.receiveShadow = true;
        this.group.add(roofRight);

        // Gable ends
        this.addGableEnds(secondFloorY, roofHeight);
    }

    private addGableEnds(secondFloorY: number, roofHeight: number): void {
        const gableShape = new THREE.Shape();
        gableShape.moveTo(-this.width / 2, 0);
        gableShape.lineTo(0, roofHeight);
        gableShape.lineTo(this.width / 2, 0);
        gableShape.lineTo(-this.width / 2, 0);

        const gableGeometry = new THREE.ExtrudeGeometry(gableShape, {
            depth: this.wallThickness,
            bevelEnabled: false
        });

        const gableFront = new THREE.Mesh(gableGeometry, this.wallMaterial);
        gableFront.position.set(0, secondFloorY + this.floorHeight, this.depth / 2 - this.wallThickness / 2);
        gableFront.castShadow = true;
        gableFront.receiveShadow = true;
        this.group.add(gableFront);

        const gableBack = new THREE.Mesh(gableGeometry, this.wallMaterial);
        gableBack.position.set(0, secondFloorY + this.floorHeight, -this.depth / 2 - this.wallThickness / 2);
        gableBack.castShadow = true;
        gableBack.receiveShadow = true;
        this.group.add(gableBack);
    }

    private addChimney(): void {
        const secondFloorY = 30 + this.floorHeight;
        const roofHeight = 120;

        // Chimney body
        const chimneyGeometry = new THREE.BoxGeometry(40, 150, 40);
        const chimney = new THREE.Mesh(chimneyGeometry, this.chimneyMaterial);
        chimney.position.set(this.width / 4, secondFloorY + this.floorHeight + roofHeight + 30, 0);
        chimney.castShadow = true;
        chimney.receiveShadow = true;
        this.group.add(chimney);

        // Chimney top
        const topGeometry = new THREE.BoxGeometry(50, 15, 50);
        const top = new THREE.Mesh(topGeometry, this.chimneyMaterial);
        top.position.set(this.width / 4, secondFloorY + this.floorHeight + roofHeight + 112, 0);
        top.castShadow = true;
        this.group.add(top);
    }

    private addPorch(): void {
        const stepGeometry = new THREE.BoxGeometry(100, 15, 40);
        const stepMaterial = new THREE.MeshStandardMaterial({ color: 0x808080 });

        const step1 = new THREE.Mesh(stepGeometry, stepMaterial);
        step1.position.set(0, 7.5, this.depth / 2 + 30);
        step1.castShadow = true;
        step1.receiveShadow = true;
        this.group.add(step1);

        const step2 = new THREE.Mesh(stepGeometry, stepMaterial);
        step2.position.set(0, 22.5, this.depth / 2 + 60);
        step2.castShadow = true;
        step2.receiveShadow = true;
        this.group.add(step2);
    }
}
