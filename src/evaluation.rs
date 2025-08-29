use std::{cmp::Ordering, str::FromStr};
use chess::{get_rank, BitBoard, Board, BoardStatus, ChessMove, Color, MoveGen};
use log::{debug,info,warn,error};
use crate::chessutil::{self, early_game_probability};



pub fn forward_pos(row: usize, color: chess::Color) -> usize{
    if color == chess::Color::White{
        return row;
    }
    return 7 - row;
}

fn get_bit(value: usize, idx: usize) -> usize{
    return (value & (1<<idx)) >> idx;
}

pub fn piece_square_table_evaluate_mg(board: &Board, color: chess::Color) -> f32{
    //using middlegame tables from https://www.chessprogramming.org/PeSTO%27s_Evaluation_Function
    
    static PST_PAWN: [i32; 64] = [
    0,   0,   0,   0,   0,   0,  0,   0,
     98, 134,  61,  95,  68, 126, 34, -11,
     -6,   7,  26,  31,  65,  56, 25, -20,
    -14,  13,   6,  21,  23,  12, 17, -23,
    -27,  -2,  -5,  12,  17,   6, 10, -25,
    -26,  -4,  -4, -10,   3,   3, 33, -12,
    -35,  -1, -20, -23, -15,  24, 38, -22,
      0,   0,   0,   0,   0,   0,  0,   0,
    ];

    static PST_KNIGHT: [i32; 64] = [
    -167, -89, -34, -49,  61, -97, -15, -107,
     -73, -41,  72,  36,  23,  62,   7,  -17,
     -47,  60,  37,  65,  84, 129,  73,   44,
      -9,  17,  19,  53,  37,  69,  18,   22,
     -13,   4,  16,  13,  28,  19,  21,   -8,
     -23,  -9,  12,  10,  19,  17,  25,  -16,
     -29, -53, -12,  -3,  -1,  18, -14,  -19,
    -105, -21, -58, -33, -17, -28, -19,  -23,
    ];

    static PST_BISHOP: [i32; 64] = [
    -29,   4, -82, -37, -25, -42,   7,  -8,
    -26,  16, -18, -13,  30,  59,  18, -47,
    -16,  37,  43,  40,  35,  50,  37,  -2,
     -4,   5,  19,  50,  37,  37,   7,  -2,
     -6,  13,  13,  26,  34,  12,  10,   4,
      0,  15,  15,  15,  14,  27,  18,  10,
      4,  15,  16,   0,   7,  21,  33,   1,
    -33,  -3, -14, -21, -13, -12, -39, -21,
    ];

    static PST_ROOK: [i32; 64] = [
   32,  42,  32,  51, 63,  9,  31,  43,
     27,  32,  58,  62, 80, 67,  26,  44,
     -5,  19,  26,  36, 17, 45,  61,  16,
    -24, -11,   7,  26, 24, 35,  -8, -20,
    -36, -26, -12,  -1,  9, -7,   6, -23,
    -45, -25, -16, -17,  3,  0,  -5, -33,
    -44, -16, -20,  -9, -1, 11,  -6, -71,
    -19, -13,   1,  17, 16,  7, -37, -26,
    ];

    static PST_QUEEN: [i32; 64] = [
    -28,   0,  29,  12,  59,  44,  43,  45,
    -24, -39,  -5,   1, -16,  57,  28,  54,
    -13, -17,   7,   8,  29,  56,  47,  57,
    -27, -27, -16, -16,  -1,  17,  -2,   1,
     -9, -26,  -9, -10,  -2,  -4,   3,  -3,
    -14,   2, -11,  -2,  -5,   2,  14,   5,
    -35,  -8,  11,   2,   8,  15,  -3,   1,
     -1, -18,  -9,  10, -15, -25, -31, -50,
    ];//moving these outside the function to static storage does nothing optimization wise
    

    static PST_KING: [i32; 64] = [
    -65,  23,  16, -15, -56, -34,   2,  13,
     29,  -1, -20,  -7,  -8,  -4, -38, -29,
     -9,  24,   2, -16, -20,   6,  22, -22,
    -17, -20, -12, -27, -30, -25, -14, -36,
    -49,  -1, -27, -39, -46, -44, -33, -51,
    -14, -14, -22, -46, -44, -30, -15, -27,
      1,   7,  -8, -64, -43, -16,   9,   8,
    -15,  36,  12, -54,   8, -28,  24,  14,
    ];

    //info!("evaluating with piece square tables");
    
    let bb_mine = board.color_combined(color);

    let bb_my_pawns = (bb_mine & board.pieces(chess::Piece::Pawn)).to_size(0);
    let bb_my_knights = (bb_mine & board.pieces(chess::Piece::Knight)).to_size(0);
    let bb_my_bishops = (bb_mine & board.pieces(chess::Piece::Bishop)).to_size(0);
    let bb_my_rooks = (bb_mine & board.pieces(chess::Piece::Rook)).to_size(0);
    let bb_my_queens = (bb_mine & board.pieces(chess::Piece::Queen)).to_size(0);
    
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
            pawn_table_value * (pawn_bit_value as i32) +
            knight_table_value * (knight_bit_value as i32) +
            bishop_table_value * (bishop_bit_value as i32) +
            rook_table_value * (rook_bit_value as i32) +
            queen_table_value * (queen_bit_value as i32);
    }

    //info!("my total evaluation: {pst_score}");

    let egp = early_game_probability(board);
    let pst_weight = 1.0 - egp;
    //info!("early game prob: {egp}");
    //info!("PST weight: {pst_weight}");

    return (pst_score as f32) * pst_weight / 24.0;
}

