/**
 * House builder module.
 * 
 * Procedurally generates a realistic two-story house with:
 * - Foundation
 * - Two floors with interior walls
 * - Doors and windows
 * - Roof with chimney
 * - Porch
 * 
 * All dimensions are in centimeters (1 unit = 1 cm).
 */

import * as THREE from 'three';

/**
 * House builder class.
 * 
 * Builds a complete two-story house with realistic dimensions and layout.
 * House footprint: 12m x 10m (1200cm x 1000cm).
 */
export class HouseBuilder {
    private readonly group: THREE.Group;

    // Realistic house dimensions (1 unit = 1 cm)
    // Total footprint: 12m x 10m (1200cm x 1000cm)
    private readonly width = 1200;      // 12 meters wide
    private readonly depth = 1000;      // 10 meters deep
    private readonly floorHeight = 270; // 2.7 meters ceiling height (standard residential)
    private readonly wallThickness = 20; // 20cm walls
    private readonly foundationHeight = 40; // 40cm foundation

    // Door dimensions (standard: 210cm tall, 90cm wide)
    private readonly doorHeight = 210;
    private readonly doorWidth = 90;

    // Window dimensions (standard: 120cm tall, 100cm wide)
    private readonly windowHeight = 120;
    private readonly windowWidth = 100;

    // Materials
    private readonly wallMaterial: THREE.MeshStandardMaterial;
    private readonly interiorWallMaterial: THREE.MeshStandardMaterial;
    private readonly roofMaterial: THREE.MeshStandardMaterial;
    private readonly doorMaterial: THREE.MeshStandardMaterial;
    private readonly windowMaterial: THREE.MeshStandardMaterial;
    private readonly frameMaterial: THREE.MeshStandardMaterial;
    private readonly foundationMaterial: THREE.MeshStandardMaterial;
    private readonly chimneyMaterial: THREE.MeshStandardMaterial;
    private readonly floorMaterial: THREE.MeshStandardMaterial;

    constructor() {
        this.group = new THREE.Group();

        // Initialize materials
        this.wallMaterial = new THREE.MeshStandardMaterial({ color: 0xf5deb3 }); // Wheat exterior
        this.interiorWallMaterial = new THREE.MeshStandardMaterial({ color: 0xfaf0e6 }); // Linen interior
        this.roofMaterial = new THREE.MeshStandardMaterial({ color: 0x8b0000 }); // Dark red
        this.doorMaterial = new THREE.MeshStandardMaterial({ color: 0x8b4513 }); // Saddle brown
        this.windowMaterial = new THREE.MeshStandardMaterial({ color: 0x87ceeb, transparent: true, opacity: 0.7 });
        this.frameMaterial = new THREE.MeshStandardMaterial({ color: 0xffffff });
        this.foundationMaterial = new THREE.MeshStandardMaterial({ color: 0x696969 });
        this.chimneyMaterial = new THREE.MeshStandardMaterial({ color: 0xa52a2a });
        this.floorMaterial = new THREE.MeshStandardMaterial({ color: 0xdeb887 }); // Burlywood
    }

    /**
     * Build the complete house.
     * 
     * Constructs all components in order:
     * - Foundation
     * - Exterior and interior walls
     * - Floors
     * - Doors and windows
     * - Roof and chimney
     * - Porch
     * 
     * @returns Complete house as a Three.js group
     */
    public build(): THREE.Group {
        this.addFoundation();
        this.addFirstFloorExteriorWalls();
        this.addFirstFloorInteriorWalls();
        this.addFirstFloorFloor();
        this.addSecondFloorExteriorWalls();
        this.addSecondFloorInteriorWalls();
        this.addFloorBetweenStories();
        this.addFrontDoor();
        this.addBackDoor();
        this.addInteriorDoors();
        this.addFirstFloorWindows();
        this.addSecondFloorWindows();
        this.addRoof();
        this.addChimney();
        this.addPorch();

        return this.group;
    }

    private addFoundation(): void {
        const geometry = new THREE.BoxGeometry(this.width + 60, this.foundationHeight, this.depth + 60);
        const foundation = new THREE.Mesh(geometry, this.foundationMaterial);
        foundation.position.y = this.foundationHeight / 2;
        foundation.castShadow = true;
        foundation.receiveShadow = true;
        this.group.add(foundation);
    }

