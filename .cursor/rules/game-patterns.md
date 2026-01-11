---
description: Game-specific patterns and conventions
globs: ['client/**/*.ts', 'server/**/*.rs']
alwaysApply: false
---

## Game Patterns

### Client-Server Communication

Messages use tagged JSON via WebSocket:
```typescript
// Client sends
{ "type": "Join", "player": { "id": "...", "username": "...", ... } }
{ "type": "Move", "player_id": "...", "position": {...}, "rotation": 0.5, "is_moving": true }

// Server broadcasts
{ "type": "WorldState", "players": [...] }
{ "type": "PlayerJoin", "player": {...} }
{ "type": "PlayerLeave", "player_id": "..." }
```

### Adding World Objects

Use `WorldObjectFactory` in `client/src/world/WorldObjectFactory.ts`:
```typescript
const tree = WorldObjectFactory.createTree(x, z);
this.scene.add(tree);
```

### Game Time

Movement speed is scaled for 60x time:
- `moveSpeed = 100` units/frame at 60 FPS = 6000 units/game-minute
- Day/night cycle managed by `DayNightCycle` class

