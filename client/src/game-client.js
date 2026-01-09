import * as THREE from 'three';

export class GameClient {
    constructor(user) {
        this.user = user;
        this.scene = null;
        this.camera = null;
        this.renderer = null;
        this.ws = null;
        this.players = new Map();
        this.myPlayer = null;
        this.keys = {};
        this.moveSpeed = 10;
        this.rotationSpeed = 0.02;

        // Camera rotation state
        this.cameraRotation = { theta: 0, phi: 0.5 }; // theta: horizontal, phi: vertical
        this.cameraDistance = 1000;
        this.cameraAnchored = true;
        this.cameraFocusPoint = null;
        this.isRightMouseDown = false;
        this.lastMousePos = { x: 0, y: 0 };
        
        // Random movement state
        this.randomMoveTimer = 0;
        this.randomMoveAction = 'idle'; // 'idle', 'forward', 'left', 'right'
    }

    init() {
        this.setupScene();
        this.setupControls();
        this.connectWebSocket();
        this.animate();
    }

    setupScene() {
        // Scene
        this.scene = new THREE.Scene();
        this.scene.background = new THREE.Color(0x87CEEB); // Sky blue

        // Camera
        this.camera = new THREE.PerspectiveCamera(
            75,
            window.innerWidth / window.innerHeight,
            10,
            100000
        );
        this.camera.position.set(0, 500, 1000);
        this.camera.lookAt(0, 0, 0);

        // Renderer
        this.renderer = new THREE.WebGLRenderer({ antialias: true });
        this.renderer.setSize(window.innerWidth, window.innerHeight);
        this.renderer.shadowMap.enabled = true;
        document.getElementById('game-container').appendChild(this.renderer.domElement);

        // Lighting
        this.ambientLight = new THREE.AmbientLight(0xffffff, 0.4);
        this.scene.add(this.ambientLight);

        this.sunLight = new THREE.DirectionalLight(0xffffff, 1.0);
        this.sunLight.position.set(0, 10000, 0);
        this.sunLight.castShadow = true;
        
        // Optimize shadows for sun
        this.sunLight.shadow.camera.left = -5000;
        this.sunLight.shadow.camera.right = 5000;
        this.sunLight.shadow.camera.top = 5000;
        this.sunLight.shadow.camera.bottom = -5000;
        this.sunLight.shadow.mapSize.width = 2048;
        this.sunLight.shadow.mapSize.height = 2048;
        
        this.scene.add(this.sunLight);

        // Sun Mesh
        const sunGeometry = new THREE.SphereGeometry(200, 32, 32);
        const sunMaterial = new THREE.MeshBasicMaterial({ color: 0xffff00 });
        this.sunMesh = new THREE.Mesh(sunGeometry, sunMaterial);
        this.scene.add(this.sunMesh);

        // Moon Mesh
        const moonGeometry = new THREE.SphereGeometry(150, 32, 32);
        const moonMaterial = new THREE.MeshBasicMaterial({ color: 0xcccccc });
        this.moonMesh = new THREE.Mesh(moonGeometry, moonMaterial);
        this.scene.add(this.moonMesh);

        // Ground
        const groundGeometry = new THREE.PlaneGeometry(10000, 10000);
        const groundMaterial = new THREE.MeshStandardMaterial({ color: 0x90EE90 });
        const ground = new THREE.Mesh(groundGeometry, groundMaterial);
        ground.rotation.x = -Math.PI / 2;
        ground.receiveShadow = true;
        this.scene.add(ground);

        // Create my player
        this.myPlayer = this.createPlayer(this.user.id, this.user.username);
        this.scene.add(this.myPlayer.mesh);
        this.players.set(this.user.id, this.myPlayer);

        // Camera follows player
        this.camera.position.set(
            this.myPlayer.mesh.position.x,
            this.myPlayer.mesh.position.y + 500,
            this.myPlayer.mesh.position.z + 1000
        );

        // Add a tree
        const tree = this.createTree(500, -500);
        this.scene.add(tree);

        // Handle window resize
        window.addEventListener('resize', () => {
            this.camera.aspect = window.innerWidth / window.innerHeight;
            this.camera.updateProjectionMatrix();
            this.renderer.setSize(window.innerWidth, window.innerHeight);
        });
    }

