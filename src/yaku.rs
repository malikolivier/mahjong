use super::game::{Fuuro, Game, KantsuInner, Te};
use super::tiles::{Fon, Hai};

#[derive(Debug, Copy, Clone)]
pub struct AgariTe<'t, 'g> {
    game: &'g Game,
    hai: &'t [Hai],
    fuuro: &'t [Fuuro],
    agarihai: Hai,
    method: WinningMethod,
    wind: Fon,
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

impl std::ops::Add for YakuValue {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        use YakuValue::*;
        match (self, other) {
            (Han(han1), Han(han2)) => Han(han1 + han2),
            (Yakuman(yakuman), Han(_)) => Yakuman(yakuman),
            (Han(_), Yakuman(yakuman)) => Yakuman(yakuman),
            (Yakuman(yakuman1), Yakuman(yakuman2)) => Yakuman(yakuman1 + yakuman2),
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum WinningCombination {
    Chiitoitsu([[Hai; 2]; 7]),
    Kokushimusou([Hai; 14]),
    Normal {
        toitsu: [Hai; 2],
        /// Usually 4 mentsu, unless the hand is open
        mentsu: Vec<[Hai; 3]>,
    },
}

impl fmt::Debug for WinningCombination {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WinningCombination::Chiitoitsu(toitsu7) => {
                let mut toitsu_list = Vec::with_capacity(7);
                for toitsu in toitsu7 {
                    toitsu_list.push(format!("{}{}", toitsu[0].to_char(), toitsu[1].to_char()));
                }
                f.debug_tuple("Chiitoitsu").field(&toitsu_list).finish()
            }
            WinningCombination::Kokushimusou(hai14) => {
                let mut hai_list = String::with_capacity(14);
                for hai in hai14 {
                    hai_list.push(hai.to_char());
                }
                f.debug_tuple("Kokushimusou").field(&hai_list).finish()
            }
            WinningCombination::Normal { toitsu, mentsu } => {
                let mut mentsu_list = Vec::with_capacity(4);
                for m in mentsu {
                    mentsu_list.push(format!(
                        "{}{}{}",
                        m[0].to_char(),
                        m[1].to_char(),
                        m[2].to_char(),
                    ));
                }
                f.debug_struct("Normal")
                    .field(
                        "toitsu",
                        &format!("{}{}", toitsu[0].to_char(), toitsu[1].to_char()),
                    )
                    .field("mentsu", &mentsu_list)
                    .finish()
            }
        }
    }
}

impl<'t, 'g> AgariTe<'t, 'g> {
    pub fn from_te(
        te: &'t Te,
        game: &'g Game,
        agarihai: Hai,
        method: WinningMethod,
        wind: Fon,
    ) -> AgariTe<'t, 'g> {
        AgariTe {
            game,
            hai: te.hai(),
            fuuro: te.fuuro(),
            agarihai,
            method,
            wind,
        }
    }

    fn hai(&self) -> impl Iterator<Item = Hai> + '_ {
        AgariTeHaiIter {
            te: self.hai.iter(),
            agarihai: Some(&self.agarihai),
        }
    }

    fn combinations(&self) -> Vec<AgariTeCombination<'_, 't, 'g>> {
        let te: Vec<_> = self.hai().collect();
        let max = match self.fuuro.len() {
            0 => 4,
            1 => 3,
            2 => 2,
            3 => 1,
            4 => 0,
            _ => unreachable!("Cannot have more than 4 fuuro!"),
        };
        let mut out = vec![];
        for combination in winning_combinations(&te, max) {
            out.push(AgariTeCombination {
                agari_te: self,
                combination,
            });
        }
        out
    }

    fn best_combination(&self) -> Option<AgariTeCombination<'_, 't, 'g>> {
        self.combinations()
            .into_iter()
            .max_by_key(AgariTeCombination::points)
    }

    fn points(&self) -> (Vec<Yaku>, YakuValue, usize) {
        if let Some(comb) = self.best_combination() {
            (comb.yaku(), comb.han(), comb.fu())
        } else {
            (vec![], YakuValue::Han(0), 0)
        }
    }
}

#[derive(Debug, Clone)]
struct AgariTeCombination<'a, 't: 'a, 'g: 't> {
    agari_te: &'a AgariTe<'t, 'g>,
    combination: WinningCombination,
}

