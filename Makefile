# Makefile builds a wasm client library, a mac server binary, and a linux server binary

# RELEASECLIENT = --release
BUILD = /Users/amitp/Sites/redblobgames/x/z

.PHONY: clean
RS_SRC = $(shell find src -type f -name '*.rs') Cargo.toml
BUILDTYPE = $(if $(RELEASECLIENT),release,debug)
WASM = target/wasm32-unknown-unknown/$(BUILDTYPE)/rust_chat_server.wasm
MAC_SERVER = target/debug/chat_server
LINUX_SERVER = target/x86_64-unknown-linux-musl/release/chat_server

all: $(BUILD)/rust_chat_server_bg.wasm $(MAC_SERVER) $(LINUX_SERVER)

run-server: $(MAC_SERVER)
	RUST_BACKTRACE=1 cargo run --bin chat_server

$(MAC_SERVER): $(RS_SRC)
	cargo build --bin chat_server

$(LINUX_SERVER): $(RS_SRC)
	TARGET_CC=x86_64-linux-musl-gcc cargo build --release --target=x86_64-unknown-linux-musl

$(WASM): $(RS_SRC)
	cargo build --lib --target wasm32-unknown-unknown $(RELEASECLIENT)

$(BUILD)/rust_chat_server_bg.wasm: $(WASM) index.html
	wasm-bindgen --target no-modules $< --out-dir $(BUILD)
	mkdir -p $(BUILD)
	cp index.html $(LINUX_SERVER) $(BUILD)/

clean:
	cargo clean
