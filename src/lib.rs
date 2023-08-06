pub mod board;
mod board2;
pub mod cell;
pub mod player;

use board2::{Board2, Move};
use itertools::Itertools;
use serde::Serialize;
use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    rc::Rc,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::wasm_bindgen_test;

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
    being_searched: HashSet<Board2>,
    nodes_searched: usize,
    max_depth: usize,
    max_transposition_table_depth: usize,
    stop: Box<dyn Fn() -> bool>,
    collect_first_move_scores: bool,
}

struct Interrupted;

impl SearchState {
    pub fn new(
        stop: Box<dyn Fn() -> bool>,
        collect_first_move_scores: bool,
        history_states: Vec<Board2>,
    ) -> SearchState {
        SearchState {
            transposition_table: HashMap::new(),
            next_transposition_table: HashMap::new(),
            being_searched: history_states.into_iter().collect(),
            nodes_searched: 0,
            max_depth: 0,
            max_transposition_table_depth: 20,
            stop,
            collect_first_move_scores,
        }
    }

    fn alpha_beta(
        &mut self,
        state: Board2,
        depth: usize,
        mut alpha: i32,
        mut beta: i32,
    ) -> Result<SearchResult, Interrupted> {
        if (self.stop)() {
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
        moves.retain(|(_, board)| !self.being_searched.contains(&board));
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

        self.being_searched.insert(state);
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
                if score > alpha && (!self.collect_first_move_scores || depth > 0) {
                    alpha = score;
                }
            } else {
                if score < best_score {
                    best_score = score;
                    best_path = best_subpath;
                    best_path.push(one_move);
                }
                if score < beta && (!self.collect_first_move_scores || depth > 0) {
                    beta = score;
                }
            }
            if depth == 0 && self.collect_first_move_scores {
                first_move_scores.push((one_move, score));
            }
            if alpha >= beta {
                break;
            }
        }
        if depth < self.max_transposition_table_depth {
            if self.next_transposition_table.len() < 30000000 {
                self.next_transposition_table.insert(
                    state.clone(),
                    (best_score, best_path.last().copied().unwrap()),
                );
            }
        }
        self.being_searched.remove(&state);

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
    pub transposition_table_size: usize,
    pub result: SearchResult,
}

pub fn find_best_move(
    state: Board2,
    stop: impl Fn() -> bool + 'static,
    partial: impl Fn(PartialSearchResult),
    collect_first_move_scores: bool,
    history_states: Vec<Board2>,
) -> Option<Move> {
    let mut search_state =
        SearchState::new(Box::new(stop), collect_first_move_scores, history_states);
    let mut result: Option<SearchResult> = None;
    loop {
        search_state.next_depth();
        let depth = search_state.max_depth;
        match search_state.alpha_beta(state, 0, i32::MIN, i32::MAX) {
            Err(Interrupted) => {
                break;
            }
            Ok(one_result) => {
                partial(PartialSearchResult {
                    depth,
                    nodes_searched: search_state.nodes_searched,
                    transposition_table_size: search_state.next_transposition_table.len(),
                    result: one_result.clone(),
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
    stop: Rc<js_sys::Uint8Array>,
    partial: js_sys::Function,
}

#[wasm_bindgen]
impl Engine {
    #[wasm_bindgen(constructor)]
    pub fn new(stop: js_sys::Uint8Array, partial: js_sys::Function) -> Engine {
        Engine {
            stop: Rc::new(stop),
            partial,
        }
    }

    pub fn find_best_move(
        &self,
        state: Vec<u8>,
        collect_first_move_scores: bool,
        history_states: Vec<u8>,
    ) -> Option<String> {
        let state = Board2::from_positions(&state);
        let stop = self.stop.clone();
        let m = find_best_move(
            state,
            move || js_sys::Atomics::load(&stop, 0).unwrap() != 0,
            |result| {
                let m = serde_json::to_string(&result).unwrap();
                let m = JsValue::from_str(&m);
                self.partial.call1(&JsValue::NULL, &m).unwrap();
            },
            collect_first_move_scores,
            history_states
                .into_iter()
                .chunks(11)
                .into_iter()
                .map(|s| Board2::from_positions(&s.collect()))
                .collect(),
        );
        m.map(|m| serde_json::to_string(&m).unwrap())
    }
}

#[wasm_bindgen_test]
fn test_basic_engine() {
    let engine = Engine::new(
        js_sys::Uint8Array::new_with_length(1),
        js_sys::Function::default(),
    );
    // does not terminate.
    println!(
        "{:?}",
        engine.find_best_move(vec![0, 1, 3, 4, 20, 21, 23, 24, 22, 2, 1], false, vec![])
    );
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
