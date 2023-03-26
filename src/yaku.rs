use log::{debug, trace};

use super::game::{Fuuro, Game, KantsuInner, Te};
use super::tiles::{Fon, Hai, SuuHai};

#[derive(Debug, Copy, Clone)]
pub struct AgariTe<'t, 'g> {
    game: &'g Game,
    hai: &'t [Hai],
    fuuro: &'t [Fuuro],
    agarihai: Hai,
    method: WinningMethod,
    wind: Fon,
    /// Ron on kakan (steal a kan)
    chankan: bool,
    /// Tsumo on kan supplementary tile
    rinshankaihou: bool,
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

    pub fn name(self) -> &'static str {
        match self {
            Menzentsumo => "門前自摸",
            Riichi => "立直",
            Ippatsu => "一発",
            Tanyao => "断么九",
            Pinfu => "平和",
            Iipeikou => "一盃口",
            Haku => "白",
            Hatsu => "発",
            Chun => "中",
            BaNoKaze => "場の風",
            JibunNoKaze => "自分の風",
            Chankan => "搶槓",
            RinshanKaihou => "嶺上開花",
            Haiteiraoyue => "海底摸月",
            Houteiraoyui => "河底撈魚 ",
            Daburii => "ダブル立直",
            Chiitoitsu => "七対子",
            Toitoi => "対々和",
            SanAnkou => "三暗刻",
            SanshokuDoukou => "三色同刻",
            SanshokuDoujun => "三色同順",
            Honroutou => "混老頭",
            Ittsuu => "一気通貫",
            Chanta => "混全帯么九",
            Shousangen => "小三元",
            Sankantsu => "三槓子",
            HonItsu => "混一色",
            Junchan => "純全帯么九",
            Ryanpeikou => "二盃口",
            Nagashimangan => "流し満貫",
            ChinItsu => "清一色",
            Tenhou => "天和",
            Chihou => "地和",
            Renhou => "人和",
            Ryuuiisou => "緑一色",
            Daisangen => "大三元",
            Shousuushii => "小四喜",
            Tsuuiisou => "字一色",
            Kokushimusou => "国士無双",
            Chuurenpoutou => "九蓮宝燈",
            Suuankou => "四暗刻",
            Chinroutou => "清老頭",
            Suukantsu => "四槓子",
            SuuankouTanki => "四槓子単騎",
            Daisuushi => "大四喜",
            JunseiChuurenpoutou => "純正九蓮宝燈",
            KokushimusouJuusanmen => "国士無双十三面",
        }
    }
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone)]
pub enum YakuValue {
    Han(usize),
    Yakuman(usize),
}
use YakuValue::*;

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
            chankan: false,
            rinshankaihou: false,
        }
    }

    pub fn chankan(mut self, chankan: bool) -> Self {
        self.chankan = chankan;
        self
    }
    pub fn rinshankaihou(mut self, rinshankaihou: bool) -> Self {
        self.rinshankaihou = rinshankaihou;
        self
    }

    /// Iterate over hidden tiles.
    fn hai(&self) -> impl Iterator<Item = Hai> + '_ {
        AgariTeHaiIter {
            te: self.hai.iter(),
            agarihai: Some(&self.agarihai),
        }
    }

    /// Iterate over all tiles, including called ones.
    fn hai_all(&self) -> impl Iterator<Item = Hai> + '_ {
        let mut open_hai = Vec::with_capacity(4 * self.fuuro.len());
        for fuuro in self.fuuro {
            match fuuro {
                Fuuro::Shuntsu { own, taken, .. } | Fuuro::Kootsu { own, taken, .. } => {
                    open_hai.push(own[0]);
                    open_hai.push(own[1]);
                    open_hai.push(*taken);
                }
                Fuuro::Kantsu(KantsuInner::Ankan { own }) => {
                    open_hai.push(own[0]);
                    open_hai.push(own[1]);
                    open_hai.push(own[2]);
                    open_hai.push(own[3]);
                }
                Fuuro::Kantsu(KantsuInner::DaiMinkan { own, taken, .. }) => {
                    open_hai.push(own[0]);
                    open_hai.push(own[1]);
                    open_hai.push(own[2]);
                    open_hai.push(*taken);
                }
                Fuuro::Kantsu(KantsuInner::ShouMinkan {
                    own, taken, added, ..
                }) => {
                    open_hai.push(own[0]);
                    open_hai.push(own[1]);
                    open_hai.push(*added);
                    open_hai.push(*taken);
                }
            }
        }
        self.hai().chain(open_hai)
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

    pub fn points(&self) -> (Vec<Yaku>, YakuValue, usize) {
        if let Some(comb) = self.best_combination() {
            let yaku = comb.yaku();
            trace!("Got yaku: {:?}", &yaku);
            (yaku, comb.han(), comb.fu())
        } else {
            trace!("No combination");
            (vec![], YakuValue::Han(0), 0)
        }
    }
}

