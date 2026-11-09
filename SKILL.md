# Time Helm

The playable world is a server-authoritative ship. Read `docs/SIMULATION.md` before changing the economy or the protocol.

## Run

- `make run-ship` builds the Bevy viewer to `client/public/ship/` and starts the Axum server. Open `http://localhost:8080/` (it redirects to `/3d/`).
- `make dev-server` starts the server without rebuilding WASM.
- `make run-ship-native` opens the same viewer in a desktop window. It connects to `ws://127.0.0.1:8080/ws` unless `TIMEHELM_WS` is set.
- Put `DATABASE_URL` in `server/.env`. The server reads `.env` from its working directory. If the variable is missing, the viewer is told the world is unavailable.

## Where code goes

- Simulation, recipes, hull, and protocol types: `sim/`. Do not depend on Bevy from this crate.
- Persistence and the WebSocket: `server/`. Call `World::step` only here.
- Rendering, camera, and the keymap: `ship-game/`. The client interpolates crew between snapshots. It does not run recipes.

## Time

One real second is one sim second. The server steps every 60 sim seconds, using the wall clock, and applies every missed step on boot. Do not reintroduce a 60× clock.

## Checks

```bash
make fmt
make lint
make test
```

`wasm-bindgen-cli` must be **0.2.130**, matching `Cargo.lock`. The WASM file is `target/wasm32-unknown-unknown/release/ship_game.wasm`.

## Protocol

Messages are tagged JSON (`type` is `snake_case`): `snapshot`, `delta`, `command_rejected`, `unavailable`, and client `command`. A viewer command is rejected. See `.cursor/rules/game-patterns.md`.
