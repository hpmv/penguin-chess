pub mod board;
mod board2;
pub mod cell;
pub mod player;

use board2::{Board2, Move};
use serde::Serialize;
use std::{
    collections::HashMap,
    fmt::Debug,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Instant,
};
use wasm_bindgen::prelude::*;

#[derive(Clone, Serialize)]
pub struct SearchResult {
    pub score: i32,
    pub best_path: Vec<Move>,
    pub first_move_scores: Vec<(Move, i32)>,
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
    transposition_table: HashMap<Board2, (i32, Move)>,
    next_transposition_table: HashMap<Board2, (i32, Move)>,
    nodes_searched: usize,
    max_depth: usize,
    max_transposition_table_depth: usize,
    stop: Arc<AtomicBool>,
}

struct Interrupted;

impl SearchState {
    pub fn new(stop: Arc<AtomicBool>) -> SearchState {
        SearchState {
            transposition_table: HashMap::new(),
            next_transposition_table: HashMap::new(),
            nodes_searched: 0,
            max_depth: 0,
            max_transposition_table_depth: 8,
            stop,
        }
    }

    fn alpha_beta(
        &mut self,
        state: Board2,
        depth: usize,
        mut alpha: i32,
        mut beta: i32,
    ) -> Result<SearchResult, Interrupted> {
        if self.stop.load(Ordering::Relaxed) {
            return Err(Interrupted);
        }
        if depth >= self.max_depth || state.ended() {
            return Ok(SearchResult {
                score: state.score(),
                best_path: vec![],
                first_move_scores: vec![],
            });
        }

        self.nodes_searched += 1;

        let maximizing = state.maximizing();
        let mut best_path = Vec::new();
        let mut best_score = if maximizing { i32::MIN } else { i32::MAX };

        let mut moves = state.all_moves();
        if moves.is_empty() {
            return Ok(SearchResult {
                score: if maximizing { -100000 } else { 100000 },
                best_path: vec![],
                first_move_scores: vec![],
            });
        }
        let prev_best_move = self.transposition_table.get(&state);
        moves.sort_by_key(|(m, state)| {
            if let Some((_, prev_best_move)) = prev_best_move {
                if m == prev_best_move {
                    return -10000000;
                }
            }
            if maximizing {
                -state.score()
            } else {
                state.score()
            }
        });

        let mut first_move_scores = Vec::new();
        for (one_move, next_state) in moves.into_iter() {
            let SearchResult {
                score,
                best_path: best_subpath,
                ..
            } = self.alpha_beta(next_state, depth + 1, alpha, beta)?;
            if maximizing {
                if score > best_score {
                    best_score = score;
                    best_path = best_subpath;
                    best_path.push(one_move);
                }
                if score > alpha {
                    alpha = score;
                }
            } else {
                if score < best_score {
                    best_score = score;
                    best_path = best_subpath;
                    best_path.push(one_move);
                }
                if score < beta {
                    beta = score;
                }
            }
            if depth == 0 {
                first_move_scores.push((one_move, score));
            }
            if alpha >= beta {
                break;
            }
        }
        if depth < self.max_transposition_table_depth {
            self.next_transposition_table.insert(
                state.clone(),
                (best_score, best_path.last().copied().unwrap()),
            );
        }

        Ok(SearchResult {
            score: best_score,
            best_path,
            first_move_scores,
        })
    }

    fn next_depth(&mut self) {
        std::mem::swap(
            &mut self.transposition_table,
            &mut self.next_transposition_table,
        );
        self.next_transposition_table.clear();

        self.max_depth += 1;
        self.nodes_searched = 0;
    }
}

#[derive(Serialize)]
pub struct PartialSearchResult {
    pub depth: usize,
    pub nodes_searched: usize,
    pub result: SearchResult,
    pub time: f64,
}

pub fn find_best_move(
    state: Board2,
    stop: Arc<AtomicBool>,
    partial: impl Fn(PartialSearchResult),
) -> Option<Move> {
    let mut search_state = SearchState::new(stop);
    let mut result: Option<SearchResult> = None;
    loop {
        search_state.next_depth();
        let depth = search_state.max_depth;
        let start_time = Instant::now();
        match search_state.alpha_beta(state, 0, i32::MIN, i32::MAX) {
            Err(Interrupted) => {
                break;
            }
            Ok(one_result) => {
                let time = start_time.elapsed().as_secs_f64();
                partial(PartialSearchResult {
                    depth,
                    nodes_searched: search_state.nodes_searched,
                    result: one_result.clone(),
                    time,
                });
                let win_found = one_result.score.abs() > 10000;
                result = Some(one_result);
                if win_found {
                    break;
                }
            }
        }
    }
    result?.best_path.last().copied()
}

#[wasm_bindgen]
pub struct Engine {
    stop: Arc<AtomicBool>,
    partial: js_sys::Function,
}

#[wasm_bindgen]
impl Engine {
    #[wasm_bindgen(constructor)]
    pub fn new(partial: js_sys::Function) -> Engine {
        Engine {
            stop: Arc::new(AtomicBool::new(false)),
            partial,
        }
    }

    pub fn find_best_move(&self, state: Vec<u8>) -> Option<String> {
        self.stop.store(false, Ordering::Relaxed);
        let state = Board2::from_positions(&state);
        let m = find_best_move(state, self.stop.clone(), |result| {
            let m = serde_json::to_string(&result).unwrap();
            let m = JsValue::from_str(&m);
            self.partial.call1(&JsValue::NULL, &m).unwrap();
        });
        m.map(|m| serde_json::to_string(&m).unwrap())
    }

    pub fn stop(&self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}

// fn analyze_board(board: Board2) -> i32 {
//     let mut search_state = SearchState::new();
//     let start_time = Instant::now();
//     let mut result = loop {
//         search_state.next_depth();
//         let result = search_state.alpha_beta(board, 0, i32::MIN, i32::MAX);
//         if start_time.elapsed().as_secs() > 20 || result.score.abs() > 10000 {
//             break result;
//         }
//     };
//     result.best_path.reverse();

//     println!(
//         "\nscore: {} depth: {}\n{}",
//         result.score,
//         search_state.max_depth,
//         result.best_path.vec_debug()
//     );
//     result.score
// }

// fn run() {
//     let mut board = Board2::new_with_king_inversed();
//     // board = board.do_move(Move::new(40, 7));
//     // board = board.do_move(Move::new(20, 8));
//     // // board = board.do_move(Move::new(5, 5));
//     // board = board.do_move(Move::new(5, 16));
//     loop {
//         println!("=======================================");
//         println!("{:?}", board);
//         println!("=======================================");
//         let mut all_moves = board
//             .all_moves()
//             .into_iter()
//             .map(|m| (m, analyze_board(m)))
//             .filter(|(m, s)| {
//                 if board.maximizing() {
//                     *s > -10000
//                 } else {
//                     *s < 10000
//                 }
//             })
//             .collect::<Vec<_>>();
//         // let m = rand::thread_rng().gen_range(0..all_moves.len());
//         all_moves.sort_by_key(|(_, s)| if board.maximizing() { -s } else { *s });
//         board = all_moves[0].0;
//     }
// }

use std::thread;

// use crate::board::VecDebug;

const STACK_SIZE: usize = 400 * 1024 * 1024;

// fn main() {
//     // Spawn thread with explicit stack size
//     let child = thread::Builder::new()
//         .stack_size(STACK_SIZE)
//         .spawn(run)
//         .unwrap();

//     // Wait for thread to join
//     child.join().unwrap();
// }
