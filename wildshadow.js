/*
 * Originally from https://www.redblobgames.com/x/1634-wildshadow-ama/
 * Copyright 2016 Red Blob Games <redblobgames@gmail.com>
 * License: Apache-2.0 <http://www.apache.org/licenses/LICENSE-2.0.html>
 */




/* GLOBALS */

var map = null;
var camera = {x: 127, y: 154, facing: 'east'}; // in tiles
var admin = (document.location.hostname == "localhost"); // no cheat protection, go ahead



/* Tile types adapted from ~redblobgames/Projects/flash10/mapgen/level_editor.as
 *     [name, color, passable, spritesheet, spriteid, ...]
 * When spritesheet is the string 'tile', spriteid will be another entry in tileTypes
 */
const tileTypes =
    [
        ["Void", 0xffffff, false, output.oryx_env, 0x73],
        ["Wall", 0xecedf9, false, output.oryx_env, 0x04, 'color', "hsla(30,40%,100%,0.5)"],
        ["Floor", 0xd9dcf3, true, output.oryx_env, 0x04, 'color', "hsla(240,10%,60%,0.5)"],
        ["Path", 0xb4b5bc, true, output.oryx_env, 0x60, 'color', "hsla(240,10%,60%,0.5)"],
        ["Water", 0x5278c5, true, output.oryx_env, 0xbc],
        ["Fountain", 0x2b4a8b, true, 'tile', 'Water', output.oryx_obj, 0x4f],
        ["Grass", 0x35863d, true, output.oryx_env, 0x48],
        ["Bench", 0x6b6b6b, true, output.oryx_env, 0x0c, 'color', "hsla(240,20%,70%,0.2)"],
        ["Tree", 0x184a1d, false, 'tile', 'Grass', output.oryx_env, 0x4d],
        ["Rock", 0x3e3e3e, true, output.oryx_env, 0x62],
        ["Gravestone", 0x00ff00, true, 'tile', 'Floor', output.oryx_obj, 0x13],
        ["Portal", 0xff0000, true, 'tile', 'Floor', output.oryx_env, 0x7c],
        ["Chicken", 0xffdcf3, true, 'tile', 'Floor', output.oryx_obj, 0x1d],
        ["Key", 0xd9fff3, true, 'tile', 'Floor', output.oryx_obj, 0x22],
        ["Bag", 0xc6b1ff, true, 'tile', 'Floor', output.oryx_obj, 0x08],
        ["Skull", 0xb1daff, true, 'tile', 'Floor', output.oryx_obj, 0x11],
        ["Cross", 0xfcffb1, true, 'tile', 'Floor', output.oryx_obj, 0x14],
        ["Sheep", 0xf2f2f2, false, 'tile', 'Grass', output.oryx_char, 0xd2],
        ["Money", 0x00af00, true, 'tile', 'Floor', output.oryx_obj, 0x00],
        ["Statue", 0x999999, false, 'tile', 'Floor', output.oryx_char, 0xa1],
        ["Flower_P", 0xff26db, true, 'tile', 'Grass', output.oryx_env, 0x6f],
        ["Flower_E", 0xd2ff1f, true, 'tile', 'Grass', output.oryx_env, 0x6e],
        // NOTE: last item is special, used for AMA markers. 0x8d is yoda, 0xc7 is a monkey, 0xb5 is an eye
        ["Marker", -1, true, 'tile', 'Grass', output.oryx_char, 0xb5],
    ];

