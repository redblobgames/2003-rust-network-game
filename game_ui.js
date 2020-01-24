/*
 * From https://www.redblobgames.com/x/2003-rust-network-game/
 * Copyright 2020 Red Blob Games <redblobgames@gmail.com>
 * License: Apache-2.0 <http://www.apache.org/licenses/LICENSE-2.0.html>
 */
'use strict';

/* global wasm_bindgen, WebSocket, FileReader, Image */

const KEY_ENTER = 13;
const ADMIN = (document.location.hostname === "localhost"); // no cheat protection, go ahead
const tileSize = 8;
const zoom = 3;

/* globals: */
let map = null;
let camera = {x: 127, y: 154, facing: 'east'}; // in tiles
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


class Spritesheet {
    image = null;
    
    loadImage(url) {
        return loadImage(url).then(image => { this.image = image; });
    }

    drawSpriteTo(ctx, x, y, spriteId, tileWidth=tileSize, tileHeight=tileSize) {
        var row = spriteId >> 4;
        var col = spriteId & 0x0f;
        ctx.drawImage(this.image,
                      col*tileSize, row*tileSize,
                      tileWidth, tileHeight,
                      x, y,
                      tileWidth, tileHeight);
    }
}

const output = {
    oryx_env: new Spritesheet(),
    oryx_obj: new Spritesheet(),
    oryx_char: new Spritesheet(),
    
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


function setMapDataFromImage(image) {
    const canvas = document.createElement('canvas');
    canvas.width = image.width;
    canvas.height = image.height;
    const ctx = canvas.getContext('2d');
    ctx.drawImage(image, 0, 0);

    var imageDataU32 = new Uint32Array(ctx.getImageData(0, 0, image.width, image.height).data.buffer);
    map = [];
    for (var y = 0; y < image.height; y++) {
        map[y] = [];
        for (var x = 0; x < image.width; x++) {
            var i = image.width * y + x;
            const colorABGR = imageDataU32[i] & 0xffffff;
            map[y][x] = colorToTile(imageDataU32[i]);
        }
    }

    wasm_bindgen.set_mapdata(image.width, image.height, imageDataU32);
}


function drawTile(ctx, x, y, tile) {
    for (var i = 3; i+1 < tileTypes[tile.type].length; i += 2) {
        if (tileTypes[tile.type][i] == 'color') {
            ctx.fillStyle = tileTypes[tile.type][i+1];
            ctx.fillRect(x, y, tileSize, tileSize);
        } else if (tileTypes[tile.type][i] == 'tile') {
            for (var j = 0; j < tileTypes.length; j++) {
                if (tileTypes[j][0] == tileTypes[tile.type][i+1]) {
                    drawTile(ctx, x, y, {type: j});
                }
            }
        } else {
            var spritesheet = tileTypes[tile.type][i];
            var spriteId = tileTypes[tile.type][i+1];
            spritesheet.drawSpriteTo(ctx, x, y, spriteId);
        }
    }
}


function draw_map(playerFacing, playerX, playerY, otherPlayerPos) {
    const FACING = ['north', 'east', 'south', 'west'];
    camera.facing = FACING[playerFacing];
    camera.x = playerX;
    camera.y = playerY;
    function roundUp(x) { return Math.floor(x/tileSize) * tileSize; }
    let v = {w: roundUp(widgets.view.clientWidth), h: roundUp(widgets.view.clientHeight)};
    widgets.view.width = v.w;
    widgets.view.height = v.h;
    
    const ctx = widgets.view.getContext('2d');
    ctx.imageSmoothingEnabled = false;

    ctx.scale(zoom, zoom);
    for (let tileY = 0; tileY < map.length; tileY++) {
        for (let tileX = 0; tileX < map[tileY].length; tileX++) {
            let screenX = (tileX-0.5 - camera.x) * tileSize + v.w/zoom/2;
            let screenY = (tileY-0.5 - camera.y) * tileSize + v.h/zoom/2;
            if (screenX > -tileSize && screenX < v.w/zoom+tileSize
                && screenY > -tileSize && screenY < v.h/zoom+tileSize) {
                drawTile(ctx, screenX, screenY, map[tileY][tileX]);
            }
        }
    }
    for (let i = 0; i < otherPlayerPos.length; i += 3) {
        let x = otherPlayerPos[i], y = otherPlayerPos[i+1], facing = FACING[otherPlayerPos[i+2]];
        let screenX = (x-0.5 - camera.x) * tileSize + v.w/zoom/2;
        let screenY = (y-0.5 - camera.y) * tileSize + v.h/zoom/2;
        if (screenX > -tileSize && screenX < v.w/zoom+tileSize
            && screenY > -tileSize && screenY < v.h/zoom+tileSize) {
            const offset = waterDepthAt(x, y);
            output.oryx_char.drawSpriteTo(ctx,
                                          screenX, screenY + offset,
                                          {'east': 0x1dc, 'south': 0x1dd, 'west': 0x1de, 'north': 0x1df}[facing],
                                          tileSize, tileSize - offset);
        }
    }

    const playerOffset = waterDepthAt(camera.x, camera.y);
    output.oryx_char.drawSpriteTo(ctx,
                          -0.5*tileSize + v.w/zoom/2, -0.5*tileSize + v.h/zoom/2 + playerOffset,
                           {'east': 0x1e0, 'south': 0x1e1, 'west': 0x1e2, 'north': 0x1e3}[camera.facing],
                           tileSize, tileSize - playerOffset);

    
    ctx.setTransform(1, 0, 0, 1, 0, 0);
}



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



function loadImage(url) {
    return new Promise((resolve, reject) => {
        const image = new Image();
        image.onload = () => resolve(image);
        image.onerror = () => reject(new Error(`failed loading ${url}`));
        image.src = url;
    });
}


Promise.all([
    output.oryx_env.loadImage("assets/lofi_environment_a.png"),
    output.oryx_obj.loadImage("assets/lofi_obj_a.png"),
    output.oryx_char.loadImage("assets/lofi_char_a.png"),
    wasm_bindgen("game_client_bg.wasm")
]).then(() => {
    loadImage("assets/nexus-init.png").then(image => {
        setMapDataFromImage(image);
        connection = new Connection(
            window.location.hostname==='localhost'
                ? "ws://localhost:9001/"
                : "wss://www.redblobgames.com/ws/"
        );
        gameLoop.loop();
    });
});