    createPlayer(id, username) {
        // ...
    }

    createTree(x, z) {
        const group = new THREE.Group();

        // Trunk (brown)
        const trunkGeometry = new THREE.BoxGeometry(40, 150, 40);
        const trunkMaterial = new THREE.MeshStandardMaterial({ color: 0x8B4513 });
        const trunk = new THREE.Mesh(trunkGeometry, trunkMaterial);
        trunk.position.y = 75; // Half of height to stand on ground
        trunk.castShadow = true;
        trunk.receiveShadow = true;
        group.add(trunk);

        // Foliage (green)
        const leavesGeometry = new THREE.BoxGeometry(200, 250, 200);
        const leavesMaterial = new THREE.MeshStandardMaterial({ color: 0x228B22 });
        const leaves = new THREE.Mesh(leavesGeometry, leavesMaterial);
        leaves.position.y = 150 + 125; // trunk height + half of leaves height
        leaves.castShadow = true;
        leaves.receiveShadow = true;
        group.add(leaves);

        group.position.set(x, 0, z);
        return group;
    }

    setupControls() {
        document.addEventListener('keydown', (e) => {
            const key = e.key.toLowerCase();
            this.keys[key] = true;

            // Toggle camera anchor
            if (key === 'c') {
                this.cameraAnchored = !this.cameraAnchored;
                console.log('Camera anchored:', this.cameraAnchored);
                const cameraStatus = document.getElementById('camera-status');
                if (cameraStatus) {
                    cameraStatus.textContent = `Camera: ${this.cameraAnchored ? 'Anchored' : 'Free'} (Press 'C' to toggle)`;
                }
            }
        });

        document.addEventListener('keyup', (e) => {
            this.keys[e.key.toLowerCase()] = false;
        });

        // Mouse controls for camera
        document.addEventListener('mousedown', (e) => {
            if (e.button === 2) { // Right click
                this.isRightMouseDown = true;
                this.lastMousePos = { x: e.clientX, y: e.clientY };
            }
        });

        document.addEventListener('mousemove', (e) => {
            if (this.isRightMouseDown) {
                const deltaX = e.clientX - this.lastMousePos.x;
                const deltaY = e.clientY - this.lastMousePos.y;

                this.cameraRotation.theta -= deltaX * 0.01;
                this.cameraRotation.phi += deltaY * 0.01;

                // Clamp vertical rotation
                this.cameraRotation.phi = Math.max(0.1, Math.min(Math.PI / 2 - 0.1, this.cameraRotation.phi));

                this.lastMousePos = { x: e.clientX, y: e.clientY };
            }
        });

        document.addEventListener('mouseup', (e) => {
            if (e.button === 2) {
                this.isRightMouseDown = false;
            }
        });

        // Zoom control
        document.addEventListener('wheel', (e) => {
            this.cameraDistance += e.deltaY;
            this.cameraDistance = Math.max(200, Math.min(5000, this.cameraDistance));
        });

        // Prevent context menu on right click
        document.addEventListener('contextmenu', (e) => {
            e.preventDefault();
        });
    }

    connectWebSocket() {
        const wsProtocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        const wsUrl = `${wsProtocol}//${window.location.host}/ws`;
        
        this.ws = new WebSocket(wsUrl);

        this.ws.onopen = () => {
            console.log('WebSocket connected');
            // Send join message
            this.sendMessage({
                type: 'Join',
                player: {
                    id: this.user.id,
                    username: this.user.username,
                    position: {
                        x: this.myPlayer.mesh.position.x,
                        y: this.myPlayer.mesh.position.y,
                        z: this.myPlayer.mesh.position.z,
                    },
                    rotation: this.myPlayer.rotation,
                },
            });
        };

        this.ws.onmessage = (event) => {
            try {
                const message = JSON.parse(event.data);
                this.handleMessage(message);
            } catch (error) {
                console.error('Failed to parse message:', error);
            }
        };

        this.ws.onerror = (error) => {
            console.error('WebSocket error:', error);
        };

        this.ws.onclose = () => {
            console.log('WebSocket disconnected');
            setTimeout(() => this.connectWebSocket(), 3000);
        };
    }