impl<'a, 't, 'g> AgariTeCombination<'a, 't, 'g> {
    fn yaku(&self) -> Vec<Yaku> {
        let mut yakus = vec![];

        if self.menzentsumo() {
            yakus.push(Yaku::Menzentsumo);
        }
        if self.riichi() {
            yakus.push(Yaku::Riichi);
        }
        // TODO: ippatsu, tanyao
        if self.pinfu() {
            yakus.push(Yaku::Pinfu);
        }
        // TODO (other yakus)

        yakus
    }

    fn han(&self) -> YakuValue {
        let closed = self.closed();
        // TODO: Add dora and uradora
        self.yaku()
            .iter()
            .fold(YakuValue::Han(0), |acc, yaku| acc + yaku.han(closed))
    }

    /// From https://majandofu.com/fu-calculation#001
    fn fu(&self) -> usize {
        if self.pinfu() && self.menzentsumo() {
            20
        } else if self.chiitoitsu() {
            25
        } else {
            let fuutei = 20;
            let agari_fu = match self.agari_te.method {
                WinningMethod::Ron => 10,
                WinningMethod::Tsumo => 2,
            };

            let mut mentsu_fu = 0;
            for fuuro in self.agari_te.fuuro {
                match fuuro {
                    Fuuro::Kootsu { taken, .. } => {
                        mentsu_fu += if taken.is_jihai_or_1_9() { 4 } else { 2 };
                    }
                    Fuuro::Kantsu(KantsuInner::Ankan {
                        own: [hai, _, _, _],
                    }) => {
                        mentsu_fu += if hai.is_jihai_or_1_9() { 32 } else { 16 };
                    }
                    Fuuro::Kantsu(KantsuInner::DaiMinkan { taken, .. })
                    | Fuuro::Kantsu(KantsuInner::ShouMinkan { taken, .. }) => {
                        mentsu_fu += if taken.is_jihai_or_1_9() { 16 } else { 8 };
                    }
                    _ => {}
                }
            }
            if let WinningCombination::Normal { mentsu, .. } = &self.combination {
                for m in mentsu {
                    // FIXME: If the agari_hai in case of ron is used for a kootsu,
                    // then the kootsu is considered to be an minkoo, not ankoo.
                    if is_kootsu(m) {
                        mentsu_fu += if m[0].is_jihai_or_1_9() { 8 } else { 4 };
                    }
                }
            }

            let mut atama_fu = 0;
            if let WinningCombination::Normal { toitsu, .. } = &self.combination {
                if toitsu[0].is_yakuhai(self.agari_te.game.wind, self.agari_te.wind) {
                    atama_fu = 2;
                }
            }

            let machi_fu = match self.machi() {
                Machi::Tanki | Machi::Penchan | Machi::Kanchan => 2,
                Machi::Ryanmen
                | Machi::Shanpon
                | Machi::KokushimusouNormal
                | Machi::KokushimusouJuusanmen => 0,
            };

            let fu = fuutei + agari_fu + mentsu_fu + atama_fu + machi_fu;
            // Round up to the next decade
            ((fu + 9) / 10) * 10
        }
    }

    fn machi(&self) -> Machi {
        match &self.combination {
            WinningCombination::Normal { toitsu, mentsu } => {
                if self.pinfu() {
                    // If the pinfu yaku is realized,
                    // use the machi that returns the less fu (Ryanmen)
                    // to get more han.
                    Machi::Ryanmen
                } else {
                    let machis = machi(*toitsu, mentsu, self.agari_te.agarihai);
                    // Return the machi with the highest fu count
                    machis.into_iter().min().expect("Has machi")
                }
            }
            WinningCombination::Kokushimusou(_) => {
                if self.agari_te.hai.contains(&self.agari_te.agarihai) {
                    Machi::KokushimusouJuusanmen
                } else {
                    Machi::KokushimusouNormal
                }
            }
            WinningCombination::Chiitoitsu(_) => {
                // A chiitoitsu can only result in a tanki machi
                Machi::Tanki
            }
        }
    }

    fn points(&self) -> (YakuValue, usize) {
        (self.han(), self.fu())
    }

    fn closed(&self) -> bool {
        self.agari_te.fuuro.is_empty()
    }

