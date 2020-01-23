/*
 * From https://www.redblobgames.com/x/2003-rust-network-game/
 * Copyright 2020 Red Blob Games <redblobgames@gmail.com>
 * License: Apache-2.0 <http://www.apache.org/licenses/LICENSE-2.0.html>
 */
'use strict';

/* global wasm_bindgen, WebSocket, FileReader */

const KEY_ENTER = 13;

let connection;

class Connection {
    constructor(url) {
        this.socket = new WebSocket(url);
        
        this.socket.onopen = event => {
            wasm_bindgen.connected();
            document.querySelector("#input").focus();
        };
        
        this.socket.onmessage = event => {
            let fileReader = new FileReader();
            fileReader.onload = e => wasm_bindgen.handle_message(new Uint8Array(e.target.result));
            fileReader.readAsArrayBuffer(event.data);
        };
        
        this.socket.onclose = event => {
            output.push("SYSTEM", `Connection closed\n{code ${event.code} reason ${event.reason}}`);
            output.set_connection_count("no");
        };
        
        this.socket.onerror = error => {
            output.push("SYSTEM", "Error (is the server running?)");
            output.set_connection_count("error");
        };
    }

    send_to_server(bytes) {
        this.socket.send(bytes);
    }
}

const widgets = {
    view: document.getElementById('view'),
    input: document.querySelector("#input"),
};

const output = {
    set_name(name) {
        const span = document.querySelector("#name");
        span.textContent = name;
    },

    set_connection_count(count) {
        const span = document.querySelector("#count");
        span.textContent = count;
    },

    push(from, text) {
        const pre = document.querySelector("#output");
        pre.textContent += `[${from}]: ${text}\n`;
    },
};

const input = {
};


/* Send events to the Rust side:
 *
 * 1. Focus can be on either the map or the input box.
 * 2. While the map has focus, every keystroke is sent to Rust.
 * 3. While the input box has focus, entire lines are sent to Rust.
 * 4. Pressing Enter switches focus between map and input box.
 *
 */
widgets.view.addEventListener('keydown', event => {
    if (event.keyCode === KEY_ENTER) {
        widgets.input.focus();
        event.preventDefault();
    } else {
        wasm_bindgen.handle_keydown(event.keyCode) && event.preventDefault();
    }
});

widgets.view.addEventListener('keyup', event => {
    wasm_bindgen.handle_keyup(event.keyCode) && event.preventDefault();
});

function formSubmit() { // called when pressing Enter on text input box
    if (widgets.input.value.length > 0) {
        wasm_bindgen.handle_text_entry(widgets.input.value);
    }
    widgets.input.value = "";
    widgets.input.blur();
    widgets.view.focus();
    return false;
}

let gameLoop = {
    TICKS_PER_SECOND: 10,
    lastTime: Date.now(),
    loop() {
        var time = Date.now();
        if (time - this.lastTime > 1000/this.TICKS_PER_SECOND) {
            this.lastTime = time;
            wasm_bindgen.game_loop();
            checkFocus();
        }
        requestAnimationFrame(gameLoop.loop.bind(gameLoop));
    },
};

// We need keyboard focus; this is a hack to tell the player to click
function checkFocus() {
    const messageBox = document.getElementById('message2');
    const activeId = document.activeElement && document.activeElement.id;
    if (activeId === 'view') {
        messageBox.textContent = "WASD or arrow keys to move; Enter to chat";
    } else if (activeId === 'input') {
        messageBox.textContent = "Enter text to chat";
    } else {
        messageBox.textContent = "Click on map to focus";
    }
}
window.addEventListener('click', checkFocus, true);
window.addEventListener('focusin', checkFocus, true);
window.addEventListener('focusout', checkFocus, true);




wasm_bindgen("game_client_bg.wasm")
    .then(() => {
        connection = new Connection(
            window.location.hostname==='localhost'
                ? "ws://localhost:9001/"
                : "wss://www.redblobgames.com/ws/"
        );
        gameLoop.loop();
    });
