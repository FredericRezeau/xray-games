default: build

all: test

test: build
	cargo test

build:
	stellar contract build
	@ls -l target/wasm32-unknown-unknown/release/*.wasm

build_cargo:
	rustup target add wasm32v1-none
	cargo build --target wasm32v1-none --release

fmt:
	cargo fmt --all

clean:
	cargo clean
