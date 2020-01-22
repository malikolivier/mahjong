use std::fmt;

use log::{debug, info, trace};
use rand::distributions::{Distribution, Standard};
use rand::seq::SliceRandom;
use rand::Rng;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::ai::{AiServer, Call, PossibleCall, TehaiIndex, TurnResult};
use super::list::OrderedList;
use super::tiles::{make_all_tiles, Fon, Hai, SuuHai, Values};
use super::yaku::{AgariTe, WinningMethod, Yaku};

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone, Serialize, Deserialize)]
pub enum Dice {
    One = 1,
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
    Six = 6,
}

impl Dice {
    pub fn into_char(self) -> char {
        std::char::from_u32(0x267F + self as u32).unwrap()
    }
}

impl Distribution<Dice> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Dice {
        match rng.gen_range(0, 6) {
            0 => Dice::One,
            1 => Dice::Two,
            2 => Dice::Three,
            3 => Dice::Four,
            4 => Dice::Five,
            _ => Dice::Six,
        }
    }
}

#[derive(Clone)]
pub struct Game {
    /// 場の風
    pub wind: Fon,
    /// Current player that should draw
    pub turn: Fon,
    honba: usize,
    kyoku: usize,
    tsumo_cnt: usize,
    /// 4 players indexed by Ton/Nan/Sha/Pee
    players: [Player; 4],
    yama: [Option<Hai>; 136],
    /// 4 rivers indexed by Ton/Nan/Sha/Pee
    hoo: [Hoo; 4],
    dice: [Dice; 2],
    /// What is left on the table in front of each player
    score: [Score; 4],
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
struct Score {
    riichi_bou: usize,
    score: isize,
}

impl Default for Game {
    fn default() -> Self {
        let mut yama = [None; 136];
        for (i, hai) in make_all_tiles().iter().cloned().enumerate() {
            yama[i] = Some(hai);
        }

        Self {
            wind: Fon::Ton,
            turn: Fon::Ton,
            honba: 0,
            kyoku: 0,
            tsumo_cnt: 0,
            players: [
                Player::new(Fon::Ton),
                Player::new(Fon::Nan),
                Player::new(Fon::Shaa),
                Player::new(Fon::Pee),
            ],
            yama,
            hoo: Default::default(),
            dice: [Dice::One, Dice::Six],
            score: [Score {
                riichi_bou: 0,
                score: 25000,
            }; 4],
        }
    }
}

impl fmt::Debug for Game {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut yama = f.debug_list();
        for hai in self.yama.iter() {
            yama.entry(hai);
        }
        let yama = yama.finish();

        f.debug_struct("Game")
            .field("wind", &self.wind)
            .field("turn", &self.turn)
            .field("honba", &self.honba)
            .field("tsumo_cnt", &self.tsumo_cnt)
            .field("players", &self.players)
            .field("yama", &yama)
            .field("hoo", &self.hoo)
            .field("dice", &self.dice)
            .finish()
    }
}

#[derive(Serialize, Deserialize)]
struct GameSerde {
    wind: Fon,
    pub turn: Fon,
    kyoku: usize,
    honba: usize,
    tsumo_cnt: usize,
    players: [Player; 4],
    yama: Vec<Option<Hai>>,
    hoo: [Hoo; 4],
    dice: [Dice; 2],
    score: [Score; 4],
}

impl Serialize for Game {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let game = GameSerde {
            wind: self.wind,
            turn: self.turn,
            kyoku: self.kyoku,
            honba: self.honba,
            tsumo_cnt: self.tsumo_cnt,
            players: self.players.clone(),
            yama: self.yama.iter().cloned().collect(),
            hoo: self.hoo.clone(),
            dice: self.dice,
            score: self.score,
        };
        game.serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for Game {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let game = GameSerde::deserialize(deserializer)?;

        assert!(game.yama.len() == 136);
        let mut yama = [None; 136];
        for i in 0..136 {
            yama[i] = game.yama[i];
        }

        Ok(Game {
            wind: game.wind,
            turn: game.turn,
            kyoku: game.kyoku,
            honba: game.honba,
            players: game.players,
            tsumo_cnt: game.tsumo_cnt,
            yama,
            hoo: game.hoo,
            dice: game.dice,
            score: game.score,
        })
    }
}

#[derive(Debug, Clone)]
pub struct GameRequest {
    pub game: Game,
    pub request: Request,
}

