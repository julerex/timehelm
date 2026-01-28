/**
 * Ship Game - A 2D Phaser game
 * 
 * Entry point for the ship game accessible at /ship
 * A cruise ship game where you can move between different decks
 */

import Phaser from 'phaser';

/**
 * Deck names for the cruise ship
 */
const DECK_NAMES = [
    'Sun Deck',
    'Pool Deck',
    'Promenade Deck',
    'Main Deck',
    'Lower Deck',
    'Engine Deck'
];

/**
 * Main game scene for the cruise ship game
 */
class ShipScene extends Phaser.Scene {
    private player: Phaser.GameObjects.Sprite | null = null;
    private cursors: Phaser.Types.Input.Keyboard.CursorKeys | null = null;
    private wasdKeys: Record<string, Phaser.Input.Keyboard.Key> | null = null;
    private pageUpKey: Phaser.Input.Keyboard.Key | null = null;
    private pageDownKey: Phaser.Input.Keyboard.Key | null = null;
    private currentDeck = 0;
    private deckDisplay: Phaser.GameObjects.Text | null = null;
    private deckLayers: Phaser.GameObjects.Container[] = [];
    private pageUpPressed = false;
    private pageDownPressed = false;
    private readonly shipSpeed = 200;
    private readonly totalDecks = DECK_NAMES.length;

    constructor() {
        super({ key: 'ShipScene' });
    }

    create(): void {
        // Set background color (ocean blue)
        this.cameras.main.setBackgroundColor('#0a1929');

        // Create deck layers
        this.createDeckLayers();

        // Create player sprite (person walking on deck)
        const graphics = this.add.graphics();
        graphics.fillStyle(0x00ff00);
        graphics.fillCircle(0, 0, 10);
        graphics.fillStyle(0x0000ff);
        graphics.fillRect(-5, 5, 10, 15);
        graphics.generateTexture('player', 20, 25);
        graphics.destroy();

        // Create player sprite at center of current deck
        this.player = this.add.sprite(400, 300, 'player');
        this.player.setOrigin(0.5, 0.5);

        // Set up keyboard input
        this.cursors = this.input.keyboard?.createCursorKeys() || null;

        // Add WASD keys for horizontal movement
        if (this.input.keyboard) {
            this.wasdKeys = this.input.keyboard.addKeys('A,D') as Record<string, Phaser.Input.Keyboard.Key>;
            // Page Up/Down for deck switching
            this.pageUpKey = this.input.keyboard.addKey(Phaser.Input.Keyboard.KeyCodes.PAGE_UP);
            this.pageDownKey = this.input.keyboard.addKey(Phaser.Input.Keyboard.KeyCodes.PAGE_DOWN);
        }

        // Create deck display text
        this.deckDisplay = this.add.text(10, 10, '', {
            fontSize: '18px',
            color: '#ffffff',
            backgroundColor: '#000000',
            padding: { x: 15, y: 10 }
        });
        this.deckDisplay.setScrollFactor(0);
        this.updateDeckDisplay();

        // Add instructions text
        const instructions = this.add.text(10, 70, 'Cruise Ship Game\nLeft/Right or A/D to move\nPage Up/Down to change decks', {
            fontSize: '14px',
            color: '#ffffff',
            backgroundColor: '#000000',
            padding: { x: 10, y: 8 }
        });
        instructions.setScrollFactor(0);

        // Show current deck
        this.showDeck(this.currentDeck);
    }

    createDeckLayers(): void {
        // Create visual layers for each deck
        for (let i = 0; i < this.totalDecks; i++) {
            const container = this.add.container(0, 0);
            
            // Create deck floor (different colors for each deck)
            const deckColor = 0x2a4a6a + (i * 0x101010);
            const floor = this.add.rectangle(400, 550, 800, 100, deckColor);
            floor.setAlpha(0.7);
            container.add(floor);

            // Add deck number label
            const label = this.add.text(50, 520, `Deck ${i + 1}: ${DECK_NAMES[i]}`, {
                fontSize: '16px',
                color: '#ffffff',
                backgroundColor: '#000000',
                padding: { x: 8, y: 4 }
            });
            container.add(label);

            // Add some decorative elements (windows, railings, etc.)
            for (let j = 0; j < 10; j++) {
                const window = this.add.rectangle(100 + j * 70, 500, 40, 30, 0x87ceeb);
                window.setAlpha(0.5);
                container.add(window);
            }

            // Initially hide all decks except the first
            container.setVisible(i === 0);
            this.deckLayers.push(container);
        }
    }

    updateDeckDisplay(): void {
        if (this.deckDisplay) {
            this.deckDisplay.setText(`Deck ${this.currentDeck + 1}/${this.totalDecks}: ${DECK_NAMES[this.currentDeck]}`);
        }
    }

    showDeck(deckIndex: number): void {
        // Hide all decks
        this.deckLayers.forEach((layer, index) => {
            layer.setVisible(index === deckIndex);
        });
        this.updateDeckDisplay();
    }

    changeDeck(direction: number): void {
        const newDeck = this.currentDeck + direction;
        if (newDeck >= 0 && newDeck < this.totalDecks) {
            this.currentDeck = newDeck;
            this.showDeck(this.currentDeck);
        }
    }

    update(): void {
        if (!this.player || !this.cursors) {
            return;
        }

        // Handle deck switching (only trigger once per key press)
        if (this.pageUpKey?.isDown && !this.pageUpPressed) {
            this.pageUpPressed = true;
            this.changeDeck(-1);
        } else if (!this.pageUpKey?.isDown) {
            this.pageUpPressed = false;
        }

        if (this.pageDownKey?.isDown && !this.pageDownPressed) {
            this.pageDownPressed = true;
            this.changeDeck(1);
        } else if (!this.pageDownKey?.isDown) {
            this.pageDownPressed = false;
        }

        const delta = this.game.loop.delta;

        // Calculate horizontal movement only
        let velocityX = 0;

        // Check arrow keys or WASD (only left/right)
        const left = this.cursors.left?.isDown || this.wasdKeys?.A?.isDown || false;
        const right = this.cursors.right?.isDown || this.wasdKeys?.D?.isDown || false;

        if (left) {
            velocityX = -this.shipSpeed;
        } else if (right) {
            velocityX = this.shipSpeed;
        }

        // Update player position
        const deltaSeconds = delta / 1000;
        this.player.x += velocityX * deltaSeconds;

        // Rotate player to face movement direction
        if (velocityX < 0) {
            this.player.setFlipX(true);
        } else if (velocityX > 0) {
            this.player.setFlipX(false);
        }

        // Keep player within bounds (horizontal only)
        const width = this.cameras.main.width;
        this.player.x = Phaser.Math.Clamp(this.player.x, 20, width - 20);
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


