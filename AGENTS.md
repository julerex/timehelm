# AGENTS.md

This file provides context for AI coding assistants working on the Time Helm project.

## Project Overview

Time Helm is an open-source, persistent-world MMO sandbox social simulation game. Think "The Sims Online" meets low-poly 3D with reality-centric mechanics.

**Key game concepts:**
- 1 real minute = 1 game hour (60x time scale)
- 360 game-days per game-year
- Players control characters indirectly via schedules and conditions
- Character behavior influenced by emotional state

## Tech Stack

| Component | Technology |
|-----------|------------|
| Frontend | TypeScript, Three.js, Vite |
| Backend | Rust, Axum, Tokio |
| Database | PostgreSQL (with SQLx) |
| Auth | Twitter/X OAuth 2.0 |
| Real-time | WebSockets |
| Deployment | fly.io |

## Quick Start

### Prerequisites
- Rust (stable)
- Node.js 18+
- PostgreSQL 14+ (or use Docker)

### Running Locally

**Terminal 1 - Database (if using Docker):**
```bash
docker run --name timehelm-db -e POSTGRES_PASSWORD=password -e POSTGRES_DB=timehelm -p 5432:5432 -d postgres:15
```

**Terminal 2 - Backend:**
```bash
cd server && cargo run
```

**Terminal 3 - Frontend:**
```bash
cd client && npm install && npm run dev
```

Access at http://localhost:5173 (Vite proxies API/WebSocket to backend on :8080)

### Using Make
```bash
make install      # Install all dependencies
make dev-server   # Run Rust server
make dev-client   # Run Vite dev server
make lint         # Lint both client and server
make build        # Production build
make deploy       # Build and deploy to fly.io
```

## Project Structure

```
timehelm/
├── client/                 # TypeScript/Three.js frontend
│   ├── src/
│   │   ├── main.ts              # Entry point
│   │   ├── game-client.ts       # Main game orchestrator
│   │   ├── camera/              # Camera control
│   │   ├── entities/            # Player and game entities
│   │   ├── environment/         # Day/night cycle, weather
│   │   ├── input/               # Keyboard/mouse handling
│   │   ├── network/             # WebSocket communication
│   │   └── world/               # World objects (trees, houses, etc.)
│   └── index.html
├── server/                 # Rust/Axum backend
│   ├── src/
│   │   ├── main.rs              # Server entry, routes
│   │   ├── auth.rs              # OAuth handling
│   │   ├── game.rs              # Game state, player management
│   │   ├── websocket.rs         # WebSocket message handling
│   │   └── db.rs                # Database operations
│   └── migrations/              # SQL migrations (auto-run on start)
├── vite.config.ts          # Dev server config with proxy
├── Makefile                # Common development commands
└── fly.toml                # Deployment config
```

## Coding Conventions

### TypeScript (Client)

- **Classes for major components** - `GameClient`, `Player`, `CameraController`, etc.
- **Readonly for immutable properties** - `private readonly user: User`
- **Null initialization with type annotation** - `private scene: THREE.Scene | null = null`
- **Section comments** - Use `// --- Section Name ---` to organize methods
- **Explicit exports** - Re-export types from index files when needed
- **No explicit return types required** (ESLint rule disabled)
- **Prefix unused params with underscore** - `(_unused) => {}`

Example structure:
```typescript
export class MyComponent {
    private readonly config: Config;
    private state: State | null = null;

    constructor(config: Config) {
        this.config = config;
    }

    // --- Public Methods ---
    public init(): void { }
    public dispose(): void { }

    // --- Private Methods ---
    private setupX(): void { }
    private handleY(): void { }
}
```

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

Pre-commit hooks run automatically via Husky + lint-staged:
- **Client:** ESLint + TypeScript type checking
- **Server:** `cargo fmt --check` + `cargo clippy`

Manual lint:
```bash
npm run lint           # Both client and server
npm run lint:client    # ESLint only
npm run lint:server    # Clippy + fmt check
```

## Key Patterns

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
make deploy  # Builds frontend, deploys to fly.io
```

Requires fly.io CLI (`flyctl`) and secrets configured. See README.md for full setup.
