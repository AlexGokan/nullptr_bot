

use std::io::{self,BufRead,Write};
use rand::Rng;
use chess::{Board,ChessMove,MoveGen};
use std::str::FromStr;

use log::{debug,info,warn,error};

struct ChessEngine{
    board: Board,
}

impl ChessEngine{
    fn new() -> Self{
        ChessEngine { 
            board: Board::default()
        }
    }

    fn handle_uci(&self){
        println!("id name nullptrbot");
        println!("id author alex");
        println!("uciok");
        io::stdout().flush().unwrap();
    }

    fn handle_isready(&self){
        println!("readyok");
        io::stdout().flush().unwrap();
    }

    fn handle_position(&mut self, tokens: &[&str]){
        if tokens.len() < 2{
            return;
        }
        if tokens[1] == "startpos"{
            self.board = Board::default();
        }
        else if tokens[1] == "fen"{
            if tokens.len() < 8{
                return;
            }
            let fen_parts: Vec<&str> = tokens[2..8].to_vec();
            let fen_str:String = fen_parts.join(" ");

            match Board::from_str(&fen_str){
                Ok(board) => self.board = board,
                Err(_) => {
                    self.board = Board::default();
                }
            }
        }

        if let Some(moves_index) = tokens.iter().position(|&x| x == "moves") {
            for &move_str in &tokens[moves_index + 1..] {
                if let Ok(chess_move) = ChessMove::from_str(move_str) {
                    self.board = self.board.make_move_new(chess_move);
                }
            }
        }
    }

    fn generate_random_move(&self) -> Option<ChessMove> {
        let movegen = MoveGen::new_legal(&self.board);
        let moves: Vec<ChessMove> = movegen.collect();
        
        if moves.is_empty() {
            return None;
        }
        
        let mut rng = rand::thread_rng();
        let random_index = rng.gen_range(0..moves.len());
        Some(moves[random_index])
    }

    fn calculate_think_time(&self, wtime: Option<i32>, btime: Option<i32>, winc: Option<i32>, binc: Option<i32>, movetime: Option<i32>, _depth: Option<i32>, infinite: Option<bool>) -> i32{
        if let Some(movetime) = movetime{
            return movetime;
        }
        else{
            if infinite.unwrap_or(false){
                return 100000000;
            }
            
            let (my_time,my_inc) = if self.board.side_to_move() == chess::Color::White{
                (wtime,winc)
            }else{
                (btime,binc)
            };

            let base_time = my_time.unwrap_or(30000)/30;
            let increment = my_inc.unwrap_or(0);

            return base_time + increment;
        }
    }

    fn generate_move(&self, _think_time: i32) -> Option<ChessMove>{
        return self.generate_random_move();

    }

    fn handle_go(&self, tokens: &[&str]) {
        let mut wtime: Option<i32> = None;
        let mut btime: Option<i32> = None;
        let mut winc: Option<i32> = None;
        let mut binc: Option<i32> = None;
        let mut movetime: Option<i32> = None;
        let mut depth: Option<i32> = None;
        let mut infinite: Option<bool> = None;

        let mut i: usize = 1;

        while i < tokens.len(){
            match tokens[i] {
                "wtime" if i + 1 < tokens.len() => {
                    wtime = tokens[i + 1].parse().ok();
                    i += 2;
                }
                "btime" if i + 1 < tokens.len() => {
                    btime = tokens[i + 1].parse().ok();
                    i += 2;
                }
                "winc" if i + 1 < tokens.len() => {
                    winc = tokens[i + 1].parse().ok();
                    i += 2;
                }
                "binc" if i + 1 < tokens.len() => {
                    binc = tokens[i + 1].parse().ok();
                    i += 2;
                }
                "movetime" if i + 1 < tokens.len() => {
                    movetime = tokens[i + 1].parse().ok();
                    i += 2;
                }
                "depth" if i + 1 < tokens.len() => {
                    depth = tokens[i + 1].parse().ok();
                    i += 2;
                }
                "infinite" => {
                    infinite = Some(true);
                    i += 1;
                }
                _ => i += 1,
            }
        }

        let think_time: i32 = self.calculate_think_time(wtime, btime, winc, binc, movetime, depth, infinite);
        //println!("Think for {}",think_time);
        info!("think for {}",think_time);
        
        if let Some(best_move) = self.generate_move(think_time) {
            println!("bestmove {}", best_move);
        } else {
            // No legal moves (checkmate or stalemate)
            println!("bestmove 0000");
        }
        io::stdout().flush().unwrap();
    }

    fn run(&mut self) {
        let stdin = io::stdin();
        


        for line in stdin.lock().lines() {

            let line = line.unwrap();
            let tokens: Vec<&str> = line.split_whitespace().collect();
            
            if tokens.is_empty() {
                continue;
            }
            
            match tokens[0] {
                "uci" => self.handle_uci(),
                "isready" => self.handle_isready(),
                "position" => self.handle_position(&tokens),
                "go" => self.handle_go(&tokens),
                "quit" => break,
                _ => {} // Ignore unknown commands
            }
        }
    }

}

fn main(){
    env_logger::Builder::from_default_env()
    .target(env_logger::Target::Stderr)
    .init();

    let mut engine = ChessEngine::new();
    engine.run();
}