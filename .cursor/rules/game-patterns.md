---
description: Game-specific patterns and conventions
globs: ["sim/**/*.rs", "server/**/*.rs", "ship-game/**/*.rs"]
alwaysApply: false
---

## Game Patterns

The ship simulation lives in `sim`. The server is the only writer. The Bevy crate renders `WorldSnapshot` values and does not step the economy.

### Time

One real second is one sim second. `World::step` advances 60 seconds. Catch-up on boot must match live stepping. Do not add a 60× clock.

### WebSocket

Messages are internally tagged JSON (`#[serde(tag = "type", rename_all = "snake_case")]`).

Server:

```json
{ "type": "snapshot", "tick_unix": 0, "rooms": [], "crew": [], "stocks": [] }
{ "type": "delta", "tick_unix": 60, "crew": [], "stocks": [] }
{ "type": "command_rejected", "reason": "viewers cannot change the simulation" }
{ "type": "unavailable", "reason": "DATABASE_URL is not set; the persistent world is unavailable" }
```

Client:

```json
{ "type": "command", "command": { "type": "set_station_recipe", "station": 1, "recipe": 0 } }
```

`Role::Viewer` is rejected by `authorize`. New mutations are new `Command` variants, still rejected for viewers until a participant role exists.

### Rooms

Add a room by extending `RoomKind` and handling it in the layout, the station table, and `color_rgba`. Matches on `RoomKind` stay exhaustive.
