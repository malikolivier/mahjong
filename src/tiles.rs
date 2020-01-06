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

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone)]
pub struct SuuHai {
    suu: Suu,
    value: Values,
    aka: bool,
}

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

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone)]
pub enum Hai {
    Suu(SuuHai),
    Ji(JiHai),
}

impl Hai {
    pub fn to_string(self) -> String {
        let c = match self {
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
            Hai::Ji(JiHai::Sangen(Sangen::Chun)) => {
                let mut s = String::new();
                // Add VS15 before mahjong Chun tile for it to be shown as char (not emoji)
                s.push(std::char::from_u32(0x1F004).unwrap());
                s.push(std::char::from_u32(0xFE0E).unwrap());
                return s;
            }
        };
        // Except for Chun, all tiles seem to be shown as half-width characters, so add space
        format!("{} ", c)
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
