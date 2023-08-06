import React, { useState, useCallback } from 'react';
import { blackKing, blackPawn, whiteKing, whitePawn } from './pieces';

const symbol = {
    'wp': whitePawn,
    'bp': blackPawn,
    'wk': whiteKing,
    'bk': blackKing
};

const ALL_DIRS = [
    [-1, -1],
    [-1, 0],
    [-1, 1],
    [0, -1],
    [0, 1],
    [1, -1],
    [1, 0],
    [1, 1],
];

export const Board = ({ board, lastMove, onMove, canMove, turnNumber }) => {
    const flat = [];
    for (let i = 0; i < 25; i++) {
        flat.push('');
    }
    flat[board[0]] = 'wp';
    flat[board[1]] = 'wp';
    flat[board[2]] = 'wp';
    flat[board[3]] = 'wp';
    flat[board[4]] = 'bp';
    flat[board[5]] = 'bp';
    flat[board[6]] = 'bp';
    flat[board[7]] = 'bp';
    flat[board[8]] = 'wk';
    flat[board[9]] = 'bk';
    const isWhitesTurn = board[10] == 1;

    const [selectedPiece, setSelectedPiece] = useState(null);
    const eligibleTargets = new Set();
    if (selectedPiece !== null) {
        for (const [dx, dy] of ALL_DIRS) {
            let target = selectedPiece;
            while (true) {
                let [i, j] = [Math.floor(target / 5), target % 5];
                let [ii, jj] = [i + dx, j + dy];
                if (ii >= 0 && ii < 5 && jj >= 0 && jj < 5 && flat[ii * 5 + jj] === '') {
                    target = ii * 5 + jj;
                } else {
                    break;
                }
            }
            if (target == 12 && (flat[selectedPiece] != 'bk' && flat[selectedPiece] != 'wk')) {
                continue;
            }
            if (target !== selectedPiece) {
                eligibleTargets.add(target);
            }
        }
    }

    const clickHandler = useCallback((e) => {
        const index = parseInt(e.currentTarget.getAttribute('pieceindex'));
        if (selectedPiece !== null) {
            if (eligibleTargets.has(index)) {
                onMove({ from: selectedPiece, to: index });
                setSelectedPiece(null);
                return;
            }
        }
        if (flat[index] != '' && selectedPiece !== index) {
            if (isWhitesTurn && (flat[index] == 'wp' || flat[index] == 'wk') ||
                !isWhitesTurn && (flat[index] == 'bp' || flat[index] == 'bk')) {
                setSelectedPiece(index);
                return;
            }
        }
        setSelectedPiece(null);
    }, [board, selectedPiece])

    return <div className="board">
        {[0, 1, 2, 3, 4].map(i => {
            return <div key={i} className="board-row">
                {[0, 1, 2, 3, 4].map(j => {
                    const index = i * 5 + j;
                    return <div key={j} className={
                        `board-cell
                        ${canMove && (isWhitesTurn && (flat[index] == 'wp' || flat[index] == 'wk') ||
                            !isWhitesTurn && (flat[index] == 'bp' || flat[index] == 'bk')) ? 'can-move-cell' : ''}
                        ${lastMove?.from === index ? 'from-cell' : ''}
                        ${lastMove?.to === index ? 'to-cell' : ''}
                        ${eligibleTargets.has(index) ? 'candidate-cell' : ''}
                        ${selectedPiece === index ? 'selected-cell' : ''}`}
                        pieceindex={index}
                        onClick={clickHandler}>
                        {flat[index] != '' &&
                            <div
                                className={`cell-piece cell-piece-${flat[index]}`}
                                dangerouslySetInnerHTML={{ __html: symbol[flat[index]] }}>
                            </div>}
                    </div>
                })}
            </div>
        })}
        {lastMove && <svg className="move-arrow" width="200" height="200">
            <line x1={20 + 40 * (lastMove.from % 5)} y1={20 + 40 * Math.floor(lastMove.from / 5)}
                x2={20 + 40 * (lastMove.to % 5)} y2={20 + 40 * Math.floor(lastMove.to / 5)}
                style={{ strokeWidth: '2px', stroke: 'green' }}></line>
        </svg>}
        <div className="turn-indicator">
            {turnNumber !== null && <div className="turn-number">{turnNumber}</div>}
            <div className={isWhitesTurn ? 'white-turn' : 'black-turn'}></div>
        </div>
    </div>;
};
