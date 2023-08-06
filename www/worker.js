import { Engine } from "penguin";

self.onmessage = ({ data: msg }) => {
    console.log(msg);
    const engine = new Engine(
        new Uint8Array(msg.stop),
        (data) => {
            console.log(data);
            self.postMessage({ info: data });
        });

    const move = engine.find_best_move(new Uint8Array(msg.search), msg.collectFirstMoveScores, msg.historyStates);
    console.log(move);
    self.postMessage({
        move,
    });
}