#[derive(Debug, Clone)]
struct AgariTeCombination<'a, 't: 'a, 'g: 't> {
    agari_te: &'a AgariTe<'t, 'g>,
    combination: WinningCombination,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum Mentsu_ {
    Anshun([Hai; 3]),
    Minshun([Hai; 3]),
    Ankou([Hai; 3]),
    Minkou([Hai; 3]),
    Ankan([Hai; 4]),
    Minkan([Hai; 4]),
}

impl Mentsu_ {
    /// Return true if this mentsu is Ankou, Minkou, Ankan or Minkan
    /// whose tile match the given predicate.
    fn count_as_kootsu_with<F>(&self, mut f: F) -> bool
    where
        F: FnMut(Hai) -> bool,
    {
        use Mentsu_::*;
        match self {
            Ankou([hai_, _, _])
            | Minkou([hai_, _, _])
            | Ankan([hai_, _, _, _])
            | Minkan([hai_, _, _, _]) => f(*hai_),
            Anshun(_) | Minshun(_) => false,
        }
    }

    /// Return true if this mentsu is Ankou, Minkou, Ankan or Minkan
    fn count_as_kootsu(&self) -> bool {
        self.count_as_kootsu_with(|_| true)
    }

    /// Return kootsu's tile (if it is kootsu)
    fn as_kootsu_hai(&self) -> Option<Hai> {
        use Mentsu_::*;
        match self {
            Ankou([hai, _, _])
            | Minkou([hai, _, _])
            | Ankan([hai, _, _, _])
            | Minkan([hai, _, _, _]) => Some(*hai),
            Anshun(_) | Minshun(_) => None,
        }
    }

    fn is_extremity(&self) -> bool {
        use Mentsu_::*;
        match self {
            Ankou([hai, _, _])
            | Minkou([hai, _, _])
            | Ankan([hai, _, _, _])
            | Minkan([hai, _, _, _]) => hai.is_jihai_or_1_9(),
            Anshun([hai1, hai2, hai3]) | Minshun([hai1, hai2, hai3]) => {
                hai1.is_jihai_or_1_9() || hai2.is_jihai_or_1_9() || hai3.is_jihai_or_1_9()
            }
        }
    }

