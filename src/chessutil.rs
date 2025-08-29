use chess::{Board, ChessMove, MoveGen, Rank};
use std::fs::{File};
use std::io::Read;
use std::fs;
use std::collections::HashMap;
use std::collections::hash_map::Entry;


pub fn make_move_new(board: &Board, cm: ChessMove) -> chess::Board{
    let mut b2 = board.clone();
    board.make_move(cm, &mut b2);
    return b2;
}

pub fn output_sorted_move_list(board: &chess::Board) -> Vec<ChessMove>{
    let movegen = MoveGen::new_legal(&board);
    
    let (captures, quiet_moves): (Vec<ChessMove>, Vec<ChessMove>) = movegen
        .partition(|m| board.piece_on(m.get_dest()).is_some());

    let (pawn_caps,other_captures): (Vec<ChessMove>, Vec<ChessMove>) = 
        captures.iter().partition(|m| {
            board.piece_on(m.get_source()).expect("there should always be a piece at the source of a move") == chess::Piece::Pawn
        });

    let (knight_caps,other_captures): (Vec<ChessMove>, Vec<ChessMove>) = 
        other_captures.iter().partition(|m| {
            board.piece_on(m.get_source()).expect("there should always be a piece at the source of a move") == chess::Piece::Knight
        });

    let (bishop_caps,other_captures): (Vec<ChessMove>, Vec<ChessMove>) = 
        other_captures.iter().partition(|m| {
            board.piece_on(m.get_source()).expect("there should always be a piece at the source of a move") == chess::Piece::Bishop
        });

    let (rook_caps,other_captures): (Vec<ChessMove>, Vec<ChessMove>) = 
        other_captures.iter().partition(|m| {
            board.piece_on(m.get_source()).expect("there should always be a piece at the source of a move") == chess::Piece::Rook
        });

    let (queen_caps,other_captures): (Vec<ChessMove>, Vec<ChessMove>) = 
        other_captures.iter().partition(|m| {
            board.piece_on(m.get_source()).expect("there should always be a piece at the source of a move") == chess::Piece::Queen
        });

    let (king_caps,other_captures): (Vec<ChessMove>, Vec<ChessMove>) = 
        other_captures.iter().partition(|m| {
            board.piece_on(m.get_source()).expect("there should always be a piece at the source of a move") == chess::Piece::King
        });
    

    let mut moves: Vec<ChessMove> = Vec::<ChessMove>::new();
    moves.extend(pawn_caps);
    moves.extend(knight_caps);
    moves.extend(bishop_caps);
    moves.extend(rook_caps);
    moves.extend(queen_caps);
    moves.extend(king_caps);
    moves.extend(other_captures);//should be empty
    moves.extend(quiet_moves);

    return moves;
}

pub fn early_game_probability(board: &Board) -> f32{
    let startpos_w = chess::get_rank(Rank::First) | chess::get_rank(Rank::Second);
    let startpos_b = chess::get_rank(Rank::Seventh) | chess::get_rank(Rank::Eighth);

    let bb_start_pawns_w = startpos_w & board.pieces(chess::Piece::Pawn) & board.color_combined(chess::Color::White);
    let bb_start_pawns_b = startpos_b & board.pieces(chess::Piece::Pawn) & board.color_combined(chess::Color::Black);

    let num_pcs_in_start = (bb_start_pawns_w.popcnt() + bb_start_pawns_b.popcnt()) as f32;
    //info!("{num_pcs_in_start} pieces in start");

    //when num_pcs is 16, it should be about 1
    //when num_pcs is < 13 or so, it should be about 0

    let denom = 1.0+2.0_f32.powf(2.0*(num_pcs_in_start-10.0));
    let prob = (-1.0/denom) + 1.0;

    return prob as f32
}

pub fn end_game_probability(board: &Board) -> f32{
    let bb_w = board.color_combined(chess::Color::White).popcnt();
    let bb_b = board.color_combined(chess::Color::Black).popcnt();

    let num_pcs = (bb_w + bb_b) as f32;
    
    let denom = 1.0 + 2_f32.powf(2.0 * (num_pcs-14.0));
    let prob = 1.0/denom;

    return prob;

}


