.PHONY: all build test clean bench

all: build test

build:
	cargo build --release --target wasm32-unknown-unknown

test:
	cargo test --workspace

bench:
	cargo bench

clean:
	cargo clean
