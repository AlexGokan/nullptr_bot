
pub struct UCIGoCommand{
    pub ponder: Option<bool>,
    pub wtime: Option<i32>,
    pub btime: Option<i32>,
    pub winc: Option<i32>,
    pub binc: Option<i32>,
    pub movestogo: Option<i32>,
    pub depth: Option<i32>,
    pub nodes: Option<i32>,
    pub mate: Option<i32>,
    pub movetime: Option<i32>,
    pub infinite: Option<bool>
}

impl UCIGoCommand{
    pub fn new(tokens: &[&str]) -> Self{
        let mut ponder: Option<bool> = None;
        let mut wtime: Option<i32> = None;
        let mut btime: Option<i32> = None;
        let mut winc: Option<i32> = None;
        let mut binc: Option<i32> = None;
        let mut movestogo: Option<i32> = None;        
        let mut depth: Option<i32> = None;
        let mut nodes: Option<i32> = None;
        let mut mate: Option<i32> = None;
        let mut movetime: Option<i32> = None;
        let mut infinite: Option<bool> = None;
        
        let mut i: usize = 1;

        while i < tokens.len(){
            match tokens[i] {
                "ponder" =>{
                    ponder = Some(true);
                }                
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
                "movestogo" if i + 1 < tokens.len() => {
                    movestogo = tokens[i + 1].parse().ok();
                    i += 2;
                }
                "depth" if i + 1 < tokens.len() => {
                    depth = tokens[i + 1].parse().ok();
                    i += 2;
                }
                "nodes" if i + 1 < tokens.len() => {
                    nodes = tokens[i + 1].parse().ok();
                    i += 2;
                }
                "mate" if i + 1 < tokens.len() => {
                    mate = tokens[i + 1].parse().ok();
                    i += 2;
                }
                "movetime" if i + 1 < tokens.len() => {
                    movetime = tokens[i + 1].parse().ok();
                    i += 2;
                }
                "infinite" => {
                    infinite = Some(true);
                    i += 1;
                }
                _ => i += 1,
            }
        }

        UCIGoCommand { 
            ponder: ponder,
            wtime: wtime, 
            btime: btime, 
            winc: winc, 
            binc: binc, 
            movestogo: movestogo, 
            depth: depth, 
            nodes: nodes, 
            mate: mate, 
            movetime: movetime,
            infinite: infinite
        }
    }
}