pub struct BookEntry{
    pub key: u64,
    pub chessmove: chess::ChessMove,
    pub weight: u16
}

impl BookEntry{
    fn new(key: u64, cm: chess::ChessMove, weight: u16) -> Self{
        BookEntry { key: key,
                    chessmove: cm,
                    weight: weight}
    }
}

pub fn load_book(filename: &str) -> HashMap<u64,Vec<BookEntry>>{
    
    let mut opening_book: HashMap<u64,Vec<BookEntry>> = HashMap::new();


    let startpos_hash: u64 = 0x463b96181691fc9c;
    
    let mut f = File::open(filename).expect("Could not open book file");
    let metadata = fs::metadata(&filename).expect("could not read book metadata");

    let mut buffer: Vec<u8> = vec![0;metadata.len() as usize];//should be u8 because metadata returns number of bytes in file
    f.read(&mut buffer).expect("buffer overflow");

    let num_entries = metadata.len() / 16;
    let mut num_good_entries = 0;

    for idx in 0..num_entries as usize{
        let offset = idx * 16;
        let key_bytes: [u8; 8] = buffer[offset..offset+8].try_into().expect("can't do the stupid array conversion when loading book");
        let key = u64::from_be_bytes(key_bytes);//should be big endian from my reading of the spec

        let move_bytes: [u8; 2] = buffer[offset+8..offset+10].try_into().expect("can't do the stupid array conversion when loading book");
        let chessmove_bits = u16::from_be_bytes(move_bytes);

        let weight_bytes: [u8; 2] = buffer[offset+10..offset+12].try_into().expect("can't do the stupid array conversion when loading book");
        let weight = u16::from_be_bytes(weight_bytes);

        let learn_bytes: [u8; 4] = buffer[offset+12..offset+16].try_into().expect("can't do the stupid array conversion when loading book");
        let learn = u32::from_be_bytes(learn_bytes);//I don't think this is actually useful?


        //turn the bit representation into a move object
        //dest file: 0,1,2
        let mut dest_file: i32 = 0;
        for idx in 0..3{
            let bit_idx: i32 = idx + 0;
            let power = idx;
            let bit: i32 = (chessmove_bits >> bit_idx&1) as i32;

            let additive = bit * 2_i32.pow(power as u32);
            dest_file += bit * 2_i32.pow(power as u32);
        }

        //dest row: 0,1,2
        let mut dest_rank: i32 = 0;
        for idx in 0..3{
            let bit_idx: i32 = idx + 3;
            let power = idx;
            let bit: i32 = (chessmove_bits >> bit_idx&1) as i32;

            let additive = bit * 2_i32.pow(power as u32);
            dest_rank += bit * 2_i32.pow(power as u32);
        }
        let dest_sq = chess::Square::make_square(chess::Rank::from_index(dest_rank as usize),chess::File::from_index(dest_file as usize));

        let mut source_file: i32 = 0;
        for idx in 0..3{
            let bit_idx: i32 = idx + 6;
            let power = idx;
            let bit: i32 = (chessmove_bits >> bit_idx&1) as i32;

            let additive = bit * 2_i32.pow(power as u32);
            source_file += bit * 2_i32.pow(power as u32);
        }

        let mut source_rank: i32 = 0;
        for idx in 0..3{
            let bit_idx: i32 = idx + 9;
            let power = idx;
            let bit: i32 = (chessmove_bits >> bit_idx&1) as i32;

            let additive = bit * 2_i32.pow(power as u32);
            source_rank += bit * 2_i32.pow(power as u32);
        }

        let source_sq = chess::Square::make_square(chess::Rank::from_index(source_rank as usize),chess::File::from_index(source_file as usize));
        
        let cm = chess::ChessMove::new(source_sq,dest_sq,None);


        let entry = BookEntry::new(key,cm,weight);

        
        if weight > 100{//an arbitrary cutoff I invented
            match opening_book.entry(key){//still don't really get this bit of code mutability-wise (thanks claude!)
                Entry::Occupied(mut e) => {
                    e.get_mut().push(entry);
                }
                Entry::Vacant(e) => {
                    e.insert((vec![entry]));
                }
            }



            num_good_entries += 1;
        }

    }


    return opening_book;

}