    handleMessage(message) {
        switch (message.type) {
            case 'WorldState':
                this.handleWorldState(message.players);
                break;
            case 'Move':
                this.handlePlayerMove(message.player_id, message.position, message.rotation, message.is_moving);
                break;
            case 'Join':
                if (message.player.id !== this.user.id) {
                    this.addPlayer(message.player);
                }
                break;
            case 'Leave':
                this.removePlayer(message.player_id);
                break;
        }
    }

    handleWorldState(players) {
        players.forEach((playerData) => {
            if (playerData.id !== this.user.id) {
                if (!this.players.has(playerData.id)) {
                    this.addPlayer(playerData);
                } else {
                    const player = this.players.get(playerData.id);
                    player.mesh.position.set(
                        playerData.position.x,
                        playerData.position.y,
                        playerData.position.z
                    );
                    player.mesh.rotation.y = playerData.rotation;
                    player.isMoving = playerData.is_moving || false;
                }
            }
        });
        this.updatePlayersList();
    }

    handlePlayerMove(playerId, position, rotation, isMoving) {
        if (playerId === this.user.id) return;

        const player = this.players.get(playerId);
        if (player) {
            player.mesh.position.set(position.x, position.y, position.z);
            player.mesh.rotation.y = rotation;
            player.isMoving = isMoving || false;
        }
    }

    addPlayer(playerData) {
        const player = this.createPlayer(playerData.id, playerData.username);
        player.mesh.position.set(
            playerData.position.x,
            playerData.position.y,
            playerData.position.z
        );
        player.mesh.rotation.y = playerData.rotation;
        player.isMoving = playerData.is_moving || false;
        this.scene.add(player.mesh);
        this.players.set(playerData.id, player);
        this.updatePlayersList();
    }

    removePlayer(playerId) {
        const player = this.players.get(playerId);
        if (player) {
            this.scene.remove(player.mesh);
            this.players.delete(playerId);
            this.updatePlayersList();
        }
    }

    updatePlayersList() {
        const list = document.getElementById('players-list');
        list.innerHTML = '';
        this.players.forEach((player) => {
            const li = document.createElement('li');
            li.textContent = player.username;
            list.appendChild(li);
        });
    }

    sendMessage(message) {
        if (this.ws && this.ws.readyState === WebSocket.OPEN) {
            this.ws.send(JSON.stringify(message));
        }
    }

    updateMovement() {
        if (!this.myPlayer) return;

        let moved = false;
        let newRotation = this.myPlayer.rotation;

        // Random movement logic
        this.randomMoveTimer -= 1;
        if (this.randomMoveTimer <= 0) {
            // Pick a new random action
            const actions = ['idle', 'forward', 'left', 'right'];
            this.randomMoveAction = actions[Math.floor(Math.random() * actions.length)];
            this.randomMoveTimer = Math.floor(Math.random() * 60) + 30; // 0.5 to 1.5 seconds at 60fps
        }

        if (this.randomMoveAction === 'forward') {
            this.myPlayer.mesh.translateZ(-this.moveSpeed);
            moved = true;
        } else if (this.randomMoveAction === 'left') {
            newRotation += this.rotationSpeed;
            moved = true;
        } else if (this.randomMoveAction === 'right') {
            newRotation -= this.rotationSpeed;
            moved = true;
        }

        this.myPlayer.isMoving = moved;

        if (moved) {
            this.myPlayer.rotation = newRotation;
            this.myPlayer.mesh.rotation.y = newRotation;

            // Send movement update
            this.sendMessage({
                type: 'Move',
                player_id: this.user.id,
                position: {
                    x: this.myPlayer.mesh.position.x,
                    y: this.myPlayer.mesh.position.y,
                    z: this.myPlayer.mesh.position.z,
                },
                rotation: this.myPlayer.rotation,
                is_moving: this.myPlayer.isMoving
            });
        }
    }

