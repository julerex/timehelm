import * as THREE from 'three';

export class DayNightCycle {
    private readonly scene: THREE.Scene;
    private readonly ambientLight: THREE.AmbientLight;
    private readonly sunLight: THREE.DirectionalLight;
    private readonly sunMesh: THREE.Mesh;
    private readonly moonMesh: THREE.Mesh;

    // Configuration
    private readonly sunRadius: number = 5000;
    private readonly dayColor = new THREE.Color(0x87ceeb);
    private readonly sunsetColor = new THREE.Color(0xff4500);
    private readonly nightColor = new THREE.Color(0x000022);

    constructor(scene: THREE.Scene) {
        this.scene = scene;

        // Ambient light
        this.ambientLight = new THREE.AmbientLight(0xffffff, 0.4);
        scene.add(this.ambientLight);

        // Sun directional light
        this.sunLight = this.createSunLight();
        scene.add(this.sunLight);

        // Sun mesh
        this.sunMesh = this.createSunMesh();
        scene.add(this.sunMesh);

        // Moon mesh
        this.moonMesh = this.createMoonMesh();
        scene.add(this.moonMesh);
    }

    // --- Public Methods ---

    public update(): void {
        const gameTime = this.calculateGameTime();

        this.updateTimeDisplay(gameTime);
        this.updateCelestialBodies(gameTime);
        this.updateLighting(gameTime);
        this.updateSkyColor(gameTime);
    }

    public getGameTime(): number {
        return this.calculateGameTime();
    }

    // --- Private Methods ---

    private createSunLight(): THREE.DirectionalLight {
        const light = new THREE.DirectionalLight(0xffffff, 1.0);
        light.position.set(0, 10000, 0);
        light.castShadow = true;

        // Shadow camera setup
        light.shadow.camera.left = -5000;
        light.shadow.camera.right = 5000;
        light.shadow.camera.top = 5000;
        light.shadow.camera.bottom = -5000;
        light.shadow.mapSize.width = 2048;
        light.shadow.mapSize.height = 2048;

        return light;
    }

    private createSunMesh(): THREE.Mesh {
        const geometry = new THREE.SphereGeometry(200, 32, 32);
        const material = new THREE.MeshBasicMaterial({ color: 0xffff00 });
        return new THREE.Mesh(geometry, material);
    }

    private createMoonMesh(): THREE.Mesh {
        const geometry = new THREE.SphereGeometry(150, 32, 32);
        const material = new THREE.MeshBasicMaterial({ color: 0xcccccc });
        return new THREE.Mesh(geometry, material);
    }

    private calculateGameTime(): number {
        // 24 hours game time = 24 minutes real time
        // 1 hour game time = 1 minute real time
        const now = new Date();
        const seconds = now.getSeconds();
        const ms = now.getMilliseconds();

        // This will give us 0-23.999... cycle every 24 minutes
        const totalRealMinutes = now.getHours() * 60 + now.getMinutes();
        return (totalRealMinutes % 24) + seconds / 60 + ms / 60000;
    }

    private updateTimeDisplay(gameTime: number): void {
        const displayHours = Math.floor(gameTime);
        const displayMinutes = Math.floor((gameTime % 1) * 60);
        const timeString = `${displayHours.toString().padStart(2, '0')}:${displayMinutes.toString().padStart(2, '0')}`;

        const timeDisplay = document.getElementById('game-time-display');
        if (timeDisplay) {
            timeDisplay.textContent = timeString;
        }
    }

    private updateCelestialBodies(gameTime: number): void {
        // Calculate sun position (rotation around X axis)
        // 0h: midnight, 6h: sunrise, 12h: noon, 18h: sunset
        const angle = (gameTime / 24) * Math.PI * 2 - Math.PI / 2;

        const sunX = 0;
        const sunY = Math.sin(angle) * this.sunRadius;
        const sunZ = Math.cos(angle) * this.sunRadius;

        this.sunMesh.position.set(sunX, sunY, sunZ);
        this.sunLight.position.set(sunX, sunY, sunZ);

        // Moon is opposite to sun
        this.moonMesh.position.set(-sunX, -sunY, -sunZ);
    }

    private updateLighting(gameTime: number): void {
        const angle = (gameTime / 24) * Math.PI * 2 - Math.PI / 2;
        const sunY = Math.sin(angle) * this.sunRadius;
        const sunUp = sunY > 0;
        const intensity = sunUp ? Math.max(0, Math.sin(angle)) : 0;

        this.sunLight.intensity = intensity;
        this.ambientLight.intensity = 0.1 + intensity * 0.3;
    }

    private updateSkyColor(gameTime: number): void {
        const angle = (gameTime / 24) * Math.PI * 2 - Math.PI / 2;
        const sunY = Math.sin(angle) * this.sunRadius;
        const sunUp = sunY > 0;
        const intensity = sunUp ? Math.max(0, Math.sin(angle)) : 0;

        if (sunUp) {
            const skyColor = this.dayColor.clone().lerp(this.sunsetColor, 1 - intensity);
            this.scene.background = skyColor;
        } else {
            this.scene.background = this.nightColor.clone();
        }
    }
}
