# Makefile builds a wasm client library, a mac server binary, and a linux server binary

# CLIENT_RELEASE = --release

.SUFFIXES:
.PHONY: clean
RS_SRC = $(shell find src -type f -name '*.rs') Cargo.toml
BUILDTYPE = $(if $(CLIENT_RELEASE),release,debug)
_MKDIRS := $(shell mkdir -p build)
CLIENT_LIB = target/wasm32-unknown-unknown/$(BUILDTYPE)/librust_network_game.rlib
CLIENT_WASM = target/wasm32-unknown-unknown/$(BUILDTYPE)/game_client.wasm
SERVER_MAC = target/debug/game_server
SERVER_LINUX = target/x86_64-unknown-linux-musl/debug/game_server

all: build/game_client.wasm build/game_server $(SERVER_MAC)

run-server: $(SERVER_MAC)
	RUST_BACKTRACE=1 $(SERVER_MAC)

$(SERVER_MAC): $(RS_SRC)
	cargo build --bin game_server

$(SERVER_LINUX): $(RS_SRC)
	TARGET_CC=x86_64-linux-musl-gcc cargo build --target=x86_64-unknown-linux-musl

$(CLIENT_LIB): $(RS_SRC)
	cargo build --lib --target wasm32-unknown-unknown $(CLIENT_RELEASE)

$(CLIENT_WASM): $(CLIENT_LIB)
	cargo build --bin game_client --target wasm32-unknown-unknown $(CLIENT_RELEASE)

build/game_server: $(SERVER_LINUX)
	cp $< $@

build/game_client.wasm: $(CLIENT_WASM) embed.html
	wasm-bindgen --target no-modules $< --out-dir build/
	cp embed.html game_ui.js build/

clean:
	cargo clean
	rm -rf target/rls
