import * as THREE from 'three';

// Custom calendar constants
// 360 days per year, 12 months of 30 days each, 24 hours per day
const MINUTES_PER_HOUR = 60;
const HOURS_PER_DAY = 24;
const MINUTES_PER_DAY = HOURS_PER_DAY * MINUTES_PER_HOUR; // 1440
const DAYS_PER_MONTH = 30;
const MONTHS_PER_YEAR = 12;
const DAYS_PER_YEAR = DAYS_PER_MONTH * MONTHS_PER_YEAR; // 360
const MINUTES_PER_YEAR = DAYS_PER_YEAR * MINUTES_PER_DAY; // 518,400

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

    // Server-synced time tracking
    // Game time = Unix seconds (1 real second = 1 game minute)
    private syncedGameTimeMinutes: number = 0;
    private syncRealTime: number = Date.now();
    private hasSynced: boolean = false;

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
        const totalMinutes = this.calculateTotalGameMinutes();
        const timeOfDayHours = this.getTimeOfDayHours(totalMinutes);

        this.updateTimeDisplay(totalMinutes);
        this.updateCelestialBodies(timeOfDayHours);
        this.updateLighting(timeOfDayHours);
        this.updateSkyColor(timeOfDayHours);
    }

    public getGameTime(): number {
        return this.calculateTotalGameMinutes();
    }

    /**
     * Sync game time from server. Game time is total minutes elapsed (Unix seconds).
     * After sync, time advances at 1 game minute per real second.
     */
    public syncTime(gameTimeMinutes: number): void {
        this.syncedGameTimeMinutes = gameTimeMinutes;
        this.syncRealTime = Date.now();
        this.hasSynced = true;
        const dateTime = this.formatDateTime(gameTimeMinutes);
        console.log(`Time synced: ${gameTimeMinutes} game minutes (${dateTime})`);
    }

    // --- Calendar Calculations ---

    /**
     * Convert total game minutes to custom calendar date/time.
     * Calendar: 360 days/year, 12 months of 30 days, 24-hour days.
     */
    private getCalendarDate(totalMinutes: number): { year: number; month: number; day: number; hour: number; minute: number } {
        const year = Math.floor(totalMinutes / MINUTES_PER_YEAR);
        const minutesInYear = totalMinutes % MINUTES_PER_YEAR;
        const dayOfYear = Math.floor(minutesInYear / MINUTES_PER_DAY);
        const month = Math.floor(dayOfYear / DAYS_PER_MONTH) + 1; // 1-12
        const day = (dayOfYear % DAYS_PER_MONTH) + 1; // 1-30
        const minutesInDay = totalMinutes % MINUTES_PER_DAY;
        const hour = Math.floor(minutesInDay / MINUTES_PER_HOUR);
        const minute = minutesInDay % MINUTES_PER_HOUR;
        
        return { year, month, day, hour, minute };
    }

    /**
     * Format date/time as YYYY/MM/DD HH:MM
     */
    private formatDateTime(totalMinutes: number): string {
        const { year, month, day, hour, minute } = this.getCalendarDate(totalMinutes);
        const yearStr = year.toString().padStart(4, '0');
        const monthStr = month.toString().padStart(2, '0');
        const dayStr = day.toString().padStart(2, '0');
        const hourStr = hour.toString().padStart(2, '0');
        const minuteStr = minute.toString().padStart(2, '0');
        return `${yearStr}/${monthStr}/${dayStr} ${hourStr}:${minuteStr}`;
    }

    /**
     * Get time of day in hours (0-24) for celestial/lighting calculations
     */
    private getTimeOfDayHours(totalMinutes: number): number {
        const minutesInDay = totalMinutes % MINUTES_PER_DAY;
        return minutesInDay / MINUTES_PER_HOUR;
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

    /**
     * Calculate total game minutes elapsed since epoch.
     * 1 real second = 1 game minute.
     */
    private calculateTotalGameMinutes(): number {
        if (!this.hasSynced) {
            return 0;
        }

        // Calculate elapsed real time since sync (in milliseconds)
        const elapsedRealMs = Date.now() - this.syncRealTime;
        
        // 1 game minute = 1 real second, so elapsed game minutes = elapsed real seconds
        const elapsedGameMinutes = Math.floor(elapsedRealMs / 1000);
        
        return this.syncedGameTimeMinutes + elapsedGameMinutes;
    }

    private updateTimeDisplay(totalMinutes: number): void {
        const timeDisplay = document.getElementById('game-time-display');
        if (timeDisplay) {
            const dateTimeStr = this.formatDateTime(totalMinutes);
            timeDisplay.innerHTML = `Minutes Elapsed: ${totalMinutes}<br>${dateTimeStr}`;
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
