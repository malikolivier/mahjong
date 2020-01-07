use super::game::Game;
use super::tiles::Fon;

pub trait AI {
    fn call(&self, game: &Game, player: Fon, allowed_calls: &[PossibleCall]) -> Option<Call>;
    fn do_turn(&self, game: &Game, player: Fon) -> TurnResult;
}

#[derive(Copy, Eq, PartialEq, PartialOrd, Ord, Clone)]
pub enum Call {
    /// Call a Chi. Includes the index of a tile in the chi.
    Chi { index: usize },
    /// Call a Pon
    Pon,
    /// Call a Kan
    Kan,
    /// Call a Ron
    Ron,
}

pub enum PossibleCall {
    Chi,
    Pon,
    Kan,
    Ron,
}

pub enum TurnResult {
    ThrowHai { index: usize, riichi: bool },
    ThrowTsumoHai { riichi: bool },
    Tsumo,
    Kyusyukyuhai,
}
