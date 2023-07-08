use std::{
    fmt::{Debug, Formatter},
    path::Display,
};

use crate::cell::CellState;

#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Board2 {
    data: u64,
}

impl Debug for Board2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let board = self.flatten();

        for row in 0..5 {
            for col in 0..5 {
                let cell = board[row * 5 + col];
                write!(f, "{:?} ", cell)?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

impl Board2 {
    pub fn new(data: u64) -> Board2 {
        Board2 { data }
    }

    pub fn flatten(self) -> [CellState; 25] {
        let mut board = [CellState::Empty; 25];
        board[((self.data >> 0) & 0b11111) as usize] = CellState::WhitePawn;
        board[((self.data >> 5) & 0b11111) as usize] = CellState::WhitePawn;
        board[((self.data >> 10) & 0b11111) as usize] = CellState::WhitePawn;
        board[((self.data >> 15) & 0b11111) as usize] = CellState::WhitePawn;
        board[((self.data >> 20) & 0b11111) as usize] = CellState::BlackPawn;
        board[((self.data >> 25) & 0b11111) as usize] = CellState::BlackPawn;
        board[((self.data >> 30) & 0b11111) as usize] = CellState::BlackPawn;
        board[((self.data >> 35) & 0b11111) as usize] = CellState::BlackPawn;
        board[((self.data >> 40) & 0b11111) as usize] = CellState::WhiteKing;
        board[((self.data >> 45) & 0b11111) as usize] = CellState::BlackKing;
        board
    }

    pub fn flatten_to_bitarray(self) -> u32 {
        let mut arr = 0;
        arr |= 1 << ((self.data >> 0) & 0b11111);
        arr |= 1 << ((self.data >> 5) & 0b11111);
        arr |= 1 << ((self.data >> 10) & 0b11111);
        arr |= 1 << ((self.data >> 15) & 0b11111);
        arr |= 1 << ((self.data >> 20) & 0b11111);
        arr |= 1 << ((self.data >> 25) & 0b11111);
        arr |= 1 << ((self.data >> 30) & 0b11111);
        arr |= 1 << ((self.data >> 35) & 0b11111);
        arr |= 1 << ((self.data >> 40) & 0b11111);
        arr |= 1 << ((self.data >> 45) & 0b11111);
        arr
    }

    fn normalize_greater(data: u64, offset: u8) -> u64 {
        if offset < 40 {
            if offset == 15 || offset == 35 {
                data
            } else {
                let pos = (data >> offset) & 0b11111;
                let next_pos = (data >> (offset + 5)) & 0b11111;
                if pos > next_pos {
                    let next_data = (data & (u64::MAX ^ (0b1111111111 << offset)))
                        | (next_pos << offset)
                        | (pos << (offset + 5));
                    Self::normalize_greater(next_data, offset + 5)
                } else {
                    data
                }
            }
        } else {
            data
        }
    }

    fn normalize_less(data: u64, offset: u8) -> u64 {
        if offset < 40 {
            if offset == 0 || offset == 20 {
                data
            } else {
                let pos = (data >> offset) & 0b11111;
                let next_pos = (data >> (offset - 5)) & 0b11111;
                if pos < next_pos {
                    let next_data = (data & (u64::MAX ^ (0b1111111111 << (offset - 5))))
                        | (next_pos << offset)
                        | (pos << (offset - 5));
                    Self::normalize_less(next_data, offset - 5)
                } else {
                    data
                }
            }
        } else {
            data
        }
    }

    pub fn ended(self) -> bool {
        (self.data >> 40) & 0x11111 == 12 || (self.data >> 45) & 0x11111 == 12
    }

    pub fn do_move(self, m: Move) -> Self {
        let mut data = self.data;
        let to = m.to as u64;
        let pos = (data >> m.from_offset) & 0b11111;
        data &= u64::MAX ^ (0b11111 << m.from_offset);
        data |= to << m.from_offset;
        data ^= 1 << 50;
        // let data_before_normalize = data;
        data = if to > pos {
            Self::normalize_greater(data, m.from_offset)
        } else {
            Self::normalize_less(data, m.from_offset)
        };
        // println!(
        //     "normalize: \n{:#051b} ->\n{:#051b}",
        //     data_before_normalize, data
        // );
        // println!("do_move: {:?} -- ({:?}) -> {:?}", self, m, Board2 { data });
        Self { data }
    }

    pub fn maximizing(self) -> bool {
        (self.data >> 50) & 1 != 0
    }

    pub fn all_moves(self) -> Vec<Board2> {
        let mut moves = Vec::new();
        let flat = self.flatten_to_bitarray();
        let offsets = if self.maximizing() {
            [0, 5, 10, 15, 40]
        } else {
            [20, 25, 30, 35, 45]
        };
        for i in offsets {
            for ray in &MOVEMENT_TABLE[((self.data >> i) & 0b11111) as usize] {
                let to = if ray[0] == x {
                    break; // no more directions
                } else if flat & (1 << ray[0]) != 0 {
                    continue; // first cell blocked
                } else if ray[1] == x {
                    ray[0]
                } else if flat & (1 << ray[1]) != 0 {
                    ray[0]
                } else if ray[2] == x {
                    ray[1]
                } else if flat & (1 << ray[2]) != 0 {
                    ray[1]
                } else if ray[3] == x {
                    ray[2]
                } else if flat & (1 << ray[3]) != 0 {
                    ray[2]
                } else {
                    ray[3]
                };
                if i < 40 && to == 12 {
                    continue;
                }
                let m = Move { from_offset: i, to };
                moves.push(self.do_move(m));
            }
        }
        moves
    }

    const CELL_WEIGHTS_PAWN: [i32; 25] = [
        0, 3, 0, 3, 0, 3, 25, 25, 25, 3, 0, 25, 0, 25, 0, 3, 25, 25, 25, 3, 0, 3, 0, 3, 0,
    ];

    const CELL_WEIGHTS_KING: [i32; 25] = [
        10, 0, 10, 0, 10, 0, 50, 50, 50, 0, 10, 50, 100000, 50, 10, 0, 50, 50, 50, 0, 10, 0, 10, 0,
        10,
    ];

    pub fn score(&self) -> i32 {
        let mut score = 0;
        let data = self.data;
        let white_king_pos = (data >> 40) & 0b11111;
        let black_king_pos = (data >> 45) & 0b11111;
        if white_king_pos == 12 {
            return 100000;
        }
        if black_king_pos == 12 {
            return -100000;
        }
        score += Self::CELL_WEIGHTS_KING[white_king_pos as usize];
        score -= Self::CELL_WEIGHTS_KING[black_king_pos as usize];
        score += Self::CELL_WEIGHTS_PAWN[((data >> 0) & 0b11111) as usize];
        score += Self::CELL_WEIGHTS_PAWN[((data >> 5) & 0b11111) as usize];
        score += Self::CELL_WEIGHTS_PAWN[((data >> 10) & 0b11111) as usize];
        score += Self::CELL_WEIGHTS_PAWN[((data >> 15) & 0b11111) as usize];
        score -= Self::CELL_WEIGHTS_PAWN[((data >> 20) & 0b11111) as usize];
        score -= Self::CELL_WEIGHTS_PAWN[((data >> 25) & 0b11111) as usize];
        score -= Self::CELL_WEIGHTS_PAWN[((data >> 30) & 0b11111) as usize];
        score -= Self::CELL_WEIGHTS_PAWN[((data >> 35) & 0b11111) as usize];
        score
    }

    pub fn new_original() -> Self {
        Self {
            data: (0 << 0
                | 1 << 5
                | 2 << 40
                | 3 << 10
                | 4 << 15
                | 20 << 20
                | 21 << 25
                | 22 << 45
                | 23 << 30
                | 24 << 35
                | 1 << 50),
        }
    }

    pub fn new_with_king_inversed() -> Self {
        Self {
            data: (0 << 0
                | 1 << 5
                | 2 << 45
                | 3 << 10
                | 4 << 15
                | 20 << 20
                | 21 << 25
                | 22 << 40
                | 23 << 30
                | 24 << 35
                | 1 << 50),
        }
    }
}

#[derive(Clone, Copy)]
pub struct Move {
    from_offset: u8,
    to: u8,
}

impl Debug for Move {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] -> {}", self.from_offset, self.to)
    }
}

