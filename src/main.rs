

use core::f32;
use std::{io::{self,BufRead,Write}, result};
use rand::Rng;
use chess::{BitBoard, Board, BoardStatus, ChessMove, MoveGen, Rank, NUM_PIECES};
use std::str::FromStr;
use std::collections::BinaryHeap;

mod evaluation;
use evaluation::evaluate;

mod ucigocommand;
use ucigocommand::UCIGoCommand;

mod search;
mod chessutil;



use log::{debug,info,warn,error};


pub struct ChessEngine{
    nodes_visited: u64,
    board: Board,
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

impl ChessEngine{
    fn new() -> Self{
        ChessEngine { 
            board: Board::default(),
            nodes_visited: 0
        }
    }

    fn handle_searchbenchmark(&mut self, tokens: &[&str]){
        self.nodes_visited = 0;
        
        let think_time: u32 = 1000000;
        let timer = std::time::Instant::now();

        let my_color = self.board.side_to_move();

        let search_depth = tokens[1].parse().unwrap();

        //let (eval,best_move) = search::iterative_deepening_search(think_time, search_depth, my_color);
        //let (eval,chessmove) = search::iterative_deepening_search(
        //    self, self.board, search_depth, think_time, self.board.side_to_move(), true);
        
        let (eval,chessmove) = search::iterative_deepening_search_with_time(
            self, self.board, search_depth,
            1.0, think_time,self.board.side_to_move(), true);
        

        let elapsed = timer.elapsed().as_millis();

        println!("Bechmark for depth: {search_depth}");
        println!("{elapsed} ms");

        let visited = self.nodes_visited;
        println!("Evaluated {visited} leafs");
        io::stdout().flush().unwrap();

    }

    fn handle_evaluate(&mut self){
        let c = self.board.side_to_move();
        let eval = evaluate(&self.board, c);
        println!("Evaluation: {eval}");
        io::stdout().flush().unwrap();

    }
    
    fn handle_uci(&mut self){
        println!("id name nullptrbot");
        println!("id author alex");
        println!("uciok");
        io::stdout().flush().unwrap();

    }

    fn handle_isready(&mut self){
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

    /*
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


    fn evaluate_minimax(&mut self, board: chess::Board, depth: usize, mut alpha: f32, mut beta: f32, my_color: chess::Color, my_move: bool) -> f32 {
        const MATE_VALUE: f32 = 100000.0;  // Much larger than any position eval
        
        if depth == 0 {
            self.nodes_visited += 1;
            return evaluate(&board, my_color);
        }
        

        let moves: Vec<ChessMove> = chessutil::output_sorted_move_list(&board);


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

    fn generate_best_move(&mut self, my_color: chess::Color, depth: usize) -> Option<ChessMove> { 
        const MATE_VALUE: f32 = 1000000.0;
        
        /*
        let movegen = MoveGen::new_legal(&self.board);
        let (captures, quiet_moves): (Vec<ChessMove>, Vec<ChessMove>) = movegen
            .partition(|m| self.board.piece_on(m.get_dest()).is_some());

        let mut moves = captures;
        moves.extend(quiet_moves);
        */

        let moves = chessutil::output_sorted_move_list(&self.board);
        
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

    fn iterative_deepening_search(&mut self, time_limit: u128, max_depth: usize, my_color: chess::Color) -> Option<ChessMove>{
        
        
        let timer = std::time::Instant::now();

        let mut best_move: Option<ChessMove> = None;
        let mut prev_best_move: Option<ChessMove> = None;

        let mut best_score: f32 = f32::NEG_INFINITY;
        
        const MATE_VALUE: f32 = 100000.0;

        'depth_loop: for depth in 1..max_depth{
            info!("=============================================");
            let mut elapsed_time = timer.elapsed().as_millis();
            if elapsed_time >= time_limit{
                break 'depth_loop;
            }else{
                info!("Depth {depth}: we have enough time!");
            }

            let mut moves = chessutil::output_sorted_move_list(&self.board);
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

            'move_loop: for chess_move in moves{
                elapsed_time = timer.elapsed().as_millis();
                info!("----Elapsed {elapsed_time}");
                
                if elapsed_time >= time_limit{
                    info!("breaking");
                    finished_this_depth = false;
                    break 'depth_loop;
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
                    break 'move_loop;
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

    */

    fn calculate_think_time_ms(&mut self, go_command: &UCIGoCommand, my_color: chess::Color) -> i32{
        //go command is in ms        
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

            let base_time = my_time.unwrap_or(30000);
            let increment = my_inc.unwrap_or(0);  

           
            //let egp = early_game_probability(&self.board);

            let proportion_of_game = 1.0/20.0;
            let time_value = ((base_time as f32)*proportion_of_game) as i32 + (increment/2);

            /*
            if egp > 0.5{
                let mut quick_time = increment + 1500;
                quick_time = quick_time.min(3000);
                return quick_time.min(time_value);//use min(3,inc+1.5) seconds, but less if we are in blitz
            }
            */
            
            return time_value;
        }
    }


    fn generate_move(&mut self, _think_time_ms: i32, my_color: chess::Color) -> Option<ChessMove>{ 
        //let (eval,chessmove) = search::search_alpha_beta(self, self.board, 8, -100000.0, 100000.0, self.board.side_to_move(), true, None, None, 1000*60*100);
        let (eval,chessmove) = search::iterative_deepening_search_with_time(self, self.board, 12, 6.0, _think_time_ms as u32, my_color, true);
        
        return chessmove;
        //return self.iterative_deepening_search(_think_time_ms as u128, 12, my_color)
    }

    fn handle_go(&mut self, tokens: &[&str]) {
        self.nodes_visited = 0;
        let my_color: chess::Color = self.board.side_to_move();
        let go_command: UCIGoCommand = UCIGoCommand::new(tokens);

        let think_time_ms: i32 = self.calculate_think_time_ms(&go_command, my_color);
        info!("think for {} ms",think_time_ms);
        

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
                "searchbenchmark" => self.handle_searchbenchmark(&tokens),
                "evaluate" => self.handle_evaluate(),
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