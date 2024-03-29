use osu_db::Replay;
use rosu_pp::{osu::OsuObjectKind, Beatmap, BeatmapExt};

use self::{error_stats::ErrorStatistics, frames::HitFrames};

mod error_stats;
mod frames;

#[cfg_attr(not(feature = "custom2"), allow(unused))]
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

    let hit_objects = map.osu_hitobjects(mods);
    let mut hit_errors = Vec::with_capacity(hit_objects.len());

    let frames = HitFrames::from_replay(replay);
    let mut frames = frames.as_slice();

    for i in 0..hit_objects.len() {
        if hit_objects[i].is_spinner() {
            continue;
        }

        let prev_end_time = i
            // not the first note
            .checked_sub(1)
            // previous note was a slider
            .and_then(|k| match hit_objects[k].kind {
                OsuObjectKind::Slider(ref slider) => Some(slider.end_time as i32),
                OsuObjectKind::Circle | OsuObjectKind::Spinner { .. } => None,
            })
            // current note is a slider
            .filter(|_| hit_objects[i].is_slider());

        let start_time = hit_objects[i].start_time as i32;
        let pos = hit_objects[i].stacked_pos();

        let time_start = start_time - hw_50;
        let time_end = start_time + hw_50;

        let start_idx = frames.partition_point(|frame| frame.time < time_start);
        let end_idx =
            start_idx + frames[start_idx..].partition_point(|frame| frame.time < time_end);

        let frame_opt = frames[start_idx..end_idx]
            .iter()
            .zip(start_idx..)
            .find(|(frame, _)| {
                frame.pos.dist_sq(pos) <= radius_sq
                    && prev_end_time.map_or(true, |prev_end_time| prev_end_time < frame.time)
            });

        let next_start = match frame_opt {
            Some((frame, j)) => {
                hit_errors.push(frame.time - start_time);

                j + 1
            }
            None => end_idx,
        };

        // simulate notelock by ignoring frames
        frames = &frames[next_start..];
    }

    let stats = ErrorStatistics::new(&hit_errors);

    stats.unstable_rate
}

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
