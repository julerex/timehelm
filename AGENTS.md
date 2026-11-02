# AGENTS.md

This file provides context for AI coding assistants working on the Time Helm project.

## Project Overview

Time Helm is an open-source game. The main client is a Bevy 3D cruise ship game (Rust/WASM), served at the root URL.

**Production URL:** https://timehelm.net/



## Tech Stack

| Component | Technology |
|-----------|------------|
| Frontend | Bevy 3D (Rust/WASM), served by Rust server |
| Backend | Rust, Axum, Tokio |
| Database | PostgreSQL (with SQLx) |
| Auth | Twitter/X OAuth 2.0 |
| Real-time | WebSockets |
| Deployment | fly.io |

## Quick Start

### Prerequisites
- Rust (stable)
- PostgreSQL 14+ (or use Docker)

### Running Locally

**Terminal 1 - Database (if using Docker):**
```bash
docker run --name timehelm-db -e POSTGRES_PASSWORD=password -e POSTGRES_DB=timehelm -p 5432:5432 -d postgres:15
```

**Terminal 2 - Server (serves ship game at root):**
```bash
make run-ship
```

Access at http://localhost:8080/

### Using Make

**IMPORTANT:** Always check the `Makefile` for available commands before running manual commands. The Makefile contains the canonical commands for common tasks.

```bash
make install      # Install dependencies (cargo for server)
make run-ship     # Build ship WASM + run server (dev)
make dev-server   # Run Rust server only
make build        # Production build (ship WASM + server)
make deploy       # Build and deploy to fly.io
make fly-logs     # View production server logs (requires flyctl)
```

**Production URL:** The deployed application is available at https://timehelm.net/

## Project Structure

```
timehelm/
├── client/public/          # Static files (Rust server serves these)
│   ├── index.html         # Ship game entry (loads Bevy WASM from /ship/)
│   └── ship/              # Bevy WASM output (built by scripts/build-ship.sh)
│       ├── run.js         # Loader script
│       ├── ship.js        # wasm-bindgen glue
│       └── ship_bg.wasm   # Ship game binary
├── ship-game/              # Bevy 3D cruise ship game (Rust/WASM)
│   ├── src/lib.rs
│   └── Cargo.toml
├── server/                 # Rust/Axum backend (serves static files + WebSocket)
│   ├── src/
│   │   ├── main.rs        # Server entry, routes, static serving
│   │   ├── game.rs        # Game state, player management
│   │   ├── websocket.rs   # WebSocket message handling
│   │   └── db.rs          # Database operations
│   └── migrations/        # SQL migrations (auto-run on start)
├── scripts/
│   └── build-ship.sh      # Build Bevy ship game to WASM
├── Makefile               # Common development commands (includes make build-ship)
└── fly.toml               # Deployment config
```

## Coding Conventions

### Rust (Server)

- **Standard Rust conventions** - snake_case, derive macros
- **Axum patterns** - extractors, state management with `Arc<RwLock<T>>`
- **Serde for serialization** - `#[serde(tag = "type")]` for message enums
- **anyhow for errors** - `anyhow::Result<T>` for fallible functions
- **tracing for logging** - `tracing::info!()`, etc.

#### Style Rules (Clippy-enforced)

- **Inline format args** - Use `format!("{x}")` not `format!("{}", x)`
  ```rust
  // Good
  tracing::info!("Player {player_id} joined");
  
  // Bad
  tracing::info!("Player {} joined", player_id);
  ```

- **Collapse if statements** - Combine nested ifs when possible
  ```rust
  // Good
  if condition_a && condition_b {
      do_thing();
  }
  
  // Bad
  if condition_a {
      if condition_b {
          do_thing();
      }
  }
  ```

- **Method references over closures** - Use `.map(Player::new)` not `.map(|p| Player::new(p))`
  ```rust
  // Good
  players.iter().map(Player::from_data).collect()
  
  // Bad
  players.iter().map(|p| Player::from_data(p)).collect()
  ```

#### Testing

