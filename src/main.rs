use std::env;

use osu_db::Replay;
use rosu_pp::Beatmap;
use rosu_ur_calc::calculate_ur;

fn main() {
    let args: Vec<String> = env::args().collect();
    let map_path = args[1].to_string();
    let replay_path = args[2].to_string();

    let map = Beatmap::from_path(map_path).unwrap();
    let replay = Replay::from_file(replay_path).unwrap();

    let unstable_rate = calculate_ur(&map, &replay);

    println!("UR: {unstable_rate:#?}");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compare_ur(map_file: &str, replay_file: &str, expected: f64) {
        let map_path = format!("./test-data/{map_file}.osu");
        let replay_path = format!("./test-data/{replay_file}.osr");

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
            "replay-osu_156352_3460700148",
            130.37,
        )
    }

    #[test]
    #[ignore]
    fn gn_unforgiving() {
        compare_ur(
            "Within Temptation - The Unforgiving (Armin) [Marathon]",
            "replay-osu_156352_3549163348",
            135.48,
        )
    }

    #[test]
    fn respektive_haitai() {
        compare_ur(
            "Ayase Rie - Yuima-ruWorld TVver. (Fycho) [Extra]",
            "replay-osu_983680_2294262584",
            87.15,
        )
    }

    #[test]
    fn respektive_high_powered() {
        compare_ur(
            "sphere - HIGH POWERED (TV Size) (Azunyan-) [POWER OVERLOAD EXPERT]",
            "replay-osu_2779503_3916842208",
            90.80,
        )
    }

    #[test]
    fn respektive_whos_world() {
        compare_ur(
            "Euchaeta - Who's World (P_O) [Who Does This World Belong To]",
            "replay-osu_3312004_4205640222",
            96.14,
        )
    }

    #[test]
    fn wolf_gitaroo() {
        compare_ur(
            "Gitaroo Man - Soft Machine (Ash) [Master Mode]",
            "replay-osu_21724_5259762",
            222.60,
        )
    }

    #[test]
    fn mrekk_demetori() {
        compare_ur(
            "Demetori - Shinkou wa Hakanaki Ningen no Tame ni ~ Jehovah's YaHVeH (Camo) [Camo & Winter's Extra Stage]",
            "replay-osu_3747453_4300226983",
            78.57,
        )
    }

    #[test]
    fn gn_barusa() {
        compare_ur(
            "Nico Nico Douga - BARUSA of MIKOSU (DJPop) [TAG4]",
            "replay-osu_24722_3095061139",
            204.31,
        )
    }

    #[test]
    fn ekoro_barusa() {
        compare_ur(
            "Nico Nico Douga - BARUSA of MIKOSU (DJPop) [TAG4]",
            "replay-osu_24722_4355141481",
            115.33,
        )
    }

    #[test]
    fn peachick_rog() {
        compare_ur(
            "07th Expansion - rog-unlimitation (AngelHoney) [AngelHoney]",
            "replay-osu_116128_1110865601",
            93.60,
        )
    }

    #[test]
    fn mimagius_usatei() {
        compare_ur(
            "IOSYS - Usatei (Card N'FoRcE) [RUN!!]",
            "replay-osu_22993_4031511629",
            218.45,
        )
    }

    #[test]
    fn badeu_mayday() {
        compare_ur(
            "TheFatRat - Mayday (feat. Laura Brehm) (Voltaeyx) [[2B] Calling Out Mayday]",
            "replay-osu_1605148_2793599598",
            241.13,
        )
    }

    #[test]
    fn rohulk_sanctus() {
        compare_ur(
            "Feryquitous - Central Nucleus (Shiirn) [Sanctus Nexum]",
            "replay-osu_1402167_2690174416",
            73.51,
        )
    }
}
