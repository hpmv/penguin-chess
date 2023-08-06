import React, { useContext } from 'react';
import { HistoryContext } from './history';
import { Board } from './board';

export const HistoryView = ({ }) => {
    const { history, dispatch } = useContext(HistoryContext);
    const reversed = [...history.boards];
    reversed.reverse();

    return <div className="history-list">
        {reversed.map(({ board, move }, ri) => {
            const i = history.boards.length - 1 - ri;
            return <div
                key={i}
                className={`history-entry ${i === history.currentIndex ? 'selected' : ''}`}
                onClick={() => dispatch({ select: i })}>
                <Board className="history-board" board={board} lastMove={move} canMove={false} turnNumber={i}></Board>
            </div>
        })}
    </div>
};