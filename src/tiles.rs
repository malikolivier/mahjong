use std::str::FromStr;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone)]
pub enum Values {
    Ii = 1,
    Ryan = 2,
    San = 3,
    Suu = 4,
    Uu = 5,
    Roo = 6,
    Chii = 7,
    Paa = 8,
    Kyuu = 9,
}
const VALUES: [Values; 9] = [
    Values::Ii,
    Values::Ryan,
    Values::San,
    Values::Suu,
    Values::Uu,
    Values::Roo,
    Values::Chii,
    Values::Paa,
    Values::Kyuu,
];

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone)]
pub enum Suu {
    Wan,
    Pin,
    Sou,
}
const SUU: [Suu; 3] = [Suu::Wan, Suu::Pin, Suu::Sou];

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone)]
pub enum JiHai {
    Fon(Fon),
    Sangen(Sangen),
}

#[derive(Debug, Ord, PartialOrd, Hash, Copy, Clone)]
pub struct SuuHai {
    pub suu: Suu,
    pub value: Values,
    pub aka: bool,
}

impl PartialEq for SuuHai {
    /// Ignore akadora during comparison
    fn eq(&self, other: &Self) -> bool {
        self.suu.eq(&other.suu) && self.value.eq(&other.value)
    }
}
impl Eq for SuuHai {}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone)]
pub enum Fon {
    Ton = 0,
    Nan = 1,
    Shaa = 2,
    Pee = 3,
}

impl Fon {
    pub fn next(self) -> Self {
        match self {
            Fon::Ton => Fon::Nan,
            Fon::Nan => Fon::Shaa,
            Fon::Shaa => Fon::Pee,
            Fon::Pee => Fon::Ton,
        }
    }
    pub fn prev(self) -> Self {
        match self {
            Fon::Ton => Fon::Pee,
            Fon::Nan => Fon::Ton,
            Fon::Shaa => Fon::Nan,
            Fon::Pee => Fon::Shaa,
        }
    }
}

const FON: [Fon; 4] = [Fon::Ton, Fon::Nan, Fon::Shaa, Fon::Pee];

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone)]
pub enum Sangen {
    Haku,
    Hatsu,
    Chun,
}
const SANGEN: [Sangen; 3] = [Sangen::Haku, Sangen::Hatsu, Sangen::Chun];

impl Sangen {
    pub fn next(self) -> Self {
        match self {
            Sangen::Haku => Sangen::Hatsu,
            Sangen::Hatsu => Sangen::Chun,
            Sangen::Chun => Sangen::Haku,
        }
    }
    pub fn prev(self) -> Self {
        match self {
            Sangen::Haku => Sangen::Chun,
            Sangen::Hatsu => Sangen::Haku,
            Sangen::Chun => Sangen::Hatsu,
        }
    }
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone)]
pub enum Hai {
    Suu(SuuHai),
    Ji(JiHai),
}
impl Values {
    pub fn next(self) -> Self {
        match self {
            Values::Ii => Values::Ryan,
            Values::Ryan => Values::San,
            Values::San => Values::Suu,
            Values::Suu => Values::Uu,
            Values::Uu => Values::Roo,
            Values::Roo => Values::Chii,
            Values::Chii => Values::Paa,
            Values::Paa => Values::Kyuu,
            Values::Kyuu => Values::Ii,
        }
    }
    pub fn prev(self) -> Self {
        match self {
            Values::Ii => Values::Kyuu,
            Values::Ryan => Values::Ii,
            Values::San => Values::Ryan,
            Values::Suu => Values::San,
            Values::Uu => Values::Suu,
            Values::Roo => Values::Uu,
            Values::Chii => Values::Roo,
            Values::Paa => Values::Chii,
            Values::Kyuu => Values::Paa,
        }
    }
}

impl Hai {
    pub fn is_suuhai(self) -> bool {
        match self {
            Hai::Suu(..) => true,
            Hai::Ji(..) => false,
        }
    }

