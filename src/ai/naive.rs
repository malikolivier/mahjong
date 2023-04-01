use crate::{
    game::{
        count_shanten, find_machi, find_machi_incomplete, Game, GameRequest, PossibleActions,
        ThrowableOnRiichi,
    },
    tiles::{Fon, Hai},
};

use super::{Call, PossibleCall, TehaiIndex, TurnResult};

/// Only call ron
pub fn handle_call(possible_calls: &[PossibleCall], _: &GameRequest) -> Option<Call> {
    for call in possible_calls {
        if *call == PossibleCall::Ron {
            return Some(Call::Ron);
        }
    }

    None
}

/// Try to minimize shanten with the more machi possible.
/// Then announce riichi when tempai.
///
/// NB:
///  - Never use kyusyukyuhai
///  - Only announce ankan if it does not reduce the number of waits (TODO)
///  - Always call shominkan (as it does not change the player's wait and likeky
///    to increase points with a dora in most cases)
pub fn handle_turn(possible_actions: &PossibleActions, request: &GameRequest) -> TurnResult {
    if possible_actions.can_tsumo {
        return TurnResult::Tsumo;
    }

    // Announce riichi whenever it cans
    if !possible_actions.can_riichi.is_empty() {
        let index = choose_riichi_tile(&possible_actions.can_riichi, request.player, &request.game);
        println!("Riichi!");
        return TurnResult::ThrowHai {
            index,
            riichi: true,
        };
    }

    if let Some(hai) = possible_actions.can_shominkan.first() {
        let index = request
            .game
            .player_te_(request.player)
            .index(*hai)
            .expect("Has kakan tile");
        return TurnResult::Kakan { index };
    }

    let index = choose_tile_to_throw(request.player, &request.game);
    TurnResult::ThrowHai {
        index,
        riichi: false,
    }
}

fn choose_tile_to_throw(player: Fon, game: &Game) -> TehaiIndex {
    if game.player_is_riichi(player) {
        // You cannot cheat and throw another tile after calling riichi!
        return TehaiIndex::Tsumohai;
    }

    let te = game.player_te_(player);
    let mut candidates: Vec<_> = te.hai_closed_all().collect();
    candidates.sort();
    candidates.dedup();

    candidates.sort_by_key(|thrown_tile| {
        // Minimize shanten
        let mut tiles: Vec<_> = te.hai_closed_all().collect();
        let index = tiles.iter().position(|h| h == thrown_tile).unwrap();
        tiles.remove(index);

        let shanten = count_shanten(&tiles);

        // Count machi to maximize their numbers on each throw
        let machi_count = find_machi_incomplete(&tiles).len();
        (shanten, usize::MAX - machi_count)
    });

    let chosen_tile = candidates[0];
    te.index(chosen_tile).expect("Candidate should be in hand")
}

fn choose_riichi_tile(
    throwable_tiles: &[ThrowableOnRiichi],
    player: Fon,
    game: &Game,
) -> TehaiIndex {
    let mut tiles = throwable_tiles.to_vec();
    tiles.sort_by_key(|tile| usize::MAX - machi_count_on_throwing_tile(tile, player, game));

    // Throw tiles so that our hand has as much waits as possible
    let tile_to_throw = tiles[0];
    // TODO: Check and avoid furiten
    match tile_to_throw {
        ThrowableOnRiichi::Te(index) => TehaiIndex::Tehai(index),
        ThrowableOnRiichi::Tsumohai => TehaiIndex::Tsumohai,
    }
}

fn machi_count_on_throwing_tile(tile: &ThrowableOnRiichi, player: Fon, game: &Game) -> usize {
    let index = match tile {
        ThrowableOnRiichi::Te(index) => TehaiIndex::Tehai(*index),
        ThrowableOnRiichi::Tsumohai => TehaiIndex::Tsumohai,
    };
    let mut te = game.player_te_(player).clone();
    te.remove(index);
    let tiles: Vec<Hai> = te.hai_closed_all().collect();

    machi_count(&tiles, player, game)
}

fn machi_count(te: &[Hai], player: Fon, game: &Game) -> usize {
    let machis = find_machi(te);
    println!("Te:{:?} Machis:{:?}", te, &machis);
    let mut count = 0;
    for machi in machis {
        // Count visible tiles on the board
        let visible = game.how_many_visible(machi, player);
        let remaining_count = 4 - visible;
        println!("Found {machi} {visible} times! Still {remaining_count} left!");
        count += remaining_count;
    }
    count
}
