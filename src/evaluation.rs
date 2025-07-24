use std::{cmp::Ordering};
use chess::{BitBoard, Board, ChessMove, Color, BoardStatus};
use log::{debug,info,warn,error};



pub fn evaluate(board: &Board, perspective: chess::Color) -> f32{
    //returns how good a board is from the perspective of a certain player
    if board.status() == BoardStatus::Stalemate{
        return 0.0;
    }
    
    let queen_val: f32 = 9.0;
    let rook_val: f32 = 5.0;
    let bishop_val: f32 = 3.5;
    let knight_val: f32 = 3.0;
    let pawn_val: f32 = 1.0;


    let bb_mine = board.color_combined(perspective);
    let bb_enemy = board.color_combined(!perspective);


    let my_val: f32 = 
        queen_val  * (board.pieces(chess::Piece::Queen)  & bb_mine).popcnt() as f32 +
        rook_val   * (board.pieces(chess::Piece::Rook)   & bb_mine).popcnt() as f32 +
        bishop_val * (board.pieces(chess::Piece::Bishop) & bb_mine).popcnt() as f32 +
        knight_val * (board.pieces(chess::Piece::Knight) & bb_mine).popcnt() as f32 +
        pawn_val   * (board.pieces(chess::Piece::Pawn)   & bb_mine).popcnt() as f32;

    let enemy_val: f32 = 
        queen_val  * (board.pieces(chess::Piece::Queen)  & bb_enemy).popcnt() as f32 +
        rook_val   * (board.pieces(chess::Piece::Rook)   & bb_enemy).popcnt() as f32 +
        bishop_val * (board.pieces(chess::Piece::Bishop) & bb_enemy).popcnt() as f32 +
        knight_val * (board.pieces(chess::Piece::Knight) & bb_enemy).popcnt() as f32 +
        pawn_val   * (board.pieces(chess::Piece::Pawn)   & bb_enemy).popcnt() as f32;

    let delta: f32 = my_val - enemy_val;

    return delta;
    
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