impl GameRequest {
    fn new(game: &Game, request: Request) -> Self {
        Self {
            game: game.clone(),
            request,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Request {
    Refresh,
    Call(Vec<PossibleCall>),
    DoTurn {
        can_tsumo: bool,
        /// List of tiles that can be thrown so that the user can call riichi.
        can_riichi: Vec<ThrowableOnRiichi>,
        can_kyusyukyuhai: bool,
        /// Can call kan on one of these tiles during own turn
        can_shominkan: Vec<Hai>,
        can_ankan: Vec<Hai>,
    },
    DisplayScore(KyokuResult),
}

#[derive(Debug, Clone)]
pub enum KyokuResult {
    Agari {
        /// List of winners with their respective Yaku.
        winners: Vec<(Fon, Vec<Yaku>)>,
        oya_agari: bool,
    },
    Ryukyoku,
}

impl Game {
    pub fn new<R: Rng>(rng: &mut R) -> Self {
        let mut game = Self::default();
        game.reset(rng);
        game
    }
    fn wall_break_index(&self) -> usize {
        let dice_result = self.dice[0] as usize + self.dice[1] as usize;
        let break_point = ((dice_result - 1) % 4) * 34 + dice_result * 2;
        break_point
    }

    fn next_tsumohai_index(&self) -> Option<usize> {
        let break_point = self.wall_break_index();
        let tsumo_cnt_max = 16 * 4 + 1 - self.kan_count();
        if self.tsumo_cnt > tsumo_cnt_max {
            None
        } else {
            let tsumohai_i = (break_point + 4 * 13 + self.tsumo_cnt) % 136;
            Some(tsumohai_i)
        }
    }

    /// Return total number of kans
    fn kan_count(&self) -> usize {
        self.players.iter().fold(0, |acc, p| acc + p.te.kan_count())
    }

    /// Return true if all kans are from the same player
    fn kan_same_player(&self) -> bool {
        let mut any_kan = false;
        for p in &self.players {
            if any_kan {
                return false;
            }
            if p.te.kan_count() > 0 {
                any_kan = true;
            }
        }
        true
    }

    pub fn play_hanchan<R: Rng>(&mut self, mut channels: [AiServer; 4], rng: &mut R) {
        loop {
            let result = self.play(&channels);
            match result {
                KyokuResult::Ryukyoku => {
                    self.honba += 1;
                    // Check tenpai, and move points and players if necessary
                }
                KyokuResult::Agari { oya_agari, .. } => {
                    if oya_agari {
                        self.honba += 1;
                    } else {
                        // Move player
                        channels.rotate_right(1);
                        self.score.rotate_right(1);

                        self.kyoku += 1;
                        if self.kyoku > 3 {
                            self.kyoku = 0;
                            self.wind = self.wind.next();
                            if self.wind > Fon::Nan {
                                info!("Hanchan completed!");
                                // TODO: Hanchan is finished
                                return;
                            }
                        }
                    }
                }
            }
            self.reset(rng);
        }
    }

    /// Play a kyoku
    pub fn play(&mut self, channels: &[AiServer; 4]) -> KyokuResult {
        self.deal();

        loop {
            if let Some(result) = self.next_turn(channels) {
                // End a kyoku
                return result;
            }
        }
    }

    /// Reset the game to the state before any tile is dealt
    fn reset<R: Rng>(&mut self, rng: &mut R) {
        let mut new_game = Self::default();

        new_game.yama.shuffle(rng);
        new_game.dice[0] = rng.gen();
        new_game.dice[1] = rng.gen();

        self.turn = new_game.turn;
        self.tsumo_cnt = new_game.tsumo_cnt;
        self.players = new_game.players;
        self.yama = new_game.yama;
        self.hoo = new_game.hoo;
        self.dice = new_game.dice;
    }

    fn deal(&mut self) {
        let break_point = self.wall_break_index();

        for i in 0..13 {
            for p in 0..4 {
                let tsumohai_i = (break_point + i + p * 13) % 136;
                if let Some(tsumohai) = self.yama[tsumohai_i] {
                    self.players[p].te.hai.insert(tsumohai);
                    self.yama[tsumohai_i] = None;
                } else {
                    info!("Already dealt!");
                    return;
                }
            }
        }
    }

    /// Make turn player draw a tile
    /// Return true if a tile is drawn. Return false if there is no tile left.
    fn draw(&mut self) -> bool {
        if let Some(tsumohai_i) = self.next_tsumohai_index() {
            let tsumohai = self.yama[tsumohai_i].expect("Yama has tile");
            self.yama[tsumohai_i] = None;
            self.players[self.turn as usize].te.set_tsumohai(tsumohai);
            self.tsumo_cnt += 1;
            true
        } else {
            false
        }
    }

    fn draw_from_rinshan(&mut self, p: Fon) {
        let break_point = self.wall_break_index();

        fn previous_tile_index(i: usize) -> usize {
            if i == 0 {
                135
            } else {
                i - 1
            }
        }

        let mut tile_index = previous_tile_index(break_point);
        while self.yama[tile_index].is_none() {
            tile_index = previous_tile_index(tile_index);
        }
        let tsumohai = self.yama[tile_index].expect("Yama has tile");
        self.yama[tile_index] = None;
        self.players[p as usize].te.set_tsumohai(tsumohai);
    }

    fn tx_refresh(&self, channels: &[AiServer; 4]) {
        for channel in channels {
            channel
                .tx
                .send(GameRequest::new(self, Request::Refresh))
                .expect("Sent!");
        }
    }

    /// Returns a boolean whose value is false if this is the last turn
    fn next_turn(&mut self, channels: &[AiServer; 4]) -> Option<KyokuResult> {
        // TODO: Ryukyoku conditions:
        //   - 4 riichi;
        //   - 4 kan from different players;
        //   - same wind thrown 4 times on first turn.

        self.tx_refresh(channels);

        // Listen for chi/pon/kan/ron
        let mut call1 = None;
        let mut call2 = None;
        let mut call3 = None;

        let allowed_calls1 = self.allowed_calls(self.turn);
        if allowed_calls1.len() > 0 {
            trace!("1. Player {} can {:?}!", self.turn as usize, allowed_calls1);
            channels[self.turn as usize]
                .tx
                .send(GameRequest::new(self, Request::Call(allowed_calls1)))
                .expect("Sent!");
            call1 = channels[self.turn as usize]
                .rx_call
                .recv()
                .expect("Received!");
        }

        let allowed_calls2 = self.allowed_calls(self.turn.next());
        if allowed_calls2.len() > 0 {
            trace!(
                "2. Player {} can {:?}!",
                self.turn.next() as usize,
                allowed_calls2
            );
            channels[self.turn.next() as usize]
                .tx
                .send(GameRequest::new(self, Request::Call(allowed_calls2)))
                .expect("Sent!");
            call2 = channels[self.turn.next() as usize]
                .rx_call
                .recv()
                .expect("Received!");
        }

        let allowed_calls3 = self.allowed_calls(self.turn.next().next());
        if allowed_calls3.len() > 0 {
            trace!(
                "3. Player {} can {:?}!",
                self.turn.next().next() as usize,
                allowed_calls3
            );
            channels[self.turn.next().next() as usize]
                .tx
                .send(GameRequest::new(self, Request::Call(allowed_calls3)))
                .expect("Sent!");
            call3 = channels[self.turn.next().next() as usize]
                .rx_call
                .recv()
                .expect("Received!");
        }

        // Check furiten by checking sutehai before they are moved by a
        // potential call.
        self.riichi_furiten_check_on_last_thrown_tile();

        let calls = [call1, call2, call3];
        trace!("Calls: {:?}", &calls);
        match calls {
            [None, None, None] => {
                if !self.draw() {
                    return Some(self.ryukyoku());
                }
                self.do_turn(channels)
            }
            _ => {
                // If any has Ron do single, double or triple ron score calculation.
                // If any has Kan or Pon, do it.
                // If any has Chi do it. In this order.
                let ron_calls: Vec<_> = calls
                    .iter()
                    .enumerate()
                    .filter_map(|(i, call)| {
                        if let Some(Call::Ron) = call {
                            Some(self.turn.next_nth(i))
                        } else {
                            None
                        }
                    })
                    .collect();
                if !ron_calls.is_empty() {
                    let result = self.agari(ron_calls, WinningMethod::Ron);
                    self.send_game_result(result.clone(), channels);
                    Some(result)
                } else {
                    if let Some(pon_kan_player_i) = calls.iter().position(|call| match call {
                        Some(Call::Pon) | Some(Call::Kan) => true,
                        _ => false,
                    }) {
                        let caller = self.turn.next_nth(pon_kan_player_i);
                        info!(
                            "Player {} called {:?}",
                            caller as usize, calls[pon_kan_player_i]
                        );
                        match calls[pon_kan_player_i] {
                            Some(Call::Pon) => {
                                self.call_pon(caller);
                                self.do_turn(channels)
                            }
                            Some(Call::Kan) => self.call_kan(caller, channels),
                            _ => unreachable!("Expect kan or pon"),
                        }
                    } else if let Some(Call::Chi { index }) = call1 {
                        self.call_chi(self.turn, index);
                        self.do_turn(channels)
                    } else {
                        unreachable!("Impossible state!");
                    }
                }
            }
        }
    }

    fn send_game_result(&self, result: KyokuResult, channels: &[AiServer; 4]) {
        for ch in channels {
            ch.tx
                .send(GameRequest::new(
                    self,
                    Request::DisplayScore(result.clone()),
                ))
                .expect("Sent!");
        }
    }

    /// Ask client for what to do then do it.
    ///
    /// Returns end game results if this turn ends the kyoku
    fn do_turn(&mut self, channels: &[AiServer; 4]) -> Option<KyokuResult> {
        channels[self.turn as usize]
            .tx
            .send(GameRequest::new(
                self,
                Request::DoTurn {
                    can_tsumo: self.can_tsumo(),
                    can_riichi: self.can_riichi(),
                    can_kyusyukyuhai: self.can_kyusyukyuhai(),
                    can_shominkan: self.can_shominkan(),
                    can_ankan: self.can_ankan(),
                },
            ))
            .expect("Sent!");
        let result = channels[self.turn as usize]
            .rx_turn
            .recv()
            .expect("Received!");
        match result {
            TurnResult::Tsumo => {
                let result = self.agari(vec![self.turn], WinningMethod::Tsumo);
                self.send_game_result(result.clone(), channels);
                Some(result)
            }
            TurnResult::Kyusyukyuhai => Some(self.ryukyoku()),
            TurnResult::Ankan { index } => self.announce_ankan(index, channels),
            TurnResult::Kakan { index } => self.announce_kakan(index, channels),
            TurnResult::ThrowHai { index, riichi } => {
                self.throw_tile(self.turn, index, riichi);
                self.turn = self.turn.next();
                None
            }
        }
    }

    fn ryukyoku(&mut self) -> KyokuResult {
        // TODO
        KyokuResult::Ryukyoku
    }

    /// Ends a game player with the given players winning.
    fn agari(&mut self, players: Vec<Fon>, winning_method: WinningMethod) -> KyokuResult {
        let loser = self.turn.prev();
        let mut winners = vec![];
        let mut oya_agari = false;
        for winner in players {
            let p = &self.players[winner as usize];
            let hupai = match winning_method {
                WinningMethod::Ron => self.last_thrown_tile().expect("Has last thrown tile"),
                WinningMethod::Tsumo => p.te.tsumo.expect("Has tsumohai"),
            };
            trace!(
                "Compute points for winner player {} (hupai: {})",
                winner as usize,
                hupai.to_char()
            );
            let points = AgariTe::from_te(&p.te, self, hupai, winning_method, winner).points();
            trace!("Points: {:?}", &points);
            winners.push((winner, points.0));

            if winner == Fon::Ton {
                oya_agari = true;
            }
        }

        KyokuResult::Agari { winners, oya_agari }
    }

    fn last_thrown_tile(&self) -> Option<Hai> {
        let player_who_threw_last_tile = self.turn.prev();
        let player_index = player_who_threw_last_tile as usize;
        self.hoo[player_index]
            .river
            .last()
            .map(|sutehai| sutehai.hai())
    }

    fn remove_last_thrown_tile(&mut self) -> Hai {
        let player_who_threw_last_tile = self.turn.prev();
        let player_index = player_who_threw_last_tile as usize;
        self.hoo[player_index]
            .river
            .pop()
            .expect("Has last thrown tile")
            .hai()
    }

    pub fn throw_tile(&mut self, p: Fon, i: TehaiIndex, riichi: bool) {
        let hai = self.players[p as usize].te.throw_and_insert(i);
        self.hoo[p as usize].river.push(if riichi {
            self.players[p as usize].riichi = Some(Riichi {
                ippatsu: true,
                furiten: self.is_furiten(p),
                machi: find_machi(self.players[p as usize].te.hai()),
            });
            self.score[p as usize].riichi_bou += 1;
            self.score[p as usize].score -= 1000;
            SuteHai::Riichi(hai)
        } else {
            if let Some(riichi) = self.players[p as usize].riichi.as_mut() {
                // Remove ippatsu if the user as called riichi
                riichi.ippatsu = false;
            }
            SuteHai::Normal(hai)
        })
    }

    /// Set ippatsu boolean to false.
    /// Used when a call is done. All ippatsu are then cancelled.
    fn remove_ippatsu(&mut self) {
        for p in &mut self.players {
            if let Some(riichi) = p.riichi.as_mut() {
                riichi.ippatsu = false;
            }
        }
    }

    fn riichi_furiten_check_on_last_thrown_tile(&mut self) {
        if let Some(hai) = self.last_thrown_tile() {
            self.riichi_furiten_check(hai);
        }
    }

    /// Check if riichi players are furiten on given hai.
    /// If any player is furiten, the furiten flag will be set.
    fn riichi_furiten_check(&mut self, hai: Hai) {
        for p in &mut self.players {
            if let Some(riichi) = p.riichi.as_mut() {
                if riichi.machi.contains(&hai) {
                    riichi.furiten = true;
                }
            }
        }
    }

    /// p: Wind of the caller.
    pub fn call_chi(&mut self, p: Fon, index: [usize; 2]) {
        let hai = self.remove_last_thrown_tile();
        debug!(
            "Chi called by player {}. Last thrown tile: {}, thrown by player {}",
            p as usize,
            hai.to_char(),
            self.turn.prev() as usize
        );
        let te = &mut self.players[p as usize].te;
        let player_diff = p as isize - self.turn.prev() as isize;
        let direction = match player_diff {
            -3 => Direction::Right,
            -2 => Direction::Front,
            -1 => Direction::Left,
            0 => unreachable!("Caller and callee cannot be the same player!"),
            1 => Direction::Right,
            2 => Direction::Front,
            3 => Direction::Left,
            _ => unreachable!("Modulo 4"),
        };
        te.open_shuntsu(hai, index);
        self.remove_ippatsu();
        self.turn = p;
    }

    /// p: Wind of the caller.
    pub fn call_pon(&mut self, p: Fon) {
        let hai = self.remove_last_thrown_tile();
        debug!(
            "Pon called by player {}. Last thrown tile: {}, thrown by player {}",
            p as usize,
            hai.to_char(),
            self.turn.prev() as usize
        );
        let te = &mut self.players[p as usize].te;
        let player_diff = p as isize - self.turn.prev() as isize;
        let direction = match player_diff {
            -3 => Direction::Right,
            -2 => Direction::Front,
            -1 => Direction::Left,
            0 => unreachable!("Caller and callee cannot be the same player!"),
            1 => Direction::Right,
            2 => Direction::Front,
            3 => Direction::Left,
            _ => unreachable!("Modulo 4"),
        };
        te.open_kootsu(hai, direction);
        self.remove_ippatsu();
        self.turn = p;
    }

    /// Returns a boolean whose value is false if this is the last turn
    pub fn call_kan(&mut self, p: Fon, channels: &[AiServer; 4]) -> Option<KyokuResult> {
        let hai = self.remove_last_thrown_tile();
        debug!(
            "Kan called by player {}. Last thrown tile: {}, thrown by player {}",
            p as usize,
            hai.to_char(),
            self.turn.prev() as usize
        );
        let te = &mut self.players[p as usize].te;
        let player_diff = p as isize - self.turn.prev() as isize;
        let direction = match player_diff {
            -3 => Direction::Right,
            -2 => Direction::Front,
            -1 => Direction::Left,
            0 => unreachable!("Caller and callee cannot be the same player!"),
            1 => Direction::Right,
            2 => Direction::Front,
            3 => Direction::Left,
            _ => unreachable!("Modulo 4"),
        };
        te.daikantsu(hai, direction);
        self.remove_ippatsu();
        self.kan_after(p, channels)
    }

    /// Returns a boolean whose value is false if this is the last turn
    pub fn announce_ankan(
        &mut self,
        i: TehaiIndex,
        channels: &[AiServer; 4],
    ) -> Option<KyokuResult> {
        let te = &mut self.players[self.turn as usize].te;
        te.ankan(i);
        self.kan_after(self.turn, channels)
    }

    /// Returns a boolean whose value is false if this is the last turn
    pub fn announce_kakan(
        &mut self,
        i: TehaiIndex,
        channels: &[AiServer; 4],
    ) -> Option<KyokuResult> {
        let te = &mut self.players[self.turn as usize].te;
        te.kakan(i);
        // TODO: Add chankan check!
        // NB: If a riichi player did not call possible ron on a chankan,
        // they will be in furiten.
        self.kan_after(self.turn, channels)
    }

    /// Returns a boolean whose value is false if this is the last turn
    pub fn kan_after(&mut self, p: Fon, channels: &[AiServer; 4]) -> Option<KyokuResult> {
        self.remove_ippatsu();
        let te = &mut self.players[p as usize].te;
        // Insert tsumohai in te, if any
        if let Some(tsumohai) = te.tsumo.take() {
            te.hai.insert(tsumohai);
        }
        // Draw from mont intouchable and do a standard turn
        self.draw_from_rinshan(p);
        self.turn = p;
        self.do_turn(channels)
    }

    pub fn to_string_repr(&self) -> String {
        let mut grid = unsafe {
            let mut grid: [[String; 25]; 25] = std::mem::zeroed();
            for i in 0..25 {
                for j in 0..25 {
                    grid[i][j] = String::from("  ");
                }
            }
            grid
        };

        // Player 3
        let top_player = &self.players[2];
        let mut offset = 0;
        for fuuro in &top_player.te.fuuro {
            match fuuro {
                Fuuro::Shuntsu { own, taken, from } | Fuuro::Kootsu { own, taken, from } => {
                    let tiles = match from {
                        Direction::Left => [own[0], own[1], *taken],
                        Direction::Front => [own[0], *taken, own[1]],
                        Direction::Right => [*taken, own[0], own[1]],
                    };
                    grid[0][offset] = tiles[0].to_string();
                    grid[0][offset + 1] = tiles[1].to_string();
                    grid[0][offset + 2] = tiles[2].to_string();
                    offset += 4;
                }
                Fuuro::Kantsu(KantsuInner::Ankan { own }) => {
                    grid[0][offset] = own[0].to_string();
                    grid[0][offset + 1] = Hai::back_char().to_string();
                    grid[0][offset + 2] = Hai::back_char().to_string();
                    grid[0][offset + 3] = own[3].to_string();
                    offset += 5;
                }
                Fuuro::Kantsu(KantsuInner::DaiMinkan { own, taken, from }) => {
                    let tiles = match from {
                        Direction::Left => [own[0], own[1], own[2], *taken],
                        Direction::Front => [own[0], *taken, own[1], own[2]],
                        Direction::Right => [*taken, own[0], own[1], own[2]],
                    };
                    grid[0][offset] = tiles[0].to_string();
                    grid[0][offset + 1] = tiles[1].to_string();
                    grid[0][offset + 2] = tiles[2].to_string();
                    grid[0][offset + 3] = tiles[3].to_string();
                    offset += 5;
                }
                Fuuro::Kantsu(KantsuInner::ShouMinkan {
                    own,
                    taken,
                    added,
                    from,
                }) => {
                    let (tiles, taken_pos) = match from {
                        Direction::Left => ([own[0], own[1], *taken], 2),
                        Direction::Front => ([own[0], *taken, own[1]], 1),
                        Direction::Right => ([*taken, own[0], own[1]], 0),
                    };
                    grid[0][offset] = tiles[0].to_string();
                    grid[0][offset + 1] = tiles[1].to_string();
                    grid[0][offset + 2] = tiles[2].to_string();
                    grid[1][offset + taken_pos] = added.to_string();
                    offset += 4;
                }
            }
        }
        for (i, hai) in top_player.te.hai.iter().enumerate() {
            grid[0][i + offset + 5] = hai.to_string();
        }
        if let Some(hai) = top_player.te.tsumo {
            grid[0][top_player.te.hai.len() + 1 + offset + 5] = hai.to_string();
        }
        for (i, sutehai) in self.hoo[2].river.iter().enumerate() {
            let hai = match sutehai {
                SuteHai::Normal(hai) | SuteHai::Riichi(hai) => hai,
            };
            grid[6 - i / 6][14 - i % 6] = hai.to_string();
        }

        // Player 1
        let bottom_player = &self.players[0];
        let mut offset = 0;
        for fuuro in &bottom_player.te.fuuro {
            match fuuro {
                Fuuro::Shuntsu { own, taken, from } | Fuuro::Kootsu { own, taken, from } => {
                    let tiles = match from {
                        Direction::Left => [*taken, own[0], own[1]],
                        Direction::Front => [own[0], *taken, own[1]],
                        Direction::Right => [own[0], own[1], *taken],
                    };
                    grid[24][24 - offset - 2] = tiles[0].to_string();
                    grid[24][24 - offset - 1] = tiles[1].to_string();
                    grid[24][24 - offset] = tiles[2].to_string();
                    offset += 4;
                }
                Fuuro::Kantsu(KantsuInner::Ankan { own }) => {
                    grid[24][24 - offset - 3] = own[0].to_string();
                    grid[24][24 - offset - 2] = Hai::back_char().to_string();
                    grid[24][24 - offset - 1] = Hai::back_char().to_string();
                    grid[24][24 - offset] = own[3].to_string();
                    offset += 5;
                }
                Fuuro::Kantsu(KantsuInner::DaiMinkan { own, taken, from }) => {
                    let tiles = match from {
                        Direction::Left => [*taken, own[0], own[1], own[2]],
                        Direction::Front => [own[0], *taken, own[1], own[2]],
                        Direction::Right => [own[0], own[1], own[2], *taken],
                    };
                    grid[24][24 - offset - 3] = tiles[0].to_string();
                    grid[24][24 - offset - 2] = tiles[1].to_string();
                    grid[24][24 - offset - 1] = tiles[2].to_string();
                    grid[24][24 - offset] = tiles[3].to_string();
                    offset += 5;
                }
                Fuuro::Kantsu(KantsuInner::ShouMinkan {
                    own,
                    taken,
                    added,
                    from,
                }) => {
                    let (tiles, taken_pos) = match from {
                        Direction::Left => ([*taken, own[0], own[1]], 2),
                        Direction::Front => ([own[0], *taken, own[1]], 1),
                        Direction::Right => ([own[0], own[1], *taken], 0),
                    };
                    grid[24][24 - offset - 2] = tiles[0].to_string();
                    grid[24][24 - offset - 1] = tiles[1].to_string();
                    grid[24][24 - offset] = tiles[2].to_string();
                    grid[23][24 - offset - taken_pos] = added.to_string();
                    offset += 4;
                }
            }
        }
        for (i, hai) in bottom_player.te.hai.iter().enumerate() {
            grid[24][i + 5] = hai.to_string();
        }
        if let Some(hai) = bottom_player.te.tsumo {
            grid[24][bottom_player.te.hai.len() + 1 + 5] = hai.to_string();
        }

        for (i, sutehai) in self.hoo[0].river.iter().enumerate() {
            let hai = match sutehai {
                SuteHai::Normal(hai) | SuteHai::Riichi(hai) => hai,
            };
            grid[17 + i / 6][9 + i % 6] = hai.to_string();
        }

        // Player 2
        let left_player = &self.players[1];
        // TODO: Called tiles not done
        for (i, hai) in left_player.te.hai.iter().enumerate() {
            grid[i + 5][0] = hai.to_string();
        }
        if let Some(hai) = left_player.te.tsumo {
            grid[left_player.te.hai.len() + 6][0] = hai.to_string();
        }

        for (i, sutehai) in self.hoo[1].river.iter().enumerate() {
            let hai = match sutehai {
                SuteHai::Normal(hai) | SuteHai::Riichi(hai) => hai,
            };
            grid[8 + i % 6][7 - i / 6] = hai.to_string();
        }

        // Player 4
        let right_player = &self.players[3];
        // TODO: Called tiles not done
        for (i, hai) in right_player.te.hai.iter().enumerate() {
            grid[24 - i - 5][24] = hai.to_string();
        }
        if let Some(hai) = right_player.te.tsumo {
            grid[24 - right_player.te.hai.len() - 6][24] = hai.to_string();
        }

        for (i, sutehai) in self.hoo[3].river.iter().enumerate() {
            let hai = match sutehai {
                SuteHai::Normal(hai) | SuteHai::Riichi(hai) => hai,
            };
            grid[15 - i % 6][15 + i / 6] = hai.to_string();
        }

        // Wall
        for (i, hai) in self.yama.iter().enumerate() {
            if let Some(hai) = hai {
                match i {
                    0..=33 => {
                        if i % 2 == 0 {
                            grid[21][20 - i / 2] = hai.to_string();
                        } else {
                            grid[22][20 - i / 2] = hai.to_string();
                        }
                    }
                    34..=67 => {
                        if i % 2 == 0 {
                            grid[20 - (i - 34) / 2][3] = hai.to_string();
                        } else {
                            grid[20 - (i - 34) / 2][2] = hai.to_string();
                        }
                    }
                    68..=101 => {
                        if i % 2 == 0 {
                            grid[3][4 + (i - 68) / 2] = hai.to_string();
                        } else {
                            grid[2][4 + (i - 68) / 2] = hai.to_string();
                        }
                    }
                    102..=std::usize::MAX => {
                        if i % 2 == 0 {
                            grid[4 + (i - 102) / 2][21] = hai.to_string();
                        } else {
                            grid[4 + (i - 102) / 2][22] = hai.to_string();
                        }
                    }
                    _ => {}
                }
            }
        }

        grid[11][10] = format!(" {}", self.dice[0].into_char());
        grid[11][13] = format!("{} ", self.dice[1].into_char());

        let mut out = String::with_capacity(22 * 21);
        for line in &grid {
            for c in line {
                out.push_str(c);
            }
            out.push('\n');
        }
        out
    }

    pub fn player_te(&self, p: Fon) -> impl Iterator<Item = &Hai> {
        self.players[p as usize].te.hai.iter()
    }
    pub fn player_te_(&self, p: Fon) -> &Te {
        &self.players[p as usize].te
    }
    pub fn player_te_mut(&mut self, p: Fon) -> &mut Te {
        &mut self.players[p as usize].te
    }
    pub fn player_tsumo(&self, p: Fon) -> Option<Hai> {
        self.players[p as usize].te.tsumo
    }
    pub fn player_is_riichi(&self, p: Fon) -> bool {
        self.players[p as usize].riichi.is_some()
    }
    pub fn player_riichi(&self, p: Fon) -> Option<&Riichi> {
        self.players[p as usize].riichi.as_ref()
    }

    fn dora_indicator(&self) -> Vec<Hai> {
        (0..=self.kan_count())
            .map(|i| {
                let break_point = self.wall_break_index();

                fn previous_tile_index(i: usize, nth: usize) -> usize {
                    if i < nth {
                        136 - (nth - i)
                    } else {
                        i - nth
                    }
                }

                let dora_index = previous_tile_index(break_point, 2 * i);
                self.yama[dora_index].expect("Dora not found")
            })
            .collect()
    }
    fn uradora_indicator(&self) -> Vec<Hai> {
        (0..=self.kan_count())
            .map(|i| {
                let break_point = self.wall_break_index();

                fn previous_tile_index(i: usize, nth: usize) -> usize {
                    if i < nth {
                        136 - (nth - i)
                    } else {
                        i - nth
                    }
                }

                let dora_index = previous_tile_index(break_point, 2 * i + 1);
                self.yama[dora_index].expect("Uradora not found")
            })
            .collect()
    }
    pub fn dora(&self) -> Vec<Hai> {
        self.dora_indicator().into_iter().map(Hai::next).collect()
    }
    pub fn uradora(&self) -> Vec<Hai> {
        self.uradora_indicator()
            .into_iter()
            .map(Hai::next)
            .collect()
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Hoo {
    river: Vec<SuteHai>,
}

impl Hoo {
    pub fn new() -> Self {
        Self { river: vec![] }
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum SuteHai {
    Normal(Hai),
    Riichi(Hai),
}

impl SuteHai {
    pub fn hai(self) -> Hai {
        match self {
            SuteHai::Normal(hai) | SuteHai::Riichi(hai) => hai,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    wind: Fon,
    te: Te,
    riichi: Option<Riichi>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Riichi {
    pub ippatsu: bool,
    machi: Vec<Hai>,
    furiten: bool,
}

impl Player {
    pub fn new(wind: Fon) -> Self {
        Self {
            wind,
            te: Default::default(),
            riichi: None,
        }
    }
}

#[derive(Default, Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct Te {
    pub hai: OrderedList<Hai>,
    fuuro: Vec<Fuuro>,
    tsumo: Option<Hai>,
}

impl Te {
    pub fn contains(&self, hai: Hai) -> bool {
        self.hai.contains(&hai) || self.tsumo == Some(hai)
    }
    pub fn index(&self, hai: Hai) -> Option<TehaiIndex> {
        self.hai.index(&hai).map(TehaiIndex::Tehai).or_else(|| {
            if self.tsumo == Some(hai) {
                Some(TehaiIndex::Tsumohai)
            } else {
                None
            }
        })
    }
    pub fn remove(&mut self, i: TehaiIndex) -> Hai {
        match i {
            TehaiIndex::Tehai(i) => self.hai.remove(i),
            TehaiIndex::Tsumohai => self.tsumo.take().expect("Has tsumohai"),
        }
    }
    /// Do a standard turn: either
    ///   - throw tsumohai; or
    ///   - insert tsumohai in te throw a tile in te
    ///
    /// Return thrown tile
    pub fn throw_and_insert(&mut self, i: TehaiIndex) -> Hai {
        match i {
            TehaiIndex::Tehai(i) => {
                let hai = self.hai.remove(i);
                if let Some(tsumohai) = self.tsumo.take() {
                    debug!("Insert tsumohai {}", tsumohai.to_string());
                    self.hai.insert(tsumohai);
                }
                debug!("Throw tehai {}", hai.to_string());
                hai
            }
            TehaiIndex::Tsumohai => {
                let hai = self.tsumo.take().expect("Has tsumohai");
                debug!("Throw tsumohai {}", hai.to_string());
                hai
            }
        }
    }

    pub fn set_tsumohai(&mut self, hai: Hai) {
        assert!(self.tsumo.is_none(), "Expect empty tsumohai");
        self.tsumo = Some(hai);
    }

    /// Make a chi in this te
    pub fn open_shuntsu(&mut self, hai: Hai, index: [usize; 2]) {
        let hai2 = self.remove(TehaiIndex::Tehai(index[0]));
        let hai3 = self.remove(TehaiIndex::Tehai(index[1]));
        let new_shuntsu = Fuuro::Shuntsu {
            own: [hai2, hai3],
            taken: hai,
            from: Direction::Right,
        };
        trace!("Make new fuuro: {:?}", &new_shuntsu);
        self.fuuro.push(new_shuntsu);
    }

    /// Make a pon in this te
    pub fn open_kootsu(&mut self, hai: Hai, direction: Direction) {
        let hai2_i = self.index(hai).expect("Has second pon tile");
        let hai2 = self.remove(hai2_i);
        let hai3_i = self.index(hai).expect("Has third pon tile");
        let hai3 = self.remove(hai3_i);
        let new_kootsu = Fuuro::Kootsu {
            own: [hai2, hai3],
            taken: hai,
            from: direction,
        };
        trace!("Make new fuuro: {:?}", &new_kootsu);
        self.fuuro.push(new_kootsu);
    }

    pub fn daikantsu(&mut self, hai: Hai, direction: Direction) {
        let hai2_i = self.index(hai).expect("Has second kan tile");
        let hai2 = self.remove(hai2_i);
        let hai3_i = self.index(hai).expect("Has third kan tile");
        let hai3 = self.remove(hai3_i);
        let hai4_i = self.index(hai).expect("Has forth kan tile");
        let hai4 = self.remove(hai4_i);
        let new_kantsu = Fuuro::Kantsu(KantsuInner::DaiMinkan {
            own: [hai2, hai3, hai4],
            taken: hai,
            from: direction,
        });
        trace!("Make new fuuro: {:?}", &new_kantsu);
        self.fuuro.push(new_kantsu);
    }

    /// Make an ankan in this te
    pub fn ankan(&mut self, i: TehaiIndex) {
        let hai1 = self.remove(i);
        let hai2_i = self.index(hai1).expect("Has second kan tile");
        let hai2 = self.remove(hai2_i);
        let hai3_i = self.index(hai1).expect("Has third kan tile");
        let hai3 = self.remove(hai3_i);
        let hai4_i = self.index(hai1).expect("Has forth kan tile");
        let hai4 = self.remove(hai4_i);
        self.fuuro.push(Fuuro::Kantsu(KantsuInner::Ankan {
            own: [hai1, hai2, hai3, hai4],
        }));
    }

    fn find_kootsu_with_hai_mut(&mut self, hai: Hai) -> Option<&mut Fuuro> {
        for fuuro in &mut self.fuuro {
            if let Fuuro::Kootsu { own: [hai1, _], .. } = fuuro {
                if *hai1 == hai {
                    return Some(fuuro);
                }
            }
        }
        None
    }

    /// Make an kakan in this te
    pub fn kakan(&mut self, i: TehaiIndex) {
        let hai = self.remove(i);
        let fuuro = self
            .find_kootsu_with_hai_mut(hai)
            .expect("Has kootsu to kakan");
        if let Fuuro::Kootsu { own, taken, from } = *fuuro {
            *fuuro = Fuuro::Kantsu(KantsuInner::ShouMinkan {
                own,
                added: hai,
                taken,
                from,
            })
        } else {
            unreachable!("Expect kootsu!");
        }
    }

    pub fn kan_count(&self) -> usize {
        self.fuuro.iter().fold(0, |acc, fuuro| {
            acc + if let Fuuro::Kantsu(_) = fuuro { 1 } else { 0 }
        })
    }

    pub fn hai(&self) -> &[Hai] {
        self.hai.as_ref()
    }
    pub fn fuuro(&self) -> &[Fuuro] {
        self.fuuro.as_ref()
    }
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone, Serialize, Deserialize)]
pub enum Direction {
    Left,
    Front,
    Right,
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum Fuuro {
    Shuntsu {
        own: [Hai; 2],
        taken: Hai,
        from: Direction,
    },
    Kootsu {
        own: [Hai; 2],
        taken: Hai,
        from: Direction,
    },
    Kantsu(KantsuInner),
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum KantsuInner {
    Ankan {
        own: [Hai; 4],
    },
    DaiMinkan {
        own: [Hai; 3],
        taken: Hai,
        from: Direction,
    },
    ShouMinkan {
        own: [Hai; 2],
        added: Hai,
        taken: Hai,
        from: Direction,
    },
}

impl Game {
    /// Return all possible chi on calling
    fn can_chi(&self) -> Vec<[usize; 2]> {
        // Cannot call chi if in riichi
        if self.players[self.turn as usize].riichi.is_some() {
            return vec![];
        }
        if let Some(hai) = self.last_thrown_tile() {
            match hai {
                Hai::Suu(SuuHai { value, .. }) => {
                    // FIXME: Take into account Kuikae
                    let right = [hai.prev().prev(), hai.prev()];
                    let middle = [hai.prev(), hai.next()];
                    let left = [hai.next(), hai.next().next()];

                    let possible_patterns = match value {
                        Values::Ii => vec![left],
                        Values::Ryan => vec![middle, left],
                        Values::Paa => vec![right, middle],
                        Values::Kyuu => vec![right],
                        _ => vec![right, middle, left],
                    };

                    let mut out = vec![];
                    for pattern in possible_patterns {
                        let pos1 = self.players[self.turn as usize].te.hai.index(&pattern[0]);
                        let pos2 = self.players[self.turn as usize].te.hai.index(&pattern[1]);
                        match (pos1, pos2) {
                            (Some(p1), Some(p2)) => {
                                let new_match = [p1, p2];
                                if !out.contains(&new_match) {
                                    out.push(new_match);
                                }
                            }
                            _ => {}
                        }
                    }
                    out
                }
                Hai::Ji(..) => vec![],
            }
        } else {
            vec![]
        }
    }

    fn can_pon(&self, player: Fon) -> bool {
        // Cannot call pon if in riichi
        if self.players[player as usize].riichi.is_some() {
            return false;
        }
        if let Some(hai) = self.last_thrown_tile() {
            let mut cnt = 0;
            for tehai in self.players[player as usize].te.hai.iter() {
                if tehai == &hai {
                    cnt += 1;
                }
            }
            cnt >= 2
        } else {
            false
        }
    }

    /// Can call kan during opponent's turn (Daiminkan)
    fn can_kan(&self, player: Fon) -> bool {
        // Cannot call kan if in riichi
        if let Some(hai) = self.last_thrown_tile() {
            if self.players[player as usize].riichi.is_some() {
                return false;
            }
            let mut cnt = 0;
            for tehai in self.players[player as usize].te.hai.iter() {
                if tehai == &hai {
                    cnt += 1;
                }
            }
            cnt >= 3
        } else {
            false
        }
    }

    fn can_ron(&self, player: Fon) -> bool {
        if let Some(hai) = self.last_thrown_tile() {
            if self.is_furiten(player) {
                false
            } else {
                let agari_te = AgariTe::from_te(
                    &self.players[player as usize].te,
                    self,
                    hai,
                    WinningMethod::Ron,
                    player,
                );
                let (yaku, _, _) = agari_te.points();
                !yaku.is_empty()
            }
        } else {
            false
        }
    }

    fn is_furiten(&self, player: Fon) -> bool {
        if let Some(riichi) = &self.players[player as usize].riichi {
            if riichi.furiten {
                return true;
            }
        }
        let machi = find_machi(&self.players[player as usize].te.hai());
        for sutehai in &self.hoo[player as usize].river {
            if machi.contains(&sutehai.hai()) {
                return true;
            }
        }
        false
    }

    fn can_tsumo(&self) -> bool {
        let te = &self.players[self.turn as usize].te;
        if let Some(tsumo_hai) = te.tsumo {
            let agari_te = AgariTe::from_te(te, self, tsumo_hai, WinningMethod::Tsumo, self.turn);
            let (yaku, _, _) = agari_te.points();
            !yaku.is_empty()
        } else {
            false
        }
    }

    fn can_riichi(&self) -> Vec<ThrowableOnRiichi> {
        let mut throwable_tiles = vec![];

        let player = &self.players[self.turn as usize];
        let enough_point = self.score[self.turn as usize].score >= 1000;
        if enough_point && player.riichi.is_none() && player.te.fuuro.is_empty() {
            if let Some(tsumohai) = player.te.tsumo {
                let mut te = vec![];
                te.extend(player.te.hai.iter().cloned());
                te.push(tsumohai);

                if is_tempai(&te) {
                    // Find tiles which can be thrown on saying riichi
                    for i in 0..te.len() {
                        let mut te_ = te.clone();
                        te_.swap_remove(i);
                        if is_tempai(&te_) {
                            throwable_tiles.push(if i == player.te.hai.len() {
                                ThrowableOnRiichi::Tsumohai
                            } else {
                                ThrowableOnRiichi::Te(i)
                            });
                        }
                    }
                }
            }
        }

        throwable_tiles
    }

    /// Can call Kyusyukyuhai if this is the first turn and there is no fuuro
    fn can_kyusyukyuhai(&self) -> bool {
        let first_turn = self.tsumo_cnt <= 4;
        let mut no_fuuro = true;
        for p in &self.players {
            if !p.te.fuuro.is_empty() {
                no_fuuro = false;
            }
        }
        if first_turn && no_fuuro {
            let mut set = std::collections::HashSet::new();
            for hai in self.players[self.turn as usize].te.hai.iter() {
                if hai.is_jihai_or_1_9() {
                    set.insert(*hai);
                }
            }
            if let Some(tsumohai) = self.players[self.turn as usize].te.tsumo {
                if tsumohai.is_jihai_or_1_9() {
                    set.insert(tsumohai);
                }
            }
            set.len() >= 9
        } else {
            false
        }
    }

    /// Can call Kan on one of these tiles during one's own turn
    fn can_shominkan(&self) -> Vec<Hai> {
        fn can_make_shominkan(all_fuuro: &[Fuuro], hai: Hai) -> bool {
            for fuuro in all_fuuro {
                if let Fuuro::Kootsu { taken, .. } = fuuro {
                    if *taken == hai {
                        return true;
                    }
                }
            }
            false
        }

        let mut candidates = vec![];
        let te = &self.players[self.turn as usize].te;
        for hai in te.hai.iter() {
            if can_make_shominkan(&te.fuuro, *hai) {
                candidates.push(*hai);
            }
        }
        if let Some(hai) = te.tsumo {
            if can_make_shominkan(&te.fuuro, hai) {
                candidates.push(hai);
            }
        }
        candidates
    }

    /// Can call Kan on one of these tiles during one's own turn
    fn can_ankan(&self) -> Vec<Hai> {
        use std::collections::{hash_map::Entry, HashMap};

        fn count(cnt_map: &mut HashMap<Hai, usize>, hai: Hai) {
            match cnt_map.entry(hai) {
                Entry::Vacant(cnt) => {
                    cnt.insert(1);
                }
                Entry::Occupied(mut cnt) => {
                    *cnt.get_mut() += 1;
                }
            }
        }

        let mut cnt_map = HashMap::new();
        for hai in self.players[self.turn as usize].te.hai.iter() {
            count(&mut cnt_map, *hai);
        }
        if let Some(hai) = self.players[self.turn as usize].te.tsumo {
            count(&mut cnt_map, hai);
        }

        // FIXME: Cannot call ankan on special case when player called riichi
        // and calling ankan would change their machi.
        cnt_map
            .iter()
            .filter(|(_, &cnt)| cnt == 4)
            .map(|(hai, _)| *hai)
            .collect()
    }

    fn allowed_calls(&self, player: Fon) -> Vec<PossibleCall> {
        let mut allowed_calls = Vec::with_capacity(4);
        if self.turn == player {
            let possible_chi = self.can_chi();
            if possible_chi.len() > 0 {
                allowed_calls.push(PossibleCall::Chi {
                    indices: possible_chi,
                });
            }
        }
        if self.can_pon(player) {
            allowed_calls.push(PossibleCall::Pon);
        }
        if self.can_kan(player) {
            allowed_calls.push(PossibleCall::Kan);
        }
        if self.can_ron(player) {
            allowed_calls.push(PossibleCall::Ron);
        }
        allowed_calls
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum ThrowableOnRiichi {
    Te(usize),
    Tsumohai,
}

fn is_tempai(te: &[Hai]) -> bool {
    count_shanten(te) == 0
}

/// Thanks https://qiita.com/tomo_hxx/items/75b5f771285e1334c0a5 !
/// http://ara.moo.jp/mjhmr/shanten.htm
fn count_shanten(te: &[Hai]) -> usize {
    let some_chi = count_chitoitsu_shanten(te);
    let some_koku = count_kokushimuso_shanten(te);
    let normal = count_normal_shanten(te);
    match (some_chi, some_koku) {
        (None, None) => normal,
        (Some(chi), None) => chi.min(normal),
        (None, Some(koku)) => koku.min(normal),
        (Some(chi), Some(koku)) => koku.min(chi).min(normal),
    }
}
/// Only works for closed hands
fn count_chitoitsu_shanten(te: &[Hai]) -> Option<usize> {
    if te.len() == 13 || te.len() == 14 {
        let mut uniq = std::collections::HashSet::new();
        for hai in te {
            uniq.insert(hai);
        }
        let haisyu_count = uniq.len();
        let mut toitsu_count = 0;
        for uniq_hai in uniq {
            let hai_count = te
                .iter()
                .filter(|hai| *hai == uniq_hai)
                .collect::<Vec<_>>()
                .len();
            if hai_count >= 2 {
                toitsu_count += 1;
            }
        }
        Some(
            6 - toitsu_count
                + if haisyu_count < 7 {
                    7 - haisyu_count
                } else {
                    0
                },
        )
    } else {
        None
    }
}
/// Only works for closed hands
fn count_kokushimuso_shanten(te: &[Hai]) -> Option<usize> {
    if te.len() == 13 || te.len() == 14 {
        let mut uniq = std::collections::HashSet::new();
        let mut any_toitsu = false;
        for hai in te {
            if hai.is_jihai_or_1_9() {
                if uniq.contains(hai) {
                    any_toitsu = true;
                }
                uniq.insert(hai);
            }
        }
        Some(13 - uniq.len() - if any_toitsu { 1 } else { 0 })
    } else {
        None
    }
}

fn count_normal_shanten(te: &[Hai]) -> usize {
    let open_mentsu_count = (14 - te.len()) / 3;
    let root = solver::GroupTree::generate(te, 0, 4 - open_mentsu_count, 0, 0);
    for tree in &root {
        trace!("{}", tree);
    }

    solver::GroupTree::shanten(&root)
}

fn find_machi(te: &[Hai]) -> Vec<Hai> {
    let open_mentsu_count = (14 - te.len()) / 3;
    let root = solver::GroupTree::generate(te, 0, 4 - open_mentsu_count, 0, 0);
    let root = solver::GroupTree::shanten0(root);

    let mut machi = vec![];
    for groups in solver::GroupTree::possible_groups_tree(&root) {
        machi.extend(groups.machi());
    }

    machi.sort();
    machi.dedup();

    machi
}

mod solver {
    use super::{Hai, SuuHai, Values};
    use std::fmt;

    #[derive(Debug, Eq, PartialEq, Copy, Clone)]
    pub enum Group {
        Mentsu([Hai; 3]),
        Taatsu([Hai; 2]),
    }
    impl fmt::Display for Group {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Group::Mentsu([h1, h2, h3]) => {
                    write!(f, "{}{}{}", h1.to_char(), h2.to_char(), h3.to_char())
                }
                Group::Taatsu([h1, h2]) => write!(f, "{}{}", h1.to_char(), h2.to_char()),
            }
        }
    }

    impl Group {
        pub fn machi(&self) -> Vec<Hai> {
            match self {
                Group::Mentsu(_) => vec![],
                Group::Taatsu([hai1, hai2]) => {
                    if hai1 == hai2 {
                        vec![*hai1]
                    } else if let (
                        Hai::Suu(SuuHai { value: value1, .. }),
                        Hai::Suu(SuuHai { value: value2, .. }),
                    ) = (hai1, hai2)
                    {
                        // Assume that the suuhai are the same color
                        match (value1, value2) {
                            // Penchan machi
                            (Values::Ii, Values::Ryan) => vec![hai2.next()],
                            (Values::Ryan, Values::Ii) => vec![hai1.next()],
                            (Values::Paa, Values::Kyuu) => vec![hai1.prev()],
                            (Values::Kyuu, Values::Paa) => vec![hai2.prev()],
                            _ => {
                                let val1 = *value1 as isize;
                                let val2 = *value2 as isize;
                                match val2 - val1 {
                                    // Kanchan machi
                                    2 => vec![hai1.next()],
                                    -2 => vec![hai2.next()],
                                    // Ryanmen machi
                                    1 => vec![hai1.prev(), hai2.next()],
                                    -1 => vec![hai2.prev(), hai1.next()],
                                    _ => unimplemented!("Unhandled group: {}", self),
                                }
                            }
                        }
                    } else {
                        unreachable!("Unexpected group: {}", self)
                    }
                }
            }
        }
    }
    impl fmt::Display for GroupTree {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            for _ in 0..self.depth {
                write!(f, "\t")?;
            }
            write!(f, "{}", &self.group)?;

            if self.children.is_empty() {
                write!(f, "\tLeft: ")?;
                for hai in &self.remaining_hai {
                    write!(f, "{}", hai.to_char())?;
                }
                write!(
                    f,
                    "\t{} mentsu\t{} taatsu",
                    self.mentsu_count, self.taatsu_count
                )?;
                if self.has_head {
                    write!(f, "\twith head")?;
                } else {
                    write!(f, "\t")?;
                }
                write!(f, "\t{}-shanten", self.shanten)?;
            }
            write!(f, "\n")?;
            for child in &self.children {
                write!(f, "{}", child)?;
            }

            Ok(())
        }
    }

    #[derive(Debug, Clone)]
    pub struct GroupTree {
        group: Group,
        children: Vec<GroupTree>,
        depth: usize,
        remaining_hai: Vec<Hai>,
        /// Number of mentsu from top to bottom (this node included)
        mentsu_count: usize,
        /// Number of taatsu  from top to bottom (this node included)
        taatsu_count: usize,
        /// Head in remaining hai?
        has_head: bool,
        shanten: usize,
    }

    #[derive(Debug, Clone)]
    pub struct Combination {
        mentsu_candidates: Vec<Group>,
        remaining: Vec<Hai>,
    }

    impl fmt::Display for Combination {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let mut group_list = Vec::with_capacity(self.mentsu_candidates.len());
            for group in &self.mentsu_candidates {
                match group {
                    Group::Mentsu(mentsu) => {
                        group_list.push(format!(
                            "{}{}{}",
                            mentsu[0].to_char(),
                            mentsu[1].to_char(),
                            mentsu[2].to_char()
                        ));
                    }
                    Group::Taatsu(taatsu) => {
                        group_list.push(format!("{}{}", taatsu[0].to_char(), taatsu[1].to_char()));
                    }
                }
            }
            let mut remaining = String::new();
            for hai in &self.remaining {
                remaining.push(hai.to_char());
            }
            f.debug_struct("Combination")
                .field("candidates", &group_list)
                .field("remaining", &remaining)
                .finish()
        }
    }

    impl Combination {
        pub fn machi(&self) -> Vec<Hai> {
            let mut machi = vec![];
            for group in &self.mentsu_candidates {
                machi.extend(group.machi());
            }

            // Add tanki machi
            if !has_head(&self.remaining) {
                for hai in &self.remaining {
                    machi.push(*hai);
                }
            }

            machi
        }
    }

    fn has_head(te: &[Hai]) -> bool {
        for (i, h1) in te.iter().enumerate() {
            for (j, h2) in te.iter().enumerate() {
                if i >= j {
                    continue;
                }
                if h1 == h2 {
                    return true;
                }
            }
        }
        false
    }

    impl GroupTree {
        pub fn shanten(trees: &[GroupTree]) -> usize {
            let mut shanten = usize::max_value();
            for tree in trees {
                shanten = shanten.min(if tree.children.is_empty() {
                    tree.shanten
                } else {
                    Self::shanten(&tree.children)
                })
            }
            shanten
        }
        pub fn possible_groups_tree(trees: &[GroupTree]) -> Vec<Combination> {
            let mut possible_groups = vec![];
            for tree in trees {
                possible_groups.extend(Self::possible_groups(tree));
            }
            possible_groups
        }
        fn possible_groups(&self) -> Vec<Combination> {
            let mut possible_groups = if self.children.is_empty() {
                vec![Combination {
                    mentsu_candidates: vec![self.group],
                    remaining: self.remaining_hai.clone(),
                }]
            } else {
                vec![]
            };

            for child in &self.children {
                let mut child_groups = child.possible_groups();
                for child_group in &mut child_groups {
                    child_group.mentsu_candidates.push(self.group);
                }
                possible_groups.extend(child_groups);
            }
            possible_groups
        }
        /// Check if has a 0-shanten child
        fn has_shanten0(&self) -> bool {
            if self.children.is_empty() {
                self.shanten == 0
            } else {
                self.children.iter().any(GroupTree::has_shanten0)
            }
        }
        /// Find all possibilities in 0-shanten by culling all branches
        /// that are not 0-shanten
        pub fn shanten0(trees: Vec<GroupTree>) -> Vec<GroupTree> {
            let mut trees_out = vec![];
            for mut tree in trees {
                if tree.has_shanten0() {
                    let children = Self::shanten0(tree.children);
                    tree.children = children;
                    trees_out.push(tree);
                }
            }
            trees_out
        }
        fn has_group(trees: &[GroupTree], group: &Group) -> bool {
            for tree in trees {
                if &tree.group == group {
                    return true;
                }
            }
            false
        }

        pub fn generate(
            te: &[Hai],
            depth: usize,
            max_depth: usize,
            mentsu_count: usize,
            taatsu_count: usize,
        ) -> Vec<Self> {
            let mut groups = vec![];
            if depth >= max_depth {
                return groups;
            }

            for hai in te {
                for mentsu in possible_mentsu(*hai) {
                    let group = Group::Mentsu(mentsu);
                    if GroupTree::has_group(&groups, &group) {
                        continue;
                    }
                    let mut te_ = te.to_owned();
                    let mut matched_mentsu = true;
                    for hai in &mentsu {
                        if let Some(pos) = te_.iter().position(|x| x == hai) {
                            te_.swap_remove(pos);
                        } else {
                            matched_mentsu = false;
                        }
                    }
                    if matched_mentsu {
                        let has_head = has_head(&te_);
                        let mut shanten =
                            8 - (4 - max_depth) - 2 * (mentsu_count + 1) - (taatsu_count);
                        if shanten != 0 && has_head {
                            shanten -= 1;
                        }
                        groups.push(GroupTree {
                            group,
                            children: GroupTree::generate(
                                &te_,
                                depth + 1,
                                max_depth,
                                mentsu_count + 1,
                                taatsu_count,
                            ),
                            depth: depth,
                            remaining_hai: te_,

                            mentsu_count: mentsu_count + 1,
                            taatsu_count,
                            has_head,
                            shanten,
                        });
                    }
                }

                for taatsu in possible_taatsu(*hai) {
                    let group = Group::Taatsu(taatsu);
                    if GroupTree::has_group(&groups, &group) {
                        continue;
                    }
                    let mut te_ = te.to_owned();
                    let mut matched_taatsu = true;
                    for hai in &taatsu {
                        if let Some(pos) = te_.iter().position(|x| x == hai) {
                            te_.swap_remove(pos);
                        } else {
                            matched_taatsu = false;
                        }
                    }
                    if matched_taatsu {
                        let has_head = has_head(&te_);
                        let mut shanten =
                            8 - (4 - max_depth) - 2 * (mentsu_count) - (taatsu_count + 1);
                        if shanten != 0 && has_head {
                            shanten -= 1;
                        }
                        groups.push(GroupTree {
                            group,
                            children: GroupTree::generate(
                                &te_,
                                depth + 1,
                                max_depth,
                                mentsu_count,
                                taatsu_count + 1,
                            ),
                            depth: depth,
                            remaining_hai: te_,
                            mentsu_count,
                            taatsu_count: taatsu_count + 1,
                            has_head,
                            shanten,
                        })
                    }
                }
            }

            groups
        }
    }

    fn possible_mentsu(hai: Hai) -> Vec<[Hai; 3]> {
        let kootsu = [hai, hai, hai];
        match hai {
            Hai::Suu(SuuHai { value, .. }) => {
                let right = [hai.prev().prev(), hai.prev(), hai];
                let middle = [hai.prev(), hai, hai.next()];
                let left = [hai, hai.next(), hai.next().next()];

                match value {
                    Values::Ii => vec![kootsu, left],
                    Values::Ryan => vec![kootsu, middle, left],
                    Values::Paa => vec![kootsu, right, middle],
                    Values::Kyuu => vec![kootsu, right],
                    _ => vec![kootsu, right, middle, left],
                }
            }
            Hai::Ji(..) => vec![kootsu],
        }
    }
    fn possible_taatsu(hai: Hai) -> Vec<[Hai; 2]> {
        let toitsu = [hai, hai];
        match hai {
            Hai::Suu(SuuHai { value, .. }) => {
                let right = [hai.prev().prev(), hai];
                let middle1 = [hai.prev(), hai];
                let middle2 = [hai, hai.next()];
                let left = [hai, hai.next().next()];

                match value {
                    Values::Ii => vec![toitsu, middle2, left],
                    Values::Ryan => vec![toitsu, middle1, middle2, left],
                    Values::Paa => vec![toitsu, right, middle1, middle2],
                    Values::Kyuu => vec![toitsu, right, middle1],
                    _ => vec![toitsu, right, middle1, middle2, left],
                }
            }
            Hai::Ji(..) => vec![toitsu],
        }
    }
}

#[cfg(test)]
pub mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::super::tiles::ParseHaiError;
    use super::*;

    struct StringifiedGameDebug<'a> {
        te: [&'a str; 4],
        tsumo: [&'a str; 4],
        hoo: [&'a str; 4],
        dice: [Dice; 2],
    }

    impl Game {
        fn from_string_debug(data: StringifiedGameDebug) -> Result<Self, ParseHaiError> {
            let mut players = [
                Player::new(Fon::Ton),
                Player::new(Fon::Nan),
                Player::new(Fon::Shaa),
                Player::new(Fon::Pee),
            ];
            let mut hoo = [Hoo::new(), Hoo::new(), Hoo::new(), Hoo::new()];

            for i in 0..4 {
                for c in data.te[i].chars() {
                    let hai = c.to_string().parse()?;
                    players[i].te.hai.insert(hai);
                }
                if let Some(c) = data.tsumo[i].chars().next() {
                    let hai = c.to_string().parse()?;
                    players[i].te.tsumo = Some(hai);
                }
                for c in data.hoo[i].chars() {
                    // FIXME: Ignore riichi
                    let hai = c.to_string().parse()?;
                    hoo[i].river.push(SuteHai::Normal(hai));
                }
            }

            Ok(Self {
                wind: Fon::Ton,
                turn: Fon::Ton,
                kyoku: 0,
                honba: 0,
                tsumo_cnt: 0,
                players,
                yama: [None; 136],
                hoo,
                dice: data.dice,
                score: [Score {
                    score: 25000,
                    riichi_bou: 0,
                }; 4],
            })
        }
    }

    pub fn te_from_string(data: &str) -> Result<Vec<Hai>, ParseHaiError> {
        let mut te = vec![];
        for c in data.chars() {
            let hai = c.to_string().parse()?;
            te.push(hai);
        }
        Ok(te)
    }

    #[test]
    fn test_chi_normal() {
        let game = Game::from_string_debug(StringifiedGameDebug {
            te: ["🀇🀈🀉🀊🀋🀌🀍🀎🀏🀙🀚🀛🀜🀝", "", "", ""],
            tsumo: ["", "", "", ""],
            hoo: ["", "", "", "🀊"],
            dice: [Dice::One, Dice::Six],
        })
        .unwrap();
        assert_eq!(game.can_chi(), vec![[1, 2], [2, 4], [4, 5]]);
    }

    #[test]
    fn test_chi_cannot_call_from_wrong_river() {
        let game = Game::from_string_debug(StringifiedGameDebug {
            te: ["🀇🀈🀉🀊🀋🀌🀍🀎🀏🀙🀚🀛🀜", "", "", ""],
            tsumo: ["", "", "", ""],
            hoo: ["", "", "🀊", ""],
            dice: [Dice::One, Dice::Six],
        })
        .unwrap();
        assert!(game.can_chi().is_empty());
    }

    #[test]
    fn test_chi_wrong_sutehai() {
        let game = Game::from_string_debug(StringifiedGameDebug {
            te: ["🀇🀈🀉🀊🀋🀌🀍🀎🀏🀙🀚🀛🀜", "", "", ""],
            tsumo: ["", "", "", ""],
            hoo: ["", "", "", "🀟"],
            dice: [Dice::One, Dice::Six],
        })
        .unwrap();
        assert!(game.can_chi().is_empty());
    }

    #[test]
    fn test_chi_middle() {
        let game = Game::from_string_debug(StringifiedGameDebug {
            te: ["🀇🀈🀉🀊🀋🀌🀍🀎🀏🀙🀛🀀🀀", "", "", ""],
            tsumo: ["", "", "", ""],
            hoo: ["", "", "", "🀚"],
            dice: [Dice::One, Dice::Six],
        })
        .unwrap();
        assert_eq!(game.can_chi(), vec![[9, 10]]);
    }

    #[test]
    fn test_kyusyukyuhai() {
        let game = Game::from_string_debug(StringifiedGameDebug {
            te: ["🀇🀇🀈🀉🀏🀙🀀🀀🀁🀂🀃🀆🀅", "", "", ""],
            tsumo: ["🀇", "", "", ""],
            hoo: ["", "", "", ""],
            dice: [Dice::One, Dice::Six],
        })
        .unwrap();
        assert!(game.can_kyusyukyuhai());
    }

    #[test]
    fn test_kyusyukyuhai_8() {
        let game = Game::from_string_debug(StringifiedGameDebug {
            te: ["🀇🀇🀈🀉🀉🀉🀙🀀🀀🀁🀂🀃🀆🀅", "", "", ""],
            tsumo: ["🀇", "", "", ""],
            hoo: ["", "", "", ""],
            dice: [Dice::One, Dice::Six],
        })
        .unwrap();
        assert!(!game.can_kyusyukyuhai());
    }

    #[test]
    fn test_chitoitsu_shanten() {
        let te = te_from_string("🀇🀇🀈🀉🀏🀙🀀🀀🀁🀂🀃🀆🀅").unwrap();
        assert_eq!(count_chitoitsu_shanten(&te), Some(4));
    }

    #[test]
    fn test_chitoitsu_shanten_edge() {
        let te = te_from_string("🀇🀇🀇🀇🀙🀙🀙🀙🀀🀀🀀🀀🀅").unwrap();
        assert_eq!(count_chitoitsu_shanten(&te), Some(6));
    }

    #[test]
    fn test_kokushimuso_shanten() {
        let te = te_from_string("🀇🀏🀙🀡🀐🀘🀀🀀🀁🀂🀃🀆🀅").unwrap();
        assert_eq!(count_kokushimuso_shanten(&te), Some(0));
    }

    #[test]
    fn test_normal_shanten() {
        let te = te_from_string("🀇🀈🀊🀋🀝🀞🀟🀐🀑🀒🀔🀕🀗🀘").unwrap();
        assert_eq!(count_normal_shanten(&te), 2);
    }

    #[test]
    fn test_normal_shanten_head() {
        let te = te_from_string("🀇🀈🀊🀋🀝🀞🀟🀐🀑🀒🀔🀕🀗🀗").unwrap();
        assert_eq!(count_normal_shanten(&te), 1);
    }

    #[test]
    fn test_normal_shanten_head_0() {
        let te = te_from_string("🀇🀈🀉🀊🀋🀌🀍🀎🀏🀙🀚🀛🀗🀗").unwrap();
        assert_eq!(count_normal_shanten(&te), 0);
    }

    #[test]
    fn test_find_machi_head_0() {
        let te = te_from_string("🀇🀈🀉🀊🀋🀍🀎🀏🀙🀚🀛🀗🀗").unwrap();
        assert_eq!(find_machi(&te), te_from_string("🀉🀌").unwrap());
    }

    #[test]
    fn test_find_machi_tanki() {
        let te = te_from_string("🀇🀈🀉🀊🀋🀌🀍🀎🀏🀙🀚🀛🀗").unwrap();
        assert_eq!(find_machi(&te), te_from_string("🀗").unwrap());
    }

    use ron;

    #[test]
    fn test_riichi() {
        let game: Game = ron::de::from_reader(std::fs::File::open("riichi.ron").unwrap()).unwrap();
        assert_eq!(game.can_riichi(), vec![ThrowableOnRiichi::Tsumohai]);
    }
}
