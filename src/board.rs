use crate::{
    cell::CellState::{self, BlackKing, BlackPawn, Empty, WhiteKing, WhitePawn},
    player::Player,
};
use std::{
    fmt::Debug,
    ops::{Index, IndexMut},
};

#[derive(Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct BoardState {
    pub state: [[CellState; 5]; 5],
}

impl Debug for BoardState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in self.state.iter() {
            for cell in row.iter() {
                write!(f, "{:?} ", cell)?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

type Pos = (usize, usize);

#[derive(Copy, Clone)]
pub enum Dir {
    Up,
    Down,
    Left,
    Right,
    UpLeft,
    UpRight,
    DownLeft,
    DownRight,
}

impl Dir {
    pub fn all() -> [Self; 8] {
        [
            Self::Up,
            Self::Down,
            Self::Left,
            Self::Right,
            Self::UpLeft,
            Self::UpRight,
            Self::DownLeft,
            Self::DownRight,
        ]
    }

    fn apply(&self, pos: Pos) -> Option<Pos> {
        // return None if exceeding board bounds.
        match self {
            Self::Up => {
                if pos.0 == 0 {
                    None
                } else {
                    Some((pos.0 - 1, pos.1))
                }
            }
            Self::Down => {
                if pos.0 == 4 {
                    None
                } else {
                    Some((pos.0 + 1, pos.1))
                }
            }
            Self::Left => {
                if pos.1 == 0 {
                    None
                } else {
                    Some((pos.0, pos.1 - 1))
                }
            }
            Self::Right => {
                if pos.1 == 4 {
                    None
                } else {
                    Some((pos.0, pos.1 + 1))
                }
            }
            Self::UpLeft => {
                if pos.0 == 0 || pos.1 == 0 {
                    None
                } else {
                    Some((pos.0 - 1, pos.1 - 1))
                }
            }
            Self::UpRight => {
                if pos.0 == 0 || pos.1 == 4 {
                    None
                } else {
                    Some((pos.0 - 1, pos.1 + 1))
                }
            }
            Self::DownLeft => {
                if pos.0 == 4 || pos.1 == 0 {
                    None
                } else {
                    Some((pos.0 + 1, pos.1 - 1))
                }
            }
            Self::DownRight => {
                if pos.0 == 4 || pos.1 == 4 {
                    None
                } else {
                    Some((pos.0 + 1, pos.1 + 1))
                }
            }
        }
    }
}

impl Index<Pos> for BoardState {
    type Output = CellState;

    fn index(&self, pos: Pos) -> &Self::Output {
        &self.state[pos.0][pos.1]
    }
}

impl IndexMut<Pos> for BoardState {
    fn index_mut(&mut self, pos: Pos) -> &mut Self::Output {
        &mut self.state[pos.0][pos.1]
    }
}

impl BoardState {
    pub fn new() -> BoardState {
        BoardState {
            state: [
                [WhitePawn, WhitePawn, WhiteKing, WhitePawn, WhitePawn],
                [Empty, Empty, Empty, Empty, Empty],
                [Empty, Empty, Empty, Empty, Empty],
                [Empty, Empty, Empty, Empty, Empty],
                [BlackPawn, BlackPawn, BlackKing, BlackPawn, BlackPawn],
            ],
        }
    }

    pub fn make_move(&self, from: Pos, dir: Dir) -> Option<BoardState> {
        let mut new_pos = from;
        while let Some(pos) = dir.apply(new_pos) {
            if self[pos] != Empty {
                break;
            }
            new_pos = pos;
        }
        if new_pos == from {
            return None;
        }
        if !self[new_pos].is_king() && new_pos == (2, 2) {
            return None;
        }
        let mut new_state = self.clone();
        new_state[new_pos] = self[from];
        new_state[from] = Empty;
        Some(new_state)
    }

    pub fn next_moves(&self, player: Player) -> Vec<BoardState> {
        let mut moves = Vec::new();
        for row in 0..5 {
            for col in 0..5 {
                let pos = (row, col);
                if self[pos].is_player(player) {
                    for dir in Dir::all() {
                        if let Some(new_state) = self.make_move(pos, dir) {
                            moves.push(new_state);
                        }
                    }
                }
            }
        }
        moves
    }

    pub fn winner(&self) -> Option<Player> {
        let mid = (2, 2);
        match self[mid] {
            WhiteKing => Some(Player::White),
            BlackKing => Some(Player::Black),
            _ => None,
        }
    }
}
