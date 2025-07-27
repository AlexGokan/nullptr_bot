use chess::{ChessMove, MoveGen};


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
