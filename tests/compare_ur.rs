use osu_db::Replay;
use rosu_pp::Beatmap;
use rosu_ur_calc::calculate_ur;

fn compare_ur(map_file: &str, replay_file: &str, expected: f64) {
    let map_path = format!("./test-data/maps/{map_file}.osu");
    let replay_path = format!("./test-data/replays/{replay_file}.osr");

    let map = Beatmap::from_path(map_path).expect("failed to parse map");
    let replay = Replay::from_file(replay_path).expect("failed to parse replay");

    let ur = (100.0 * calculate_ur(&map, &replay)).round() / 100.0;

    assert!(
        (ur - expected).abs() <= f64::EPSILON,
        "expected {expected}, got {ur}",
    );
}

#[test]
#[ignore]
fn respektive_unforgiving() {
    compare_ur(
        "Within Temptation - The Unforgiving (Armin) [Marathon]",
        "respektive_unforgiving",
        130.37,
    )
}

#[test]
#[ignore]
fn gn_unforgiving() {
    compare_ur(
        "Within Temptation - The Unforgiving (Armin) [Marathon]",
        "gn_unforgiving",
        135.48,
    )
}

#[test]
fn respektive_haitai() {
    compare_ur(
        "Ayase Rie - Yuima-ruWorld TVver. (Fycho) [Extra]",
        "respektive_haitai",
        87.15,
    )
}

#[test]
fn respektive_high_powered() {
    compare_ur(
        "sphere - HIGH POWERED (TV Size) (Azunyan-) [POWER OVERLOAD EXPERT]",
        "respektive_high_powered",
        90.80,
    )
}

#[test]
fn respektive_whos_world() {
    compare_ur(
        "Euchaeta - Who's World (P_O) [Who Does This World Belong To]",
        "respektive_whos_world",
        96.14,
    )
}

#[test]
fn wolf_gitaroo() {
    compare_ur(
        "Gitaroo Man - Soft Machine (Ash) [Master Mode]",
        "wolf_gitaroo",
        222.60,
    )
}

#[test]
fn mrekk_demetori() {
    compare_ur(
            "Demetori - Shinkou wa Hakanaki Ningen no Tame ni ~ Jehovah's YaHVeH (Camo) [Camo & Winter's Extra Stage]",
            "mrekk_demetori",
            78.57,
        )
}

#[test]
fn gn_barusa() {
    compare_ur(
        "Nico Nico Douga - BARUSA of MIKOSU (DJPop) [TAG4]",
        "gn_barusa",
        204.31,
    )
}

#[test]
fn ekoro_barusa() {
    compare_ur(
        "Nico Nico Douga - BARUSA of MIKOSU (DJPop) [TAG4]",
        "ekoro_barusa",
        115.33,
    )
}

#[test]
fn peachick_rog() {
    compare_ur(
        "07th Expansion - rog-unlimitation (AngelHoney) [AngelHoney]",
        "peachick_rog",
        93.60,
    )
}

#[test]
fn mismagius_usatei() {
    compare_ur(
        "IOSYS - Usatei (Card N'FoRcE) [RUN!!]",
        "mismagius_usatei",
        218.45,
    )
}

#[test]
fn badeu_mayday() {
    compare_ur(
        "TheFatRat - Mayday (feat. Laura Brehm) (Voltaeyx) [[2B] Calling Out Mayday]",
        "badeu_mayday",
        241.13,
    )
}

#[test]
fn rohulk_sanctus() {
    compare_ur(
        "Feryquitous - Central Nucleus (Shiirn) [Sanctus Nexum]",
        "rohulk_sanctus",
        73.51,
    )
}

#[test]
fn gn_strange() {
    compare_ur(
        "DJ Sharpnel - StrangeProgram (happy30) [Lesjuh's TAG]",
        "gn_strange",
        150.93,
    )
}

#[test]
fn whitecat_flamewall() {
    compare_ur(
        "Camellia - Flamewall (Sotarks) [ETERNAL SACRED FIRE]",
        "whitecat_flamewall",
        112.92,
    )
}

#[test]
fn respektive_sink() {
    compare_ur(
        "Chroma - sink to the deep sea world (None1637) [AR10]",
        "respektive_sink",
        252.50,
    )
}
