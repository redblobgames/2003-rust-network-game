/*
 * From https://www.redblobgames.com/x/2003-rust-network-game/
 * Copyright 2020 Red Blob Games <redblobgames@gmail.com>
 * License: Apache-2.0 <http://www.apache.org/licenses/LICENSE-2.0.html>
 */
'use strict';

/* global wasm_bindgen, WebSocket, FileReader */

let socket;

function send_to_server(bytes) {
    socket.send(bytes);
}

function set_name(name) {
    const span = document.querySelector("#name");
    span.textContent = name;
}

function set_connection_count(count) {
    const span = document.querySelector("#count");
    span.textContent = count;
}

function add_to_output(from, text) {
    const pre = document.querySelector("#output");
    pre.textContent += `[${from}]: ${text}\n`;
}

function sendText() {
    let input = document.querySelector("#input");
    if (input.value.length > 0) {
        wasm_bindgen.handle_input(input.value);
    }
    input.value = "";
    return false;
}

function setUpWebSocket() {
    socket = new WebSocket(window.location.hostname==='localhost'? "ws://localhost:9001/" : "wss://www.redblobgames.com/ws/");
    
    socket.onopen = event => {
        wasm_bindgen.connected();
        document.querySelector("#input").focus();
    };
    
    socket.onmessage = event => {
        let fileReader = new FileReader();
        fileReader.onload = e => wasm_bindgen.handle_message(new Uint8Array(e.target.result));
        fileReader.readAsArrayBuffer(event.data);
    };
    
    socket.onclose = event => {
        add_to_output("SYSTEM", `Connection closed\n{code ${event.code} reason ${event.reason}}`);
        set_connection_count("no");
    };
    
    socket.onerror = error => {
        add_to_output("SYSTEM", "Error (is the server running?)");
        set_connection_count("error");
    };
}

wasm_bindgen("game_client_bg.wasm")
    .then(setUpWebSocket);
