#![cfg_attr(not(feature = "custom1"), allow(unused))]

use osu_db::Replay;
use rosu_pp::{Beatmap, BeatmapExt};

use self::{error_stats::ErrorStatistics, frames::HitFrames, hit_object::HitObject};

mod error_stats;
mod frames;
mod hit_object;

pub fn calculate_ur(map: &Beatmap, replay: &Replay) -> f64 {
    let mods = replay.mods.bits() & !(NC | HT);

    let mut od = map.od as f64;
    let mut cs = map.cs as f64;

    if (mods & HR) > 0 {
        od = (od * 1.4).min(10.0);
        cs = (cs * 1.3).min(10.0);
    } else if (mods & EZ) > 0 {
        od = (od / 2.0).max(0.0);
        cs = (cs / 2.0).max(0.0);
    }

    let hw_50 = hit_window_50(od);
    let radius_sq = radius_sq(cs);

    let frames = HitFrames::from_replay(replay);
    let hit_objects = map.osu_hitobjects(mods);
    let mut hit_objects: Vec<_> = hit_objects.iter().map(|h| HitObject::new(h)).collect();
    let mut hit_errors = Vec::with_capacity(hit_objects.len());

    for frame in frames {
        let time_start = frame.time - hw_50;
        let time_end = frame.time + hw_50;

        let start_idx = hit_objects.partition_point(|h| h.start_time() < time_start);

        let end_idx =
            start_idx + hit_objects[start_idx..].partition_point(|h| h.start_time() <= time_end);

        let h_opt = hit_objects[start_idx..end_idx]
            .iter()
            .zip(start_idx..)
            .find(|(h, _)| {
                !h.matched_frame && frame.pos.dist_sq(h.pos()) <= radius_sq && !h.ignore()
            });

        let Some((h, i)) = h_opt else { continue };

        if let Some(prev) = i.checked_sub(1).map(|i| &hit_objects[i]) {
            // notelock
            if !prev.matched_frame && frame.time - hw_50 <= prev.start_time() {
                continue;
            }

            // sliderlock
            if let Some(prev_end_time) = prev.slider_end_time() {
                if h.is_slider() && frame.time < prev_end_time {
                    continue;
                }
            }
        }

        hit_errors.push(frame.time - h.start_time());
        hit_objects[i].matched_frame = true;
    }

    let stats = ErrorStatistics::new(&hit_errors);

    stats.unstable_rate
}

const HIT_WINDOW_MISS: i32 = 400;

const EZ: u32 = 1 << 1;
const HR: u32 = 1 << 4;
const DT: u32 = 1 << 6;
const HT: u32 = 1 << 8;
const NC: u32 = DT + (1 << 9);

fn hit_window_50(od: f64) -> i32 {
    (150.0 + 50.0 * (5.0 - od) / 5.0) as i32
}

fn radius_sq(cs: f64) -> f32 {
    let adjusted_cs = (cs - 5.0) / 5.0;
    let sprite_display_size = (64.0 * (1.0 - 0.7 * adjusted_cs)) as f32;
    const BROKEN_GAMEFIELD_ROUNDING_ALLOWANCE: f32 = 1.00041;
    let radius = sprite_display_size / 2.0 * BROKEN_GAMEFIELD_ROUNDING_ALLOWANCE;

    radius * radius
}
