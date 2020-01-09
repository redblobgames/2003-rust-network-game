# Makefile builds both a wasm library (for use by a client) and a server binary

# RELEASECLIENT = --release
BUILD = /Users/amitp/Sites/redblobgames/x/z

.PHONY: clean
RS_SRC = $(shell find src -type f -name '*.rs') Cargo.toml
BUILDTYPE = $(if $(RELEASECLIENT),release,debug)
WASM = target/wasm32-unknown-unknown/$(BUILDTYPE)/rust_chat_server.wasm

all: $(BUILD)/rust_chat_server_bg.wasm target/debug/chat_server

run-server:
	RUST_BACKTRACE=1 cargo run --bin chat_server

target/debug/chat_server: $(RS_SRC)
	cargo build --bin chat_server

$(WASM): $(RS_SRC)
	cargo build --lib --target wasm32-unknown-unknown $(RELEASECLIENT)

$(BUILD)/rust_chat_server_bg.wasm: $(WASM) index.html
	wasm-bindgen --target no-modules $(WASM) --out-dir $(BUILD)
	cp index.html $(BUILD)/
	ls -l $(BUILD)/*.wasm

clean:
	cargo clean
