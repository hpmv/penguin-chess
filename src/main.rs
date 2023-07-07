pub mod board;
pub mod cell;
pub mod player;

use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
};

use board::BoardState;
use player::Player;

struct SearchResult {
    pub winner: Option<Player>,
    pub best_move: usize,
}

#[derive(Hash, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct GameState {
    board: BoardState,
    player: Player,
}

impl GameState {
    pub fn new() -> GameState {
        GameState {
            board: BoardState::new(),
            player: Player::White,
        }
    }

    pub fn next_moves(&self) -> Vec<GameState> {
        self.board
            .next_moves(self.player)
            .into_iter()
            .map(|board| GameState {
                board,
                player: self.player.opponent(),
            })
            .collect()
    }

    pub fn winner(&self) -> Option<Player> {
        self.board.winner()
    }
}

impl Debug for GameState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}\n{:?}", self.player, self.board)
    }
}

struct SearchState {
    optimal: HashMap<GameState, SearchResult>,
    visiting: HashSet<GameState>,
}

impl SearchState {
    pub fn new() -> SearchState {
        SearchState {
            optimal: HashMap::new(),
            visiting: HashSet::new(),
        }
    }

    fn search(&mut self, state: GameState, depth: usize) {
        if self.optimal.contains_key(&state) {
            return;
        }
        if let Some(winner) = state.winner() {
            self.optimal.insert(
                state,
                SearchResult {
                    winner: Some(winner),
                    best_move: 0,
                },
            );
            return;
        }

        // println!("Searching state at depth {}", depth);
        // println!("{:?}", state);

        self.visiting.insert(state.clone());
        let mut best_move = 0;
        let mut best_score = 0;
        for (i, next_state) in state.next_moves().iter().enumerate() {
            if self.visiting.contains(next_state) {
                continue;
            }
            self.search(next_state.clone(), depth + 1);
            let score = match self.optimal.get(&next_state).unwrap().winner {
                Some(winner) => {
                    if winner == state.player {
                        1
                    } else {
                        -1
                    }
                }
                _ => 0,
            };
            if score > best_score {
                best_score = score;
                best_move = i;
            }
        }
        self.optimal.insert(
            state.clone(),
            SearchResult {
                winner: if best_score == 1 {
                    Some(state.player)
                } else if best_score == -1 {
                    Some(state.player.opponent())
                } else {
                    None
                },
                best_move,
            },
        );
        self.visiting.remove(&state);
    }
}

fn run() {
    let mut search_state = SearchState::new();
    search_state.search(GameState::new(), 0);
    // println!("{:?}", search_state.optimal);
}

use std::thread;

const STACK_SIZE: usize = 400 * 1024 * 1024;

fn main() {
    // Spawn thread with explicit stack size
    let child = thread::Builder::new()
        .stack_size(STACK_SIZE)
        .spawn(run)
        .unwrap();

    // Wait for thread to join
    child.join().unwrap();
}
