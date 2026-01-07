# Build stage for Rust
FROM rust:1.75 as rust-builder

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

# Build stage for frontend
FROM node:20 as frontend-builder

WORKDIR /app
COPY client/package.json client/package-lock.json* ./
RUN npm install
COPY client ./
RUN npm run build

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from builder
COPY --from=rust-builder /app/server/target/release/timehelm-server /app/server

# Copy frontend build
COPY --from=frontend-builder /app/dist /app/client/dist

# Expose port
EXPOSE 8080

# Run the server
CMD ["/app/server"]

