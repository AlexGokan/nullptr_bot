

use chess::{BoardStatus, ChessMove};

use crate::{chessutil, evaluation::evaluate, ChessEngine};

use log::{debug,info,warn,error};


pub fn search_alpha_beta(engine: &mut ChessEngine, board: chess::Board, depth: usize, mut alpha: f32, mut beta: f32, 
                        my_color: chess::Color, my_move: bool, movelist: Option<Vec<ChessMove>>,
                        timer: Option<&std::time::Instant>, time_limit: u32
                        )
                         -> (f32, Option<ChessMove>,bool){
    //can take a pre-supplied move list
    const MATE_VALUE: f32 = 100000.0;


    if depth == 0{
        let eval = evaluate(&board, my_color);
        engine.nodes_visited += 1;
        return (eval,None,true);
    }

    let mut moves: Vec<ChessMove> = vec![];
    match movelist{
        Some(ml) => {
            moves = ml;
        }
        None => {
            moves = chessutil::output_sorted_move_list(&board);
        }
    }
    
    let num_moves = moves.len();
    //info!("{num_moves} moves!!!!!!");

    if moves.is_empty(){
        match board.status(){
            BoardStatus::Stalemate => {
                return (0.0,None,true);
            }
            BoardStatus::Checkmate => {
                if board.side_to_move() == my_color{
                    let eval = -MATE_VALUE - (depth as f32);
                    return (eval,None,true)
                }else{
                    let eval = MATE_VALUE + (depth as f32);
                    return (eval,None,true)
                }
            }
            BoardStatus::Ongoing => {}
            _ => {}
        }
    }

    
    if my_move{
        let mut max_eval: f32 = -MATE_VALUE;
        let mut best_move: Option<ChessMove> = None;

        let mut finished_inner_search = false;

        for chess_move in moves{
            match timer{
                Some(t) => {
                    let elapsed = t.elapsed().as_millis() as u32;
                    if elapsed >= time_limit{
                        return (max_eval,best_move,false);
                    }
                }
                None => {}
            }
            let board_copy = board.clone();
            let mut new_board = board.clone();
            board_copy.make_move(chess_move, &mut new_board);

            let (eval,_,finished_this_inner_search) = search_alpha_beta(engine, new_board, depth-1, alpha, beta, my_color, false, None, timer, time_limit);
            finished_inner_search = finished_this_inner_search;

            if eval > max_eval{
                max_eval = eval;
                best_move = Some(chess_move);
            }

            alpha = alpha.max(eval);
            if beta <= alpha{
                break;
            }
        }
        return (max_eval,best_move,finished_inner_search);

    }else{
        let mut min_eval: f32 = MATE_VALUE;
        let mut worst_move: Option<ChessMove> = None;

        let mut finished_inner_search = false;
        for chess_move in moves{
            match timer{
                Some(t) => {
                    let elapsed = t.elapsed().as_millis() as u32;
                    //info!("----sub-elapsed: {elapsed}");

                    if elapsed >= time_limit{
                        return (min_eval,worst_move,false);
                    }
                }
                None => {
                    //info!("couldn't match timer");
                }
            }
            let board_copy = board.clone();
            let mut new_board = board.clone();
            board_copy.make_move(chess_move, &mut new_board);

            let (eval,_,finished_this_inner_search) = search_alpha_beta(engine, new_board, depth-1, alpha, beta, my_color, true, None, timer, time_limit);
            finished_inner_search = finished_this_inner_search;

            if eval < min_eval{
                min_eval = eval;
                worst_move = Some(chess_move);
            }
            
            beta = beta.min(eval);
            if beta <= alpha{
                break;
            }
        }
        return (min_eval,worst_move,finished_inner_search);

    }
}

