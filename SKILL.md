# Time Helm Development Skills

This file defines custom skills and workflows for the Time Helm project.

## Custom Commands

### `/run-server`
Runs the Rust backend server with proper environment setup.

### `/run-client`
Runs the Vite development server for the frontend.

### `/run-full-stack`
Starts both the database (if needed), backend, and frontend in the correct order.

### `/format-rust`
Automatically formats Rust code using `cargo fmt` in the server directory.

### `/lint-all`
Runs linting for both client and server code.

## Hooks

### Pre-commit
- Run `cargo fmt` on Rust files
- Run ESLint on TypeScript files
- Run TypeScript type checking

### Post-code-generation
- Format generated Rust code automatically
- Check for common patterns (e.g., ensure WebSocket messages use tagged enums)

## Domain Knowledge

### Project Structure
- Frontend: TypeScript + Three.js in `client/`
- Backend: Rust + Axum in `server/`
- Database: PostgreSQL with SQLx migrations

### Key Concepts
- 1 real minute = 1 game hour (60x time scale)
- 360 game-days per game-year
- Players controlled via schedules and conditions
- Character behavior influenced by emotional state

