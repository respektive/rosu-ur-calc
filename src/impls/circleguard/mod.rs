use osu_db::Replay;
use rosu_pp::{osu::OsuObjectKind, Beatmap, BeatmapExt};

use self::{error_stats::ErrorStatistics, frames::HitFrames};

mod error_stats;
mod frames;

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
    let mut hit_errors = Vec::with_capacity(hit_objects.len());

    let mut hitobj_i = 0;
    let mut frame_i = 0;

    let sliderbug_fixed = true; // TODO

    while hitobj_i < hit_objects.len() && frame_i < frames.len() {
        let hitobj = &hit_objects[hitobj_i];
        let frame = frames[frame_i];

        let hitobj_t = hitobj.start_time as i32;

        let hitobj_end_time = match hitobj.kind {
            OsuObjectKind::Circle => hitobj_t + hw_50,
            OsuObjectKind::Slider(ref slider) => slider.end_time as i32 + hw_50,
            OsuObjectKind::Spinner { end_time } => end_time as i32,
        };

        let notelock_end_time = if !sliderbug_fixed {
            let mut notelock_end_time = hitobj_t + hw_50;

            if !hitobj.is_circle() {
                notelock_end_time = notelock_end_time.min(hitobj_end_time);
            }

            notelock_end_time
        } else {
            let mut notelock_end_time = hitobj_end_time;

            if hitobj.is_circle() {
                notelock_end_time += 1;
            }

            notelock_end_time
        };

        if frame.time < hitobj_t - HIT_WINDOW_MISS {
            frame_i += 1;

            continue;
        }

        if frame.time <= hitobj_t - hw_50 {
            if frame.pos.dist_sq(hitobj.pos) <= radius_sq && !hitobj.is_spinner() {
                if hitobj.is_slider() && sliderbug_fixed {
                    while frames[frame_i].time < notelock_end_time {
                        frame_i += 1;

                        if frame_i >= frames.len() {
                            break;
                        }
                    }
                } else {
                    frame_i += 1;
                }

                hitobj_i += 1;
            } else {
                frame_i += 1;
            }
        } else if frame.time >= notelock_end_time {
            hitobj_i += 1;
        } else if frame.time < hitobj_t + hw_50
            && frame.pos.dist_sq(hitobj.pos) <= radius_sq
            && !hitobj.is_spinner()
        {
            hit_errors.push(frame.time - hitobj_t);

            if hitobj.is_slider() && sliderbug_fixed {
                while frames[frame_i].time < notelock_end_time {
                    frame_i += 1;

                    if frame_i >= frames.len() {
                        break;
                    }
                }
            } else {
                frame_i += 1;
            }

            hitobj_i += 1;
        } else {
            frame_i += 1;
        }
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