    private addFirstFloorExteriorWalls(): void {
        const baseY = this.foundationHeight + this.floorHeight / 2;

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

    private addFirstFloorInteriorWalls(): void {
        const baseY = this.foundationHeight + this.floorHeight / 2;
        
        // First floor layout:
        // Left side: Living room (front) + Kitchen (back)
        // Right side: Entry/Hallway (front) + Dining + Bathroom (back)
        
        // Main vertical wall dividing left and right (with opening for living room)
        // This wall runs from back to about 2/3 forward, leaving entry area open
        const mainDividerLength = this.depth * 0.6;
        const mainDividerGeometry = new THREE.BoxGeometry(this.wallThickness, this.floorHeight, mainDividerLength);
        const mainDivider = new THREE.Mesh(mainDividerGeometry, this.interiorWallMaterial);
        mainDivider.position.set(0, baseY, -this.depth / 2 + mainDividerLength / 2 + this.wallThickness);
        mainDivider.castShadow = true;
        mainDivider.receiveShadow = true;
        this.group.add(mainDivider);

        // Horizontal wall separating kitchen from living room (left side)
        const kitchenWallLength = this.width / 2 - this.wallThickness;
        const kitchenWallGeometry = new THREE.BoxGeometry(kitchenWallLength, this.floorHeight, this.wallThickness);
        const kitchenWall = new THREE.Mesh(kitchenWallGeometry, this.interiorWallMaterial);
        kitchenWall.position.set(-this.width / 4 - this.wallThickness / 2, baseY, 0);
        kitchenWall.castShadow = true;
        kitchenWall.receiveShadow = true;
        this.group.add(kitchenWall);

        // Bathroom walls (back right corner) - creates a 250cm x 200cm bathroom
        const bathroomWidth = 250;
        const bathroomDepth = 200;
        
        // Bathroom front wall
        const bathroomFrontGeometry = new THREE.BoxGeometry(bathroomWidth, this.floorHeight, this.wallThickness);
        const bathroomFront = new THREE.Mesh(bathroomFrontGeometry, this.interiorWallMaterial);
        bathroomFront.position.set(this.width / 2 - bathroomWidth / 2, baseY, -this.depth / 2 + bathroomDepth);
        bathroomFront.castShadow = true;
        bathroomFront.receiveShadow = true;
        this.group.add(bathroomFront);

        // Bathroom left wall
        const bathroomLeftGeometry = new THREE.BoxGeometry(this.wallThickness, this.floorHeight, bathroomDepth);
        const bathroomLeft = new THREE.Mesh(bathroomLeftGeometry, this.interiorWallMaterial);
        bathroomLeft.position.set(this.width / 2 - bathroomWidth, baseY, -this.depth / 2 + bathroomDepth / 2);
        bathroomLeft.castShadow = true;
        bathroomLeft.receiveShadow = true;
        this.group.add(bathroomLeft);
    }

    private addFirstFloorFloor(): void {
        const floorY = this.foundationHeight + 1;
        
        // Main floor (living room + entry + dining)
        const mainFloorGeometry = new THREE.BoxGeometry(
            this.width - this.wallThickness * 2,
            2,
            this.depth - this.wallThickness * 2
        );
        const mainFloor = new THREE.Mesh(mainFloorGeometry, this.floorMaterial);
        mainFloor.position.y = floorY;
        mainFloor.receiveShadow = true;
        this.group.add(mainFloor);
    }

    private addSecondFloorExteriorWalls(): void {
        const secondFloorY = this.foundationHeight + this.floorHeight;
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

    private addSecondFloorInteriorWalls(): void {
        const secondFloorY = this.foundationHeight + this.floorHeight;
        const baseY = secondFloorY + this.floorHeight / 2;
        
        // Second floor layout:
        // Left front: Bedroom 1 (4m x 4m)
        // Left back: Bedroom 2 (4m x 4m)  
        // Right front: Master Bedroom (6m x 5m)
        // Right back: Master Bath (3m x 2.5m) + Hallway
        // Center: Hallway connecting all rooms
        
        // Central hallway wall (left side) - runs most of the depth
        const hallwayWidth = 120; // 1.2m wide hallway
        const hallwayWallLength = this.depth - 200; // Leave space at front
        const hallwayLeftGeometry = new THREE.BoxGeometry(this.wallThickness, this.floorHeight, hallwayWallLength);
        const hallwayLeft = new THREE.Mesh(hallwayLeftGeometry, this.interiorWallMaterial);
        hallwayLeft.position.set(-hallwayWidth / 2, baseY, -this.depth / 2 + hallwayWallLength / 2 + this.wallThickness);
        hallwayLeft.castShadow = true;
        hallwayLeft.receiveShadow = true;
        this.group.add(hallwayLeft);

        // Hallway right wall
        const hallwayRight = new THREE.Mesh(hallwayLeftGeometry, this.interiorWallMaterial);
        hallwayRight.position.set(hallwayWidth / 2, baseY, -this.depth / 2 + hallwayWallLength / 2 + this.wallThickness);
        hallwayRight.castShadow = true;
        hallwayRight.receiveShadow = true;
        this.group.add(hallwayRight);

        // Wall between bedroom 1 and bedroom 2 (horizontal, left side)
        const bedroomDividerLength = this.width / 2 - hallwayWidth / 2 - this.wallThickness;
        const bedroomDividerGeometry = new THREE.BoxGeometry(bedroomDividerLength, this.floorHeight, this.wallThickness);
        const bedroomDivider = new THREE.Mesh(bedroomDividerGeometry, this.interiorWallMaterial);
        bedroomDivider.position.set(-this.width / 4 - hallwayWidth / 4, baseY, 0);
        bedroomDivider.castShadow = true;
        bedroomDivider.receiveShadow = true;
        this.group.add(bedroomDivider);

        // Master bathroom walls (back right corner) - 300cm x 250cm
        const masterBathWidth = 300;
        const masterBathDepth = 250;
        
        // Master bath front wall
        const masterBathFrontGeometry = new THREE.BoxGeometry(masterBathWidth, this.floorHeight, this.wallThickness);
        const masterBathFront = new THREE.Mesh(masterBathFrontGeometry, this.interiorWallMaterial);
        masterBathFront.position.set(this.width / 2 - masterBathWidth / 2, baseY, -this.depth / 2 + masterBathDepth);
        masterBathFront.castShadow = true;
        masterBathFront.receiveShadow = true;
        this.group.add(masterBathFront);

        // Master bath left wall
        const masterBathLeftGeometry = new THREE.BoxGeometry(this.wallThickness, this.floorHeight, masterBathDepth);
        const masterBathLeft = new THREE.Mesh(masterBathLeftGeometry, this.interiorWallMaterial);
        masterBathLeft.position.set(this.width / 2 - masterBathWidth, baseY, -this.depth / 2 + masterBathDepth / 2);
        masterBathLeft.castShadow = true;
        masterBathLeft.receiveShadow = true;
        this.group.add(masterBathLeft);
    }

    private addFloorBetweenStories(): void {
        const secondFloorY = this.foundationHeight + this.floorHeight;
        const geometry = new THREE.BoxGeometry(
            this.width - this.wallThickness,
            20, // 20cm thick floor
            this.depth - this.wallThickness
        );
        const floor = new THREE.Mesh(geometry, this.floorMaterial);
        floor.position.y = secondFloorY;
        floor.castShadow = true;
        floor.receiveShadow = true;
        this.group.add(floor);
    }

    private addFrontDoor(): void {
        // Front door (90cm x 210cm)
        const doorGeometry = new THREE.BoxGeometry(this.doorWidth, this.doorHeight, this.wallThickness + 5);
        const door = new THREE.Mesh(doorGeometry, this.doorMaterial);
        door.position.set(200, this.foundationHeight + this.doorHeight / 2, this.depth / 2);
        door.castShadow = true;
        door.userData.occlusionType = 'door';
        door.userData.floorLevel = 1;
        this.group.add(door);

        // Door frame
        const frameGeometry = new THREE.BoxGeometry(this.doorWidth + 20, this.doorHeight + 20, this.wallThickness + 2);
        const doorFrame = new THREE.Mesh(frameGeometry, this.frameMaterial);
        doorFrame.position.set(200, this.foundationHeight + this.doorHeight / 2 + 5, this.depth / 2 - 2);
        doorFrame.userData.occlusionType = 'door';
        doorFrame.userData.floorLevel = 1;
        this.group.add(doorFrame);

        // Door handle
        const handleGeometry = new THREE.BoxGeometry(12, 12, 15);
        const handleMaterial = new THREE.MeshStandardMaterial({ color: 0xffd700 });
        const handle = new THREE.Mesh(handleGeometry, handleMaterial);
        handle.position.set(200 + 35, this.foundationHeight + this.doorHeight / 2, this.depth / 2 + 10);
        handle.userData.occlusionType = 'door';
        handle.userData.floorLevel = 1;
        this.group.add(handle);
    }

    private addBackDoor(): void {
        // Back door to kitchen (90cm x 210cm)
        const doorGeometry = new THREE.BoxGeometry(this.doorWidth, this.doorHeight, this.wallThickness + 5);
        const door = new THREE.Mesh(doorGeometry, this.doorMaterial);
        door.position.set(-300, this.foundationHeight + this.doorHeight / 2, -this.depth / 2);
        door.castShadow = true;
        door.userData.occlusionType = 'door';
        door.userData.floorLevel = 1;
        this.group.add(door);

        // Door frame
        const frameGeometry = new THREE.BoxGeometry(this.doorWidth + 20, this.doorHeight + 20, this.wallThickness + 2);
        const doorFrame = new THREE.Mesh(frameGeometry, this.frameMaterial);
        doorFrame.position.set(-300, this.foundationHeight + this.doorHeight / 2 + 5, -this.depth / 2 + 2);
        doorFrame.userData.occlusionType = 'door';
        doorFrame.userData.floorLevel = 1;
        this.group.add(doorFrame);
    }

    private addInteriorDoors(): void {
        // Interior doors (80cm x 200cm)
        const interiorDoorWidth = 80;
        const interiorDoorHeight = 200;
        const interiorDoorGeometry = new THREE.BoxGeometry(interiorDoorWidth, interiorDoorHeight, this.wallThickness + 2);
        const interiorFrameGeometry = new THREE.BoxGeometry(interiorDoorWidth + 15, interiorDoorHeight + 15, this.wallThickness);
        
        // Bathroom door (first floor)
        const bathroomDoorY = this.foundationHeight + interiorDoorHeight / 2;
        const bathroomDoor = new THREE.Mesh(interiorDoorGeometry, this.doorMaterial);
        bathroomDoor.position.set(this.width / 2 - 250 - 40, bathroomDoorY, -this.depth / 2 + 200);
        bathroomDoor.castShadow = true;
        bathroomDoor.userData.occlusionType = 'door';
        bathroomDoor.userData.floorLevel = 1;
        this.group.add(bathroomDoor);

        const bathroomFrame = new THREE.Mesh(interiorFrameGeometry, this.frameMaterial);
        bathroomFrame.position.set(this.width / 2 - 250 - 40, bathroomDoorY + 5, -this.depth / 2 + 200 - 2);
        bathroomFrame.userData.occlusionType = 'door';
        bathroomFrame.userData.floorLevel = 1;
        this.group.add(bathroomFrame);
    }

    private addFirstFloorWindows(): void {
        const windowGeometry = new THREE.BoxGeometry(this.windowWidth, this.windowHeight, this.wallThickness + 5);
        const windowFrameGeometry = new THREE.BoxGeometry(this.windowWidth + 20, this.windowHeight + 20, this.wallThickness + 2);
        const windowY = this.foundationHeight + 100 + this.windowHeight / 2; // 100cm from floor

        // Front wall windows (3 windows)
        const frontPositions = [-400, -100, 400];
        for (const xPos of frontPositions) {
            const frame = new THREE.Mesh(windowFrameGeometry, this.frameMaterial);
            frame.position.set(xPos, windowY, this.depth / 2 - 2);
            this.group.add(frame);

            const window = new THREE.Mesh(windowGeometry, this.windowMaterial);
            window.position.set(xPos, windowY, this.depth / 2);
            this.group.add(window);
        }

        // Back wall windows (3 windows)
        const backPositions = [-400, -100, 300];
        for (const xPos of backPositions) {
            const frame = new THREE.Mesh(windowFrameGeometry, this.frameMaterial);
            frame.position.set(xPos, windowY, -this.depth / 2 + 2);
            this.group.add(frame);

            const window = new THREE.Mesh(windowGeometry, this.windowMaterial);
            window.position.set(xPos, windowY, -this.depth / 2);
            this.group.add(window);
        }

        // Side windows
        const sideWindowGeometry = new THREE.BoxGeometry(this.wallThickness + 5, this.windowHeight, this.windowWidth);
        const sideWindowFrameGeometry = new THREE.BoxGeometry(this.wallThickness + 2, this.windowHeight + 20, this.windowWidth + 20);

        // Left side windows (2)
        const leftPositions = [-200, 200];
        for (const zPos of leftPositions) {
            const frame = new THREE.Mesh(sideWindowFrameGeometry, this.frameMaterial);
            frame.position.set(-this.width / 2 + 2, windowY, zPos);
            this.group.add(frame);

            const window = new THREE.Mesh(sideWindowGeometry, this.windowMaterial);
            window.position.set(-this.width / 2, windowY, zPos);
            this.group.add(window);
        }

        // Right side windows (2)
        for (const zPos of leftPositions) {
            const frame = new THREE.Mesh(sideWindowFrameGeometry, this.frameMaterial);
            frame.position.set(this.width / 2 - 2, windowY, zPos);
            this.group.add(frame);

            const window = new THREE.Mesh(sideWindowGeometry, this.windowMaterial);
            window.position.set(this.width / 2, windowY, zPos);
            this.group.add(window);
        }
    }

    private addSecondFloorWindows(): void {
        const secondFloorY = this.foundationHeight + this.floorHeight;
        const windowGeometry = new THREE.BoxGeometry(this.windowWidth, this.windowHeight, this.wallThickness + 5);
        const windowFrameGeometry = new THREE.BoxGeometry(this.windowWidth + 20, this.windowHeight + 20, this.wallThickness + 2);
        const windowY = secondFloorY + 100 + this.windowHeight / 2;

        // Front wall windows (4 windows for bedrooms)
        const frontPositions = [-450, -200, 150, 400];
        for (const xPos of frontPositions) {
            const frame = new THREE.Mesh(windowFrameGeometry, this.frameMaterial);
            frame.position.set(xPos, windowY, this.depth / 2 - 2);
            this.group.add(frame);

            const window = new THREE.Mesh(windowGeometry, this.windowMaterial);
            window.position.set(xPos, windowY, this.depth / 2);
            this.group.add(window);
        }

        // Back wall windows (3 windows)
        const backPositions = [-400, -100, 350];
        for (const xPos of backPositions) {
            const frame = new THREE.Mesh(windowFrameGeometry, this.frameMaterial);
            frame.position.set(xPos, windowY, -this.depth / 2 + 2);
            this.group.add(frame);

            const window = new THREE.Mesh(windowGeometry, this.windowMaterial);
            window.position.set(xPos, windowY, -this.depth / 2);
            this.group.add(window);
        }

        // Side windows
        const sideWindowGeometry = new THREE.BoxGeometry(this.wallThickness + 5, this.windowHeight, this.windowWidth);
        const sideWindowFrameGeometry = new THREE.BoxGeometry(this.wallThickness + 2, this.windowHeight + 20, this.windowWidth + 20);

        // Left side windows (2)
        const sidePositions = [-200, 200];
        for (const zPos of sidePositions) {
            const frame = new THREE.Mesh(sideWindowFrameGeometry, this.frameMaterial);
            frame.position.set(-this.width / 2 + 2, windowY, zPos);
            this.group.add(frame);

            const window = new THREE.Mesh(sideWindowGeometry, this.windowMaterial);
            window.position.set(-this.width / 2, windowY, zPos);
            this.group.add(window);
        }

        // Right side windows (2)
        for (const zPos of sidePositions) {
            const frame = new THREE.Mesh(sideWindowFrameGeometry, this.frameMaterial);
            frame.position.set(this.width / 2 - 2, windowY, zPos);
            this.group.add(frame);

            const window = new THREE.Mesh(sideWindowGeometry, this.windowMaterial);
            window.position.set(this.width / 2, windowY, zPos);
            this.group.add(window);
        }
    }

    private addRoof(): void {
        const secondFloorY = this.foundationHeight + this.floorHeight;
        const roofHeight = 300; // 3 meter tall roof peak
        const roofOverhang = 80; // 80cm overhang

        const roofLength = this.depth + roofOverhang * 2;
        const roofWidth = Math.sqrt(
            Math.pow(this.width / 2 + roofOverhang, 2) + Math.pow(roofHeight, 2)
        );

        const roofGeometry = new THREE.BoxGeometry(roofWidth, 25, roofLength);
        const roofAngle = Math.atan2(roofHeight, this.width / 2 + roofOverhang);

        // Left roof slope
        const roofLeft = new THREE.Mesh(roofGeometry, this.roofMaterial);
        roofLeft.rotation.z = roofAngle;
        roofLeft.position.set(
            -(this.width / 4 + roofOverhang / 2) * Math.cos(roofAngle),
            secondFloorY + this.floorHeight + roofHeight / 2 + 25,
            0
        );
        roofLeft.castShadow = true;
        roofLeft.receiveShadow = true;
        roofLeft.userData.occlusionType = 'roof';
        this.group.add(roofLeft);

        // Right roof slope
        const roofRight = new THREE.Mesh(roofGeometry, this.roofMaterial);
        roofRight.rotation.z = -roofAngle;
        roofRight.position.set(
            (this.width / 4 + roofOverhang / 2) * Math.cos(roofAngle),
            secondFloorY + this.floorHeight + roofHeight / 2 + 25,
            0
        );
        roofRight.castShadow = true;
        roofRight.receiveShadow = true;
        roofRight.userData.occlusionType = 'roof';
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
        gableFront.userData.occlusionType = 'roof';
        this.group.add(gableFront);

        const gableBack = new THREE.Mesh(gableGeometry, this.wallMaterial);
        gableBack.position.set(0, secondFloorY + this.floorHeight, -this.depth / 2 - this.wallThickness / 2);
        gableBack.castShadow = true;
        gableBack.receiveShadow = true;
        gableBack.userData.occlusionType = 'roof';
        this.group.add(gableBack);
    }

    private addChimney(): void {
        const secondFloorY = this.foundationHeight + this.floorHeight;
        const roofHeight = 300;

        // Chimney body (60cm x 60cm, realistic size)
        const chimneyGeometry = new THREE.BoxGeometry(60, 350, 60);
        const chimney = new THREE.Mesh(chimneyGeometry, this.chimneyMaterial);
        chimney.position.set(this.width / 4, secondFloorY + this.floorHeight + roofHeight, 0);
        chimney.castShadow = true;
        chimney.receiveShadow = true;
        this.group.add(chimney);

        // Chimney top cap
        const topGeometry = new THREE.BoxGeometry(80, 20, 80);
        const top = new THREE.Mesh(topGeometry, this.chimneyMaterial);
        top.position.set(this.width / 4, secondFloorY + this.floorHeight + roofHeight + 185, 0);
        top.castShadow = true;
        this.group.add(top);
    }

    private addPorch(): void {
        // Covered front porch (300cm x 150cm)
        const porchWidth = 300;
        const porchDepth = 150;
        
        // Porch floor/platform
        const porchFloorGeometry = new THREE.BoxGeometry(porchWidth, 20, porchDepth);
        const porchMaterial = new THREE.MeshStandardMaterial({ color: 0x808080 });
        const porchFloor = new THREE.Mesh(porchFloorGeometry, porchMaterial);
        porchFloor.position.set(200, this.foundationHeight - 10, this.depth / 2 + porchDepth / 2 + 10);
        porchFloor.castShadow = true;
        porchFloor.receiveShadow = true;
        this.group.add(porchFloor);

        // Porch steps (3 steps, each 15cm tall, 30cm deep)
        const stepWidth = porchWidth;
        for (let i = 0; i < 3; i++) {
            const stepGeometry = new THREE.BoxGeometry(stepWidth, 15, 30);
            const step = new THREE.Mesh(stepGeometry, porchMaterial);
            step.position.set(
                200, 
                7.5 + i * 15, 
                this.depth / 2 + porchDepth + 25 + (2 - i) * 30
            );
            step.castShadow = true;
            step.receiveShadow = true;
            this.group.add(step);
        }

        // Porch columns (4 columns)
        const columnGeometry = new THREE.BoxGeometry(20, 250, 20);
        const columnMaterial = new THREE.MeshStandardMaterial({ color: 0xffffff });
        const columnPositions = [
            { x: 200 - porchWidth / 2 + 30, z: this.depth / 2 + porchDepth - 20 },
            { x: 200 + porchWidth / 2 - 30, z: this.depth / 2 + porchDepth - 20 },
        ];

        for (const pos of columnPositions) {
            const column = new THREE.Mesh(columnGeometry, columnMaterial);
            column.position.set(pos.x, this.foundationHeight + 125, pos.z);
            column.castShadow = true;
            column.receiveShadow = true;
            this.group.add(column);
        }

        // Porch roof
        const porchRoofGeometry = new THREE.BoxGeometry(porchWidth + 40, 15, porchDepth + 20);
        const porchRoof = new THREE.Mesh(porchRoofGeometry, this.roofMaterial);
        porchRoof.position.set(200, this.foundationHeight + 260, this.depth / 2 + porchDepth / 2);
        porchRoof.castShadow = true;
        porchRoof.receiveShadow = true;
        porchRoof.userData.occlusionType = 'roof';
        this.group.add(porchRoof);
    }
}


