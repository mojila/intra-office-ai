@PHONY: all

setup:
	cd web && npm install
build:
	cd web && npm run build
run-web:
	cd web && npm run dev
run:
	cargo run