    updateAnimations() {
        this.players.forEach((player) => {
            // For other players, detect movement by position change
            if (player.id !== this.user.id) {
                const pos = player.mesh.position;
                if (player.lastPosition) {
                    const dist = Math.sqrt(
                        Math.pow(pos.x - player.lastPosition.x, 2) + 
                        Math.pow(pos.z - player.lastPosition.z, 2)
                    );
                    player.isMoving = dist > 0.1;
                }
                player.lastPosition = { x: pos.x, y: pos.y, z: pos.z };
            }

            if (player.isMoving) {
                player.walkCycle += 0.15;
                const swing = Math.sin(player.walkCycle) * 0.6;
                
                // Animate legs
                if (player.leftLeg) player.leftLeg.rotation.x = swing;
                if (player.rightLeg) player.rightLeg.rotation.x = -swing;
                
                // Animate arms (opposite to legs)
                if (player.leftArm) player.leftArm.rotation.x = -swing;
                if (player.rightArm) player.rightArm.rotation.x = swing;
                
                // Slight body bob
                const torso = player.mesh.children[0];
                if (torso) torso.position.y = 110 + Math.abs(Math.cos(player.walkCycle)) * 5;
            } else {
                // Reset to idle pose
                player.walkCycle = 0;
                if (player.leftLeg) player.leftLeg.rotation.x = THREE.MathUtils.lerp(player.leftLeg.rotation.x, 0, 0.2);
                if (player.rightLeg) player.rightLeg.rotation.x = THREE.MathUtils.lerp(player.rightLeg.rotation.x, 0, 0.2);
                if (player.leftArm) player.leftArm.rotation.x = THREE.MathUtils.lerp(player.leftArm.rotation.x, 0, 0.2);
                if (player.rightArm) player.rightArm.rotation.x = THREE.MathUtils.lerp(player.rightArm.rotation.x, 0, 0.2);
                
                const torso = player.mesh.children[0];
                if (torso) torso.position.y = THREE.MathUtils.lerp(torso.position.y, 110, 0.2);
            }
        });
    }

    animate() {
        requestAnimationFrame(() => this.animate());
        this.updateTime();
        this.updateMovement();
        this.updateAnimations();
        this.updateCamera();
        this.renderer.render(this.scene, this.camera);
    }