    fn is_kan(&self) -> bool {
        use Mentsu_::*;
        match self {
            Ankan(_) | Minkan(_) => true,
            Ankou(_) | Minkou(_) | Anshun(_) | Minshun(_) => false,
        }
    }
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
        if self.ippatsu() {
            yakus.push(Yaku::Ippatsu);
        }
        if self.tanyao() {
            yakus.push(Yaku::Tanyao);
        }
        if self.pinfu() {
            yakus.push(Yaku::Pinfu);
        }
        if self.iipeikou() {
            yakus.push(Yaku::Iipeikou);
        }
        if self.haku() {
            yakus.push(Yaku::Haku);
        }
        if self.hatsu() {
            debug!("Hatsu validated");
            yakus.push(Yaku::Hatsu);
        }
        if self.chun() {
            yakus.push(Yaku::Chun);
        }
        if self.ba_no_kaze() {
            yakus.push(Yaku::BaNoKaze);
        }
        if self.jibun_no_kaze() {
            yakus.push(Yaku::JibunNoKaze);
        }
        if self.chankan() {
            yakus.push(Yaku::Chankan);
        }
        if self.rinshankaihou() {
            yakus.push(Yaku::RinshanKaihou);
        }
        if self.haiteiraoyue() {
            yakus.push(Yaku::Haiteiraoyue);
        }
        if self.houteiraoyui() {
            yakus.push(Yaku::Houteiraoyui);
        }
        if self.daburii() {
            yakus.push(Yaku::Daburii);
        }
        if self.chiitoitsu() {
            yakus.push(Yaku::Chiitoitsu);
        }
        if self.toitoi() {
            yakus.push(Yaku::Toitoi);
        }
        if self.sanankou() {
            yakus.push(Yaku::SanAnkou);
        }
        if self.sanshokudoukou() {
            yakus.push(Yaku::SanshokuDoukou);
        }
        if self.sanshokudoujun() {
            yakus.push(Yaku::SanshokuDoujun);
        }
        if self.honroutou() {
            yakus.push(Yaku::Honroutou);
        }
        if self.ittsuu() {
            yakus.push(Yaku::Ittsuu);
        }
        if self.chanta() {
            yakus.push(Yaku::Chanta);
        }
        if self.shousangen() {
            yakus.push(Yaku::Shousangen);
        }
        if self.sankantsu() {
            yakus.push(Yaku::Sankantsu);
        }
        if self.honitsu() {
            yakus.push(Yaku::HonItsu);
        }
        if self.junchan() {
            yakus.push(Yaku::Junchan);
        }
        if self.ryanpeikou() {
            yakus.push(Yaku::Ryanpeikou);
        }
        if self.chinitsu() {
            yakus.push(Yaku::ChinItsu);
        }
        // TODO (other yakus)

