use super::game::{Fuuro, Game, Te};
use super::tiles::Hai;

#[derive(Debug, Copy, Clone)]
pub struct AgariTe<'t, 'g> {
    game: &'g Game,
    hai: &'t [Hai],
    fuuro: &'t [Fuuro],
    agarihai: Hai,
    method: WinningMethod,
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub enum WinningMethod {
    Ron,
    Tsumo,
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub enum Yaku {
    Menzentsumo,
    Riichi,
    Ippatsu,
    Tanyao,
    Pinfu,
    Iipeikou,
    Haku,
    Hatsu,
    Chun,
    BaNoKaze,
    JibunNoKaze,
    Chankan,
    RinshanKaihou,
    Haiteiraoyue,
    Houteiraoyui,
    Daburii,
    Chiitoitsu,
    Toitoi,
    SanAnkou,
    SanshokuDoukou,
    SanshokuDoujun,
    Honroutou,
    Ittsuu,
    Chanta,
    Shousangen,
    Sankantsu,
    HonItsu,
    Junchan,
    Ryanpeikou,
    Nagashimangan,
    ChinItsu,
    Tenhou,
    Chihou,
    Renhou,
    Ryuuiisou,
    Daisangen,
    Shousuushii,
    Tsuuiisou,
    Kokushimusou,
    Chuurenpoutou,
    Suuankou,
    Chinroutou,
    Suukantsu,
    SuuankouTanki,
    Daisuushi,
    JunseiChuurenpoutou,
    KokushimusouJuusanmen,
}

use Yaku::*;

const YAKU: [Yaku; 47] = [
    Menzentsumo,
    Riichi,
    Ippatsu,
    Tanyao,
    Pinfu,
    Iipeikou,
    Haku,
    Hatsu,
    Chun,
    BaNoKaze,
    JibunNoKaze,
    Chankan,
    RinshanKaihou,
    Haiteiraoyue,
    Houteiraoyui,
    Daburii,
    Chiitoitsu,
    Toitoi,
    SanAnkou,
    SanshokuDoukou,
    SanshokuDoujun,
    Honroutou,
    Ittsuu,
    Chanta,
    Shousangen,
    Sankantsu,
    HonItsu,
    Junchan,
    Ryanpeikou,
    Nagashimangan,
    ChinItsu,
    Tenhou,
    Chihou,
    Renhou,
    Ryuuiisou,
    Daisangen,
    Shousuushii,
    Tsuuiisou,
    Kokushimusou,
    Chuurenpoutou,
    Suuankou,
    Chinroutou,
    Suukantsu,
    SuuankouTanki,
    Daisuushi,
    JunseiChuurenpoutou,
    KokushimusouJuusanmen,
];

impl Yaku {
    pub fn han(self, closed: bool) -> YakuValue {
        use YakuValue::*;
        match self {
            Menzentsumo => Han(1),
            Riichi => Han(1),
            Ippatsu => Han(1),
            Tanyao => Han(1),
            Pinfu => Han(1),
            Iipeikou => Han(1),
            Haku => Han(1),
            Hatsu => Han(1),
            Chun => Han(1),
            BaNoKaze => Han(1),
            JibunNoKaze => Han(1),
            Chankan => Han(1),
            RinshanKaihou => Han(1),
            Haiteiraoyue => Han(1),
            Houteiraoyui => Han(1),
            Daburii => Han(2),
            Chiitoitsu => Han(2),
            Toitoi => Han(2),
            SanAnkou => Han(2),
            SanshokuDoukou => Han(2),
            SanshokuDoujun => Han(if closed { 2 } else { 1 }),
            Honroutou => Han(2),
            Ittsuu => Han(if closed { 2 } else { 1 }),
            Chanta => Han(if closed { 2 } else { 1 }),
            Shousangen => Han(2),
            Sankantsu => Han(2),
            HonItsu => Han(if closed { 3 } else { 2 }),
            Junchan => Han(if closed { 3 } else { 2 }),
            Ryanpeikou => Han(3),
            Nagashimangan => Han(5),
            ChinItsu => Han(if closed { 6 } else { 5 }),
            Tenhou => Yakuman(1),
            Chihou => Yakuman(1),
            Renhou => Yakuman(1),
            Ryuuiisou => Yakuman(1),
            Daisangen => Yakuman(1),
            Shousuushii => Yakuman(1),
            Tsuuiisou => Yakuman(1),
            Kokushimusou => Yakuman(1),
            Chuurenpoutou => Yakuman(1),
            Suuankou => Yakuman(1),
            Chinroutou => Yakuman(1),
            Suukantsu => Yakuman(1),
            SuuankouTanki => Yakuman(2),
            Daisuushi => Yakuman(2),
            JunseiChuurenpoutou => Yakuman(2),
            KokushimusouJuusanmen => Yakuman(2),
        }
    }
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone)]
pub enum YakuValue {
    Han(usize),
    Yakuman(usize),
}

#[derive(Debug, Copy, Clone)]
pub enum WinningCombination {
    Chiitoitsu([[Hai; 2]; 7]),
    Kokushimusou([Hai; 14]),
    Normal {
        toitsu: [Hai; 2],
        mentsu: [[Hai; 3]; 4],
    },
}

impl<'t, 'g> AgariTe<'t, 'g> {
    pub fn fu(&self) -> usize {
        // TODO: Check if this is correct!
        let win_bonus = match self.method {
            WinningMethod::Ron => 30,
            WinningMethod::Tsumo => 20,
        };
        win_bonus
    }