pub fn piece_square_table_evaluate_eg(board: &Board, color: chess::Color) -> f32{
    //using endgame tables from https://www.chessprogramming.org/PeSTO%27s_Evaluation_Function
    
    static PST_PAWN: [i32; 64] = [
    0,   0,   0,   0,   0,   0,   0,   0,
    178, 173, 158, 134, 147, 132, 165, 187,
     94, 100,  85,  67,  56,  53,  82,  84,
     32,  24,  13,   5,  -2,   4,  17,  17,
     13,   9,  -3,  -7,  -7,  -8,   3,  -1,
      4,   7,  -6,   1,   0,  -5,  -1,  -8,
     13,   8,   8,  10,  13,   0,   2,  -7,
      0,   0,   0,   0,   0,   0,   0,   0,
    ];

    static PST_KNIGHT: [i32; 64] = [
    -58, -38, -13, -28, -31, -27, -63, -99,
    -25,  -8, -25,  -2,  -9, -25, -24, -52,
    -24, -20,  10,   9,  -1,  -9, -19, -41,
    -17,   3,  22,  22,  22,  11,   8, -18,
    -18,  -6,  16,  25,  16,  17,   4, -18,
    -23,  -3,  -1,  15,  10,  -3, -20, -22,
    -42, -20, -10,  -5,  -2, -20, -23, -44,
    -29, -51, -23, -15, -22, -18, -50, -64,
    ];

    static PST_BISHOP: [i32; 64] = [
    -14, -21, -11,  -8, -7,  -9, -17, -24,
     -8,  -4,   7, -12, -3, -13,  -4, -14,
      2,  -8,   0,  -1, -2,   6,   0,   4,
     -3,   9,  12,   9, 14,  10,   3,   2,
     -6,   3,  13,  19,  7,  10,  -3,  -9,
    -12,  -3,   8,  10, 13,   3,  -7, -15,
    -14, -18,  -7,  -1,  4,  -9, -15, -27,
    -23,  -9, -23,  -5, -9, -16,  -5, -17,
    ];

    static PST_ROOK: [i32; 64] = [
   13, 10, 18, 15, 12,  12,   8,   5,
    11, 13, 13, 11, -3,   3,   8,   3,
     7,  7,  7,  5,  4,  -3,  -5,  -3,
     4,  3, 13,  1,  2,   1,  -1,   2,
     3,  5,  8,  4, -5,  -6,  -8, -11,
    -4,  0, -5, -1, -7, -12,  -8, -16,
    -6, -6,  0,  2, -9,  -9, -11,  -3,
    -9,  2,  3, -1, -5, -13,   4, -20,
    ];

    static PST_QUEEN: [i32; 64] = [
     -9,  22,  22,  27,  27,  19,  10,  20,
    -17,  20,  32,  41,  58,  25,  30,   0,
    -20,   6,   9,  49,  47,  35,  19,   9,
      3,  22,  24,  45,  57,  40,  57,  36,
    -18,  28,  19,  47,  31,  34,  39,  23,
    -16, -27,  15,   6,   9,  17,  10,   5,
    -22, -23, -30, -16, -16, -23, -36, -32,
    -33, -28, -22, -43,  -5, -32, -20, -41,
    ];
    
    static PST_KING: [i32; 64] = [
    -74, -35, -18, -18, -11,  15,   4, -17,
    -12,  17,  14,  17,  17,  38,  23,  11,
     10,  17,  23,  15,  20,  45,  44,  13,
     -8,  22,  24,  27,  26,  33,  26,   3,
    -18,  -4,  21,  24,  27,  23,   9, -11,
    -19,  -3,  11,  21,  23,  16,   7,  -9,
    -27, -11,   4,  13,  14,   4,  -5, -17,
    -53, -34, -21, -11, -28, -14, -24, -43
    ];

    //info!("evaluating with piece square tables");
    
    let bb_mine = board.color_combined(color);

    let bb_my_pawns = (bb_mine & board.pieces(chess::Piece::Pawn)).to_size(0);
    let bb_my_knights = (bb_mine & board.pieces(chess::Piece::Knight)).to_size(0);
    let bb_my_bishops = (bb_mine & board.pieces(chess::Piece::Bishop)).to_size(0);
    let bb_my_rooks = (bb_mine & board.pieces(chess::Piece::Rook)).to_size(0);
    let bb_my_queens = (bb_mine & board.pieces(chess::Piece::Queen)).to_size(0);
    
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
            pawn_table_value * (pawn_bit_value as i32) +
            knight_table_value * (knight_bit_value as i32) +
            bishop_table_value * (bishop_bit_value as i32) +
            rook_table_value * (rook_bit_value as i32) +
            queen_table_value * (queen_bit_value as i32);
    }

    //info!("my total evaluation: {pst_score}");

    let egp = early_game_probability(board);
    let pst_weight = 1.0 - egp;
    //info!("early game prob: {egp}");
    //info!("PST weight: {pst_weight}");

    return (pst_score as f32) * pst_weight / 24.0;
}

