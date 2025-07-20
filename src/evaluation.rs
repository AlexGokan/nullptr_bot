use std::{cmp::Ordering};
use chess::{BitBoard, Board, ChessMove, Color};
use log::{debug,info,warn,error};



pub fn evaluate(board: &Board) -> f32{
    //returns how good a board is for white
    let queen_val: f32 = 9.0;
    let rook_val: f32 = 5.0;
    let bishop_val: f32 = 3.5;
    let knight_val: f32 = 3.0;
    let pawn_val: f32 = 1.0;


    let bbwhite = board.color_combined(chess::Color::White);
    let bbblack = board.color_combined(chess::Color::Black);

    let white_val: f32 = 
        queen_val  * (board.pieces(chess::Piece::Queen)  & bbwhite).popcnt() as f32 +
        rook_val   * (board.pieces(chess::Piece::Rook)   & bbwhite).popcnt() as f32 +
        bishop_val * (board.pieces(chess::Piece::Bishop) & bbwhite).popcnt() as f32 +
        knight_val * (board.pieces(chess::Piece::Knight) & bbwhite).popcnt() as f32 +
        pawn_val   * (board.pieces(chess::Piece::Pawn)   & bbwhite).popcnt() as f32;

    let black_val: f32 = 
        queen_val  * (board.pieces(chess::Piece::Queen)  & bbblack).popcnt() as f32 +
        rook_val   * (board.pieces(chess::Piece::Rook)   & bbblack).popcnt() as f32 +
        bishop_val * (board.pieces(chess::Piece::Bishop) & bbblack).popcnt() as f32 +
        knight_val * (board.pieces(chess::Piece::Knight) & bbblack).popcnt() as f32 +
        pawn_val   * (board.pieces(chess::Piece::Pawn)   & bbblack).popcnt() as f32;

    let delta: f32 = white_val - black_val;

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