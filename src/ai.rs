use super::game::Game;

pub trait AI: Sync + Send {
    fn call(&self, game: &Game, player_index: usize) -> Option<Call>;
}

#[derive(Copy, Eq, PartialEq, PartialOrd, Ord, Clone)]
pub enum Call {
    Tsumo = 1, // Draw a tile
    Chi = 2,   // Call a Chi
    Pon = 3,   // Call a Pon
    Kan = 4,   // Call a Kan
    Ron = 5,   // Call a Ron
}

pub struct PossibleCalls([bool; 4]);