    pub fn next(self) -> Self {
        match self {
            Hai::Suu(SuuHai { suu, value, .. }) => Hai::Suu(SuuHai {
                suu,
                value: value.next(),
                aka: false,
            }),
            Hai::Ji(JiHai::Fon(fon)) => Hai::Ji(JiHai::Fon(fon.next())),
            Hai::Ji(JiHai::Sangen(sangen)) => Hai::Ji(JiHai::Sangen(sangen.next())),
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Hai::Suu(SuuHai { suu, value, .. }) => Hai::Suu(SuuHai {
                suu,
                value: value.prev(),
                aka: false,
            }),
            Hai::Ji(JiHai::Fon(fon)) => Hai::Ji(JiHai::Fon(fon.prev())),
            Hai::Ji(JiHai::Sangen(sangen)) => Hai::Ji(JiHai::Sangen(sangen.prev())),
        }
    }
}

impl Hai {
    pub fn to_char(self) -> char {
        match self {
            Hai::Suu(SuuHai {
                suu: Suu::Wan,
                value,
                ..
            }) => std::char::from_u32(0x1F007 + value as u32 - 1).unwrap(),
            Hai::Suu(SuuHai {
                suu: Suu::Pin,
                value,
                ..
            }) => std::char::from_u32(0x1F019 + value as u32 - 1).unwrap(),
            Hai::Suu(SuuHai {
                suu: Suu::Sou,
                value,
                ..
            }) => std::char::from_u32(0x1F010 + value as u32 - 1).unwrap(),
            Hai::Ji(JiHai::Fon(fon)) => std::char::from_u32(0x1F000 + fon as u32).unwrap(),
            Hai::Ji(JiHai::Sangen(Sangen::Haku)) => std::char::from_u32(0x1F006).unwrap(),
            Hai::Ji(JiHai::Sangen(Sangen::Hatsu)) => std::char::from_u32(0x1F005).unwrap(),
            Hai::Ji(JiHai::Sangen(Sangen::Chun)) => std::char::from_u32(0x1F004).unwrap(),
        }
    }

    /// Convert to terminal-friendly strings for display
    pub fn to_string(self) -> String {
        match self {
            Hai::Ji(JiHai::Sangen(Sangen::Chun)) => {
                let mut s = String::new();
                // Add VS15 before mahjong Chun tile for it to be shown as char (not emoji)
                s.push(std::char::from_u32(0x1F004).unwrap());
                s.push(std::char::from_u32(0xFE0E).unwrap());
                s
            }
            _ => {
                // Except for Chun, all tiles seem to be shown as half-width characters, so add space
                format!("{} ", self.to_char())
            }
        }
    }

    pub fn back_char() -> char {
        std::char::from_u32(0x1F02B).unwrap()
    }
}

pub fn make_all_tiles() -> [Hai; 136] {
    let mut hai = [Hai::Ji(JiHai::Sangen(Sangen::Hatsu)); 136];
    let mut cnt = 0;

    for _ in 0..4 {
        for suu in &SUU {
            for value in &VALUES {
                hai[cnt] = Hai::Suu(SuuHai {
                    suu: *suu,
                    value: *value,
                    aka: false,
                });
                cnt += 1;
            }
        }

        for fon in &FON {
            hai[cnt] = Hai::Ji(JiHai::Fon(*fon));
            cnt += 1;
        }

        for sangen in &SANGEN {
            hai[cnt] = Hai::Ji(JiHai::Sangen(*sangen));
            cnt += 1;
        }
    }
    hai
}

#[derive(Debug, Clone)]
pub enum ParseHaiError {
    EmptyString,
    NoMahjongCharFound { string: String },
}

impl FromStr for Hai {
    type Err = ParseHaiError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(c) = s.chars().next() {
            for hai in make_all_tiles().iter() {
                if hai.to_char() == c {
                    return Ok(*hai);
                }
            }

            Err(ParseHaiError::NoMahjongCharFound {
                string: s.to_owned(),
            })
        } else {
            Err(ParseHaiError::EmptyString)
        }
    }
}
