use crate::yaku::YakuValue;

const KO_MANGAN: isize = 8000;
const OYA_MANGAN: isize = KO_MANGAN * 3 / 2;
const KO_HANEMAN: isize = 12000;
const OYA_HANEMAN: isize = KO_HANEMAN * 3 / 2;
const KO_BAIMAN: isize = 16000;
const OYA_BAIMAN: isize = KO_BAIMAN * 3 / 2;
const KO_SANBAIMAN: isize = 24000;
const OYA_SANBAIMAN: isize = KO_SANBAIMAN * 3 / 2;
const KO_YAKUMAN: isize = 32000;
const OYA_YAKUMAN: isize = KO_YAKUMAN * 3 / 2;

pub fn points_ron_oya(han: YakuValue, fu: usize) -> isize {
    match han {
        YakuValue::Han(1) => match fu {
            30 => 1500,
            40 => 2000,
            50 => 2400,
            60 => 2900,
            70 => 3400,
            80 => 3900,
            90 => 4400,
            100 => 4800,
            110 => 5300,
            _ => unreachable!("Impossible fu value for {:?}: {fu}", han),
        },
        YakuValue::Han(2) => match fu {
            25 => 2400,
            30 => 2900,
            40 => 3900,
            50 => 4800,
            60 => 5800,
            70 => 6800,
            80 => 7700,
            90 => 8700,
            100 => 9600,
            110 => 10600,
            _ => unreachable!("Impossible fu value for {:?}: {fu}", han),
        },
        YakuValue::Han(3) => match fu {
            25 => 4800,
            30 => 5800,
            40 => 7700,
            50 => 9600,
            60 => 11600,
            61.. => OYA_MANGAN,
            _ => unreachable!("Impossible fu value for {:?}: {fu}", han),
        },
        YakuValue::Han(4) => match fu {
            25 => 9600,
            30 => 11600,
            31.. => OYA_MANGAN,
            _ => unreachable!("Impossible fu value for {:?}: {fu}", han),
        },
        YakuValue::Han(han) => match han {
            5 => OYA_MANGAN,
            6 | 7 => OYA_HANEMAN,
            8 | 9 | 10 => OYA_BAIMAN,
            11 | 12 => OYA_SANBAIMAN,
            13.. => OYA_YAKUMAN,
            _ => unreachable!("Impossible han value: {han}"),
        },
        YakuValue::Yakuman(yakuman) => yakuman as isize * OYA_YAKUMAN,
    }
}

pub fn points_ron_ko(han: YakuValue, fu: usize) -> isize {
    match han {
        YakuValue::Han(1) => match fu {
            30 => 1000,
            40 => 1300,
            50 => 1600,
            60 => 2000,
            70 => 2300,
            80 => 2600,
            90 => 2900,
            100 => 3200,
            110 => 3600,
            _ => unreachable!("Impossible fu value for {:?}: {fu}", han),
        },
        YakuValue::Han(2) => match fu {
            25 => 1600,
            30 => 2000,
            40 => 2600,
            50 => 3200,
            60 => 3900,
            70 => 4500,
            80 => 5200,
            90 => 5800,
            100 => 6400,
            110 => 7100,
            _ => unreachable!("Impossible fu value for {:?}: {fu}", han),
        },
        YakuValue::Han(3) => match fu {
            25 => 3200,
            30 => 3900,
            40 => 5200,
            50 => 6400,
            60 => 7700,
            61.. => KO_MANGAN,
            _ => unreachable!("Impossible fu value for {:?}: {fu}", han),
        },
        YakuValue::Han(4) => match fu {
            25 => 6400,
            30 => 7700,
            31.. => KO_MANGAN,
            _ => unreachable!("Impossible fu value for {:?}: {fu}", han),
        },
        YakuValue::Han(han) => match han {
            5 => KO_MANGAN,
            6 | 7 => KO_HANEMAN,
            8 | 9 | 10 => KO_BAIMAN,
            11 | 12 => KO_SANBAIMAN,
            13.. => KO_YAKUMAN,
            _ => unreachable!("Impossible han value: {han}"),
        },
        YakuValue::Yakuman(yakuman) => yakuman as isize * KO_YAKUMAN,
    }
}

