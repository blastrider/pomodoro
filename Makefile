.PHONY: fmt clippy test build run

fmt:
	cargo fmt

clippy:
	cargo clippy -- -D warnings

test:
	cargo test

build:
	cargo build --release

run:
	cargo run -- $(ARGS)
