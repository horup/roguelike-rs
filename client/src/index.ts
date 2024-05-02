import { Application, Sprite, Assets, Text, Texture, Container } from 'pixi.js';
import { Message } from '../../server/bindings/Message';
const app = new Application();

interface WSEvent {
    connected?:boolean;
    disconnected?:boolean;
    msg?:Message;
}
let wsEvents = [] as WSEvent[]; 
let socket:WebSocket = null;
function openWebSocket() {
    if (socket != null) {
        socket.onopen = null;
    }
    socket = new WebSocket("ws://localhost:8080");
    socket.onopen = ()=>{
        wsEvents.push({connected:true});
    }
    socket.onclose = ()=>{
        wsEvents.push({disconnected:true});
        setTimeout(()=>{
            openWebSocket();
        }, 2000);
    }
    socket.onmessage = (ev)=>{
        wsEvents.push({msg:JSON.parse(ev.data)});
    }
}
function sendMessage(msg:Message) {
    socket.send(JSON.stringify(msg));
}

let world:Container = new Container();
let textures = {} as {[key:string]:Texture}
let tiles = {} as {[index:string]:Sprite}

function update() {
    for (let ev of wsEvents) {
        if (ev.connected) {
            console.log("connected");
            sendMessage({
                joinAsPlayer:{
                    id:self.crypto.randomUUID(),
                    name:"Player"
                }
            })
        }
        else if (ev.disconnected) {
            console.log("disconnected");
        }
        else if (ev.msg) {
            let msg = ev.msg;
            if ('tileUpdate' in msg) {
                let tu = msg.tileUpdate;
                let [x, y] = [tu.index[0], tu.index[1]];
                let index = tu.index.toString();
                if (tiles[index] == null) {
                    let texture = tu.wall ? 'wall' : 'floor';
                    let t = textures[texture];
                    const sprite = new Sprite();
                    sprite.x = x;
                    sprite.y = y;
                    world.addChild(sprite);
                    tiles[index] = sprite;
                }
                let sprite = tiles[index];
                let texture = tu.wall ? 'wall' : 'floor';
                let t = textures[texture];
                sprite.texture = t;
                sprite.width = 1;
                sprite.height = 1;
            }
        }
    }
    wsEvents = [];
}

async function main() {
    // Wait for the Renderer to be available
    await app.init();
    world.scale.set(16.0);

    // The application will create a canvas element for you that you
    // can then insert into the DOM
    document.body.appendChild(app.canvas);

    // load the texture we need
    //const texture = await Assets.load('assets/imgs/bunny.png');

    // This creates a texture from a 'bunny.png' image
    textures["floor"] = await Assets.load('assets/imgs/floor.png');
    textures["player"] = await Assets.load('assets/imgs/player.png');
    textures["wall"] = await Assets.load('assets/imgs/wall.png');
    textures["door"] = await Assets.load('assets/imgs/door.png');
    console.log(textures);
    /*const bunny = new Sprite(texture);

    // Setup the position of the bunny
    bunny.x = app.renderer.width / 2;
    bunny.y = app.renderer.height / 2;

    // Rotate around the center
    bunny.anchor.x = 0.5;
    bunny.anchor.y = 0.5;

    // Add the bunny to the scene we are building
    app.stage.addChild(bunny);*/

    const text = new Text({
        text: 'Hello Pixi!',
        style: {
            fontFamily: 'Arial',
            fontSize: 24,
            fill: 0xff1010,
            align: 'center',
        }
    });
    app.stage.addChild(world);
    app.stage.addChild(text);

    openWebSocket();

    // Listen for frame updates
    app.ticker.add(() => {
        update();
    });
}

main();