// https://www.realmeye.com/wiki/development-and-release-history -- too bad I can't load this in the iframe
var discoveries = 0; // how many of the regions has the player discovered?
const regions = [
    {left: 116, top: 143, right: 125, bottom: 152, message: "In a permadeath game, we wanted deaths to have meaning."},
    {left: camera.x, top: camera.y, right: camera.x, bottom: camera.y, message: "Welcome to the Nexus Museum. Enjoy your visit! WASD or arrows to move around."},
    {left: 132, top: 159, right: 140, bottom: 169, message: "Build 111 introduced the ability to buy keys for gold."},
    {left: 165, top: 153, right: 172, bottom: 158, message: "The creativity of players was amazing. We loved our players. They made art, stories, music, animations, newsletters, guides, sculptures, a wiki, papercrafts, videos, tools. Much of this was lost when Swatsec hacked the forums.", url: "https://web.archive.org/web/20130430134856/https://forums.wildshadow.com/node/198296#a"},
    {left: 146, top: 183, right: 155, bottom: 193, message: "For the game jam entry, we had been given sprite art to use, and we couldn't make our own sprites. We had chicken leg sprites. That's why there's a Chicken Leg of Doom."},
    {left: 149, top: 158, right: 155, bottom: 176, message: "In the first version of the game there was no Nexus and there was only one Realm. Dungeons didn't launch until build 74. The Nexus didn't appear until build 106."},
    {left: 171, top: 177, right: 182, bottom: 184, message: "Permadeath in an MMO? Inconceivable.", url: "http://www.lostgarden.com/2011/06/realm-of-mad-god-released.html"},
    {left: 179, top: 144, right: 187, bottom: 150, message: "Buildings like this were “set pieces” that we added to the map for character (and later, quests)"},
    {left: 103, top: 177, right: 109, bottom: 184, message: "For almost the first two years of the game, when another player died you could loot their gravestones get their items."},
    {left: 150, top: 177, right: 151, bottom: 182, message: "Build 120 introduced fountains that heal you."},
    {left: 127, top: 186, right: 134, bottom: 196, message: "In early versions of the game, all loot was public so you had to grab it before anyone else grabbed it."},
    {left: 37, top: 18, right: 45, bottom: 24, message: "What are you doing here?"},
    {left: 187, top: 206, right: 211, bottom: 212, message: "The world you are in now was an “alternate Nexus” that was in the game for a brief period of time."},
    {left: 44, top: 193, right: 56, bottom: 205, message: "There is no secret sheep level."},
    {left: 60, top: 136, right: 79, bottom: 156, message: "Map design: rivers link the godlands to the beaches. Roads take you to other areas with the same difficulty but different types of monsters."},
    {left: 125, top: 123, right: 133, bottom: 131, message: "The game was originally 3D implemented in software, but later the terrain was made flat to simplify the look and implementation. Walls continued to be 3D.", url: "https://joanofarcrotmg.wordpress.com/2013/03/19/rotmg-is-a-3d-game/"},
    {left: 78, top: 225, right: 86, bottom: 233, message: "The map generator occasionally produced unreachable areas."},
    {left: 109, top: 185, right: 125, bottom: 186, message: "The game was designed to let you play without tedious preliminaries. Other MMOs put installing the game, creating an account, choosing a race/class, customizing your character's appearance, selecting a name, downloading patches, and other steps in your way. We wanted you to be able to click a button and start playing right away instead of waiting half an hour to get started.", url: "https://web.archive.org/web/20130502033340/https://forums.wildshadow.com/node/1087"},
    {left: 167, top: 54, right: 176, bottom: 64, message: "Selling dyes, character slots, keys, health pots, and vault space gave Wild Shadow the funds to continue development."},
    {left: 37, top: 116, right: 50, bottom: 128, message: "Most of what the game offers was originally not designed in. It was meant to be a simple game jam entry where you leveled up from 1 to 20 in an hour. No dungeons, no stat potions, no stars. Oryx spawned in the main map instead of in a dungeon. Players loved the game and wanted more, so Wild Shadow kept adding features and content long after the game jam was over."},
];

// Tutorial was originally shared, so people could meet there, trade, etc. The nexus didn't exist back then.

// This line generated by running ./pick-sheep-locations.py:
var ama_locations =  [[67, 186], [173, 73], [49, 195], [231, 173], [54, 203], [103, 96], [93, 189], [177, 103], [61, 201], [65, 40], [93, 154], [124, 59], [84, 138], [178, 168], [83, 187], [64, 125], [58, 191], [134, 55], [49, 204], [182, 219], [45, 198], [77, 181], [163, 70], [102, 159], [47, 189], [171, 78], [134, 47], [158, 112], [136, 51], [93, 114], [44, 118], [95, 93], [175, 107], [70, 128], [87, 189], [179, 162], [77, 132], [170, 149], [81, 134], [128, 43], [174, 128], [181, 172], [101, 43], [186, 164], [167, 139], [228, 176], [171, 132], [167, 74], [38, 118], [110, 160], [55, 126], [53, 193], [88, 144], [56, 197], [97, 185], [116, 158], [187, 226], [85, 150], [176, 76], [124, 47], [165, 83], [126, 66], [169, 115], [67, 36], [80, 184], [98, 39], [181, 158], [41, 202], [59, 205], [86, 110], [174, 146], [123, 53]] ;

