

use core::f32;
use std::{io::{self,BufRead,Write}, result};
use rand::Rng;
use chess::{BitBoard, Board, BoardStatus, ChessMove, MoveGen, NUM_PIECES};
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


    fn output_sorted_move_list(&self, board: chess::Board) -> Vec<ChessMove>{
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

    fn evaluate_minimax(&self, board: chess::Board, depth: usize, mut alpha: f32, mut beta: f32, my_color: chess::Color, my_move: bool) -> f32 {
        const MATE_VALUE: f32 = 100000.0;  // Much larger than any position eval
        
        if depth == 0 {
            return evaluate(&board, my_color);
        }
        

        let moves: Vec<ChessMove> = self.output_sorted_move_list(board);


        // Handle terminal positions (checkmate/stalemate)
        if moves.is_empty() {
            return match board.status() {
                BoardStatus::Stalemate => 0.0,
                BoardStatus::Checkmate => {
                    if board.side_to_move() == my_color {
                        -MATE_VALUE - (depth as f32)  // We're in checkmate - bad, but prefer later mates
                    } else {
                        MATE_VALUE + (depth as f32)   // Opponent in checkmate - good, prefer faster mates
                    }
                }
                _ => 0.0,
            };
        }

        
        if my_move {
            let mut max_eval = -MATE_VALUE;
            for chess_move in moves {
                let board_copy = board.clone();
                let mut new_board = board.clone();
                board_copy.make_move(chess_move, &mut new_board);
                let eval = self.evaluate_minimax(new_board, depth - 1, alpha, beta, my_color, false);
                
                max_eval = max_eval.max(eval);
                alpha = alpha.max(eval);
                
                if beta <= alpha {
                    break; // Alpha-beta pruning
                }
            }
            max_eval
        } else {
            let mut min_eval = MATE_VALUE;
            for chess_move in moves {
                let board_copy = board.clone();
                let mut new_board = board.clone();
                board_copy.make_move(chess_move, &mut new_board);
                let eval = self.evaluate_minimax(new_board, depth - 1, alpha, beta, my_color, true);
                
                min_eval = min_eval.min(eval);
                beta = beta.min(eval);
                
                if beta <= alpha {
                    break; // Alpha-beta pruning
                }
            }
            min_eval
        }
    }

    fn generate_best_move(&self, my_color: chess::Color, depth: usize) -> Option<ChessMove> { 
        const MATE_VALUE: f32 = 1000000.0;
        
        /*
        let movegen = MoveGen::new_legal(&self.board);
        let (captures, quiet_moves): (Vec<ChessMove>, Vec<ChessMove>) = movegen
            .partition(|m| self.board.piece_on(m.get_dest()).is_some());

        let mut moves = captures;
        moves.extend(quiet_moves);
        */

        let moves = self.output_sorted_move_list(self.board);
        
        if moves.is_empty() {
            return None;
        }
        
        let mut best_move = None;
        let mut best_eval = -MATE_VALUE;
        let mut alpha = -MATE_VALUE;
        let beta = MATE_VALUE;
        
        info!("evaluating {} moves at the top level", moves.len());
        
        for chess_move in moves {
            info!("====================================================");
            info!("going into eval function for {}", chess_move);
            
            let board_copy = self.board.clone();
            let mut new_board = self.board.clone();
            board_copy.make_move(chess_move, &mut new_board);
            let eval = self.evaluate_minimax(new_board, depth - 1, alpha, beta, my_color, false);
            
            info!("move: {} evaluation: {}", chess_move, eval);
            
            if eval > best_eval {
                best_eval = eval;
                best_move = Some(chess_move);
            }
            
            alpha = alpha.max(eval);
            
            // Alpha-beta pruning at root level (though less useful here)
            if beta <= alpha {
                break;
            }
        }
    
    best_move
}

    fn iterative_deepening_search(&self, time_limit: u128, max_depth: usize, my_color: chess::Color) -> Option<ChessMove>{
        
        
        let timer = std::time::Instant::now();

        let mut best_move: Option<ChessMove> = None;
        let mut prev_best_move: Option<ChessMove> = None;

        let mut best_score: f32 = f32::NEG_INFINITY;
        
        const MATE_VALUE: f32 = 100000.0;

        for depth in 2..max_depth{
            info!("=============================================");
            let mut elapsed_time = timer.elapsed().as_millis();
            if elapsed_time >= time_limit{
                break;
            }else{
                info!("we have enough time!");
            }

            let mut moves = self.output_sorted_move_list(self.board);
            let num_moves = moves.len();
            info!("{num_moves} moves");
            

            
            match prev_best_move{//prepend previous best move to list if it exists
                Some(cm) => {
                    moves.retain(|&m| m!= cm);
                    moves.insert(0, cm);

                }
                None =>{}
            }
            

            let num_moves = moves.len();
            info!("after prepending we have {num_moves} elements");


            let mut curr_best_move: Option<ChessMove> = None;
            let mut curr_best_score: f32 = f32::NEG_INFINITY;
            let mut alpha: f32 = -MATE_VALUE;
            let mut beta: f32 = MATE_VALUE;

            let mut finished_this_depth = true;

            for chess_move in moves{
                elapsed_time = timer.elapsed().as_millis();
                info!("----Elapsed {elapsed_time}");
                
                if (time_limit - elapsed_time) < 250{//give it 250 milliseconds to break instead of a percentage
                //if (elapsed_time as f32) >= (time_limit as f32) * 0.90{
                    info!("breaking");
                    finished_this_depth = false;
                    break;
                }
                

                let board_copy = self.board.clone();//make the move onto a new board
                let mut new_board = self.board.clone();
                board_copy.make_move(chess_move, &mut new_board);

                //let score = -1.0 * self.evaluate_minimax(new_board, depth-1, -beta, -alpha, my_color, false);
                let score = self.evaluate_minimax(new_board, depth - 1, alpha, beta, my_color, false);

                info!("{chess_move} : {score}");

                if score > curr_best_score{
                    curr_best_score = score;
                    curr_best_move = Some(chess_move);
                }

                alpha = alpha.max(score);
                if alpha >= beta{
                    break;
                }
            }

            if finished_this_depth{
                match curr_best_move{
                    Some(curr_best_move_cm) => {
                        best_move = curr_best_move;
                        best_score = curr_best_score;
                        prev_best_move = curr_best_move;
                    }
                    None => {}
                }
            }

        }

        return best_move;

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

    fn calculate_think_time_ms(&self, go_command: &UCIGoCommand, my_color: chess::Color) -> i32{
        //go command is in ms
        
        
        if let Some(movetime) = go_command.movetime{
            warn!("Received a UCI go command with movetime of {movetime}");
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

            let base_time = my_time.unwrap_or(30000);
            let increment = my_inc.unwrap_or(0);  

            println!("time remain: {base_time} {increment}");      
           
            let num_my_pcs = self.board.color_combined(my_color).popcnt();
            let num_opp_pcs = self.board.color_combined(!my_color).popcnt();
            let total_num_pcs = num_my_pcs + num_opp_pcs;

            if total_num_pcs == 32{
                return 5;
            }




            return ((base_time as f32)/30.0) as i32 + increment;
        }
    }


    fn generate_move(&self, _think_time_ms: i32, my_color: chess::Color) -> Option<ChessMove>{
        
        return self.iterative_deepening_search(_think_time_ms as u128, 12, my_color)
       // return self.generate_best_move(my_color,8);

    }

    fn handle_go(&self, tokens: &[&str]) {

        let my_color: chess::Color = self.board.side_to_move();
        let go_command: UCIGoCommand = UCIGoCommand::new(tokens);

        let think_time_ms: i32 = self.calculate_think_time_ms(&go_command, my_color);
        info!("think for {} ms",think_time_ms);
        
        

        //let eval = evaluate(&self.board, my_color);
        //info!("Eval: {eval}");
        
        //std::process::exit(0);

        if let Some(best_move) = self.generate_move(think_time_ms, my_color) {
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