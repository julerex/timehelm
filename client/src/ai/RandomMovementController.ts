type RandomMoveAction = 'idle' | 'forward' | 'left' | 'right';

export interface MovementBounds {
    minX: number;
    maxX: number;
    minZ: number;
    maxZ: number;
}

export interface MovementConfig {
    moveSpeed: number;
    rotationSpeed: number;
    bounds: MovementBounds;
}

export interface MovementState {
    timer: number;
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

