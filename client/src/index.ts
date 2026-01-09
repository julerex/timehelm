// Main entry point - re-exports for clean API

// Core game client
export { GameClient } from './game-client';
export type { User } from './game-client';

// Entities
export { Player } from './entities/Player';
export type { Position, PlayerData } from './entities/Player';

// Systems
export { CameraController } from './camera/CameraController';
export { InputManager } from './input/InputManager';
export { NetworkManager } from './network/NetworkManager';
export type { NetworkEventHandlers, WebSocketMessage } from './network/NetworkManager';

// Environment
export { DayNightCycle } from './environment/DayNightCycle';

// World building
export { WorldObjectFactory } from './world/WorldObjectFactory';
