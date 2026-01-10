/**
 * Input manager module.
 * 
 * Handles keyboard, mouse, and wheel input events.
 * Provides callback-based event system for game controls.
 */

/**
 * Mouse position interface.
 */
export interface MousePosition {
    /** X coordinate */
    x: number;
    /** Y coordinate */
    y: number;
}

/**
 * Callback type for key press events.
 */
export type InputEventCallback = (key: string) => void;

/**
 * Input manager class.
 * 
 * Manages all input events:
 * - Keyboard key presses
 * - Mouse drag (right mouse button)
 * - Mouse wheel
 * - Context menu prevention
 */
export class InputManager {
    private keys: Record<string, boolean> = {};
    private isRightMouseDown: boolean = false;
    private lastMousePos: MousePosition = { x: 0, y: 0 };

    // Event callbacks
    private onKeyPressCallbacks: InputEventCallback[] = [];
    private onMouseDragCallbacks: ((deltaX: number, deltaY: number) => void)[] = [];
    private onWheelCallbacks: ((delta: number) => void)[] = [];

    constructor() {
        this.setupEventListeners();
    }

    // --- Getters ---

    public isKeyPressed(key: string): boolean {
        return this.keys[key] || false;
    }

    public getLastMousePosition(): MousePosition {
        return { ...this.lastMousePos };
    }

    // --- Event Registration ---

    public onKeyPress(callback: InputEventCallback): void {
        this.onKeyPressCallbacks.push(callback);
    }

    public onMouseDrag(callback: (deltaX: number, deltaY: number) => void): void {
        this.onMouseDragCallbacks.push(callback);
    }

    public onWheel(callback: (delta: number) => void): void {
        this.onWheelCallbacks.push(callback);
    }

    // --- Cleanup ---

    public dispose(): void {
        document.removeEventListener('keydown', this.handleKeyDown);
        document.removeEventListener('keyup', this.handleKeyUp);
        document.removeEventListener('mousedown', this.handleMouseDown);
        document.removeEventListener('mousemove', this.handleMouseMove);
        document.removeEventListener('mouseup', this.handleMouseUp);
        document.removeEventListener('wheel', this.handleWheel);
        document.removeEventListener('contextmenu', this.handleContextMenu);
    }

    // --- Private Methods ---

    private setupEventListeners(): void {
        document.addEventListener('keydown', this.handleKeyDown);
        document.addEventListener('keyup', this.handleKeyUp);
        document.addEventListener('mousedown', this.handleMouseDown);
        document.addEventListener('mousemove', this.handleMouseMove);
        document.addEventListener('mouseup', this.handleMouseUp);
        document.addEventListener('wheel', this.handleWheel);
        document.addEventListener('contextmenu', this.handleContextMenu);
    }

    private handleKeyDown = (e: KeyboardEvent): void => {
        const key = e.key.toLowerCase();
        this.keys[key] = true;

        for (const callback of this.onKeyPressCallbacks) {
            callback(key);
        }
    };

    private handleKeyUp = (e: KeyboardEvent): void => {
        this.keys[e.key.toLowerCase()] = false;
    };

    private handleMouseDown = (e: MouseEvent): void => {
        if (e.button === 2) {
            this.isRightMouseDown = true;
            this.lastMousePos = { x: e.clientX, y: e.clientY };
        }
    };

    private handleMouseMove = (e: MouseEvent): void => {
        if (this.isRightMouseDown) {
            const deltaX = e.clientX - this.lastMousePos.x;
            const deltaY = e.clientY - this.lastMousePos.y;

            for (const callback of this.onMouseDragCallbacks) {
                callback(deltaX, deltaY);
            }

            this.lastMousePos = { x: e.clientX, y: e.clientY };
        }
    };

    private handleMouseUp = (e: MouseEvent): void => {
        if (e.button === 2) {
            this.isRightMouseDown = false;
        }
    };

    private handleWheel = (e: WheelEvent): void => {
        for (const callback of this.onWheelCallbacks) {
            callback(e.deltaY);
        }
    };

    private handleContextMenu = (e: Event): void => {
        e.preventDefault();
    };
}
