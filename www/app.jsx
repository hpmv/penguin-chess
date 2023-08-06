import React, { useReducer, useEffect, useState, useCallback } from "react";
import { Board } from "./board";
import "./site.scss";
import { HistoryContext, reduceHistory } from "./history";
import { HistoryView } from "./HistoryView";
import { applyMove } from "./move";

export const App = ({ }) => {
    const [history, dispatch] = useReducer(reduceHistory, {
        boards: [],
        currentIndex: 0
    });
    const [thoughts, setThoughts] = useState([]);
    const [thinking, setThinking] = useState(false);
    const [stopBuffer, setStopBuffer] = useState(null);
    const [collectFirstMoveScores, setCollectFirstMoveScores] = useState(false);
    const board = history.boards[history.currentIndex]?.board;
    useEffect(() => {
        if (!board) {
            return;
        }
        const stop = new SharedArrayBuffer(1);
        setStopBuffer(stop);
        const worker = window.worker;
        worker.postMessage({
            search: board,
            stop,
            collectFirstMoveScores,
            historyStates: history.boards.slice(0, history.currentIndex + 1).map(({ board }) => board).flat(),
        });
        worker.onmessage = ({ data: msg }) => {
            if (msg.info) {
                setThoughts(thoughts => [JSON.parse(msg.info), ...thoughts]);
            }
        };
        setThinking(true);
        return () => {
            Atomics.store(new Uint8Array(stop), 0, 1);
            setThinking(false);
            setThoughts([]);
            setStopBuffer(null);
            worker.onmessage = undefined;
        };
    }, [board, collectFirstMoveScores]);

    let thinkingMove = null;
    if (thoughts.length > 0) {
        const thought = thoughts[0];
        thinkingMove = thought.result.best_path[thought.result.best_path.length - 1];
    }

    const stopThinking = useCallback(() => {
        Atomics.store(new Uint8Array(stopBuffer), 0, 1);
    }, [stopBuffer]);

    return (
        <HistoryContext.Provider value={{ history, dispatch }}>
            <div className="app-container">
                <div className="controls">
                    <button onClick={() => dispatch({ newGame: [0, 1, 3, 4, 20, 21, 23, 24, 22, 2, 1] })}>New Game</button>
                    <button onClick={stopThinking} disabled={!thinking}>Stop Thinking</button>
                    <button onClick={() => dispatch({ move: thinkingMove })} disabled={thinkingMove === null}>
                        Make move {thinkingMove == null ? "?" : `${thinkingMove.from} -> ${thinkingMove.to}`}
                    </button>
                    <button onClick={() => setCollectFirstMoveScores(x => !x)}>
                        Collect First Move Scores (slower): {collectFirstMoveScores ? "YES" : "no"}
                    </button>
                </div>
                <div className="top-panel">
                    {history.boards.length > 0 && <div className="top-panel-board">
                        <Board board={board} onMove={(move) => dispatch({ move })} canMove={true}></Board>
                    </div>}
                    <HistoryView></HistoryView>
                </div>
                <div className="bottom-panel">
                    <div className="thoughts">
                        {thoughts.map(({
                            depth,
                            nodes_searched: nodesSearched,
                            transposition_table_size: transpositionTableSize,
                            result: {
                                score,
                                best_path: bestPath
                            } }) => {
                            return <div key={depth} className="thought">
                                <div className="depth-header">
                                    Depth {depth}, Score: {score} Nodes: {humanDisplay(nodesSearched)} Transposition Table: {humanDisplay(transpositionTableSize)}
                                </div>
                                <div className="thought-boards">
                                    {(() => {
                                        const boards = [{ board, move: undefined }];
                                        for (let i = bestPath.length - 1; i >= 0; i--) {
                                            boards.push({ move: bestPath[i], board: applyMove(boards[boards.length - 1].board, bestPath[i]) });
                                        }
                                        const rendered = [];
                                        for (let i = 1; i < boards.length; i++) {
                                            rendered.push(
                                                <div className="thought-board">
                                                    <Board board={boards[i].board} lastMove={boards[i].move} canMove={false}
                                                        turnNumber={history.currentIndex + i}></Board>
                                                </div>
                                            );
                                        }
                                        return rendered;
                                    })()}
                                </div>
                            </div>;
                        })}
                    </div>
                </div>
            </div>
        </HistoryContext.Provider>
    );
};

function humanDisplay(n) {
    if (n < 1000) {
        return '' + n;
    }
    if (n < 1000000) {
        return (Math.floor(n / 100) / 10) + 'K';
    }
    return (Math.floor(n / 100000) / 10) + 'M';
}