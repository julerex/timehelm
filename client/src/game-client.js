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
        const ambientLight = new THREE.AmbientLight(0xffffff, 0.6);
        this.scene.add(ambientLight);

        const directionalLight = new THREE.DirectionalLight(0xffffff, 0.8);
        directionalLight.position.set(10, 10, 5);
        directionalLight.castShadow = true;
        this.scene.add(directionalLight);

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

        if (this.keys['w'] || this.keys['arrowup']) {
            this.myPlayer.mesh.translateZ(-this.moveSpeed);
            moved = true;
        }
        if (this.keys['s'] || this.keys['arrowdown']) {
            this.myPlayer.mesh.translateZ(this.moveSpeed);
            moved = true;
        }
        if (this.keys['a'] || this.keys['arrowleft']) {
            newRotation += this.rotationSpeed;
            moved = true;
        }
        if (this.keys['d'] || this.keys['arrowright']) {
            newRotation -= this.rotationSpeed;
            moved = true;
        }

        if (moved) {
            this.myPlayer.rotation = newRotation;
            this.myPlayer.mesh.rotation.y = newRotation;

            // Update camera to follow player
            const offset = new THREE.Vector3(0, 5, 10);
            offset.applyAxisAngle(new THREE.Vector3(0, 1, 0), newRotation);
            this.camera.position.set(
                this.myPlayer.mesh.position.x + offset.x,
                this.myPlayer.mesh.position.y + offset.y,
                this.myPlayer.mesh.position.z + offset.z
            );
            this.camera.lookAt(this.myPlayer.mesh.position);

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
        this.updateMovement();
        this.renderer.render(this.scene, this.camera);
    }
}

