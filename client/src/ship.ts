/**
 * Ship Game - A 2D Phaser game
 * 
 * Entry point for the ship game accessible at /ship
 */

import Phaser from 'phaser';

/**
 * Main game scene for the ship game
 */
class ShipScene extends Phaser.Scene {
    private ship: Phaser.GameObjects.Sprite | null = null;
    private cursors: Phaser.Types.Input.Keyboard.CursorKeys | null = null;
    private wasdKeys: Record<string, Phaser.Input.Keyboard.Key> | null = null;
    private readonly shipSpeed = 200;

    constructor() {
        super({ key: 'ShipScene' });
    }

    create(): void {
        // Set background color
        this.cameras.main.setBackgroundColor('#1a1a2e');

        // Create a simple ship sprite (using a rectangle for now)
        // In a real game, you'd load an image asset
        const graphics = this.add.graphics();
        graphics.fillStyle(0x00ff00);
        graphics.fillTriangle(0, -20, -15, 15, 15, 15);
        graphics.generateTexture('ship', 30, 30);
        graphics.destroy();

        // Create ship sprite at center
        this.ship = this.add.sprite(400, 300, 'ship');
        this.ship.setOrigin(0.5, 0.5);

        // Set up keyboard input
        this.cursors = this.input.keyboard?.createCursorKeys() || null;

        // Add WASD keys as alternative
        if (this.input.keyboard) {
            this.wasdKeys = this.input.keyboard.addKeys('W,S,A,D') as Record<string, Phaser.Input.Keyboard.Key>;
        }

        // Add instructions text
        const instructions = this.add.text(10, 10, 'Ship Game\nArrow Keys or WASD to move', {
            fontSize: '16px',
            color: '#ffffff',
            backgroundColor: '#000000',
            padding: { x: 10, y: 5 }
        });
        instructions.setScrollFactor(0);
    }

    update(): void {
        if (!this.ship || !this.cursors) {
            return;
        }

        const delta = this.game.loop.delta;

        // Calculate movement
        let velocityX = 0;
        let velocityY = 0;

        // Check arrow keys or WASD
        const left = this.cursors.left?.isDown || this.wasdKeys?.A?.isDown || false;
        const right = this.cursors.right?.isDown || this.wasdKeys?.D?.isDown || false;
        const up = this.cursors.up?.isDown || this.wasdKeys?.W?.isDown || false;
        const down = this.cursors.down?.isDown || this.wasdKeys?.S?.isDown || false;

        if (left) {
            velocityX = -this.shipSpeed;
        } else if (right) {
            velocityX = this.shipSpeed;
        }

        if (up) {
            velocityY = -this.shipSpeed;
        } else if (down) {
            velocityY = this.shipSpeed;
        }

        // Normalize diagonal movement
        if (velocityX !== 0 && velocityY !== 0) {
            const length = Math.sqrt(velocityX * velocityX + velocityY * velocityY);
            velocityX = (velocityX / length) * this.shipSpeed;
            velocityY = (velocityY / length) * this.shipSpeed;
        }

        // Update ship position
        const deltaSeconds = delta / 1000;
        this.ship.x += velocityX * deltaSeconds;
        this.ship.y += velocityY * deltaSeconds;

        // Rotate ship to face movement direction
        if (velocityX !== 0 || velocityY !== 0) {
            const angle = Math.atan2(velocityY, velocityX) * (180 / Math.PI) + 90;
            this.ship.rotation = Phaser.Math.DegToRad(angle);
        }

        // Keep ship within bounds
        const width = this.cameras.main.width;
        const height = this.cameras.main.height;
        this.ship.x = Phaser.Math.Clamp(this.ship.x, 0, width);
        this.ship.y = Phaser.Math.Clamp(this.ship.y, 0, height);
    }
}

/**
 * Phaser game configuration
 */
const config: Phaser.Types.Core.GameConfig = {
    type: Phaser.AUTO,
    width: 800,
    height: 600,
    parent: 'ship-game-container',
    backgroundColor: '#1a1a2e',
    scene: ShipScene,
    physics: {
        default: 'arcade',
        arcade: {
            gravity: { x: 0, y: 0 },
            debug: false
        }
    }
};

// Initialize the game
new Phaser.Game(config);