    updateCamera() {
        if (!this.myPlayer) return;

        // Handle arrow keys for horizontal camera rotation
        if (this.keys['arrowleft']) {
            this.cameraRotation.theta += 0.02;
        }
        if (this.keys['arrowright']) {
            this.cameraRotation.theta -= 0.02;
        }

        let targetX, targetY, targetZ;
        let baseRotation = 0;

        if (this.cameraAnchored) {
            targetX = this.myPlayer.mesh.position.x;
            targetY = this.myPlayer.mesh.position.y;
            targetZ = this.myPlayer.mesh.position.z;
            baseRotation = this.myPlayer.rotation;
            this.cameraFocusPoint = null; // Reset when anchored
        } else {
            if (!this.cameraFocusPoint) {
                this.cameraFocusPoint = new THREE.Vector3(
                    this.myPlayer.mesh.position.x,
                    this.myPlayer.mesh.position.y,
                    this.myPlayer.mesh.position.z
                );
            }

            // Move focus point with WASD when unanchored
            const moveSpeed = 50;
            if (this.keys['w']) {
                this.cameraFocusPoint.z -= Math.cos(this.cameraRotation.theta) * moveSpeed;
                this.cameraFocusPoint.x -= Math.sin(this.cameraRotation.theta) * moveSpeed;
            }
            if (this.keys['s']) {
                this.cameraFocusPoint.z += Math.cos(this.cameraRotation.theta) * moveSpeed;
                this.cameraFocusPoint.x += Math.sin(this.cameraRotation.theta) * moveSpeed;
            }
            if (this.keys['a']) {
                this.cameraFocusPoint.x -= Math.cos(this.cameraRotation.theta) * moveSpeed;
                this.cameraFocusPoint.z += Math.sin(this.cameraRotation.theta) * moveSpeed;
            }
            if (this.keys['d']) {
                this.cameraFocusPoint.x += Math.cos(this.cameraRotation.theta) * moveSpeed;
                this.cameraFocusPoint.z -= Math.sin(this.cameraRotation.theta) * moveSpeed;
            }

            targetX = this.cameraFocusPoint.x;
            targetY = this.cameraFocusPoint.y;
            targetZ = this.cameraFocusPoint.z;
            baseRotation = 0; // Don't use player rotation when unanchored
        }

        const relativeTheta = baseRotation + this.cameraRotation.theta;
        
        const x = Math.sin(relativeTheta) * Math.cos(this.cameraRotation.phi) * this.cameraDistance;
        const y = Math.sin(this.cameraRotation.phi) * this.cameraDistance;
        const z = Math.cos(relativeTheta) * Math.cos(this.cameraRotation.phi) * this.cameraDistance;

        this.camera.position.set(
            targetX + x,
            targetY + y,
            targetZ + z
        );
        
        // Look at the target point
        const target = new THREE.Vector3(
            targetX,
            targetY + 150,
            targetZ
        );
        this.camera.lookAt(target);
    }

    updateTime() {
        // 24 hours game time = 24 minutes real time
        // 1 hour game time = 1 minute real time
        const now = new Date();
        const minutes = now.getMinutes();
        const seconds = now.getSeconds();
        const ms = now.getMilliseconds();
        
        // This will give us 0-23.999... cycle every 24 minutes
        const totalRealMinutes = now.getHours() * 60 + now.getMinutes();
        const gameTime = (totalRealMinutes % 24) + (seconds / 60) + (ms / 60000);
        this.gameTime = gameTime;

        // Update time display
        const displayHours = Math.floor(gameTime);
        const displayMinutes = Math.floor((gameTime % 1) * 60);
        const timeString = `${displayHours.toString().padStart(2, '0')}:${displayMinutes.toString().padStart(2, '0')}`;
        const timeDisplay = document.getElementById('game-time-display');
        if (timeDisplay) {
            timeDisplay.textContent = timeString;
        }

        // Calculate sun position (rotation around X axis)
        // 0h: midnight, 6h: sunrise, 12h: noon, 18h: sunset
        const angle = (gameTime / 24) * Math.PI * 2 - Math.PI / 2;
        const radius = 5000;
        
        const sunX = 0;
        const sunY = Math.sin(angle) * radius;
        const sunZ = Math.cos(angle) * radius;

        this.sunMesh.position.set(sunX, sunY, sunZ);
        this.sunLight.position.set(sunX, sunY, sunZ);

        // Update moon position (opposite to sun)
        this.moonMesh.position.set(-sunX, -sunY, -sunZ);

        // Adjust intensity and background color
        const sunUp = sunY > 0;
        const intensity = sunUp ? Math.max(0, Math.sin(angle)) : 0;
        
        this.sunLight.intensity = intensity;
        this.ambientLight.intensity = 0.1 + intensity * 0.3;

        // Sky color
        if (sunUp) {
            // Day sky: light blue
            const dayColor = new THREE.Color(0x87CEEB);
            const sunsetColor = new THREE.Color(0xFF4500);
            const skyColor = dayColor.clone().lerp(sunsetColor, 1 - intensity);
            this.scene.background = skyColor;
        } else {
            // Night sky: dark blue/black
            const nightColor = new THREE.Color(0x000022);
            this.scene.background = nightColor;
        }
    }
}

