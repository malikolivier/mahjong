use std::fmt;

use log::debug;
use rand::distributions::{Distribution, Standard};
use rand::seq::SliceRandom;
use rand::Rng;

use super::ai::{AiServer, PossibleCall, TurnResult};
use super::list::OrderedList;
use super::tiles::{make_all_tiles, Fon, Hai, ParseHaiError, SuuHai, Values};

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone)]
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
    wind: Fon,
    /// Current player that should draw
    pub turn: Fon,
    honba: usize,
    // turn_player: usize,
    tsumo_cnt: usize,
    /// 4 players indexed by Ton/Nan/Sha/Pee
    players: [Player; 4],
    yama: [Option<Hai>; 136],
    /// 4 rivers indexed by Ton/Nan/Sha/Pee
    hoo: [Hoo; 4],
    dice: [Dice; 2],
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
    DoTurn,
}

impl Game {
    pub fn new<R: Rng>(rng: &mut R) -> Self {
        let mut yama = [None; 136];
        for (i, hai) in make_all_tiles().iter().cloned().enumerate() {
            yama[i] = Some(hai);
        }

        yama.shuffle(rng);
        let dice1 = rng.gen();
        let dice2 = rng.gen();

        Self {
            wind: Fon::Ton,
            turn: Fon::Ton,
            honba: 0,
            tsumo_cnt: 0,
            players: [
                Player::new(Fon::Ton),
                Player::new(Fon::Nan),
                Player::new(Fon::Shaa),
                Player::new(Fon::Pee),
            ],
            yama,
            hoo: Default::default(),
            dice: [dice1, dice2],
        }
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
            let tsumohai_i = (break_point + 4 * 14 + self.tsumo_cnt) % 136;
            Some(tsumohai_i)
        }
    }

    fn kan_count(&self) -> usize {
        let mut cnt = 0;
        for p in &self.players {
            for fuuro in &p.te.fuuro {
                if let Fuuro::Kantsu(_) = fuuro {
                    cnt += 1;
                }
            }
        }
        cnt
    }

    pub fn play(&mut self, channels: [AiServer; 4]) {
        self.deal();

        while self.next_turn(&channels) {}
    }

    fn deal(&mut self) {
        let break_point = self.wall_break_index();

        for i in 0..=13 {
            for p in 0..4 {
                let tsumohai_i = (break_point + i + p * 14) % 136;
                if let Some(tsumohai) = self.yama[tsumohai_i] {
                    self.players[p].te.hai.insert(tsumohai);
                    self.yama[tsumohai_i] = None;
                } else {
                    unreachable!()
                }
            }
        }
    }

    /// Make turn player draw a tile
    /// Return true if a tile is drawn. Return false if there is no tile left.
    fn draw(&mut self) -> bool {
        if let Some(tsumohai_i) = self.next_tsumohai_index() {
            let tsumohai = self.yama[tsumohai_i];
            self.yama[tsumohai_i] = None;
            self.players[self.turn as usize].te.tsumo = tsumohai;
            self.tsumo_cnt += 1;
            true
        } else {
            false
        }
    }

    /// Returns a boolean whose value is false if this is the last turn
    fn tx_refresh(&self, channels: &[AiServer; 4]) {
        for channel in channels {
            channel
                .tx
                .send(GameRequest::new(self, Request::Refresh))
                .expect("Sent!");
        }
    }

    /// Returns a boolean whose value is false if this is the last turn
    fn next_turn(&mut self, channels: &[AiServer; 4]) -> bool {
        self.tx_refresh(channels);

        // Listen for chi/pon/kan/ron
        let mut call1 = None;
        let mut call2 = None;
        let mut call3 = None;

        let allowed_calls1 = self.allowed_calls(self.turn);
        if allowed_calls1.len() > 0 {
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
            channels[self.turn.next() as usize]
                .tx
                .send(GameRequest::new(self, Request::Call(allowed_calls2)))
                .expect("Sent!");
            call2 = channels[self.turn.next() as usize]
                .rx_call
                .recv()
                .expect("Received!");
        }

        let allowed_calls3 = self.allowed_calls(self.turn);
        if allowed_calls3.len() > 0 {
            channels[self.turn.next().next() as usize]
                .tx
                .send(GameRequest::new(self, Request::Call(allowed_calls3)))
                .expect("Sent!");
            call3 = channels[self.turn.next().next() as usize]
                .rx_call
                .recv()
                .expect("Received!");
        }

        match [call1, call2, call3] {
            [None, None, None] => {
                if !self.draw() {
                    self.ryukyoku();
                    return false;
                }
                channels[self.turn as usize]
                    .tx
                    .send(GameRequest::new(self, Request::DoTurn))
                    .expect("Sent!");
                let result = channels[self.turn as usize]
                    .rx_turn
                    .recv()
                    .expect("Received!");
                match result {
                    TurnResult::Tsumo => self.agari(self.turn),
                    TurnResult::Kyusyukyuhai => self.ryukyoku(),
                    TurnResult::ThrowHai { index, riichi } => {
                        self.throw_tile(self.turn, index, riichi);
                        self.turn = self.turn.next();
                        true
                    }
                    TurnResult::ThrowTsumoHai { riichi } => {
                        self.throw_tsumo(self.turn, riichi);
                        self.turn = self.turn.next();
                        true
                    }
                }
            }
            _ => unimplemented!("Someone called!"),
        }
    }

    fn ryukyoku(&mut self) -> bool {
        // TODO
        false
    }

    fn agari(&mut self, _player: Fon) -> bool {
        // TODO
        false
    }

    fn last_thrown_tile(&self) -> Option<Hai> {
        let player_who_threw_last_tile = self.turn.prev();
        let player_index = player_who_threw_last_tile as usize;
        self.hoo[player_index]
            .river
            .last()
            .map(|sutehai| sutehai.hai())
    }

    pub fn throw_tsumo(&mut self, p: Fon, riichi: bool) {
        let hai = self.players[p as usize]
            .te
            .tsumo
            .take()
            .expect("Has tsumohai");
        debug!("Throw tsumohai {}", hai.to_string());
        self.hoo[p as usize].river.push(if riichi {
            SuteHai::Riichi(hai)
        } else {
            SuteHai::Normal(hai)
        })
    }

    pub fn throw_tile(&mut self, p: Fon, i: usize, riichi: bool) {
        let hai = self.players[p as usize].te.hai.remove(i);
        if let Some(tsumohai) = self.players[p as usize].te.tsumo.take() {
            debug!("Insert tsumohai {}", tsumohai.to_string());
            self.players[p as usize].te.hai.insert(tsumohai);
        }
        debug!("Throw tehai {}", hai.to_string());
        self.hoo[p as usize].river.push(if riichi {
            SuteHai::Riichi(hai)
        } else {
            SuteHai::Normal(hai)
        })
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
            grid[0][i + offset] = hai.to_string();
        }
        if let Some(hai) = top_player.te.tsumo {
            grid[0][top_player.te.hai.len() + 1 + offset] = hai.to_string();
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
            grid[24][i] = hai.to_string();
        }
        if let Some(hai) = bottom_player.te.tsumo {
            grid[24][bottom_player.te.hai.len() + 1] = hai.to_string();
        }

        for (i, sutehai) in self.hoo[0].river.iter().enumerate() {
            let hai = match sutehai {
                SuteHai::Normal(hai) | SuteHai::Riichi(hai) => hai,
            };
            grid[17 + i / 6][9 + i % 6] = hai.to_string();
        }

        // Player 2
        let bottom_player = &self.players[1];
        let mut offset = 0;
        // TODO: Called tiles not done
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
            grid[i + 5][0] = hai.to_string();
        }
        if let Some(hai) = bottom_player.te.tsumo {
            grid[bottom_player.te.hai.len() + 6][0] = hai.to_string();
        }

        for (i, sutehai) in self.hoo[1].river.iter().enumerate() {
            let hai = match sutehai {
                SuteHai::Normal(hai) | SuteHai::Riichi(hai) => hai,
            };
            grid[8 + i % 6][7 - i / 6] = hai.to_string();
        }

        // TODO: Add player 4

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
    pub fn player_tsumo(&self, p: Fon) -> Option<Hai> {
        self.players[p as usize].te.tsumo
    }
}