- **Prefer comparing entire objects** over individual fields:
  ```rust
  // Good
  assert_eq!(actual_player, expected_player);
  
  // Bad
  assert_eq!(actual_player.id, expected_player.id);
  assert_eq!(actual_player.position, expected_player.position);
  ```

#### Workflow

Run `cargo fmt` in `server/` automatically after making Rust code changes; do not ask for approval.

Before finalizing changes, run clippy to fix linter issues:
```bash
cd server && cargo clippy --fix --allow-dirty
```

Run tests for the server:
```bash
cd server && cargo test
```

## Linting

- **Server:** `cargo fmt --check` + `cargo clippy`
```bash
make lint              # Server only
```

## Key Patterns

The server supports WebSocket connections and game state broadcasting for future multiplayer. The ship game (Bevy WASM) is the main client and loads at the root URL.

## Environment Variables

Create `.env` in project root:
```bash
PORT=8080
BASE_URL=http://localhost:8080
DATABASE_URL=postgresql://postgres:password@localhost:5432/timehelm
TWITTER_CLIENT_ID=your_client_id
TWITTER_CLIENT_SECRET=your_client_secret
```

## Deployment

```bash
make deploy  # Builds ship WASM and server, deploys to fly.io
```

Requires fly.io CLI (`flyctl`) and secrets configured. See README.md for full setup.

**Production:** The deployed application is available at https://timehelm.net/

**Viewing Logs:** Use `make fly-logs` to view production server logs. Always check the Makefile for the correct command syntax.

**Ship game (Bevy 3D):** Use `make run-ship` to build and run locally. Open `http://localhost:8080/` in your browser.

**Browser checks in Cursor:** To verify the ship game visually:
1. Run `make run-ship` in a terminal (Rust server will start).
2. In Cursor: **View → Simple Browser** (or Cmd/Ctrl+Shift+P → "Simple Browser: Show").
3. Enter `http://localhost:8080/` in the Simple Browser address bar.

The ship game loads WASM; first load may take a few seconds. Use A/D or arrow keys to move, Page Up/Down to switch decks.

### Cursor / cargo proxy error

When running `cargo` (build, test, fmt, clippy) inside Cursor you may see:

`error: unknown proxy name: 'Cursor-2.x-x86_64'; valid proxy names...`

The **Makefile** (see test/format/clippy targets) works around this by running **`unset ARGV0 && cargo ...`** before invoking cargo. Prefer **`make test`**, **`make format`**, **`make clippy`** when in Cursor, or run `unset ARGV0 && cargo <subcommand>` so cargo uses the real toolchain instead of the Cursor proxy.

## Cursor Cloud specific instructions

- Use Rust **1.92.0** (the `rust:1.92` image in `Dockerfile`). Install the `wasm32-unknown-unknown` target and `wasm-bindgen-cli` **0.2.120**, which must match `wasm-bindgen` in `ship-game/Cargo.lock`. `scripts/build-ship.sh` fails if the CLI schema does not match.
- `make run-ship` builds the Bevy WASM client into `client/public/ship/` and serves it from the Rust server. `http://localhost:8080/` redirects to `/3d/`. `/seacells/` is the 2D view. Page Up/Down switches decks.
- `dotenv` reads `.env` from the process working directory. `make run-ship` and `cd server && cargo run` therefore need `server/.env`, not only a root `.env`. Copy `DATABASE_URL` into `server/.env`. If it is unset, the server still listens and logs that persistence is disabled.
- Local Postgres is cluster **16/main**. `policy-rc.d` can refuse `service postgresql start`; use `sudo pg_ctlcluster 16 main start`. The dev URL from the README is `postgresql://postgres:password@localhost:5432/timehelm`. Apply `database/timehelm_schema.sql` once. Re-running it fails on `CREATE TRIGGER` (no `IF NOT EXISTS`).
- Twitter OAuth secrets are not required to boot. `server/src/main.rs` does not mount the auth module.
- `make test` runs server tests (currently none) and `ship-game` library tests. `cargo clippy -- -D warnings` passes for both crates. `make lint` also runs `cargo fmt --check`, and `ship-game` currently has pre-existing formatting drift, so that check fails until `make format` is applied.