pub fn quiescence_search(engine: &mut ChessEngine, board: chess::Board, qs_depth_hard_limit: u32, mut alpha: f32, mut beta: f32,
                        my_color: chess::Color, my_move: bool,
                        )
                        -> (f32, Option<ChessMove>, bool){

    if qs_depth_hard_limit <= 0{
        engine.nodes_visited += 1;
        let eval = evaluate(&board, my_color);
        return (eval,None,true);
    }
    
    let stand_pat = evaluate(&board, my_color);
    engine.nodes_visited += 1;
    
    if my_move{//maximizing
        if stand_pat >= beta{
            return (stand_pat,None,true);
        }
        alpha = alpha.max(stand_pat);

        let all_moves = chessutil::output_sorted_move_list(&board);
        let (captures, quiet_moves): (Vec<ChessMove>, Vec<ChessMove>) = all_moves.iter()
            .partition(|m| board.piece_on(m.get_dest()).is_some());

        for cap in captures{
            let board_copy = board.clone();
            let mut new_board = board.clone();
            board_copy.make_move(cap, &mut new_board);
            let (eval,_,_) = quiescence_search(engine, new_board, qs_depth_hard_limit-1,alpha, beta, my_color, false);
            if eval >= beta{
                return (beta,Some(cap),true);
            }
            alpha = alpha.max(eval);
        }
    }
    else{//minimizing
        if stand_pat <= alpha{
            return (stand_pat,None,true);
        }
        beta = beta.min(stand_pat);
        let all_moves = chessutil::output_sorted_move_list(&board);
        let (captures, quiet_moves): (Vec<ChessMove>, Vec<ChessMove>) = all_moves.iter()
            .partition(|m| board.piece_on(m.get_dest()).is_some());

        for cap in captures{
            let board_copy = board.clone();
            let mut new_board = board.clone();
            board_copy.make_move(cap, &mut new_board);
            let (eval,_,_) = quiescence_search(engine, new_board, qs_depth_hard_limit-1,alpha, beta, my_color, true);
            if eval <= alpha{
                return (alpha,Some(cap),true);
            }
            beta = beta.min(eval);
        }
    }
    return (stand_pat,None,true);//fallback case
}

pub fn search_alpha_beta_with_quiescence(engine: &mut ChessEngine, board: chess::Board, depth: usize, mut alpha: f32, mut beta: f32, 
                        my_color: chess::Color, my_move: bool, movelist: Option<Vec<ChessMove>>,
                        timer: Option<&std::time::Instant>, time_limit: u32
                        )
                         -> (f32, Option<ChessMove>,bool){
    /*
    This function is pretty bad. There is probably a bug, but also in most moves we are not hitting the ply limit
    We time out on most moves, so therefore going deeper on a few of them doesn't really do much I think
     */

    const MATE_VALUE: f32 = 100000.0;


    if depth == 0{
        let (eval,_,_) =  quiescence_search(engine, board, 1,alpha, beta, my_color, my_move);
        return (eval,None,true);
    }

    let mut moves: Vec<ChessMove> = vec![];
    match movelist{
        Some(ml) => {
            moves = ml;
        }
        None => {
            moves = chessutil::output_sorted_move_list(&board);
        }
    }
    
    let num_moves = moves.len();
    //info!("{num_moves} moves!!!!!!");

    if moves.is_empty(){
        match board.status(){
            BoardStatus::Stalemate => {
                return (0.0,None,true);
            }
            BoardStatus::Checkmate => {
                if board.side_to_move() == my_color{
                    let eval = -MATE_VALUE - (depth as f32);
                    return (eval,None,true)
                }else{
                    let eval = MATE_VALUE + (depth as f32);
                    return (eval,None,true)
                }
            }
            BoardStatus::Ongoing => {}
            _ => {}
        }
    }

    
    if my_move{
        let mut max_eval: f32 = -MATE_VALUE;
        let mut best_move: Option<ChessMove> = None;

        let mut finished_inner_search = false;

        for chess_move in moves{
            match timer{
                Some(t) => {
                    let elapsed = t.elapsed().as_millis() as u32;
                    if elapsed >= time_limit{
                        return (max_eval,best_move,false);
                    }
                }
                None => {}
            }
            let board_copy = board.clone();
            let mut new_board = board.clone();
            board_copy.make_move(chess_move, &mut new_board);

            let move_is_capture = board.piece_on(chess_move.get_dest()).is_some();

            
            let (eval,_,finished_this_inner_search) = search_alpha_beta_with_quiescence(engine, new_board, depth-1, alpha, beta, my_color, false, None, timer, time_limit);
            
            finished_inner_search = finished_this_inner_search;

            if eval > max_eval{
                max_eval = eval;
                best_move = Some(chess_move);
            }

            alpha = alpha.max(eval);
            if beta <= alpha{
                break;
            }
        }
        return (max_eval,best_move,finished_inner_search);

    }else{
        let mut min_eval: f32 = MATE_VALUE;
        let mut worst_move: Option<ChessMove> = None;

        let mut finished_inner_search = false;
        for chess_move in moves{
            match timer{
                Some(t) => {
                    let elapsed = t.elapsed().as_millis() as u32;
                    //info!("----sub-elapsed: {elapsed}");

                    if elapsed >= time_limit{
                        return (min_eval,worst_move,false);
                    }
                }
                None => {
                    //info!("couldn't match timer");
                }
            }
            let board_copy = board.clone();
            let mut new_board = board.clone();
            board_copy.make_move(chess_move, &mut new_board);

            //if chess_move is a capture, search to depth d, and not d-1
            let move_is_capture = board.piece_on(chess_move.get_dest()).is_some();

                
            let (eval,_,finished_this_inner_search) = search_alpha_beta_with_quiescence(engine, new_board, depth-1, alpha, beta, my_color, false, None, timer, time_limit);
        
            
            finished_inner_search = finished_this_inner_search;

            if eval < min_eval{
                min_eval = eval;
                worst_move = Some(chess_move);
            }
            
            beta = beta.min(eval);
            if beta <= alpha{
                break;
            }
        }
        return (min_eval,worst_move,finished_inner_search);

    }
}


