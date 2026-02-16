.PHONY: dev build deploy install lint lint-client lint-server fmt build-ship run-ship

build-ship:
	./scripts/build-ship.sh

# Run the ship game locally. Builds WASM, then starts Vite. Open in browser:
#   http://localhost:5173/
run-ship: build-ship
	cd client && npm run dev

install:
	npm install
	cd client && npm install
	cd server && cargo build

lint: lint-client lint-server

lint-client:
	cd client && npm run lint
	cd client && npm run typecheck

lint-server:
	cd server && cargo fmt --check
	cd server && cargo clippy -- -D warnings

fmt:
	cd server && cargo fmt

dev-server:
	cd server && cargo run

dev-client:
	cd client && npm run dev

build:
	./scripts/build-ship.sh
	cd client && npm install && npm run build
	cd server && cargo build --release

deploy:
	./build.sh
	fly deploy

fly-db-connect:
	fly mpg connect $(PG_CLUSTER_ID)	

fly-logs:
	fly logs --app timehelm