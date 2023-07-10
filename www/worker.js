import { Engine } from "wynn";
// import wasm from "../pkg/wynn_bg.wasm";

self.onmessage = ({ data: msg }) => {
    console.log(msg);
    const engine = new Engine(
        new Uint8Array(msg.stop),
        (data) => {
            console.log(data);
        });

    console.log(engine.find_best_move(new Uint8Array(msg.search)));

}