pub fn iterative_deepening_search_with_time(engine: &mut ChessEngine, board: chess::Board, max_depth: usize, base_time_scale: f32, hard_time_limit: u32, my_color: chess::Color, my_move: bool)
                                    -> (f32, Option<ChessMove>){


    let base_time = ((hard_time_limit as f32) / base_time_scale) as u32;

    const MATE_VALUE: f32 = 100000.0;
    
    let timer = std::time::Instant::now();

    let mut target_duration = base_time;
    let mut difficulty_mult: f32 = 1.0;  


    let mut best_move: Option<ChessMove> = None;
    let mut prev_best_move: Option<ChessMove> = None;

    let mut best_score: f32 = -MATE_VALUE;

    let mut score_history: Vec<i32>  = vec![];

    'depth_loop: for depth in 1..max_depth{
        let mut elapsed_time = timer.elapsed().as_millis() as u32;
        info!("Depth: {depth}   Elapsed: {elapsed_time}");
        if elapsed_time >= target_duration{
            break 'depth_loop;
        }
        if elapsed_time >= hard_time_limit{
            break 'depth_loop;
        }

        let mut moves = chessutil::output_sorted_move_list(&board);
        match prev_best_move{//prepend the previous best move to list if it exists
                //there exist other schemes such as using the list of moves sorted by evaluation
                //but in general just using the best move first is ok
                //storing the entire list and sorting it provides minimal gain for a lot (?) of computation
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
        

        let ab_time_limit = target_duration.min(hard_time_limit);//in case the hard limit is sooner than our new limit, send that one
        let (eval,bm,finished_ab) = search_alpha_beta(engine, board, depth, -MATE_VALUE, MATE_VALUE, my_color, my_move, Some(moves), Some(&timer),ab_time_limit);
                
        
        if eval > curr_best_score{
            curr_best_score = eval;
            curr_best_move = bm;
        }

    
        if finished_ab{
            match curr_best_move{
                Some(cbm) =>{
                    best_move = curr_best_move;
                    best_score = curr_best_score;
                    prev_best_move = curr_best_move;
                }
                None => {}
            }
        }

        //---------adjust timer for unstable evaluations-------------
        //best move instability
        if prev_best_move.is_some() && curr_best_move != prev_best_move{
            difficulty_mult *= 1.30;
        }

        //score history instability
        score_history.push((best_score*1000.0) as i32);//since floats don't have a max()
        if score_history.len() >= 3{
            let default_eval: i32 = 0;
            let max_recent_score = score_history[score_history.len()-3..].iter().max().unwrap_or(&default_eval);
            let min_recent_score = score_history[score_history.len()-3..].iter().min().unwrap_or(&default_eval);
            if ((max_recent_score - min_recent_score) as f32)/1000.0 > 10.0{//this 15 number is arbitrary
                difficulty_mult *= 1.25;
            }
            
        }

        //close alternatives check, not currently possible

        //tactical indicator check
        //if we are currently in check
        let num_checkers = board.checkers().popcnt();
        if num_checkers > 0{
            difficulty_mult *= 1.2;
        }

        //if our best move puts the opponent in check
        match best_move{
            Some(bm) => {
                let board_after_bm = board.make_move_new(bm);
                let num_checkers = board_after_bm.checkers().popcnt();
                if num_checkers > 0{
                    difficulty_mult *= 1.2;
                }
            }
            None => {}
        }
        


        match best_move{
            Some(bm) => {
                //if our best move is a capture
                let bm_is_a_capture = board.piece_on(bm.get_dest()).is_some();
                if bm_is_a_capture{
                    difficulty_mult *= 1.2;
                }

                match board.piece_on(bm.get_source()){
                    Some(p) => {
                        //if our best move is a promotion
                        if p == chess::Piece::Pawn && (bm.get_dest().get_rank()  == chess::Rank::Eighth || bm.get_dest().get_rank()  == chess::Rank::First){
                            difficulty_mult *= 1.2;//promotion
                        }
                    }
                    None => {}
                }
                
            }
            None => {}
        }

        target_duration = ((base_time as f32) * difficulty_mult.min(base_time_scale)) as u32;
    
        elapsed_time = timer.elapsed().as_millis() as u32;

        
        info!("End of D{depth}     Elapsed: {elapsed_time}");
        info!("Difficulty mult for next iteration: {difficulty_mult}");
        info!("New time limit: {target_duration}");


    }

    return (best_score,best_move);

}