    fn menzentsumo(&self) -> bool {
        self.closed() && self.agari_te.method == WinningMethod::Tsumo
    }

    fn riichi(&self) -> bool {
        self.agari_te.game.player_riichi(self.agari_te.wind)
    }

    fn pinfu(&self) -> bool {
        if let WinningCombination::Normal { toitsu, mentsu } = &self.combination {
            let machis = machi(*toitsu, mentsu, self.agari_te.agarihai);
            self.closed()
                && machis.contains(&Machi::Ryanmen)
                && !toitsu[0].is_yakuhai(self.agari_te.game.wind, self.agari_te.wind)
                && mentsu.iter().all(|m| !is_kootsu(m))
        } else {
            false
        }
    }

    fn chiitoitsu(&self) -> bool {
        if let WinningCombination::Chiitoitsu(_) = self.combination {
            true
        } else {
            false
        }
    }
}

fn is_kootsu(mentsu: &[Hai; 3]) -> bool {
    mentsu[0] == mentsu[1] && mentsu[1] == mentsu[2]
}

/// Find all the machi for this winning hand (normal 4 mentsu 1 toitsu te only).
fn machi(toitsu: [Hai; 2], mentsu: &[[Hai; 3]], agari_hai: Hai) -> Vec<Machi> {
    let mut machis = vec![];
    if toitsu.contains(&agari_hai) {
        machis.push(Machi::Tanki);
    } else {
        for m in mentsu {
            let mut m = m.to_owned();
            m.sort();
            if m.contains(&agari_hai) {
                if is_kootsu(&m) {
                    machis.push(Machi::Shanpon);
                } else {
                    if m[0] == agari_hai {
                        if m[2].is_jihai_or_1_9() {
                            machis.push(Machi::Penchan);
                        } else {
                            machis.push(Machi::Ryanmen);
                        }
                    } else if m[1] == agari_hai {
                        machis.push(Machi::Kanchan);
                    } else if m[2] == agari_hai {
                        if m[0].is_jihai_or_1_9() {
                            machis.push(Machi::Penchan);
                        } else {
                            machis.push(Machi::Ryanmen);
                        }
                    } else {
                        unreachable!("Contains agari hai");
                    }
                }
            }
        }
    }
    machis
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
pub enum Machi {
    Tanki,
    Penchan,
    Kanchan,
    Ryanmen,
    Shanpon,
    // Nobetan can be considered as Tanki + Ryanmen
    // Nobetan,
    KokushimusouNormal,
    KokushimusouJuusanmen,
}

#[derive(Debug, Clone)]
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

fn winning_combinations(te: &[Hai], max: usize) -> Vec<WinningCombination> {
    let mut out = vec![];

    if let Some(comb) = try_chiitoitsu(te) {
        out.push(WinningCombination::Chiitoitsu(comb));
    }
    if let Some(comb) = try_kokushimuso(te) {
        out.push(WinningCombination::Kokushimusou(comb));
    }
    out.extend(try_normal_combinations(te, max));

    out
}

fn try_normal_combinations(te: &[Hai], max: usize) -> Vec<WinningCombination> {
    let mut combs = vec![];
    for head in all_heads(te) {
        for comb in pickup_mentsu_comb(&head.remaining, max) {
            combs.push(WinningCombination::Normal {
                toitsu: head.head,
                mentsu: comb,
            })
        }
    }
    combs
}

fn pickup_mentsu_comb(remaining: &[Hai], max: usize) -> Vec<Vec<[Hai; 3]>> {
    let mut out = vec![];

    // Find all possible kootsu with a given te
    let all_kootsu_ = all_kootsu(remaining);
    for kootsu in all_kootsu_ {
        // Find all possible shuntsu with a given te
        for shuntsu in all_shuntsu(&kootsu.remaining) {
            if kootsu.mentsu.len() + shuntsu.mentsu.len() == max {
                let mut mentsu_4 = Vec::with_capacity(max);
                for mentsu in kootsu.mentsu.iter().chain(shuntsu.mentsu.iter()) {
                    mentsu_4.push(*mentsu);
                }
                // Sort tiles to have a pretty result
                mentsu_4.sort();
                out.push(mentsu_4);
            }
        }
    }

    // Reverse
    let all_shuntsu_ = all_shuntsu(remaining);
    for shuntsu in all_shuntsu_ {
        for kootsu in all_kootsu(&shuntsu.remaining) {
            if kootsu.mentsu.len() + shuntsu.mentsu.len() == max {
                let mut mentsu_4 = Vec::with_capacity(max);
                for mentsu in kootsu.mentsu.iter().chain(shuntsu.mentsu.iter()) {
                    mentsu_4.push(*mentsu);
                }
                mentsu_4.sort();
                out.push(mentsu_4);
            }
        }
    }

    // Normalize
    out.sort();
    out.dedup();

    out
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct Mentsu {
    mentsu: Vec<[Hai; 3]>,
    remaining: Vec<Hai>,
}

impl Mentsu {
    fn normalize(&mut self) {
        self.mentsu.sort();
        self.remaining.sort();
    }
}

use std::fmt;
impl fmt::Debug for Mentsu {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut mentsu_list = Vec::with_capacity(self.mentsu.len());
        for mentsu in &self.mentsu {
            mentsu_list.push(format!(
                "{}{}{}",
                mentsu[0].to_char(),
                mentsu[1].to_char(),
                mentsu[2].to_char()
            ));
        }
        let mut remaining = String::new();
        for hai in &self.remaining {
            remaining.push(hai.to_char());
        }
        f.debug_struct("Mentsu")
            .field("mentsu", &mentsu_list)
            .field("remaining", &remaining)
            .finish()
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Head {
    head: [Hai; 2],
    remaining: Vec<Hai>,
}

fn all_heads(te: &[Hai]) -> Vec<Head> {
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

    // Normalize
    heads.sort();
    heads.dedup();

    heads
}

fn all_kootsu(te: &[Hai]) -> Vec<Mentsu> {
    let mut kootsu = vec![Mentsu {
        // Include the trivial mentsu (all remainining and no mentsu)
        mentsu: vec![],
        remaining: te.to_owned(),
    }];
    for hai in te {
        let mut te_ = te.to_owned();
        if let Some(pos) = te_.iter().position(|x| x == hai) {
            te_.swap_remove(pos);
        } else {
            unreachable!("Hai should be there");
        }

        if let Some(pos) = te_.iter().position(|x| x == hai) {
            let hai2 = te_.swap_remove(pos);
            if let Some(pos) = te_.iter().position(|x| x == hai) {
                let hai3 = te_.swap_remove(pos);
                let this_kootsu = [*hai, hai2, hai3];
                let all_remaining_kootsu = all_kootsu(&te_);
                kootsu.push(Mentsu {
                    mentsu: vec![this_kootsu],
                    remaining: te_,
                });
                for remaining_kootsu in all_remaining_kootsu {
                    let mut mentsu = vec![this_kootsu];
                    mentsu.extend(remaining_kootsu.mentsu);
                    kootsu.push(Mentsu {
                        mentsu,
                        remaining: remaining_kootsu.remaining,
                    })
                }
            }
        }
    }

    for i in kootsu.iter_mut() {
        i.normalize()
    }
    kootsu.sort();
    kootsu.dedup();

    kootsu
}

fn all_shuntsu(te: &[Hai]) -> Vec<Mentsu> {
    fn possible_shuntsu(hai: Hai) -> Vec<[Hai; 3]> {
        use super::tiles::{SuuHai, Values};
        match hai {
            Hai::Suu(SuuHai { value, .. }) => {
                let right = [hai.prev().prev(), hai.prev(), hai];
                let middle = [hai.prev(), hai, hai.next()];
                let left = [hai, hai.next(), hai.next().next()];

                match value {
                    Values::Ii => vec![left],
                    Values::Ryan => vec![middle, left],
                    Values::Paa => vec![right, middle],
                    Values::Kyuu => vec![right],
                    _ => vec![right, middle, left],
                }
            }
            Hai::Ji(..) => vec![],
        }
    }

    #[derive(Debug)]
    struct ShuntsuList {
        shuntsu: [Hai; 3],
        next: Vec<ShuntsuList>,
        remaining: Vec<Hai>,
    }

    fn find_shuntsu(te: &[Hai]) -> Vec<ShuntsuList> {
        let mut out_shuntsu = vec![];
        for hai in te {
            for shuntsu in possible_shuntsu(*hai) {
                let mut te_ = te.to_owned();
                let mut matched_shuntsu = true;
                for hai in &shuntsu {
                    if let Some(pos) = te_.iter().position(|x| x == hai) {
                        te_.swap_remove(pos);
                    } else {
                        matched_shuntsu = false;
                    }
                }
                if matched_shuntsu {
                    let next = find_shuntsu(&te_);
                    out_shuntsu.push(ShuntsuList {
                        shuntsu,
                        next,
                        remaining: te_,
                    });
                }
            }
        }
        out_shuntsu
    }

    fn shuntsu_list_to_mentsu(head: ShuntsuList) -> Vec<Mentsu> {
        let mut out = vec![];
        out.push(Mentsu {
            mentsu: vec![head.shuntsu],
            remaining: head.remaining,
        });
        for li in head.next {
            let mentsu_li = shuntsu_list_to_mentsu(li);
            for mentsu in mentsu_li {
                let mut shuntsu_list = vec![head.shuntsu];
                shuntsu_list.extend(mentsu.mentsu);
                out.push(Mentsu {
                    mentsu: shuntsu_list,
                    remaining: mentsu.remaining,
                })
            }
        }
        out
    }

    let mut out = vec![Mentsu {
        // Include the trivial mentsu (all remainining and no mentsu)
        mentsu: vec![],
        remaining: te.to_owned(),
    }];
    for shuntsu in find_shuntsu(te) {
        out.extend(shuntsu_list_to_mentsu(shuntsu));
    }
    for i in out.iter_mut() {
        i.normalize()
    }
    out.sort();
    out.dedup();

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

    #[test]
    fn test_all_kootsu() {
        let te = te_from_string("🀇🀇🀇🀈🀈🀉🀊🀋🀌🀎🀎🀎").unwrap();
        let result = all_kootsu(&te);
        assert_eq!(
            result,
            vec![
                mentsu_from_str(&[], "🀇🀇🀇🀈🀈🀉🀊🀋🀌🀎🀎🀎").unwrap(),
                mentsu_from_str(&["🀇🀇🀇"], "🀈🀈🀉🀊🀋🀌🀎🀎🀎").unwrap(),
                mentsu_from_str(&["🀇🀇🀇", "🀎🀎🀎"], "🀈🀈🀉🀊🀋🀌").unwrap(),
                mentsu_from_str(&["🀎🀎🀎"], "🀇🀇🀇🀈🀈🀉🀊🀋🀌").unwrap(),
            ]
        );
    }

    #[test]
    fn test_all_shuntsu() {
        let te = te_from_string("🀇🀇🀈🀈🀉🀉🀊🀋🀌🀎🀎🀎").unwrap();
        let result = all_shuntsu(&te);
        assert_eq!(
            result,
            vec![
                mentsu_from_str(&[], "🀇🀇🀈🀈🀉🀉🀊🀋🀌🀎🀎🀎").unwrap(),
                mentsu_from_str(&["🀇🀈🀉"], "🀇🀈🀉🀊🀋🀌🀎🀎🀎").unwrap(),
                mentsu_from_str(&["🀇🀈🀉", "🀇🀈🀉"], "🀊🀋🀌🀎🀎🀎").unwrap(),
                mentsu_from_str(&["🀇🀈🀉", "🀇🀈🀉", "🀊🀋🀌"], "🀎🀎🀎").unwrap(),
                mentsu_from_str(&["🀇🀈🀉", "🀈🀉🀊"], "🀇🀋🀌🀎🀎🀎").unwrap(),
                mentsu_from_str(&["🀇🀈🀉", "🀉🀊🀋"], "🀇🀈🀌🀎🀎🀎").unwrap(),
                mentsu_from_str(&["🀇🀈🀉", "🀊🀋🀌"], "🀇🀈🀉🀎🀎🀎").unwrap(),
                mentsu_from_str(&["🀈🀉🀊"], "🀇🀇🀈🀉🀋🀌🀎🀎🀎").unwrap(),
                mentsu_from_str(&["🀉🀊🀋"], "🀇🀇🀈🀈🀉🀌🀎🀎🀎").unwrap(),
                mentsu_from_str(&["🀊🀋🀌"], "🀇🀇🀈🀈🀉🀉🀎🀎🀎").unwrap(),
            ]
        );
    }

    #[test]
    fn test_find_winning_comb_normal() {
        let te = te_from_string("🀇🀇🀈🀈🀉🀉🀊🀋🀌🀌🀌🀎🀎🀎").unwrap();
        let result = winning_combinations(&te, 4);
        assert_eq!(
            result,
            vec![normal_winning_combination_from_str("🀌🀌", ["🀇🀈🀉", "🀇🀈🀉", "🀊🀋🀌", "🀎🀎🀎"]).unwrap()]
        );
    }

    #[test]
    fn test_find_winning_comb_normal_many() {
        let te = te_from_string("🀇🀇🀇🀈🀈🀈🀉🀉🀉🀊🀋🀌🀌🀌").unwrap();
        let result = winning_combinations(&te, 4);
        assert_eq!(
            result,
            vec![
                normal_winning_combination_from_str("🀉🀉", ["🀇🀇🀇", "🀈🀈🀈", "🀉🀊🀋", "🀌🀌🀌"]).unwrap(),
                normal_winning_combination_from_str("🀌🀌", ["🀇🀇🀇", "🀈🀈🀈", "🀉🀉🀉", "🀊🀋🀌"]).unwrap(),
                normal_winning_combination_from_str("🀌🀌", ["🀇🀈🀉", "🀇🀈🀉", "🀇🀈🀉", "🀊🀋🀌"]).unwrap(),
            ]
        );
    }

    #[test]
    fn test_find_winning_comb_ryanpeikou() {
        let te = te_from_string("🀇🀇🀈🀈🀉🀉🀊🀊🀋🀋🀌🀌🀍🀍").unwrap();
        let result = winning_combinations(&te, 4);
        assert_eq!(
            result,
            vec![
                chiitoitsu_winning_combination_from_str(["🀇🀇", "🀈🀈", "🀉🀉", "🀊🀊", "🀋🀋", "🀌🀌", "🀍🀍"])
                    .unwrap(),
                normal_winning_combination_from_str("🀇🀇", ["🀈🀉🀊", "🀈🀉🀊", "🀋🀌🀍", "🀋🀌🀍"]).unwrap(),
                normal_winning_combination_from_str("🀊🀊", ["🀇🀈🀉", "🀇🀈🀉", "🀋🀌🀍", "🀋🀌🀍"]).unwrap(),
                normal_winning_combination_from_str("🀍🀍", ["🀇🀈🀉", "🀇🀈🀉", "🀊🀋🀌", "🀊🀋🀌"]).unwrap(),
            ]
        );
    }

    use super::super::tiles::ParseHaiError;
    fn mentsu_from_str(mentsu: &[&str], remaining: &str) -> Result<Mentsu, ParseHaiError> {
        let mut mentsu_out = vec![];
        for m in mentsu {
            let m = te_from_string(m)?;
            assert_eq!(m.len(), 3);
            mentsu_out.push([m[0], m[1], m[2]]);
        }
        Ok(Mentsu {
            mentsu: mentsu_out,
            remaining: te_from_string(remaining)?,
        })
    }

    fn chiitoitsu_winning_combination_from_str(
        toitsu: [&str; 7],
    ) -> Result<WinningCombination, ParseHaiError> {
        let mut toitsu_out = Vec::with_capacity(7);
        for t in &toitsu {
            let t = te_from_string(t)?;
            assert_eq!(t.len(), 2);
            toitsu_out.push([t[0], t[1]]);
        }
        Ok(WinningCombination::Chiitoitsu([
            toitsu_out[0],
            toitsu_out[1],
            toitsu_out[2],
            toitsu_out[3],
            toitsu_out[4],
            toitsu_out[5],
            toitsu_out[6],
        ]))
    }

    fn normal_winning_combination_from_str(
        toitsu: &str,
        mentsu: [&str; 4],
    ) -> Result<WinningCombination, ParseHaiError> {
        let toitsu = te_from_string(toitsu)?;
        assert_eq!(toitsu.len(), 2);
        let toitsu_out = [toitsu[0], toitsu[1]];

        let mut mentsu_out = Vec::with_capacity(4);
        for m in &mentsu {
            let m = te_from_string(m)?;
            assert_eq!(m.len(), 3);
            mentsu_out.push([m[0], m[1], m[2]]);
        }
        Ok(WinningCombination::Normal {
            toitsu: toitsu_out,
            mentsu: mentsu_out,
        })
    }
}
