

use chess::{BoardStatus, ChessMove};

use crate::{chessutil, evaluation::evaluate, ChessEngine};

use log::{debug,info,warn,error};


pub fn search_alpha_beta(engine: &mut ChessEngine, board: chess::Board, depth: usize, mut alpha: f32, mut beta: f32, 
                        my_color: chess::Color, my_move: bool, movelist: Option<Vec<ChessMove>>,
                        timer: Option<&std::time::Instant>, time_limit: u128
                        )
                         -> (f32, Option<ChessMove>){
    //can take a pre-supplied move list

    const MATE_VALUE: f32 = 100000.0;


    if depth == 0{
        let eval = evaluate(&board, my_color);
        engine.nodes_visited += 1;
        return (eval,None);
    }

    let mut moves: Vec<ChessMove> = vec![];
    match movelist{
        Some(ml) => {
            moves = ml;
        }
        None => {
            moves = chessutil::output_sorted_move_list(board);
        }
    }
    
    let num_moves = moves.len();
    //info!("{num_moves} moves!!!!!!");

    if moves.is_empty(){
        match board.status(){
            BoardStatus::Stalemate => {
                return (0.0,None);
            }
            BoardStatus::Checkmate => {
                if board.side_to_move() == my_color{
                    let eval = -MATE_VALUE - (depth as f32);
                    return (eval,None)
                }else{
                    let eval = MATE_VALUE + (depth as f32);
                    return (eval,None)
                }
            }
            BoardStatus::Ongoing => {}
            _ => {}
        }
    }

    
    if my_move{
        let mut max_eval: f32 = -MATE_VALUE;
        let mut best_move: Option<ChessMove> = None;

        for chess_move in moves{
            match timer{
                Some(t) => {
                    let elapsed = t.elapsed().as_millis();
                    if elapsed >= time_limit{
                        return (max_eval,best_move);
                    }
                }
                None => {}
            }
            let board_copy = board.clone();
            let mut new_board = board.clone();
            board_copy.make_move(chess_move, &mut new_board);

            let (eval,_) = search_alpha_beta(engine, new_board, depth-1, alpha, beta, my_color, false, None, timer, time_limit);

            if eval > max_eval{
                max_eval = eval;
                best_move = Some(chess_move);
            }

            alpha = alpha.max(eval);
            if beta <= alpha{
                break;
            }
        }
        return (max_eval,best_move);

    }else{
        let mut min_eval: f32 = MATE_VALUE;
        let mut worst_move: Option<ChessMove> = None;
        for chess_move in moves{
            match timer{
                Some(t) => {
                    let elapsed = t.elapsed().as_millis();
                    //info!("----sub-elapsed: {elapsed}");

                    if elapsed >= time_limit{
                        return (min_eval,worst_move);
                    }
                }
                None => {
                    //info!("couldn't match timer");
                }
            }
            let board_copy = board.clone();
            let mut new_board = board.clone();
            board_copy.make_move(chess_move, &mut new_board);

            let (eval,_) = search_alpha_beta(engine, new_board, depth-1, alpha, beta, my_color, true, None, timer, time_limit);

            if eval < min_eval{
                min_eval = eval;
                worst_move = Some(chess_move);
            }
            
            beta = beta.min(eval);
            if beta <= alpha{
                break;
            }
        }
        return (min_eval,worst_move);

    }
}

pub fn iterative_deepening_search(engine: &mut ChessEngine, board: chess::Board, max_depth: usize, time_limit: u128, my_color: chess::Color, my_move: bool) -> (f32, Option<ChessMove>){
    const MATE_VALUE: f32 = 100000.0;
    
    let timer = std::time::Instant::now();

    let mut best_move: Option<ChessMove> = None;
    let mut prev_best_move: Option<ChessMove> = None;

    let mut best_score: f32 = -MATE_VALUE;


    'depth_loop: for depth in 1..max_depth{
        let mut elapsed_time = timer.elapsed().as_millis();
        info!("Depth: {depth}   Elapsed: {elapsed_time}");
        if elapsed_time >= time_limit{
            break 'depth_loop;
        }

        let mut moves = chessutil::output_sorted_move_list(board);
        match prev_best_move{//prepend previous best move to list if it exists
                Some(cm) => {
                    moves.retain(|&m| m!= cm);
                    moves.insert(0, cm);

                }
                None =>{}
            }
        
        let num_moves = moves.len();

        let mut curr_best_move: Option<ChessMove> = None;
        let mut curr_best_score: f32 = -MATE_VALUE;

        let mut finished_this_depth = true;
        
        let (eval,bm) = search_alpha_beta(engine, board, depth, -MATE_VALUE, MATE_VALUE, my_color, my_move, Some(moves), Some(&timer),time_limit);
        if eval > curr_best_score{
            curr_best_score = eval;
            curr_best_move = bm;
        }

    
        if finished_this_depth{
            match curr_best_move{
                Some(cbm) =>{
                    best_move = curr_best_move;
                    best_score = curr_best_score;
                    prev_best_move = curr_best_move;
                }
                None => {}
            }
        }
    
        elapsed_time = timer.elapsed().as_millis();
        info!("End of D{depth}     Elapsed: {elapsed_time}");

    }

    return (best_score,best_move);

}