pub fn iterative_deepening_search(engine: &mut ChessEngine, board: chess::Board, max_depth: usize, time_limit: u32, my_color: chess::Color, my_move: bool)
                                    -> (f32, Option<ChessMove>){
    const MATE_VALUE: f32 = 100000.0;
    
    let timer = std::time::Instant::now();

    let mut best_move: Option<ChessMove> = None;
    let mut prev_best_move: Option<ChessMove> = None;

    let mut best_score: f32 = -MATE_VALUE;


    'depth_loop: for depth in 1..max_depth{
        let mut elapsed_time = timer.elapsed().as_millis() as u32;
        info!("Depth: {depth}   Elapsed: {elapsed_time}");
        if elapsed_time >= time_limit{
            break 'depth_loop;
        }

        let mut moves = chessutil::output_sorted_move_list(&board);
        match prev_best_move{//prepend the previous best move to list if it exists
                //there exist other schemes such as using the list of moves sorted by evaluation
                //but in general just using the best move first is ok
                //storing the entire list and sorting it provides minimal gain for a lot (?) of computation
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
        
        //let (eval,bm,finished_ab) = search_alpha_beta_with_quiescence(engine, board, depth, -MATE_VALUE, MATE_VALUE, my_color, my_move, Some(moves), Some(&timer),time_limit);
        let (eval,bm,finished_ab) = search_alpha_beta(engine, board, depth, -MATE_VALUE, MATE_VALUE, my_color, my_move, Some(moves), Some(&timer),time_limit);
        
        if eval > curr_best_score{
            curr_best_score = eval;
            curr_best_move = bm;
        }

    
        if finished_ab{
            match curr_best_move{
                Some(cbm) =>{
                    best_move = curr_best_move;
                    best_score = curr_best_score;
                    prev_best_move = curr_best_move;
                }
                None => {}
            }
        }
    
        elapsed_time = timer.elapsed().as_millis() as u32;
        info!("End of D{depth}     Elapsed: {elapsed_time}");

    }

    return (best_score,best_move);

}



