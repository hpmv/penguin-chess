use std::fmt::Debug;

use crate::player::Player;

#[derive(Hash, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum CellState {
    Empty,
    WhitePawn,
    BlackPawn,
    WhiteKing,
    BlackKing,
}

impl Debug for CellState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "_"),
            Self::WhitePawn => write!(f, "o"),
            Self::BlackPawn => write!(f, "x"),
            Self::WhiteKing => write!(f, "@"),
            Self::BlackKing => write!(f, "*"),
        }
    }
}

impl CellState {
    pub fn is_king(&self) -> bool {
        match self {
            Self::WhiteKing | Self::BlackKing => true,
            _ => false,
        }
    }

    pub fn is_player(&self, player: Player) -> bool {
        match (self, player) {
            (Self::WhitePawn, Player::White) => true,
            (Self::BlackPawn, Player::Black) => true,
            (Self::WhiteKing, Player::White) => true,
            (Self::BlackKing, Player::Black) => true,
            _ => false,
        }
    }
}
