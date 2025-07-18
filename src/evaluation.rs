use std::{cmp::Ordering};
use chess::{Board,ChessMove,BitBoard};
use log::{debug,info,warn,error};



pub fn evaluate(board: &Board) -> f32{
    let bb: &BitBoard = board.combined();
    let mut set_squares = bb.popcnt() as f32;
    set_squares = set_squares * -1.0;

    return set_squares;
}

#[derive(Debug, PartialEq)] 
pub struct EvaluatedMove{
    pub evaluation: f32,
    pub chessmove: ChessMove
}

impl EvaluatedMove{
    pub fn new(m: ChessMove, eval: f32) -> Self{
        EvaluatedMove{
            evaluation: eval,
            chessmove: m
        }
    }
}

impl Ord for EvaluatedMove {
    fn cmp(&self, other: &Self) -> Ordering {
        self.evaluation.partial_cmp(&other.evaluation)
            .unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for EvaluatedMove {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for EvaluatedMove {}  // Required for Ord