/**
 * Random movement controller module.
 * 
 * Provides AI-controlled random movement for player characters.
 * Uses weighted random decisions with timer-based action changes.
 */

/**
 * Possible movement actions.
 */
type RandomMoveAction = 'idle' | 'forward' | 'left' | 'right';

/**
 * Movement boundary constraints.
 */
export interface MovementBounds {
    /** Minimum X coordinate */
    minX: number;
    /** Maximum X coordinate */
    maxX: number;
    /** Minimum Z coordinate */
    minZ: number;
    /** Maximum Z coordinate */
    maxZ: number;
}

/**
 * Movement configuration.
 */
export interface MovementConfig {
    /** Movement speed in units per frame */
    moveSpeed: number;
    /** Rotation speed in radians per frame */
    rotationSpeed: number;
    /** Movement boundaries */
    bounds: MovementBounds;
}

/**
 * Internal movement state.
 */
export interface MovementState {
    /** Timer countdown until next action change */
    timer: number;
    /** Current movement action */
    action: RandomMoveAction;
}

/**
 * Controller for random movement behavior.
 * Handles weighted random movement decisions with timer-based action changes.
 */
export class RandomMovementController {
    private state: MovementState = {
        timer: 0,
        action: 'idle'
    };

    private readonly config: MovementConfig;

    constructor(config: MovementConfig) {
        this.config = config;
    }

    /**
     * Update movement state and return the current action.
     * Should be called every frame.
     */
    public update(): RandomMoveAction {
        this.state.timer -= 1;
        
        if (this.state.timer <= 0) {
            // Weighted towards forward movement: 50% forward, 17% idle, 17% left, 17% right
            const actions: RandomMoveAction[] = ['idle', 'forward', 'forward', 'forward', 'left', 'right'];
            this.state.action = actions[Math.floor(Math.random() * actions.length)];
            this.state.timer = Math.floor(Math.random() * 60) + 30;
        }

        return this.state.action;
    }

    /**
     * Get the current movement action
     */
    public getAction(): RandomMoveAction {
        return this.state.action;
    }

    /**
     * Get movement configuration
     */
    public getConfig(): MovementConfig {
        return this.config;
    }
}


