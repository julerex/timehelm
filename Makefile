.PHONY: dev build deploy install lint lint-server fmt build-ship run-ship

build-ship:
	./scripts/build-ship.sh

# Run the ship game locally. Builds WASM, then starts Rust server. Open in browser:
#   http://localhost:8080/
run-ship: build-ship
	cd server && cargo run

install:
	npm install
	cd server && cargo build

lint: lint-server

lint-server:
	cd server && cargo fmt --check
	cd server && cargo clippy -- -D warnings

fmt:
	cd server && cargo fmt

dev-server:
	cd server && cargo run

build:
	./scripts/build-ship.sh
	cd server && cargo build --release

deploy:
	./build.sh
	fly deploy

fly-db-connect:
	fly mpg connect $(PG_CLUSTER_ID)	

fly-logs:
	fly logs --app timehelm
