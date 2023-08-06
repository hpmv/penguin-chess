export function applyMove(board, move) {
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

    flat[move.to] = flat[move.from];
    flat[move.from] = '';
    const whites = [];
    const blacks = [];
    let whiteKing, blackKing;
    for (let i = 0; i < 25; i++) {
        if (flat[i] == 'wp') {
            whites.push(i);
        } else if (flat[i] == 'bp') {
            blacks.push(i);
        } else if (flat[i] == 'wk') {
            whiteKing = i;
        } else if (flat[i] == 'bk') {
            blackKing = i;
        }
    }
    return [...whites, ...blacks, whiteKing, blackKing, isWhitesTurn ? 0 : 1];
}