#[derive(Default, Debug, Clone)]
pub struct Hoo {
    river: Vec<SuteHai>,
}

impl Hoo {
    pub fn new() -> Self {
        Self { river: vec![] }
    }
}

#[derive(Debug, Copy, Clone)]
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

#[derive(Debug, Clone)]
pub struct Player {
    wind: Fon,
    te: Te,
}

impl Player {
    pub fn new(wind: Fon) -> Self {
        Self {
            wind,
            te: Default::default(),
        }
    }
}

#[derive(Default, Debug, Eq, PartialEq, Clone)]
pub struct Te {
    hai: OrderedList<Hai>,
    fuuro: Vec<Fuuro>,
    tsumo: Option<Hai>,
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone)]
pub enum Direction {
    Left,
    Front,
    Right,
}

#[derive(Debug, Eq, PartialEq, Clone)]
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

#[derive(Debug, Eq, PartialEq, Clone)]
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
        if let Some(hai) = self.last_thrown_tile() {
            match hai {
                Hai::Suu(SuuHai { value, .. }) => {
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

    fn can_kan(&self, player: Fon) -> bool {
        if let Some(hai) = self.last_thrown_tile() {
            let mut cnt = 0;
            for tehai in self.players[player as usize].te.hai.iter() {
                if tehai == &hai {
                    cnt += 1;
                }
            }
            cnt >= 3
        } else {
            // TODO: Take into account Shouminkan
            false
        }
    }

    fn can_ron(&self, player: Fon) -> bool {
        if let Some(hai) = self.last_thrown_tile() {
            // TODO: Check all yakus
            false
        } else {
            // TODO: Take into account Shouminkan
            false
        }
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

struct StringifiedGameDebug<'a> {
    te: [&'a str; 4],
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
            for c in data.hoo[i].chars() {
                // FIXME: Ignore riichi
                let hai = c.to_string().parse()?;
                hoo[i].river.push(SuteHai::Normal(hai));
            }
        }

        Ok(Self {
            wind: Fon::Ton,
            turn: Fon::Ton,
            honba: 0,
            tsumo_cnt: 0,
            players,
            yama: [None; 136],
            hoo,
            dice: data.dice,
        })
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_chi_normal() {
        use std::collections::HashSet;

        let game = Game::from_string_debug(StringifiedGameDebug {
            te: ["🀇🀈🀉🀊🀋🀌🀍🀎🀏🀙🀚🀛🀜🀝", "", "", ""],
            hoo: ["", "", "", "🀊"],
            dice: [Dice::One, Dice::Six],
        })
        .unwrap();
        assert_eq!(game.can_chi(), vec![[1, 2], [2, 4], [4, 5]]);
    }

    #[test]
    fn test_chi_cannot_call_from_wrong_river() {
        let game = Game::from_string_debug(StringifiedGameDebug {
            te: ["🀇🀈🀉🀊🀋🀌🀍🀎🀏🀙🀚🀛🀜🀝", "", "", ""],
            hoo: ["", "", "🀊", ""],
            dice: [Dice::One, Dice::Six],
        })
        .unwrap();
        assert!(game.can_chi().is_empty());
    }

    #[test]
    fn test_chi_wrong_sutehai() {
        let game = Game::from_string_debug(StringifiedGameDebug {
            te: ["🀇🀈🀉🀊🀋🀌🀍🀎🀏🀙🀚🀛🀜🀝", "", "", ""],
            hoo: ["", "", "", "🀟"],
            dice: [Dice::One, Dice::Six],
        })
        .unwrap();
        assert!(game.can_chi().is_empty());
    }

    #[test]
    fn test_chi_middle() {
        let game = Game::from_string_debug(StringifiedGameDebug {
            te: ["🀇🀈🀉🀊🀋🀌🀍🀎🀏🀙🀛🀀🀀🀀", "", "", ""],
            hoo: ["", "", "", "🀚"],
            dice: [Dice::One, Dice::Six],
        })
        .unwrap();
        assert_eq!(game.can_chi(), vec![[9, 10]]);
    }
}
