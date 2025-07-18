use std::{cmp::Ordering, io::{self,BufRead,Write}};
use rand::Rng;
use chess::{Board,ChessMove,MoveGen};
use std::str::FromStr;

use log::{debug,info,warn,error};



pub fn evaluate(_cm: &Board) -> f32{
    return 0.0;
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