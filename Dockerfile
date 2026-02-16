# Build stage for Rust server
FROM rust:1.92 as rust-builder

WORKDIR /app

# Copy Cargo files for dependency caching
COPY server/Cargo.toml ./server/
RUN mkdir -p server/src && echo "fn main() {}" > server/src/main.rs
# Build dependencies only (for caching)
RUN cd server && cargo build --release || true

# Copy source code (this will overwrite the dummy main.rs)
COPY server ./server

# Build the application
WORKDIR /app/server
RUN cargo build --release

# Build stage for ship game (Bevy WASM)
FROM rust:1.92 as ship-builder

WORKDIR /app
RUN rustup target add wasm32-unknown-unknown
RUN cargo install wasm-bindgen-cli

COPY ship-game ./ship-game
RUN RUSTFLAGS='--cfg getrandom_backend="wasm_js"' cargo build --manifest-path ship-game/Cargo.toml --target wasm32-unknown-unknown --release

RUN mkdir -p /app/client-public/ship && \
    wasm-bindgen --no-typescript --target web \
        --out-dir /app/client-public/ship \
        --out-name ship \
        /app/ship-game/target/wasm32-unknown-unknown/release/ship_game.wasm

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from builder
COPY --from=rust-builder /app/server/target/release/timehelm-server /app/server

# Copy static files: client/public (index.html, run.js) + ship WASM
COPY client/public ./client/public
COPY --from=ship-builder /app/client-public/ship ./client/public/ship

# Expose port
EXPOSE 8080

# Run the server
CMD ["/app/server"]