    pub fn from_te(
        te: &'t Te,
        game: &'g Game,
        agarihai: Hai,
        method: WinningMethod,
    ) -> AgariTe<'t, 'g> {
        AgariTe {
            game,
            hai: te.hai(),
            fuuro: te.fuuro(),
            agarihai,
            method,
        }
    }

    fn hai(&self) -> impl Iterator<Item = Hai> + '_ {
        AgariTeHaiIter {
            te: self.hai.iter(),
            agarihai: Some(&self.agarihai),
        }
    }
}

struct AgariTeHaiIter<'t> {
    te: std::slice::Iter<'t, Hai>,
    agarihai: Option<&'t Hai>,
}

impl<'t> Iterator for AgariTeHaiIter<'t> {
    type Item = Hai;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(hai) = self.te.next() {
            Some(*hai)
        } else if let Some(hai) = self.agarihai.take() {
            Some(*hai)
        } else {
            None
        }
    }
}

fn winning_combinations(te: &[Hai]) -> Vec<WinningCombination> {
    let mut out = vec![];

    if let Some(comb) = try_chiitoitsu(te) {
        out.push(WinningCombination::Chiitoitsu(comb));
    }
    if let Some(comb) = try_kokushimuso(te) {
        out.push(WinningCombination::Kokushimusou(comb));
    }

    #[derive(Debug)]
    struct Head {
        head: [Hai; 2],
        remaining: Vec<Hai>,
    }

    // Find all possible heads
    let mut heads = vec![];
    for hai in te {
        let mut te_ = te.to_owned();
        if let Some(pos) = te_.iter().position(|x| x == hai) {
            te_.swap_remove(pos);
        } else {
            unreachable!("Hai should be there");
        }

        if let Some(pos) = te_.iter().position(|x| x == hai) {
            let hai2 = te_.swap_remove(pos);
            heads.push(Head {
                head: [*hai, hai2],
                remaining: te_,
            });
        }
    }

    struct Kootsu {
        kootsu: Vec<[Hai; 3]>,
        remaining: Vec<Hai>,
    }

    fn all_kootsu(te: &[Hai]) -> Kootsu {
        let mut kootsu = vec![];
        let mut remaining = te.to_owned();
        for hai in te {
            let mut te_ = remaining.clone();
            if let Some(pos) = te_.iter().position(|x| x == hai) {
                te_.swap_remove(pos);
            } else {
                unreachable!("Hai should be there");
            }

            if let Some(pos) = te_.iter().position(|x| x == hai) {
                let hai2 = te_.swap_remove(pos);
                if let Some(pos) = te_.iter().position(|x| x == hai) {
                    let hai3 = te_.swap_remove(pos);
                    kootsu.push([*hai, hai2, hai3]);
                    remaining = te_;
                }
            }
        }
        Kootsu { kootsu, remaining }
    }

    // Find all possible kootsu with a given te
    // Find all possible shuntsu with a given te

    out
}