function colorToTile(colorABGR) {
    let colorRGB = (colorABGR & 0xff << 16) | (colorABGR & 0xff00) | (colorABGR & 0xff0000 >> 16);
    var result = {type: 0};
    for (var i = 0; i < tileTypes.length; i++) {
        if (colorRGB == tileTypes[i][1]) {
            result.type = i;
        }
    }
    return result;
}

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
            map[y][x] = colorToTile(imageDataU32[i]);
        }
    }

    wasm_bindgen.set_mapdata(image.width, image.height, imageDataU32);
}

// The map index is something I used during development to make a list
// of interesting positions (knights, chests, sheep, etc.) so that I
// could place items there. See ./pick-sheep-locations.py
function buildMapIndex() {
    // mapIndex[typename] will be [[x,y],[x,y],...]
    var mapIndex = {};

    for (var y = 0; y < map.length; y++) {
        for (var x = 0; x < map[y].length; x++) {
            var typeName = tileTypes[map[y][x].type][0];
            if (!mapIndex[typeName]) { mapIndex[typeName] = []; }
            mapIndex[typeName].push([x, y]);
        }
    }

    return mapIndex;
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



/* **********************************************************************
 * Output
 */

/* Make the player sprite sink into water, deeper if surrounded by water */
function waterDepthAt(x, y) {
    function waterAt(x, y) {
        if (y < 0 || y >= map.length || x < 0 || x >= map[y].length) return 0;
        var tileType = tileTypes[map[y][x].type][0];
        return (tileType == 'Water' || tileType == 'Fountain')? 1 : 0;
    }

    var D = 3, distanceToLand = D;
    for (var dx = -D; dx <= D; dx++) {
        for (var dy = -D; dy <= D; dy++) {
            if (!waterAt(x + dx, y + dy)) {
                var distance = Math.abs(dx) + Math.abs(dy);
                if (distance < distanceToLand) distanceToLand = distance;
            }
        }
    }
    return distanceToLand;
}

function drawMap() {
    if (output.oryx_char.image == null || output.oryx_env.image == null || output.oryx_obj.image == null || map == null) return;

    var view = document.getElementById('view');

    function roundUp(x) { return Math.floor(x/tileSize) * tileSize; }
    var v = {w: roundUp(view.clientWidth), h: roundUp(view.clientHeight)};
    view.width = v.w;
    view.height = v.h;
    
    var ctx = view.getContext('2d');
    ctx.imageSmoothingEnabled = false;

    ctx.scale(zoom, zoom);
    for (var tileY = 0; tileY < map.length; tileY++) {
        for (var tileX = 0; tileX < map[tileY].length; tileX++) {
            var screenX = (tileX-0.5 - camera.x) * tileSize + v.w/zoom/2;
            var screenY = (tileY-0.5 - camera.y) * tileSize + v.h/zoom/2;
            if (screenX > -tileSize && screenX < v.w/zoom+tileSize
                && screenY > -tileSize && screenY < v.h/zoom+tileSize) {
                drawTile(ctx, screenX, screenY, map[tileY][tileX]);
            }
        }
    }
    var playerOffset = waterDepthAt(camera.x, camera.y);
    output.oryx_char.drawSpriteTo(ctx,
                          -0.5*tileSize + v.w/zoom/2, -0.5*tileSize + v.h/zoom/2 + playerOffset,
                           {'east': 0x1e0, 'south': 0x1e1, 'west': 0x1e2, 'north': 0x1e3}[camera.facing],
                           tileSize, tileSize - playerOffset);
    
    ctx.setTransform(1, 0, 0, 1, 0, 0);
}

function updateMessages() {
    var message = "";
    if (admin) { message = "x=" + camera.x + " y=" + camera.y; }
    var iframeSrc = "";
    for (var i = 0; i < regions.length; i++) {
        if (regions[i].left <= camera.x && camera.x <= regions[i].right
            && regions[i].top <= camera.y && camera.y <= regions[i].bottom) {
            if (!regions[i].discovered) {
                regions[i].discovered = true;
                discoveries++;
            }
            message = regions[i].message;
            iframeSrc = regions[i].url || "";
        }
    }

    document.getElementById('message').textContent = message;
}
