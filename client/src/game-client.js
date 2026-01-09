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
        this.moveSpeed = 0.1;
        this.rotationSpeed = 0.02;

        // Camera rotation state
        this.cameraRotation = { theta: 0, phi: 0.5 }; // theta: horizontal, phi: vertical
        this.cameraDistance = 10;
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
            0.1,
            1000
        );
        this.camera.position.set(0, 5, 10);
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
        this.sunLight.position.set(0, 100, 0);
        this.sunLight.castShadow = true;
        
        // Optimize shadows for sun
        this.sunLight.shadow.camera.left = -50;
        this.sunLight.shadow.camera.right = 50;
        this.sunLight.shadow.camera.top = 50;
        this.sunLight.shadow.camera.bottom = -50;
        this.sunLight.shadow.mapSize.width = 2048;
        this.sunLight.shadow.mapSize.height = 2048;
        
        this.scene.add(this.sunLight);

        // Sun Mesh
        const sunGeometry = new THREE.SphereGeometry(2, 32, 32);
        const sunMaterial = new THREE.MeshBasicMaterial({ color: 0xffff00 });
        this.sunMesh = new THREE.Mesh(sunGeometry, sunMaterial);
        this.scene.add(this.sunMesh);

        // Moon Mesh
        const moonGeometry = new THREE.SphereGeometry(1.5, 32, 32);
        const moonMaterial = new THREE.MeshBasicMaterial({ color: 0xcccccc });
        this.moonMesh = new THREE.Mesh(moonGeometry, moonMaterial);
        this.scene.add(this.moonMesh);

        // Ground
        const groundGeometry = new THREE.PlaneGeometry(100, 100);
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
            this.myPlayer.mesh.position.y + 5,
            this.myPlayer.mesh.position.z + 10
        );

        // Handle window resize
        window.addEventListener('resize', () => {
            this.camera.aspect = window.innerWidth / window.innerHeight;
            this.camera.updateProjectionMatrix();
            this.renderer.setSize(window.innerWidth, window.innerHeight);
        });
    }

    createPlayer(id, username) {
        // Simple low-poly character
        const group = new THREE.Group();

        // Body (cube)
        const bodyGeometry = new THREE.BoxGeometry(0.8, 1.2, 0.6);
        const bodyMaterial = new THREE.MeshStandardMaterial({ color: 0x4a90e2 });
        const body = new THREE.Mesh(bodyGeometry, bodyMaterial);
        body.position.y = 0.6;
        body.castShadow = true;
        group.add(body);

        // Head (cube)
        const headGeometry = new THREE.BoxGeometry(0.6, 0.6, 0.6);
        const headMaterial = new THREE.MeshStandardMaterial({ color: 0xffdbac });
        const head = new THREE.Mesh(headGeometry, headMaterial);
        head.position.y = 1.5;
        head.castShadow = true;
        group.add(head);

        // Name label
        const canvas = document.createElement('canvas');
        const context = canvas.getContext('2d');
        canvas.width = 256;
        canvas.height = 64;
        context.fillStyle = 'rgba(0, 0, 0, 0.8)';
        context.fillRect(0, 0, canvas.width, canvas.height);
        context.fillStyle = 'white';
        context.font = '24px Arial';
        context.textAlign = 'center';
        context.fillText(username, canvas.width / 2, canvas.height / 2 + 8);

        const texture = new THREE.CanvasTexture(canvas);
        const spriteMaterial = new THREE.SpriteMaterial({ map: texture });
        const sprite = new THREE.Sprite(spriteMaterial);
        sprite.position.y = 2.5;
        sprite.scale.set(2, 0.5, 1);
        group.add(sprite);

        return {
            id,
            username,
            mesh: group,
            position: { x: 0, y: 0, z: 0 },
            rotation: 0,
        };
    }

    setupControls() {
        document.addEventListener('keydown', (e) => {
            this.keys[e.key.toLowerCase()] = true;
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
            this.cameraDistance += e.deltaY * 0.01;
            this.cameraDistance = Math.max(2, Math.min(50, this.cameraDistance));
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
                this.handlePlayerMove(message.player_id, message.position, message.rotation);
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
                }
            }
        });
        this.updatePlayersList();
    }

    handlePlayerMove(playerId, position, rotation) {
        if (playerId === this.user.id) return;

        const player = this.players.get(playerId);
        if (player) {
            player.mesh.position.set(position.x, position.y, position.z);
            player.mesh.rotation.y = rotation;
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
            });
        }
    }

    animate() {
        requestAnimationFrame(() => this.animate());
        this.updateTime();
        this.updateMovement();
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

        // Calculate camera position relative to player
        // theta: horizontal rotation, phi: vertical rotation
        const relativeTheta = this.myPlayer.rotation + this.cameraRotation.theta;
        
        const x = Math.sin(relativeTheta) * Math.cos(this.cameraRotation.phi) * this.cameraDistance;
        const y = Math.sin(this.cameraRotation.phi) * this.cameraDistance;
        const z = Math.cos(relativeTheta) * Math.cos(this.cameraRotation.phi) * this.cameraDistance;

        this.camera.position.set(
            this.myPlayer.mesh.position.x + x,
            this.myPlayer.mesh.position.y + y,
            this.myPlayer.mesh.position.z + z
        );
        
        // Look slightly above the player's feet (at the head area)
        const target = new THREE.Vector3(
            this.myPlayer.mesh.position.x,
            this.myPlayer.mesh.position.y + 1.5,
            this.myPlayer.mesh.position.z
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
        const radius = 50;
        
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