/*
0  1  2  3  4
5  6  7  8  9
10 11 12 13 14
15 16 17 18 19
20 21 22 23 24
*/

const x: u8 = 0xff;
const z: [u8; 4] = [0xff; 4];
const nil2: [[u8; 4]; 8] = [[0xff; 4]; 8];

const MOVEMENT_TABLE: [[[u8; 4]; 8]; 32] = [
    [
        [1, 2, 3, 4],
        [6, 12, 18, 24],
        [5, 10, 15, 20],
        z,
        z,
        z,
        z,
        z,
    ],
    [
        [0, x, x, x],
        [5, x, x, x],
        [6, 11, 16, 21],
        [7, 13, 19, x],
        [2, 3, 4, x],
        z,
        z,
        z,
    ],
    [
        [1, 0, x, x],
        [6, 10, x, x],
        [7, 12, 17, 22],
        [8, 14, x, x],
        [3, 4, x, x],
        z,
        z,
        z,
    ],
    [
        [2, 1, 0, x],
        [7, 11, 15, x],
        [8, 13, 18, 23],
        [9, x, x, x],
        [4, x, x, x],
        z,
        z,
        z,
    ],
    [
        [3, 2, 1, 0],
        [8, 12, 16, 20],
        [9, 14, 19, 24],
        z,
        z,
        z,
        z,
        z,
    ],
    [
        [0, x, x, x],
        [10, 15, 20, x],
        [11, 17, 23, x],
        [6, 7, 8, 9],
        [1, x, x, x],
        z,
        z,
        z,
    ],
    [
        [1, x, x, x],
        [0, x, x, x],
        [5, x, x, x],
        [10, x, x, x],
        [11, 16, 21, x],
        [12, 18, 24, x],
        [7, 8, 9, x],
        [2, x, x, x],
    ],
    [
        [2, x, x, x],
        [1, x, x, x],
        [6, 5, x, x],
        [11, 15, x, x],
        [12, 17, 22, x],
        [13, 19, x, x],
        [8, 9, x, x],
        [3, x, x, x],
    ],
    [
        [3, x, x, x],
        [2, x, x, x],
        [7, 6, 5, x],
        [12, 16, 20, x],
        [13, 18, 23, x],
        [14, x, x, x],
        [9, x, x, x],
        [4, x, x, x],
    ],
    [
        [4, x, x, x],
        [3, x, x, x],
        [8, 7, 6, 5],
        [13, 17, 21, x],
        [14, 19, 24, x],
        z,
        z,
        z,
    ],
    [
        [5, 0, x, x],
        [15, 20, x, x],
        [16, 22, x, x],
        [11, 12, 13, 14],
        [6, 2, x, x],
        z,
        z,
        z,
    ],
    [
        [6, 1, x, x],
        [5, x, x, x],
        [10, x, x, x],
        [15, x, x, x],
        [16, 21, x, x],
        [17, 23, x, x],
        [12, 13, 14, x],
        [7, 3, x, x],
    ],
    nil2,
    [
        [8, 3, x, x],
        [7, 1, x, x],
        [12, 11, 10, x],
        [17, 21, x, x],
        [18, 23, x, x],
        [19, x, x, x],
        [14, x, x, x],
        [9, x, x, x],
    ],
    [
        [9, 4, x, x],
        [8, 2, x, x],
        [13, 12, 11, 10],
        [18, 22, x, x],
        [19, 24, x, x],
        z,
        z,
        z,
    ],
    [
        [10, 5, 0, x],
        [20, x, x, x],
        [21, x, x, x],
        [16, 17, 18, 19],
        [11, 7, 3, x],
        z,
        z,
        z,
    ],
    [
        [11, 6, 1, x],
        [10, x, x, x],
        [15, x, x, x],
        [20, x, x, x],
        [21, x, x, x],
        [22, x, x, x],
        [17, 18, 19, x],
        [12, 8, 4, x],
    ],
    [
        [12, 7, 2, x],
        [11, 5, x, x],
        [16, 15, x, x],
        [21, x, x, x],
        [22, x, x, x],
        [23, x, x, x],
        [18, 19, x, x],
        [13, 9, x, x],
    ],
    [
        [13, 8, 3, x],
        [12, 6, 0, x],
        [17, 16, 15, x],
        [22, x, x, x],
        [23, x, x, x],
        [24, x, x, x],
        [19, x, x, x],
        [14, x, x, x],
    ],
    [
        [14, 9, 4, x],
        [13, 7, 1, x],
        [18, 17, 16, 15],
        [23, x, x, x],
        [24, x, x, x],
        z,
        z,
        z,
    ],
    [
        [15, 10, 5, 0],
        [21, 22, 23, 24],
        [16, 12, 8, 4],
        z,
        z,
        z,
        z,
        z,
    ],
    [
        [16, 11, 6, 1],
        [15, x, x, x],
        [20, x, x, x],
        [22, 23, 24, x],
        [17, 13, 9, x],
        z,
        z,
        z,
    ],
    [
        [17, 12, 7, 2],
        [16, 10, x, x],
        [21, 20, x, x],
        [23, 24, x, x],
        [18, 14, x, x],
        z,
        z,
        z,
    ],
    [
        [18, 13, 8, 3],
        [17, 11, 5, x],
        [22, 21, 20, x],
        [24, x, x, x],
        [19, x, x, x],
        z,
        z,
        z,
    ],
    [
        [19, 14, 9, 4],
        [18, 12, 6, 0],
        [23, 22, 21, 20],
        z,
        z,
        z,
        z,
        z,
    ],
    nil2,
    nil2,
    nil2,
    nil2,
    nil2,
    nil2,
    nil2,
];
