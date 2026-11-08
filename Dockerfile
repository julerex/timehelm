# Build stage for the Rust server. The repo is a Cargo workspace, so the
# binary is /app/target/release/timehelm-server rather than a per-crate target.
FROM rust:1.92 AS rust-builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY sim ./sim
COPY server ./server
COPY ship-game/Cargo.toml ./ship-game/Cargo.toml
RUN mkdir -p ship-game/src && printf 'pub fn run_native() {}\n' > ship-game/src/lib.rs \
    && printf 'fn main() {}\n' > ship-game/src/main.rs
RUN cargo build --release -p timehelm-server
COPY ship-game ./ship-game
RUN cargo build --release -p timehelm-server

# Build stage for the Bevy viewer.
FROM rust:1.92 AS ship-builder

WORKDIR /app
# zstd-sys compiles its C sources for wasm32-unknown-unknown. cc-rs uses clang
# for that target; the stock Rust image only has gcc, which has no wasm backend.
RUN apt-get update && apt-get install -y --no-install-recommends clang \
    && rm -rf /var/lib/apt/lists/*
RUN rustup target add wasm32-unknown-unknown
# Must match wasm-bindgen in Cargo.lock (web-sys 0.3.107 requires 0.2.130).
RUN cargo install wasm-bindgen-cli --version 0.2.130

COPY Cargo.toml Cargo.lock ./
COPY sim ./sim
COPY server/Cargo.toml ./server/Cargo.toml
RUN mkdir -p server/src && printf 'fn main() {}\n' > server/src/main.rs
COPY ship-game ./ship-game
RUN RUSTFLAGS='--cfg getrandom_backend="wasm_js"' cargo build --release -p ship-game --target wasm32-unknown-unknown

RUN mkdir -p /app/client-public/ship && \
    wasm-bindgen --no-typescript --target web \
        --out-dir /app/client-public/ship \
        --out-name ship \
        /app/target/wasm32-unknown-unknown/release/ship_game.wasm

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=rust-builder /app/target/release/timehelm-server /app/server
COPY client/public ./client/public
COPY --from=ship-builder /app/client-public/ship ./client/public/ship

EXPOSE 8080

CMD ["/app/server"]
