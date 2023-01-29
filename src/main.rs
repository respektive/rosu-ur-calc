use osu_db::Replay;
use rosu_pp::{Beatmap, BeatmapExt};
use std::{collections::HashSet, env};

fn main() {
    let args: Vec<String> = env::args().collect();
    let map_path = args[1].to_string();
    let replay_path = args[2].to_string();

    let map = Beatmap::from_path(map_path).unwrap();
    let replay = Replay::from_file(replay_path).unwrap();

    let unstable_rate = calculate_ur(&map, &replay);

    println!("UR: {unstable_rate:#?}");
}

fn calculate_ur(map: &Beatmap, replay: &Replay) -> f64 {
    let mods = replay
        .mods
        .without(osu_db::Mod::DoubleTime)
        .without(osu_db::Mod::HalfTime)
        .without(osu_db::Mod::Nightcore)
        .bits();

    let replay_data: Vec<_> = replay
        .replay_data
        .as_ref()
        .unwrap()
        .iter()
        .scan(0, |time_elapsed, action| {
            *time_elapsed += action.delta;

            Some(ReplayData {
                timestamp: *time_elapsed as f64,
                x: action.x,
                y: action.y,
                keys: Buttons::from_f32(action.z),
            })
        })
        .collect();

    let attrs = map.attributes().mods(mods).build();

    //let radius = 64.0 * (1.0 - 7.0 * (attrs.cs as f32 - 5.0) / 5.0) / 2.0;
    //let scale = (1.0 - 7.0 * (attrs.cs as f32 - 5.0) / 5.0) / 2.0;
    let radius = 23.05 - (attrs.cs as f32 - 7.0) * 4.4825;
    let hit_window_50 = 199.5 - attrs.od * 10.0;
    //let hit_window_50 = (150.0 + 50.0 * (5.0 - attrs.od) / 5.0) - 0.5;

    let mut hit_errors: Vec<f64> = Vec::new();
    let mut used_frames: HashSet<u64> = HashSet::new();
    let mut prev_hit = true;

    let hit_objects = map.osu_hitobjects(mods);
    for (i, obj) in hit_objects.iter().enumerate() {
        if obj.is_spinner() {
            continue;
        }

        let mut hit = false;
        for (j, frame) in replay_data.iter().enumerate() {
            let prev_frame_keys: Buttons = match j > 0 {
                false => Buttons(0),
                true => replay_data[j - 1].keys,
            };
            let latest_hit = match obj.is_slider() {
                false => obj.start_time + hit_window_50,
                true => (obj.start_time + hit_window_50).min(obj.end_time().round()),
            };

            if frame.timestamp < obj.start_time - hit_window_50
                || used_frames.contains(&frame.timestamp.to_bits())
            {
                continue;
            } else if frame.timestamp > latest_hit {
                break;
            }

            let in_circle = (frame.x - obj.stacked_pos().x) * (frame.x - obj.stacked_pos().x)
                + (frame.y - obj.stacked_pos().y) * (frame.y - obj.stacked_pos().y)
                < (radius * radius);

            let m1 = frame.keys.m1() && !prev_frame_keys.m1();
            let m2 = frame.keys.m2() && !prev_frame_keys.m2();
            let k1 = frame.keys.k1() && !prev_frame_keys.k1();
            let k2 = frame.keys.k2() && !prev_frame_keys.k2();
            let press = m1 || m2 || k1 || k2;

            let mut notelock = false;
            if i > 0 {
                notelock =
                    !prev_hit && frame.timestamp < hit_objects[i - 1].start_time + hit_window_50;

                if hit_objects[i - 1].is_slider() {
                    let in_prev_cirle = (frame.x - hit_objects[i - 1].stacked_pos().x)
                        * (frame.x - hit_objects[i - 1].stacked_pos().x)
                        + (frame.y - hit_objects[i - 1].stacked_pos().y)
                            * (frame.y - hit_objects[i - 1].stacked_pos().y)
                        < (radius * radius);
                    let sliderlock =
                        press && in_prev_cirle && frame.timestamp < hit_objects[i - 1].end_time();
                    notelock = notelock || sliderlock;
                }
            }

            if in_circle && press && !notelock {
                hit_errors.push(frame.timestamp - obj.start_time);
                used_frames.insert(frame.timestamp.to_bits());
                hit = true;
                break;
            }
        }
        prev_hit = hit;
    }

    let sum: f64 = hit_errors.iter().sum();
    let len = hit_errors.len() as f64;
    let avg = sum / len;

    let mut variance: f64 = 0.0;
    for hit in hit_errors {
        variance += (hit - avg) * (hit - avg);
    }
    variance /= len;

    variance.sqrt() * 10.0
}

#[derive(Debug)]
struct ReplayData {
    timestamp: f64,
    x: f32,
    y: f32,
    keys: Buttons,
}

#[derive(Debug, Copy, Clone)]
struct Buttons(u8);

impl Buttons {
    const M1: u8 = 1 << 0;
    const M2: u8 = 1 << 1;
    const K1: u8 = 1 << 2;
    const K2: u8 = 1 << 3;

    fn from_f32(float: f32) -> Self {
        let mut bits = float as u8;

        if (bits & Self::K1) > 0 {
            assert!((bits & Self::M1) > 0);
            bits -= Self::M1;
        }

        if (bits & Self::K2) > 0 {
            assert!((bits & Self::M2) > 0);
            bits -= Self::M2;
        }

        Self(bits)
    }

    fn m1(self) -> bool {
        (self.0 & Self::M1) > 0
    }

    fn m2(self) -> bool {
        (self.0 & Self::M2) > 0
    }

    fn k1(self) -> bool {
        (self.0 & Self::K1) > 0
    }

    fn k2(self) -> bool {
        (self.0 & Self::K2) > 0
    }
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
    fn respektive_unforgiving() {
        compare_ur(
            "Within Temptation - The Unforgiving (Armin) [Marathon]",
            "replay-osu_156352_3460700148",
            130.37,
        )
    }

    #[test]
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
}
