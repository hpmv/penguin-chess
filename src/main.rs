pub mod board;
pub mod cell;
pub mod player;

use std::{collections::HashMap, fmt::Debug};

use board::BoardState;
use player::Player;

struct SearchResult {
    pub score: i32,
    pub best_path: Vec<BoardState>,
}

impl Debug for SearchResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}\n", self.score)?;
        for state in self.best_path.iter() {
            write!(f, "{:?}\n", state)?;
        }
        Ok(())
    }
}

struct SearchState {
    transposition_table: Vec<HashMap<BoardState, (i32, BoardState)>>,
    next_transposition_table: Vec<HashMap<BoardState, (i32, BoardState)>>,
    same_search_transposition_hits: Vec<usize>,
    nodes_searched: Vec<usize>,
    max_depth: usize,
    max_transposition_table_depth: usize,
}

impl SearchState {
    pub fn new() -> SearchState {
        SearchState {
            transposition_table: Vec::new(),
            next_transposition_table: Vec::new(),
            same_search_transposition_hits: Vec::new(),
            nodes_searched: Vec::new(),
            max_depth: 0,
            max_transposition_table_depth: 10,
        }
    }

    fn alpha_beta(
        &mut self,
        state: BoardState,
        maximizing: bool,
        depth: usize,
        mut alpha: i32,
        mut beta: i32,
    ) -> SearchResult {
        if depth >= self.max_depth {
            return SearchResult {
                score: state.score(),
                best_path: vec![state],
            };
        }
        if let Some(result) = self.next_transposition_table[depth].get(&state) {
            self.same_search_transposition_hits[depth] += 1;
            return SearchResult {
                score: result.0,
                best_path: vec![result.1.clone(), state],
            };
        }

        self.nodes_searched[depth] += 1;

        let mut best_path = Vec::new();
        let mut best_score = if maximizing { i32::MIN } else { i32::MAX };
        let player = if maximizing {
            Player::White
        } else {
            Player::Black
        };

        let mut moves = state.next_moves(player);
        if moves.is_empty() {
            return SearchResult {
                score: if maximizing { -100000 } else { 100000 },
                best_path: vec![state],
            };
        }
        let prev_best_move = self.transposition_table[depth].get(&state);
        moves.sort_by_key(|state| {
            if let Some((_, prev_best_move)) = prev_best_move {
                if state == prev_best_move {
                    return -10000000;
                }
            }
            if maximizing {
                -state.score()
            } else {
                state.score()
            }
        });

        for next_state in moves.into_iter() {
            let SearchResult {
                score,
                best_path: best_subpath,
            } = self.alpha_beta(next_state, !maximizing, depth + 1, alpha, beta);
            if maximizing {
                if score > best_score {
                    best_score = score;
                    best_path = best_subpath;
                }
                if score > alpha {
                    alpha = score;
                }
            } else {
                if score < best_score {
                    best_score = score;
                    best_path = best_subpath;
                }
                if score < beta {
                    beta = score;
                }
            }
            if alpha >= beta {
                break;
            }
        }
        if depth < self.max_transposition_table_depth {
            self.next_transposition_table[depth].insert(
                state.clone(),
                (best_score, best_path.last().cloned().unwrap()),
            );
        }

        best_path.push(state);

        SearchResult {
            score: best_score,
            best_path,
        }
    }

    fn next_depth(&mut self) {
        std::mem::swap(
            &mut self.transposition_table,
            &mut self.next_transposition_table,
        );
        for i in 0..self.max_depth {
            self.next_transposition_table[i].clear();
        }
        self.transposition_table.push(HashMap::new());
        self.next_transposition_table.push(HashMap::new());

        self.max_depth += 1;
        self.same_search_transposition_hits.push(0);
        self.nodes_searched.push(0);

        for i in 0..self.max_depth {
            println!(
                "Depth {} transposition hits: {} / {}, table size = {}",
                i,
                self.same_search_transposition_hits[i],
                self.nodes_searched[i],
                self.transposition_table[i].len(),
            );
            self.same_search_transposition_hits[i] = 0;
            self.nodes_searched[i] = 0;
        }
    }
}

fn run() {
    let mut search_state = SearchState::new();
    loop {
        search_state.next_depth();
        let result = search_state.alpha_beta(
            BoardState::new_with_king_inversed(),
            true,
            0,
            i32::MIN,
            i32::MAX,
        );
        println!("Depth {} result: {:?}", search_state.max_depth, result);
    }
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
