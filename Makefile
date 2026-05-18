.PHONY: dev build deploy install lint lint-server lint-ship fmt test build-ship run-ship build-ship-native run-ship-native run-ship-native-gl run-ship-native-vulkan \
	gh-check ci ci-list ci-here ci-watch ci-watch-here ci-view ci-log ci-status \
	inspect-ship write-ship-default refresh-ship-sides

# Compressed ship save for agent analysis (override: make inspect-ship SAVE_SHIP=path/to/file.ship.zst)
SAVE_SHIP ?= saved_ships/latest.ship.zst
ifdef SAVE
SAVE_SHIP := $(SAVE)
endif

# Cursor sets ARGV0 to its binary path, which breaks cargo's proxy detection.
CARGO = env -u ARGV0 cargo

# GitHub Actions (requires: gh CLI authenticated via `gh auth login`)
WORKFLOW ?= deploy.yml
CI_BRANCH ?= main
CI_LIMIT ?= 15

gh-check:
	@command -v gh >/dev/null 2>&1 || { echo >&2 "GitHub CLI (gh) required: https://cli.github.com/"; exit 1; }

# Default: recent runs for deploy workflow on main (override: make ci CI_BRANCH=my-branch)
ci: ci-list

ci-list: gh-check
	gh run list --workflow=$(WORKFLOW) --branch=$(CI_BRANCH) --limit=$(CI_LIMIT)

# Same as ci-list but uses your current git branch
ci-here: gh-check
	gh run list --workflow=$(WORKFLOW) --branch=$$(git rev-parse --abbrev-ref HEAD 2>/dev/null) --limit=$(CI_LIMIT)

# Wait until the latest matching run finishes (Ctrl+C to stop watching)
ci-watch: gh-check
	gh run watch $$(gh run list --workflow=$(WORKFLOW) --branch=$(CI_BRANCH) --limit=1 --json databaseId -q '.[0].databaseId')

ci-watch-here: gh-check
	gh run watch $$(gh run list --workflow=$(WORKFLOW) --branch=$$(git rev-parse --abbrev-ref HEAD 2>/dev/null) --limit=1 --json databaseId -q '.[0].databaseId')

# Summary + steps for the latest matching run
ci-view: gh-check
	gh run view $$(gh run list --workflow=$(WORKFLOW) --branch=$(CI_BRANCH) --limit=1 --json databaseId -q '.[0].databaseId')

# Full log for the latest matching run (verbose)
ci-log: gh-check
	gh run view $$(gh run list --workflow=$(WORKFLOW) --branch=$(CI_BRANCH) --limit=1 --json databaseId -q '.[0].databaseId') --log

# One-line status per recent run (good for agents / scripts)
ci-status: gh-check
	gh run list --workflow=$(WORKFLOW) --branch=$(CI_BRANCH) --limit=$(CI_LIMIT) --json status,conclusion,displayTitle,url,updatedAt \
		-q '.[] | "\(.status)\t\(.conclusion // "-")\t\(.displayTitle)\t\(.updatedAt)\t\(.url)"'

WGPU_BACKEND ?= vulkan

build-ship:
	./scripts/build-ship.sh

# Run the ship game locally. Builds WASM, then starts Rust server. Open in browser:
#   http://localhost:8080/
run-ship: build-ship
	cd server && $(CARGO) run

build-ship-native:
	cd ship-game && $(CARGO) build

run-ship-native:
	cd ship-game && WGPU_BACKEND=$(WGPU_BACKEND) $(CARGO) run

run-ship-native-gl:
	cd ship-game && WGPU_BACKEND=gl $(CARGO) run

run-ship-native-vulkan:
	cd ship-game && WGPU_BACKEND=vulkan $(CARGO) run

install:
	cd server && $(CARGO) build

lint: lint-server lint-ship

lint-server:
	cd server && $(CARGO) fmt --check
	cd server && $(CARGO) clippy -- -D warnings

lint-ship:
	cd ship-game && $(CARGO) fmt --check
	cd ship-game && $(CARGO) clippy -- -D warnings

fmt:
	cd server && $(CARGO) fmt
	cd ship-game && $(CARGO) fmt

test:
	cd server && $(CARGO) test
	cd ship-game && $(CARGO) test

dev-server:
	cd server && $(CARGO) run

build:
	./scripts/build-ship.sh
	cd server && $(CARGO) build --release

deploy:
	./build.sh
	fly deploy

fly-db-connect:
	fly mpg connect $(PG_CLUSTER_ID)	

fly-logs:
	fly logs --app timehelm

# Deserialize a saved ship and print CellBox / deck summary (for agents).
inspect-ship:
	cd ship-game && SAVE_SHIP=$(abspath $(SAVE_SHIP)) $(CARGO) test --lib ship_save::tests::inspect_ship_save -- --exact --ignored --nocapture

# Reassign perimeter cabin doors/windows on a save (default: saved_ships/latest.ship.zst).
refresh-ship-sides:
	cd ship-game && SAVE_SHIP=$(abspath $(SAVE_SHIP)) $(CARGO) test --lib ship_save::tests::refresh_ship_sides_save -- --exact --ignored --nocapture

# Write procedural default layout to saved_ships/default.ship.zst (for testing inspect-ship).
write-ship-default:
	@mkdir -p saved_ships client/public/saved_ships
	cd ship-game && $(CARGO) test --lib ship_save::tests::write_default_ship_save -- --exact --nocapture
	@cp -f saved_ships/*.ship.zst client/public/saved_ships/ 2>/dev/null || true
	@cd ship-game && $(CARGO) test --lib ship_save::tests::sync_public_save_manifest -- --exact --nocapture
