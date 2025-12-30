.PHONY: dev build deploy install

install:
	cd client && npm install
	cd server && cargo build

dev-server:
	cd server && cargo run

dev-client:
	cd client && npm run dev

build:
	cd client && npm install && npm run build
	cd server && cargo build --release

deploy:
	./build.sh
	fly deploy