pub fn evaluate_for_color(board: &Board, color: chess::Color) -> f32{
    let queen_val: f32 = 9.0;
    let rook_val: f32 = 5.0;
    let bishop_val: f32 = 3.1;
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
    let pst_score_mg =  (piece_square_table_evaluate_mg(board, color) as f32) * 0.10;//not sure if .25 is an appropriate weight idk
    let pst_score_eg =  (piece_square_table_evaluate_eg(board, color) as f32) * 0.10;//not sure if .25 is an appropriate weight idk
    
    let eg_prob = chessutil::end_game_probability(&board);
    let pst_score = (eg_prob * pst_score_eg) + ((1.0-eg_prob) * pst_score_mg);

    //-------doubled pawns----------------------
    
    let my_pawn_bb = bb_my_color.0 & board.pieces(chess::Piece::Pawn).0;
    let mut doubled_penalty = 0.0;
    if color == chess::Color::White{
        let num_doubled_pawns = (my_pawn_bb & (my_pawn_bb << 8)).count_ones();
        doubled_penalty = -0.4*(num_doubled_pawns as f32);

    }else{
        let num_doubled_pawns = (my_pawn_bb & (my_pawn_bb >> 8)).count_ones();
        doubled_penalty = -0.4*(num_doubled_pawns as f32);
    }


    //------King safety-------
    //is there a pawn right in front of my king?
    let my_king_bb = bb_my_color.0 & board.pieces(chess::Piece::King).0;
    let mut king_safety_bonus: f32 = 0.0;
    if color == chess::Color::White{
        let square_in_front_of_king = my_king_bb << 8;
        let bb_pawns_in_front_of_king = square_in_front_of_king & my_pawn_bb;
        king_safety_bonus = 1.2 * bb_pawns_in_front_of_king.count_ones() as f32;
    }
    else{
        let square_in_front_of_king = my_king_bb >> 8;
        let bb_pawns_in_front_of_king = square_in_front_of_king & my_pawn_bb;
        king_safety_bonus = 1.2 * bb_pawns_in_front_of_king.count_ones() as f32;
    }

    king_safety_bonus = king_safety_bonus * (1.0-eg_prob);//taper this off towards the end game


    //info!("Piece val: {piece_val}");
    //info!("Control val: {control_score}");
    //info!("PST val: {pst_score}");

    return piece_val + control_score + pst_score + doubled_penalty + king_safety_bonus;
}


pub fn evaluate(board: &Board, perspective: chess::Color) -> f32{
    //returns how good a board is from the perspective of a certain player
    
    let eval_me: f32 = evaluate_for_color(board, perspective);
    let eval_opp: f32 = evaluate_for_color(board, !perspective);
    
    return eval_me - eval_opp;
    
}
