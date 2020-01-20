/*
 * From https://www.redblobgames.com/x/2003-rust-network-game/
 * Copyright 2020 Red Blob Games <redblobgames@gmail.com>
 * License: Apache-2.0 <http://www.apache.org/licenses/LICENSE-2.0.html>
 */
'use strict';

/* global wasm_bindgen, WebSocket, FileReader */

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


const output = {
    view: document.getElementById('view'),
    
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


// Send events to the Rust side ; TODO: should distinguish
// between text input area having focus and not
output.view.addEventListener('keydown', event => {
    wasm_bindgen.handle_keydown(event.keyCode);
});
output.view.addEventListener('keyup', event => {
    wasm_bindgen.handle_keyup(event.keyCode);
});


// We need keyboard focus; this is a hack to tell the player to click
function checkFocus() {
    const focusMessage = "Click to focus";
    const messageBox = document.getElementById('message');
    if (document.hasFocus) {
        if (!document.hasFocus()) {
            messageBox.textContent = focusMessage;
        } else if (messageBox.textContent == focusMessage) {
            messageBox.textContent = "";
        }
    }
}
window.addEventListener('click', checkFocus, true);
window.addEventListener('focusin', checkFocus, true);
window.addEventListener('focusout', checkFocus, true);



function formSubmit() {
    let input = document.querySelector("#input");
    if (input.value.length > 0) {
        wasm_bindgen.handle_input(input.value);
    }
    input.value = "";
    return false;
}


wasm_bindgen("game_client_bg.wasm")
    .then(() => {
        connection = new Connection(
            window.location.hostname==='localhost'
                ? "ws://localhost:9001/"
                : "wss://www.redblobgames.com/ws/"
        );
    });
