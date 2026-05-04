.PHONY: dev build deploy install lint lint-server fmt build-ship run-ship build-ship-native run-ship-native run-ship-native-gl run-ship-native-vulkan \
	gh-check ci ci-list ci-here ci-watch ci-watch-here ci-view ci-log ci-status

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
	cd server && cargo run

build-ship-native:
	cd ship-game && cargo build

run-ship-native:
	cd ship-game && WGPU_BACKEND=$(WGPU_BACKEND) cargo run

run-ship-native-gl:
	cd ship-game && WGPU_BACKEND=gl cargo run

run-ship-native-vulkan:
	cd ship-game && WGPU_BACKEND=vulkan cargo run

install:
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
