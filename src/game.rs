use rand::distributions::{Distribution, Standard};
use rand::seq::SliceRandom;
use rand::Rng;

use std::collections::BTreeSet;

use super::tiles::{make_all_tiles, Fon, Hai};

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
    turn: Fon,
    players: [Player; 4],
    yama: [Option<Hai>; 136],
    dice: [Dice; 2],
}

use std::fmt;

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
            .field("players", &self.players)
            .field("yama", &yama)
            .field("dice", &self.dice)
            .finish()
    }
}

impl Game {
    pub fn new<R: Rng>(rng: &mut R) -> Self {
        let mut yama = [None; 136];
        for (i, hai) in make_all_tiles().iter().cloned().enumerate() {
            yama[i] = Some(hai);
        }

        // yama.shuffle(rng);
        let dice1 = rng.gen();
        let dice2 = rng.gen();

        Self {
            wind: Fon::Ton,
            turn: Fon::Ton,
            players: [
                Player::new(Fon::Ton),
                Player::new(Fon::Nan),
                Player::new(Fon::Shaa),
                Player::new(Fon::Pee),
            ],
            yama,
            dice: [dice1, dice2],
        }
    }

    pub fn deal(&mut self) {
        let dice_result = self.dice[0] as usize + self.dice[1] as usize;
        let break_point = ((dice_result - 1) % 4) * 34 + dice_result * 2;

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

        let tsumohai_i = (break_point + 14 + 3 * 14) % 136;
        let tsumohai = self.yama[tsumohai_i];
        self.yama[tsumohai_i] = None;
        self.players[0].te.tsumo = tsumohai;
    }

    pub fn throw_tsumo(&mut self) {
        self.players[0].te.tsumo = None;
    }

    pub fn throw_tile(&mut self, i: usize) {
        if let Some(hai) = self.players[0].te.hai.iter().nth(i).cloned() {
            self.players[0].te.hai.remove(&hai);
        }
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

        // TODO: Add player 2 and 4

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

    pub fn player1_te(&self) -> impl Iterator<Item = &Hai> {
        self.players[0].te.hai.iter()
    }
    pub fn player1_tsumo(&self) -> Option<Hai> {
        self.players[0].te.tsumo
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
    hai: BTreeSet<Hai>,
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