        yakus
    }

    fn dora_count(&self) -> usize {
        let mut dora = self.agari_te.game.dora();
        if self.riichi() || self.daburii() {
            dora.extend(self.agari_te.game.uradora());
        }

        let mut dora_cnt = 0;
        for hai in self.agari_te.hai_all() {
            if dora.contains(&hai) {
                dora_cnt += 1;
            }
        }
        dora_cnt
    }

    fn han(&self) -> YakuValue {
        let closed = self.closed();
        let dora_cnt = self.dora_count();
        self.yaku()
            .iter()
            .fold(YakuValue::Han(dora_cnt), |acc, yaku| acc + yaku.han(closed))
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
            if let Some(mentsu) = self.mentsu() {
                for m in mentsu {
                    mentsu_fu += match m {
                        Mentsu_::Ankan([hai, _, _, _]) => {
                            if hai.is_jihai_or_1_9() {
                                32
                            } else {
                                16
                            }
                        }
                        Mentsu_::Minkan([hai, _, _, _]) => {
                            if hai.is_jihai_or_1_9() {
                                16
                            } else {
                                8
                            }
                        }
                        Mentsu_::Ankou([hai, _, _]) => {
                            if hai.is_jihai_or_1_9() {
                                8
                            } else {
                                4
                            }
                        }
                        Mentsu_::Minkou([hai, _, _]) => {
                            if hai.is_jihai_or_1_9() {
                                4
                            } else {
                                2
                            }
                        }
                        Mentsu_::Anshun(_) | Mentsu_::Minshun(_) => 0,
                    };
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

    fn mentsu(&self) -> Option<impl Iterator<Item = Mentsu_>> {
        if let WinningCombination::Normal { mentsu, .. } = &self.combination {
            let mut out = vec![];

            let hupai = self.agari_te.agarihai;
            let ron = self.agari_te.method == WinningMethod::Ron;
            for m in mentsu {
                let mentsu_ = if is_kootsu(m) {
                    let minkoo = ron
                        && hupai == m[0]
                        && !mentsu.iter().any(|m| !is_kootsu(m) && m.contains(&hupai));
                    if minkoo {
                        Mentsu_::Minkou(*m)
                    } else {
                        Mentsu_::Ankou(*m)
                    }
                } else {
                    // Note: If we are pedantic,
                    // this mentsu may possible be a minshun. However,
                    // this will not change how points are counted.
                    // So we leave it as is.
                    Mentsu_::Anshun(*m)
                };
                out.push(mentsu_);
            }

            for fuuro in self.agari_te.fuuro {
                let mentsu = match fuuro {
                    Fuuro::Shuntsu { own, taken, .. } => Mentsu_::Minshun([own[0], own[1], *taken]),
                    Fuuro::Kootsu { own, taken, .. } => Mentsu_::Minkou([own[0], own[1], *taken]),
                    Fuuro::Kantsu(KantsuInner::Ankan { own }) => {
                        Mentsu_::Ankan([own[0], own[1], own[2], own[3]])
                    }
                    Fuuro::Kantsu(KantsuInner::DaiMinkan { own, taken, .. }) => {
                        Mentsu_::Minkan([own[0], own[1], own[2], *taken])
                    }
                    Fuuro::Kantsu(KantsuInner::ShouMinkan {
                        own, taken, added, ..
                    }) => Mentsu_::Minkan([own[0], own[1], *taken, *added]),
                };
                out.push(mentsu);
            }
            Some(out.into_iter())
        } else {
            None
        }
    }

    fn menzentsumo(&self) -> bool {
        self.closed() && self.agari_te.method == WinningMethod::Tsumo
    }

    fn riichi(&self) -> bool {
        self.agari_te.game.player_is_riichi(self.agari_te.wind) && !self.daburii()
    }

    fn ippatsu(&self) -> bool {
        if let Some(riichi) = self.agari_te.game.player_riichi(self.agari_te.wind) {
            riichi.ippatsu
        } else {
            false
        }
    }

    fn tanyao(&self) -> bool {
        self.agari_te.hai_all().all(|hai| !hai.is_jihai_or_1_9())
    }

    fn iipeikou(&self) -> bool {
        if self.closed() {
            if let WinningCombination::Normal { mentsu, .. } = &self.combination {
                use std::collections::hash_map::Entry;
                let mut cnt = std::collections::HashMap::new();
                for m in mentsu {
                    if !is_kootsu(m) {
                        match cnt.entry(m) {
                            Entry::Occupied(mut e) => {
                                *e.get_mut() += 1;
                            }
                            Entry::Vacant(e) => {
                                e.insert(1);
                            }
                        }
                    }
                }
                cnt.iter().filter(|(_, n)| **n == 2 || **n == 3).count() == 1
            } else {
                false
            }
        } else {
            false
        }
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

    fn haku(&self) -> bool {
        if let Some(mut mentsu) = self.mentsu() {
            mentsu.any(|m| m.count_as_kootsu_with(|hai| hai.is_haku()))
        } else {
            false
        }
    }

    fn hatsu(&self) -> bool {
        debug!("Check hatsu yaku");
        if let Some(mut mentsu) = self.mentsu() {
            mentsu.any(|m| {
                trace!("Check hatsu on Mentsu {:?}", &m);
                m.count_as_kootsu_with(|hai| hai.is_hatsu())
            })
        } else {
            false
        }
    }

    fn chun(&self) -> bool {
        if let Some(mut mentsu) = self.mentsu() {
            mentsu.any(|m| m.count_as_kootsu_with(|hai| hai.is_chun()))
        } else {
            false
        }
    }

    fn ba_no_kaze(&self) -> bool {
        if let Some(mut mentsu) = self.mentsu() {
            mentsu.any(|m| m.count_as_kootsu_with(|hai| hai.is_fon(self.agari_te.game.wind)))
        } else {
            false
        }
    }

    fn jibun_no_kaze(&self) -> bool {
        if let Some(mut mentsu) = self.mentsu() {
            mentsu.any(|m| m.count_as_kootsu_with(|hai| hai.is_fon(self.agari_te.wind)))
        } else {
            false
        }
    }

    fn chankan(&self) -> bool {
        self.agari_te.chankan
    }

    fn rinshankaihou(&self) -> bool {
        self.agari_te.rinshankaihou
    }

    fn haiteiraoyue(&self) -> bool {
        self.agari_te.method == WinningMethod::Tsumo
            && self.agari_te.game.next_tsumohai_index().is_none()
    }

    fn houteiraoyui(&self) -> bool {
        self.agari_te.method == WinningMethod::Ron
            && self.agari_te.game.next_tsumohai_index().is_none()
    }

    fn daburii(&self) -> bool {
        if let Some(riichi) = self.agari_te.game.player_riichi(self.agari_te.wind) {
            riichi.double
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

    fn toitoi(&self) -> bool {
        if let Some(mut mentsu) = self.mentsu() {
            mentsu.all(|m| Mentsu_::count_as_kootsu(&m))
        } else {
            false
        }
    }

    fn sanankou(&self) -> bool {
        if let Some(mentsu) = self.mentsu() {
            let mut ankou_cnt = 0;
            for m in mentsu {
                match m {
                    Mentsu_::Ankan(_) | Mentsu_::Ankou(_) => ankou_cnt += 1,
                    _ => {}
                }
            }
            ankou_cnt == 3
        } else {
            false
        }
    }

    fn sanshokudoukou(&self) -> bool {
        if let Some(mentsu) = self.mentsu() {
            let mut suuhai_grid = [[false; 3]; 9];
            for m in mentsu {
                if let Some(hai) = m.as_kootsu_hai() {
                    if let Hai::Suu(SuuHai { value, suu, .. }) = hai {
                        suuhai_grid[value as usize - 1][suu as usize] = true;
                    }
                }
            }
            suuhai_grid
                .iter()
                .any(|[wan, pin, sou]| *wan && *pin && *sou)
        } else {
            false
        }
    }

    fn sanshokudoujun(&self) -> bool {
        if let Some(mentsu) = self.mentsu() {
            let mut suuhai_grid = [[false; 3]; 9];
            for m in mentsu {
                match m {
                    Mentsu_::Anshun([hai1, hai2, hai3]) | Mentsu_::Minshun([hai1, hai2, hai3]) => {
                        if let Hai::Suu(SuuHai { value, suu, .. }) = hai1.min(hai2).min(hai3) {
                            suuhai_grid[value as usize - 1][suu as usize] = true;
                        }
                    }
                    _ => {}
                }
            }
            suuhai_grid
                .iter()
                .any(|[wan, pin, sou]| *wan && *pin && *sou)
        } else {
            false
        }
    }

    fn honroutou(&self) -> bool {
        self.agari_te.hai_all().all(Hai::is_jihai_or_1_9)
    }

    fn ittsuu(&self) -> bool {
        if let Some(mentsu) = self.mentsu() {
            let mut suuhai_grid = [[false; 9]; 3];
            for m in mentsu {
                match m {
                    Mentsu_::Anshun([hai1, hai2, hai3]) | Mentsu_::Minshun([hai1, hai2, hai3]) => {
                        if let Hai::Suu(SuuHai { value, suu, .. }) = hai1.min(hai2).min(hai3) {
                            suuhai_grid[suu as usize][value as usize - 1] = true;
                        }
                    }
                    _ => {}
                }
            }
            suuhai_grid
                .iter()
                .any(|[ii, _, _, suu, _, _, chii, _, _]| *ii && *suu && *chii)
        } else {
            false
        }
    }

    fn chanta(&self) -> bool {
        if let WinningCombination::Normal { toitsu, .. } = self.combination {
            if let Some(mut mentsu) = self.mentsu() {
                !self.honroutou()
                    && !self.junchan()
                    && toitsu.iter().all(|h| h.is_jihai_or_1_9())
                    && mentsu.all(|m| Mentsu_::is_extremity(&m))
            } else {
                false
            }
        } else {
            false
        }
    }

    fn shousangen(&self) -> bool {
        if let WinningCombination::Normal { toitsu, .. } = self.combination {
            if let Some(mentsu) = self.mentsu() {
                let sangenpai_cnt = mentsu
                    .filter(|m| m.count_as_kootsu_with(Hai::is_sangen))
                    .count();
                sangenpai_cnt == 2 && toitsu[0].is_sangen()
            } else {
                false
            }
        } else {
            false
        }
    }

    fn sankantsu(&self) -> bool {
        if let Some(mentsu) = self.mentsu() {
            let kan_cnt = mentsu.filter(|m| m.is_kan()).count();
            kan_cnt == 3
        } else {
            false
        }
    }

    fn honitsu(&self) -> bool {
        let mut has_jihai = false;
        let mut suu_found = None;
        for hai in self.agari_te.hai_all() {
            match hai {
                Hai::Ji(_) => has_jihai = true,
                Hai::Suu(SuuHai { suu, .. }) => {
                    if suu_found.is_none() {
                        suu_found = Some(suu);
                    } else if suu_found != Some(suu) {
                        return false;
                    }
                }
            }
        }
        suu_found.is_some() && has_jihai
    }

    fn junchan(&self) -> bool {
        if let WinningCombination::Normal { toitsu, .. } = self.combination {
            if let Some(mut mentsu) = self.mentsu() {
                toitsu.iter().all(|h| h.is_1_9())
                    && mentsu.all(|m| Mentsu_::is_extremity(&m))
                    && self.agari_te.hai_all().all(Hai::is_suuhai)
            } else {
                false
            }
        } else {
            false
        }
    }

    fn ryanpeikou(&self) -> bool {
        if self.closed() {
            if let WinningCombination::Normal { mentsu, .. } = &self.combination {
                use std::collections::hash_map::Entry;
                let mut cnt = std::collections::HashMap::new();
                for m in mentsu {
                    if !is_kootsu(m) {
                        match cnt.entry(m) {
                            Entry::Occupied(mut e) => {
                                *e.get_mut() += 1;
                            }
                            Entry::Vacant(e) => {
                                e.insert(1);
                            }
                        }
                    }
                }
                cnt.iter().filter(|(_, n)| **n >= 2).count() == 2
                    || cnt.iter().filter(|(_, n)| **n == 4).count() == 1
            } else {
                false
            }
        } else {
            false
        }
    }

    fn chinitsu(&self) -> bool {
        let mut suu_found = None;
        for hai in self.agari_te.hai_all() {
            match hai {
                Hai::Suu(SuuHai { suu, .. }) => {
                    if suu_found.is_none() {
                        suu_found = Some(suu);
                    } else if suu_found != Some(suu) {
                        return false;
                    }
                }
                _ => return false,
            }
        }
        suu_found.is_some()
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
        use super::tiles::Values;
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

    #[test]
    fn test_iipeikou() {
        let yaku = yaku_from_str_ron("🀇🀇🀈🀈🀉🀉🀊🀋🀌🀍🀙🀚🀛", "🀍").unwrap();
        assert_eq!(yaku, vec![Yaku::Iipeikou]);
    }

    #[test]
    fn test_iipeikou_triple_ron() {
        let yaku = yaku_from_str_ron("🀇🀇🀇🀈🀈🀈🀉🀉🀍🀍🀙🀚🀛", "🀉").unwrap();
        assert_eq!(yaku, vec![Yaku::Iipeikou]);
    }

    #[test]
    fn test_iipeikou_triple_sanankou_ambiguous_tsumo() {
        let yaku = yaku_from_str_tsumo("🀇🀇🀇🀈🀈🀈🀉🀉🀍🀍🀙🀚🀛", "🀉").unwrap();
        assert_eq!(yaku, vec![Yaku::Menzentsumo, Yaku::SanAnkou]);
    }

    #[test]
    fn test_pinfu_iipeikou() {
        let yaku = yaku_from_str_ron("🀇🀇🀈🀈🀉🀉🀊🀋🀌🀍🀍🀚🀛", "🀙").unwrap();
        assert_eq!(yaku, vec![Yaku::Pinfu, Yaku::Iipeikou]);
    }

    #[test]
    fn test_sanankou_toitoi() {
        let yaku = yaku_from_str_ron("🀇🀇🀇🀈🀈🀈🀉🀉🀉🀍🀍🀙🀙", "🀙").unwrap();
        assert_eq!(yaku, vec![Yaku::Toitoi, Yaku::SanAnkou]);
    }

    #[test]
    fn test_sanshokudoukou() {
        let yaku = yaku_from_str_ron("🀇🀇🀇🀇🀈🀉🀍🀍🀙🀙🀐🀐🀐", "🀙").unwrap();
        assert_eq!(yaku, vec![Yaku::SanshokuDoukou]);
    }

    #[test]
    fn test_sanshokudoujun() {
        let yaku = yaku_from_str_ron("🀇🀈🀉🀙🀙🀚🀛🀐🀑🀒🀐🀑🀒", "🀙").unwrap();
        assert_eq!(
            yaku,
            vec![Yaku::Iipeikou, Yaku::SanshokuDoujun, Yaku::Junchan]
        );
    }

    #[test]
    fn test_sanankou_toitoi_sanshokudoukou_honroutou() {
        let yaku = yaku_from_str_ron("🀇🀇🀇🀏🀏🀏🀙🀙🀐🀐🀐🀅🀅", "🀙").unwrap();
        assert_eq!(
            yaku,
            vec![
                Yaku::Toitoi,
                Yaku::SanAnkou,
                Yaku::SanshokuDoukou,
                Yaku::Honroutou
            ]
        );
    }

    #[test]
    fn test_ittsuu() {
        let yaku = yaku_from_str_ron("🀇🀈🀉🀊🀋🀌🀍🀎🀏🀙🀚🀛🀜", "🀜").unwrap();
        assert_eq!(yaku, vec![Yaku::Ittsuu]);
    }

    #[test]
    fn test_chanta() {
        let yaku = yaku_from_str_ron("🀇🀈🀉🀍🀎🀏🀙🀚🀛🀃🀃🀆🀆", "🀆").unwrap();
        assert_eq!(yaku, vec![Yaku::Haku, Yaku::Chanta]);
    }

    #[test]
    fn test_shousangen() {
        let yaku = yaku_from_str_ron("🀇🀈🀉🀃🀃🀃🀆🀆🀅🀅🀅🀄🀄", "🀆").unwrap();
        assert_eq!(
            yaku,
            vec![
                Yaku::Haku,
                Yaku::Hatsu,
                Yaku::Chanta,
                Yaku::Shousangen,
                Yaku::HonItsu
            ]
        );
    }

    #[test]
    fn test_honitsu() {
        let yaku = yaku_from_str_tsumo("🀇🀇🀇🀈🀉🀊🀋🀌🀍🀎🀏🀄🀄", "🀄").unwrap();
        assert_eq!(
            yaku,
            vec![Yaku::Menzentsumo, Yaku::Chun, Yaku::Ittsuu, Yaku::HonItsu]
        );
    }

    #[test]
    fn test_junchan() {
        let yaku = yaku_from_str_ron("🀏🀏🀐🀐🀐🀖🀗🀘🀙🀚🀛🀡🀡", "🀡").unwrap();
        assert_eq!(yaku, vec![Yaku::Junchan]);
    }

    #[test]
    fn test_ryanpeikou() {
        let yaku = yaku_from_str_ron("🀌🀌🀍🀍🀎🀎🀑🀑🀒🀒🀓🀓🀃", "🀃").unwrap();
        assert_eq!(yaku, vec![Yaku::Ryanpeikou]);
    }

    #[test]
    fn test_ryanpeikou_edge_case() {
        let yaku = yaku_from_str_ron("🀌🀌🀌🀌🀍🀍🀍🀍🀎🀎🀎🀎🀃", "🀃").unwrap();
        assert_eq!(yaku, vec![Yaku::HonItsu, Yaku::Ryanpeikou]);
    }

    #[test]
    fn test_chinitsu() {
        let yaku = yaku_from_str_ron("🀇🀌🀌🀌🀌🀍🀍🀍🀍🀎🀎🀎🀎", "🀇").unwrap();
        assert_eq!(yaku, vec![Yaku::Ryanpeikou, Yaku::ChinItsu]);
    }

    #[test]
    fn test_pinfu_tsumo_fu() {
        let fu = fu_from_str_tsumo("🀇🀇🀈🀈🀉🀉🀊🀋🀌🀍🀍🀚🀛", "🀙").unwrap();
        assert_eq!(fu, 20);
    }

    #[test]
    fn test_sanankou_fu() {
        let fu = fu_from_str_tsumo("🀇🀇🀇🀈🀈🀈🀉🀉🀍🀍🀙🀚🀛", "🀉").unwrap();
        assert_eq!(fu, 40);
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

    fn yaku_from_str_ron(tehai: &str, hupai: &str) -> Result<Vec<Yaku>, ParseHaiError> {
        yaku_from_str(tehai, hupai, WinningMethod::Ron)
    }
    fn yaku_from_str_tsumo(tehai: &str, hupai: &str) -> Result<Vec<Yaku>, ParseHaiError> {
        yaku_from_str(tehai, hupai, WinningMethod::Tsumo)
    }
    fn fu_from_str_ron(tehai: &str, hupai: &str) -> Result<usize, ParseHaiError> {
        fu_from_str(tehai, hupai, WinningMethod::Ron)
    }
    fn fu_from_str_tsumo(tehai: &str, hupai: &str) -> Result<usize, ParseHaiError> {
        fu_from_str(tehai, hupai, WinningMethod::Tsumo)
    }

    fn fu_from_str(
        tehai: &str,
        hupai: &str,
        method: WinningMethod,
    ) -> Result<usize, ParseHaiError> {
        let (_, _, fu) = points_from_str(tehai, hupai, method)?;
        Ok(fu)
    }
    fn yaku_from_str(
        tehai: &str,
        hupai: &str,
        method: WinningMethod,
    ) -> Result<Vec<Yaku>, ParseHaiError> {
        let (yaku, _, _) = points_from_str(tehai, hupai, method)?;
        Ok(yaku)
    }

    fn points_from_str(
        tehai: &str,
        hupai: &str,
        method: WinningMethod,
    ) -> Result<(Vec<Yaku>, YakuValue, usize), ParseHaiError> {
        let player_wind = Fon::Ton;
        let mut game = Game::default();
        {
            let te = game.player_te_mut(player_wind);
            for hai in te_from_string(tehai)? {
                te.hai.insert(hai);
            }
        }
        let te = game.player_te_(player_wind);
        let hupai = te_from_string(hupai)?;
        assert_eq!(hupai.len(), 1);
        let agarihai = hupai[0];

        Ok(AgariTe::from_te(te, &game, agarihai, method, Fon::Ton).points())
    }
}
