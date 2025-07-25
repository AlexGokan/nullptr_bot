use std::{cmp::Ordering, str::FromStr};
use chess::{get_rank, BitBoard, Board, BoardStatus, ChessMove, Color, MoveGen};
use log::{debug,info,warn,error};

pub fn forward_pos(row: usize, color: chess::Color) -> usize{
    if color == chess::Color::White{
        return row;
    }
    return 7 - row;
}

pub fn evaluate_for_color(board: &Board, color: chess::Color) -> f32{
    let queen_val: f32 = 9.0;
    let rook_val: f32 = 5.0;
    let bishop_val: f32 = 3.5;
    let knight_val: f32 = 3.0;
    let pawn_val: f32 = 1.0;

    let bb_my_color = board.color_combined(color);

    //increase pawn value the further up the board it is
    let mut pawn_sum: f32 = 0.0;
    let base_val: f32 = 1.20;
    if color == chess::Color::White{
        for row in 0..8{
            let bb_row = get_rank(chess::Rank::from_index(row)) & bb_my_color;
            let rank_value: f32 = base_val.powf(forward_pos(row, color) as f32);
            let pawns_on_row = (board.pieces(chess::Piece::Pawn) & bb_row).popcnt() as f32;
            pawn_sum += pawns_on_row*rank_value*pawn_val;
        }
    }
    if color == chess::Color::Black{
        for row in (0..8).rev(){
            let bb_row = get_rank(chess::Rank::from_index(row)) & bb_my_color;
            let rank_value: f32 = base_val.powf(forward_pos(row, color) as f32);
            let pawns_on_row = (board.pieces(chess::Piece::Pawn) & bb_row).popcnt() as f32;
            pawn_sum += pawns_on_row*rank_value*pawn_val;
            
        }
    }

    let piece_val: f32 = 
        queen_val  * (board.pieces(chess::Piece::Queen)  & bb_my_color).popcnt() as f32 +
        rook_val   * (board.pieces(chess::Piece::Rook)   & bb_my_color).popcnt() as f32 +
        bishop_val * (board.pieces(chess::Piece::Bishop) & bb_my_color).popcnt() as f32 +
        knight_val * (board.pieces(chess::Piece::Knight) & bb_my_color).popcnt() as f32 +
        pawn_sum;
        //pawn_val   * (board.pieces(chess::Piece::Pawn)   & bb_my_color).popcnt() as f32;

    let center_squares: u64 = (1u64 << 27) | (1u64 << 28) | (1u64 << 35) | (1u64 << 36);
    let bb_center_squares = chess::BitBoard::new(center_squares);
    
    let outer_ring: u64 = (1u64 << 42) | (1u64 << 43) | (1u64 << 44) |  (1u64 << 45) | // c6, d6, e6, f6
                      (1u64 << 34) | (1u64 << 37) |                  // c5, f5
                      (1u64 << 26) | (1u64 << 29) |                  // c4, f4
                      (1u64 << 18) | (1u64 << 19) | (1u64 << 20) | (1u64 << 21); // c3, d3, e3, f3
    let bb_center_outer_ring = chess::BitBoard::new(outer_ring);

    let inner_control_score = (bb_center_squares & bb_my_color).popcnt();
    let outer_control_score = (bb_center_outer_ring & bb_my_color).popcnt();

    let control_score: f32 = ((inner_control_score as f32) * 0.4) + ((outer_control_score as f32) * 0.2);
    
    //let move_count = MoveGen::new_legal(&board).count();
    //let flexibility_score = (move_count as f32) * 0.25;
    
    
    return piece_val + control_score;
}


pub fn evaluate(board: &Board, perspective: chess::Color) -> f32{
    //returns how good a board is from the perspective of a certain player
    /*
    if board.status() == BoardStatus::Stalemate{
        info!("stalemate");
        return 0.0;
    }
    */
    
    let eval_me: f32 = evaluate_for_color(board, perspective);
    let eval_opp: f32 = evaluate_for_color(board, !perspective);
    
    return eval_me - eval_opp;
    
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