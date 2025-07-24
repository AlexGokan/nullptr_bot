

use core::f32;
use std::{io::{self,BufRead,Write}, result};
use rand::Rng;
use chess::{BitBoard, Board, BoardStatus, ChessMove, MoveGen};
use std::str::FromStr;
use std::collections::BinaryHeap;

mod evaluation;
use evaluation::evaluate;

mod ucigocommand;
use ucigocommand::UCIGoCommand;



use log::{debug,info,warn,error};

use crate::evaluation::EvaluatedMove;

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

    fn evaluate_minimax(&self, board: chess::Board, depth: u16, mut alpha: f32, mut beta: f32, my_color: chess::Color, my_move: bool) -> f32{
        if depth == 0{
            return evaluate(&board, my_color);
        }
        else{
            
            let movegen = MoveGen::new_legal(&board);
            let (captures, quiet_moves): (Vec<ChessMove>, Vec<ChessMove>) = movegen
            .partition(|m| board.piece_on(m.get_dest()).is_some());

            let mut moves = captures;
            moves.extend(quiet_moves);
            

            let num_moves = moves.len();
            info!(" ---- {num_moves} response moves");
            if my_move{//I am a maximizing player
                if num_moves == 0 && board.status() == BoardStatus::Stalemate{
                    return 0.0;
                }


                let mut max_eval = f32::NEG_INFINITY;
                for cm in moves{
                    info!("depth {depth}         evaluating response {cm}");
                    let board_copy = board.clone();
                    let mut new_board = board.clone();
                    board_copy.make_move(cm, &mut new_board);

                    let eval = self.evaluate_minimax(new_board, depth-1, alpha, beta, my_color, !my_move);
                    if eval >= beta{
                        break;
                    }
                    alpha = alpha.max(eval);
                    max_eval = max_eval.max(eval);
                }
                return max_eval;

            }else{//my opponent is a minimzing player
                if num_moves == 0 && board.status() == BoardStatus::Stalemate{
                    return 0.0;
                }
                let mut min_eval = f32::INFINITY;             
                for cm in moves{
                    info!("depth {depth}         evaluating response {cm}");
                    let board_copy = board.clone();
                    let mut new_board = board.clone();
                    board_copy.make_move(cm, &mut new_board);

                    let eval = self.evaluate_minimax(new_board, depth-1, alpha, beta, my_color, !my_move);
                    if eval <= alpha{
                        break;
                    }
                    beta = beta.min(eval);
                    min_eval = min_eval.min(eval);
                }
                return min_eval;
            }
        }
    }

    fn generate_slightly_smart_move(&self, my_color: chess::Color) -> Option<ChessMove>{
        let movegen = MoveGen::new_legal(&self.board);
        let moves: Vec<ChessMove> = movegen.collect();
        
        if moves.is_empty() {
            return None;
        }
        
        let mut move_heap: BinaryHeap<EvaluatedMove> = BinaryHeap::new();
        
        let num_moves = moves.len();
        info!("evaluating {num_moves} moves at the top level");
        for cm in moves{
            info!("====================================================");
            //let eval = evaluate(&self.board.make_move_new(cm),my_color);
            info!("going into eval function for {cm}");
            let nb = self.board.make_move_new(cm);
            let mut alpha: f32 = f32::NEG_INFINITY;
            let mut beta: f32 = f32::INFINITY;
            let eval = self.evaluate_minimax(nb, 8, alpha,beta, my_color, false);

            info!("move: {} evaluation: {}",cm,eval);
            let em: EvaluatedMove = EvaluatedMove::new(cm,eval);
            move_heap.push(em);
        }
        
        let best_move: Option<&EvaluatedMove> = move_heap.peek();//this is a max heap
        //since I have ensured that the heap will always have some element in it, why do I have to return an Option?
        

        let extracted_best_move: Option<ChessMove> = match best_move{
            Some(em) => Some(em.chessmove),
            None => None
        };//feels like ther should be a better way to do this?
        //let extracted_best_move: Option<ChessMove> = move_heap.peek().map(|em| em.chessmove);

        return extracted_best_move;
    }

    fn calculate_think_time(&self, go_command: &UCIGoCommand) -> i32{
        if let Some(movetime) = go_command.movetime{
            return movetime;
        }
        else{
            if go_command.infinite.unwrap_or(false){
                return 100000000;
            }
            
            let (my_time,my_inc) = 
                if self.board.side_to_move() == chess::Color::White{
                    (go_command.wtime,go_command.winc)
                }else{
                    (go_command.btime,go_command.binc)
                };

            let base_time = my_time.unwrap_or(30000)/30;
            let increment = my_inc.unwrap_or(0);

            return base_time + increment;
        }
    }


    fn generate_move(&self, _think_time: i32, my_color: chess::Color) -> Option<ChessMove>{
        return self.generate_slightly_smart_move(my_color);

    }

    fn handle_go(&self, tokens: &[&str]) {
        let go_command: UCIGoCommand = UCIGoCommand::new(tokens);

        let think_time: i32 = self.calculate_think_time(&go_command);
        info!("think for {}",think_time);
        
        let my_color: chess::Color = self.board.side_to_move();

        if let Some(best_move) = self.generate_move(think_time, my_color) {
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