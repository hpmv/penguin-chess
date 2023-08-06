import { createContext } from "react";
import { applyMove } from "./move";

export function reduceHistory(oldHistory, action) {
    const history = {
        boards: [...oldHistory.boards],
        currentIndex: oldHistory.currentIndex,
    };
    if (action.move) {
        history.boards.splice(history.currentIndex + 1);
        history.boards.push({ move: action.move, board: applyMove(history.boards[history.currentIndex].board, action.move) });
        history.currentIndex++;
    } else if (action.select != undefined) {
        history.currentIndex = action.select;
    } else if (action.newGame) {
        history.boards = [{ board: action.newGame, move: undefined }];
        history.currentIndex = 0;
    }
    console.log(action);
    console.log(history);
    return history;
}

export const HistoryContext = createContext({
    boards: [], currentIndex: -1
});