fn try_chiitoitsu(te: &[Hai]) -> Option<[[Hai; 2]; 7]> {
    let mut chiitoitsu = [[Option::<Hai>::None; 2]; 7];
    for hai in te {
        if let Some(pos) = chiitoitsu
            .iter()
            .position(|[some_hai1, some_hai2]| *some_hai1 == Some(*hai) && some_hai2.is_none())
        {
            chiitoitsu[pos][1] = Some(*hai);
        } else if let Some(empty_pos) = chiitoitsu
            .iter()
            .position(|[some_hai1, some_hai2]| some_hai1.is_none() && some_hai2.is_none())
        {
            if chiitoitsu
                .iter()
                .find(|[some_hai1, _]| some_hai1 == &Some(*hai))
                .is_none()
            {
                chiitoitsu[empty_pos][0] = Some(*hai);
            }
        }
    }
    match chiitoitsu {
        [[Some(hai1), Some(hai2)], [Some(hai3), Some(hai4)], [Some(hai5), Some(hai6)], [Some(hai7), Some(hai8)], [Some(hai9), Some(hai10)], [Some(hai11), Some(hai12)], [Some(hai13), Some(hai14)]] => {
            Some([
                [hai1, hai2],
                [hai3, hai4],
                [hai5, hai6],
                [hai7, hai8],
                [hai9, hai10],
                [hai11, hai12],
                [hai13, hai14],
            ])
        }
        _ => None,
    }
}

fn try_kokushimuso(te: &[Hai]) -> Option<[Hai; 14]> {
    let mut kokushimuso = [Option::<Hai>::None; 14];
    let mut head_found = false;
    for hai in te {
        if hai.is_jihai_or_1_9() {
            if kokushimuso.contains(&Some(*hai)) {
                if head_found {
                    // Cannot have two heads!
                    return None;
                }
                head_found = true;
            }
            if let Some(empty_pos) = kokushimuso.iter().position(Option::is_none) {
                kokushimuso[empty_pos] = Some(*hai)
            }
        } else {
            return None;
        }
    }
    match kokushimuso {
        [Some(hai1), Some(hai2), Some(hai3), Some(hai4), Some(hai5), Some(hai6), Some(hai7), Some(hai8), Some(hai9), Some(hai10), Some(hai11), Some(hai12), Some(hai13), Some(hai14)] => {
            Some([
                hai1, hai2, hai3, hai4, hai5, hai6, hai7, hai8, hai9, hai10, hai11, hai12, hai13,
                hai14,
            ])
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::super::game::tests::te_from_string;
    use super::*;

    #[test]
    fn test_chitoitsu_agari() {
        let te = te_from_string("🀇🀇🀈🀈🀏🀏🀙🀙🀀🀀🀁🀁🀆🀆").unwrap();
        let comb = try_chiitoitsu(&te);
        assert!(comb.is_some());
    }

    #[test]
    fn test_chitoitsu_kan_edgecase() {
        let te = te_from_string("🀇🀇🀇🀇🀏🀏🀙🀙🀀🀀🀁🀁🀆🀆").unwrap();
        let comb = try_chiitoitsu(&te);
        assert!(comb.is_none());
    }

    #[test]
    fn test_chitoitsu_iishanten() {
        let te = te_from_string("🀇🀈🀈🀊🀏🀏🀙🀙🀀🀀🀁🀁🀆🀆").unwrap();
        let comb = try_chiitoitsu(&te);
        assert!(comb.is_none());
    }

    #[test]
    fn test_kokushimuso_agari() {
        let te = te_from_string("🀇🀏🀙🀡🀐🀘🀀🀀🀁🀂🀃🀆🀅🀄").unwrap();
        assert!(try_kokushimuso(&te).is_some());
    }
}
