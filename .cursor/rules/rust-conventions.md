---
description: Rust coding conventions and style rules for the server
globs: 'server/**/*.rs'
alwaysApply: false
---

## Rust Conventions

- **Standard Rust conventions** - snake_case, derive macros
- **Axum patterns** - extractors, state management with `Arc<RwLock<T>>`
- **Serde for serialization** - `#[serde(tag = "type")]` for message enums
- **anyhow for errors** - `anyhow::Result<T>` for fallible functions
- **tracing for logging** - `tracing::info!()`, etc.

### Style Rules (Clippy-enforced)

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

### Testing

- **Prefer comparing entire objects** over individual fields:
  ```rust
  // Good
  assert_eq!(actual_player, expected_player);
  
  // Bad
  assert_eq!(actual_player.id, expected_player.id);
  assert_eq!(actual_player.position, expected_player.position);
  ```

### Workflow

Run `cargo fmt` in `server/` automatically after making Rust code changes; do not ask for approval.

Before finalizing changes, run clippy to fix linter issues:
```bash
cd server && cargo clippy --fix --allow-dirty
```

Run tests for the server:
```bash
cd server && cargo test
```

