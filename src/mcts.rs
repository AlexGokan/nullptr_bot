
use crate::{chessutil, evaluation::evaluate};
use chess::{Board, BoardStatus, ChessMove, Game, MoveGen};
use log::{debug,info,warn,error};
use rand::Rng;

use chessutil::output_sorted_move_list;

pub struct GametreeNode{
    pub board: chess::Board,
    pub children: Vec<GametreeNode>,
    pub num_times_visited: i32,
    pub num_times_won: i32,
    pub move_leading_to_me: Option<ChessMove>
}

impl GametreeNode{
    pub fn new(b: chess::Board, m: Option<ChessMove>) -> Self{
        GametreeNode { board: b, 
            children: Vec::new(),
            num_times_visited: 0, 
            num_times_won: 0,
            move_leading_to_me: m}
    }
}

pub fn mcts_selection_expansion(root: &mut GametreeNode) -> &GametreeNode{
    /*
    select succesive child nodes until a leaf L is reached
    a leaf is any node that has a potential child (and?) from which no playout has been performed
    a leaf is therefore any non-terminal board that hasn't had a playout done

    I don't feel bad about making this function recursive since it will be at most like 150 nodes deep I think
    although I should probably make it iterative later for speed, idk
    */

    if(root.board.status() == chess::BoardStatus::Ongoing && root.num_times_visited == 0) {
        return root;
    }else{
        //if there is no children list, generate it
        if(root.children.len() == 0){
            let mg_moves = MoveGen::new_legal(&root.board);
            for chessmove in mg_moves{
                let b2 = root.board.make_move_new(chessmove);
                root.children.push(GametreeNode::new(b2,Some(chessmove)));
            }
        }

        //pick one child from random from the list
        let rand_idx = rand::thread_rng().gen_range(0..root.children.len());
        
        return mcts_selection_expansion(&mut root.children[rand_idx]);
        
    }

}



pub fn mcts_simulation(){

}

pub fn mcts_backpropogation(){

}

pub fn pure_mcts_search(board: chess::Board, playout_depth: i32){
    let mut root = GametreeNode::new(board,None);


    let selected_node: &GametreeNode = mcts_selection_expansion(&mut root);
}

pub fn playout(board: chess::Board){
    let mut b2 = board.clone();
    
    let mut rng = rand::thread_rng();

    let mut moves_played: u16 = 0;
    while(b2.status() == BoardStatus::Ongoing && moves_played < 150){
        let movegen: Vec<ChessMove> = output_sorted_move_list(&b2);
        let num_moves = movegen.len();

        let rand_idx = rng.gen_range(0..num_moves);

        let m = movegen[rand_idx];
        let mut b2copy = b2.clone();
        b2.make_move(m, &mut b2copy);
        b2 = b2copy;

        info!("{m}");
        moves_played += 1;
    }
    if(moves_played >= 150){
        info!("Hit depth limit");
    }else{
        info!("Game finished");
    }


    

}