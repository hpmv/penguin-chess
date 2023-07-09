import { Engine } from "wynn";
// import wasm from "../pkg/wynn_bg.wasm";

async function test() {
    // await init(wasm);

    const engine = new Engine((data) => {
        console.log(data);
    });

    setTimeout(() => engine.stop(), 2000);
    console.log(engine.find_best_move(new Uint8Array([0, 1, 3, 4, 20, 21, 23, 24, 22, 2, 1])));

}
test();