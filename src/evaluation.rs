use std::{cmp::Ordering, str::FromStr};
use chess::{get_rank, BitBoard, Board, BoardStatus, ChessMove, Color, MoveGen};
use log::{debug,info,warn,error};

use crate::early_game_probability;



pub fn forward_pos(row: usize, color: chess::Color) -> usize{
    if color == chess::Color::White{
        return row;
    }
    return 7 - row;
}

fn get_bit(value: usize, idx: usize) -> usize{
    return (value & (1<<idx)) >> idx;
}

pub fn piece_square_table_evaluate(board: &Board, color: chess::Color) -> f32{
    static PST_PAWN: [i32; 64] = [
    0,  0,  0,  0,  0,  0,  0,  0,
    50, 50, 50, 50, 50, 50, 50, 50,
    10, 10, 20, 30, 30, 20, 10, 10,
    5,  5, 10, 25, 25, 10,  5,  5,
    0,  0,  0, 20, 20,  0,  0,  0,
    5, -5,-10,  0,  0,-10, -5,  5,
    5, 10, 10,-20,-20, 10, 10,  5,
    0,  0,  0,  0,  0,  0,  0,  0,
    ];

    static PST_KNIGHT: [i32; 64] = [
    -50,-40,-30,-30,-30,-30,-40,-50,
    -40,-20,  0,  0,  0,  0,-20,-40,
    -30,  0, 10, 15, 15, 10,  0,-30,
    -30,  5, 15, 20, 20, 15,  5,-30,
    -30,  0, 15, 20, 20, 15,  0,-30,
    -30,  5, 10, 15, 15, 10,  5,-30,
    -40,-20,  0,  5,  5,  0,-20,-40,
    -50,-40,-30,-30,-30,-30,-40,-50,
    ];

    static PST_BISHOP: [i32; 64] = [
    -20,-10,-10,-10,-10,-10,-10,-20,
    -10,  0,  0,  0,  0,  0,  0,-10,
    -10,  0,  5, 10, 10,  5,  0,-10,
    -10,  5,  5, 10, 10,  5,  5,-10,
    -10,  0, 10, 10, 10, 10,  0,-10,
    -10, 10, 10, 10, 10, 10, 10,-10,
    -10,  5,  0,  0,  0,  0,  5,-10,
    -20,-10,-10,-10,-10,-10,-10,-20,
    ];

    static PST_ROOK: [i32; 64] = [
    0,  0,  0,  0,  0,  0,  0,  0,
    5, 10, 10, 10, 10, 10, 10,  5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    0,  0,  0,  5,  5,  0,  0,  0
    ];

    static PST_QUEEN: [i32; 64] = [
    -20,-10,-10, -5, -5,-10,-10,-20,
    -10,  0,  0,  0,  0,  0,  0,-10,
    -10,  0,  5,  5,  5,  5,  0,-10,
    -5,  0,  5,  5,  5,  5,  0, -5,
    0,  0,  5,  5,  5,  5,  0, -5,
    -10,  5,  5,  5,  5,  5,  0,-10,
    -10,  0,  5,  0,  0,  0,  0,-10,
    -20,-10,-10, -5, -5,-10,-10,-20
    ];//moving these outside the function to static storage does nothing optimization wise
    
    //info!("evaluating with piece square tables");
    
    let bb_my_pawns = (board.color_combined(color) & board.pieces(chess::Piece::Pawn)).to_size(0);
    let bb_my_knights = (board.color_combined(color) & board.pieces(chess::Piece::Knight)).to_size(0);
    let bb_my_bishops = (board.color_combined(color) & board.pieces(chess::Piece::Bishop)).to_size(0);
    let bb_my_rooks = (board.color_combined(color) & board.pieces(chess::Piece::Rook)).to_size(0);
    let bb_my_queens = (board.color_combined(color) & board.pieces(chess::Piece::Queen)).to_size(0);
    
    let mut pst_score: i32 = 0;

    for nominal_idx in 0..64{
        let mut idx = nominal_idx;
        if color == chess::Color::Black{
            idx = 63-idx;
        }
        
        let pawn_bit_value = get_bit(bb_my_pawns,idx);//looking upwards from white's perspective
        let knight_bit_value = get_bit(bb_my_knights,idx);
        let bishop_bit_value = get_bit(bb_my_bishops,idx);
        let rook_bit_value = get_bit(bb_my_rooks,idx);
        let queen_bit_value = get_bit(bb_my_queens,idx);
        
        let col = nominal_idx%8;
        let row = (nominal_idx-col)/8;//will range from 0-7
        let new_row = 7-row;//ranges from 7-0
        let new_idx = (new_row*8)+col;

        let pawn_table_value = PST_PAWN[new_idx];
        let knight_table_value = PST_KNIGHT[new_idx];
        let bishop_table_value = PST_BISHOP[new_idx];
        let rook_table_value = PST_ROOK[new_idx];
        let queen_table_value = PST_QUEEN[new_idx];

        pst_score += 
            pawn_table_value * (pawn_bit_value as i32);
            knight_table_value * (knight_bit_value as i32);
            bishop_table_value * (bishop_bit_value as i32);
            rook_table_value * (rook_bit_value as i32);
            queen_table_value * (queen_bit_value as i32);
    }

    //info!("my total evaluation: {pst_score}");

    let egp = early_game_probability(board);
    let pst_weight = 1.0 - egp;
    //info!("early game prob: {egp}");
    //info!("PST weight: {pst_weight}");

    return (pst_score as f32) * pst_weight;
}

pub fn evaluate_for_color(board: &Board, color: chess::Color) -> f32{
    let queen_val: f32 = 9.0;
    let rook_val: f32 = 5.0;
    let bishop_val: f32 = 3.5;
    let knight_val: f32 = 3.0;
    let pawn_val: f32 = 1.0;

    let bb_my_color = board.color_combined(color);

        
    
    //----------------Pawn Value as it moves up the board ---------------------
    
    
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
    
    


    //-----------------Generic piece value----------------------------
    let piece_val: f32 = 
        queen_val  * (board.pieces(chess::Piece::Queen)  & bb_my_color).popcnt() as f32 +
        rook_val   * (board.pieces(chess::Piece::Rook)   & bb_my_color).popcnt() as f32 +
        bishop_val * (board.pieces(chess::Piece::Bishop) & bb_my_color).popcnt() as f32 +
        knight_val * (board.pieces(chess::Piece::Knight) & bb_my_color).popcnt() as f32 +
        pawn_sum;


    //----------------Presence in the middle squares---------------------
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
    

    //--------piece square table-----------------
    let pst_score =  (piece_square_table_evaluate(board, color) as f32) * 0.10;//not sure if .25 is an appropriate weight idk
    
    //info!("Piece val: {piece_val}");
    //info!("Control val: {control_score}");
    //info!("PST val: {pst_score}");

    return piece_val + control_score + pst_score;
}


pub fn evaluate(board: &Board, perspective: chess::Color) -> f32{
    //returns how good a board is from the perspective of a certain player
    
    let eval_me: f32 = evaluate_for_color(board, perspective);
    let eval_opp: f32 = evaluate_for_color(board, !perspective);
    
    return eval_me - eval_opp;
    
}
