use super::game::Game;
use super::tiles::Fon;

pub trait AI {
    fn call(&self, game: &Game, player: Fon, allowed_calls: &[Call]) -> Option<Call>;
    fn do_turn(&self, game: &Game, player: Fon) -> TurnResult;
}

#[derive(Copy, Eq, PartialEq, PartialOrd, Ord, Clone)]
pub enum Call {
    Chi = 1, // Call a Chi
    Pon = 2, // Call a Pon
    Kan = 3, // Call a Kan
    Ron = 4, // Call a Ron
}

pub enum TurnResult {
    ThrowHai { index: usize, riichi: bool },
    ThrowTsumoHai { riichi: bool },
    Tsumo,
    Kyusyukyuhai,
}
