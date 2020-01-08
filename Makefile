# Makefile builds both a wasm library (for use by a client) and a server binary

.PHONY: clean
RS_SRC = $(shell find src/ -type f -name '*.rs') Cargo.toml
WASM = target/wasm32-unknown-unknown/debug/rust_chat_server.wasm
BUILD = /Users/amitp/Sites/redblobgames/x/z

all: $(BUILD)/rust_chat_server_bg.wasm target/debug/chat_server

run-server:
	cargo run --bin chat_server

target/debug/chat_server: $(RS_SRC)
	cargo build --bin chat_server

$(WASM): $(RS_SRC)
	cargo build --lib --target wasm32-unknown-unknown

$(BUILD)/rust_chat_server_bg.wasm: $(WASM) index.html
	wasm-bindgen --target no-modules $(WASM) --out-dir $(BUILD)
	cp index.html $(BUILD)/

clean:
	cargo clean
