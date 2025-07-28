use rand::prelude::*;
use std::collections::HashMap;
use chess::{BitBoard, Board, BoardStatus, ChessMove, MoveGen, Rank};

pub struct ZobristHasher{
    pub rand_arrays: [[u64; 12];64],
    pub black_to_move: u64,
    pub gamestate_hashmap: HashMap<u64,u8>,
}

impl ZobristHasher{
    
    
    pub fn new() -> Self{
        let mut rng = rand::thread_rng();
        
        let mut arr: [[u64; 12]; 64] = [[0; 12]; 64];
        
        for i in 0..64 {
            for j in 0..12 {
                let x = rng.r#gen::<u64>();
                arr[i][j] = x;
            }
        }

        ZobristHasher { 
            rand_arrays: arr,
            black_to_move: rng.r#gen::<u64>(),
            gamestate_hashmap: HashMap::new()
        
        }
    }


    pub fn hash_board(&self, board: &Board) -> u64{
        let mut h: u64 = 0;
        if board.side_to_move() == chess::Color::Black{
            h = h ^ self.black_to_move;
        }

        for i in 0..64{
            let sq = unsafe{chess::Square::new(i as u8)};
            let p = board.piece_on(sq);

            match p{
                Some(piece) =>{
                    let color_on = board.color_on(sq).unwrap_or(chess::Color::White);
                    let mut piece_idx: usize = 0;
                    if piece == chess::Piece::Pawn{
                        piece_idx = 0;
                    }else if piece == chess::Piece::Knight{
                        piece_idx = 1;
                    }else if piece == chess::Piece::Bishop{
                        piece_idx = 2;
                    }else if piece == chess::Piece::Rook{
                        piece_idx = 3;
                    }else if piece == chess::Piece::Queen{
                        piece_idx = 4;
                    }else if piece == chess::Piece::King{
                        piece_idx = 5;
                    }
                    if color_on == chess::Color::Black{
                        piece_idx += 6;
                    }

                    h = h ^ self.rand_arrays[i][piece_idx];

                }
                None => {}
            }
        }

        return h;
    }

    pub fn check_table(&self, board_hash: u64) -> u8{
        let count = self.gamestate_hashmap.get(&board_hash).unwrap_or(&0);

        return *count as u8;
    }

    pub fn insert_board(&mut self, board: &Board){
        let h = self.hash_board(board);
        let boardcount = self.gamestate_hashmap.entry(h).or_insert(0);
        *boardcount += 1;
        //insert a 0 if its not in table, and then add 1 to it
    }

    pub fn to_string(&mut self) -> String{
        let mut s: String = "--Count-----Hash---------------------\n".to_owned();
        
        for (h,count) in &self.gamestate_hashmap{
            let ss: String = format!("--{}-------- {}\n",count,h);
            s.push_str(&ss);
        }

        return s;
    }



}