pub fn points_tsumo_oya(han: YakuValue, fu: usize) -> isize {
    match han {
        YakuValue::Han(1) => match fu {
            30 => 500,
            40 => 700,
            50 => 800,
            60 => 1000,
            70 => 1200,
            80 => 1300,
            90 => 1500,
            100 => 1600,
            110 => 1800,
            _ => unreachable!("Impossible fu value for {:?}: {fu}", han),
        },
        YakuValue::Han(2) => match fu {
            20 => 700,
            30 => 1000,
            40 => 1300,
            50 => 1600,
            60 => 2000,
            70 => 2300,
            80 => 2600,
            90 => 2900,
            100 => 3200,
            110 => 3600,
            _ => unreachable!("Impossible fu value for {:?}: {fu}", han),
        },
        YakuValue::Han(3) => match fu {
            20 => 1300,
            25 => 1600,
            30 => 2000,
            40 => 2600,
            50 => 3200,
            60 => 3900,
            61.. => OYA_MANGAN / 3,
            _ => unreachable!("Impossible fu value for {:?}: {fu}", han),
        },
        YakuValue::Han(4) => match fu {
            20 => 2600,
            25 => 3200,
            30 => 3900,
            31.. => OYA_MANGAN / 3,
            _ => unreachable!("Impossible fu value for {:?}: {fu}", han),
        },
        YakuValue::Han(han) => match han {
            5 => OYA_MANGAN / 3,
            6 | 7 => OYA_HANEMAN / 3,
            8 | 9 | 10 => OYA_BAIMAN / 3,
            11 | 12 => OYA_SANBAIMAN / 3,
            13.. => OYA_YAKUMAN / 3,
            _ => unreachable!("Impossible han value: {han}"),
        },
        YakuValue::Yakuman(yakuman) => yakuman as isize * OYA_YAKUMAN / 3,
    }
}
pub fn points_tsumo_ko(han: YakuValue, fu: usize) -> (isize, isize) {
    let oya = points_tsumo_oya(han, fu);
    let ko = match han {
        YakuValue::Han(1) => match fu {
            30 => 300,
            40 | 50 => 400,
            60 => 500,
            70 => 600,
            80 => 700,
            90 | 100 => 800,
            110 => 900,
            _ => unreachable!("Impossible fu value for {:?}: {fu}", han),
        },
        YakuValue::Han(2) => match fu {
            20 => 400,
            30 => 500,
            40 => 700,
            50 => 800,
            60 => 1000,
            70 => 1200,
            80 => 1300,
            90 => 1500,
            100 => 1600,
            110 => 1800,
            _ => unreachable!("Impossible fu value for {:?}: {fu}", han),
        },
        YakuValue::Han(3) => match fu {
            20 => 700,
            25 => 800,
            30 => 1000,
            40 => 1300,
            50 => 1600,
            60 => 2000,
            61.. => OYA_MANGAN / 6,
            _ => unreachable!("Impossible fu value for {:?}: {fu}", han),
        },
        YakuValue::Han(4) => match fu {
            20 => 1300,
            25 => 1600,
            30 => 2000,
            31.. => OYA_MANGAN / 6,
            _ => unreachable!("Impossible fu value for {:?}: {fu}", han),
        },
        YakuValue::Han(han) => match han {
            5 => OYA_MANGAN / 6,
            6 | 7 => OYA_HANEMAN / 6,
            8 | 9 | 10 => OYA_BAIMAN / 6,
            11 | 12 => OYA_SANBAIMAN / 6,
            13.. => OYA_YAKUMAN / 6,
            _ => unreachable!("Impossible han value: {han}"),
        },
        YakuValue::Yakuman(yakuman) => yakuman as isize * OYA_YAKUMAN / 6,
    };
    (oya, ko)
}
