import { Application, Sprite, Assets, Text } from 'pixi.js';
    
const app = new Application();

async function main() {

    // Wait for the Renderer to be available
    await app.init();

    // The application will create a canvas element for you that you
    // can then insert into the DOM
    document.body.appendChild(app.canvas);

    // load the texture we need
    const texture = await Assets.load('bunny.png');

    // This creates a texture from a 'bunny.png' image
    const bunny = new Sprite(texture);

    // Setup the position of the bunny
    bunny.x = app.renderer.width / 2;
    bunny.y = app.renderer.height / 2;

    // Rotate around the center
    bunny.anchor.x = 0.5;
    bunny.anchor.y = 0.5;

    // Add the bunny to the scene we are building
    app.stage.addChild(bunny);

    const text = new Text({
        text: 'Hello Pixi!',
        style: {
            fontFamily: 'Arial',
            fontSize: 24,
            fill: 0xff1010,
            align: 'center',
        }
    });
    app.stage.addChild(text);

    // Listen for frame updates
    app.ticker.add(() => {
        // each frame we spin the bunny around a bit
        bunny.x += 0.1;
    });
}

main();