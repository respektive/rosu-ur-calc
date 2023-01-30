use std::mem;

use osu_db::{Mod, Replay};
use rosu_pp::{
    osu::{OsuObject, OsuObjectKind},
    Beatmap, BeatmapExt,
};

use crate::models::Buttons;

pub fn calculate_ur(map: &Beatmap, replay: &Replay) -> f64 {
    let mods = replay
        .mods
        .without(Mod::DoubleTime)
        .without(Mod::HalfTime)
        .without(Mod::Nightcore)
        .bits();

    let attrs = map.attributes().mods(mods).build();

    let adjusted_cs = ((attrs.cs - 5.0) / 5.0) as f32;
    let sprite_display_size = 512.0 / 8.0 * (1.0 - 0.7 * adjusted_cs);
    const BROKEN_GAMEFIELD_ROUNDING_ALLOWANCE: f32 = 1.00041;
    let radius = sprite_display_size / 2.0 / 1.0 * BROKEN_GAMEFIELD_ROUNDING_ALLOWANCE;
    let radius_sq = radius * radius;

    let hit_window_50 = map_difficulty_range(map.od, 200.0, 150.0, 100.0, mods) as i32 as f64;
    let preempt = map_difficulty_range(map.ar, 1800.0, 1200.0, 450.0, mods) as i32 as f64;

    let frames = parse_hit_frames(replay);
    let hit_objects = map.osu_hitobjects(mods);
    let mut hit_errors = Vec::with_capacity(hit_objects.len());
    let mut hit_objects = HitObjects::new(hit_objects, preempt);

    for &frame in frames.iter() {
        let hit_objects_minimal = hit_objects.minimal(frame.time);

        let h = hit_objects_minimal
            .hit_objects
            .iter()
            .zip(hit_objects_minimal.hits.iter())
            .enumerate()
            .find(|(_, (h, &is_hit))| hit_test(h, is_hit, preempt, hit_window_50, frame, radius_sq))
            .map(|(i, (h, _))| (i, h));

        let Some((i, h)) = h else {
            continue;
        };

        match check_click_action(h, i, &hit_objects_minimal, hit_window_50, frame) {
            ClickAction::Hit => match h.kind {
                OsuObjectKind::Circle => {
                    hit_objects_minimal.hits[i] = true;
                    let accuracy = (frame.time - h.start_time).abs();

                    if accuracy < hit_window_50 {
                        hit_errors.push(frame.time - h.start_time);
                    }
                }
                OsuObjectKind::Slider(_) => {
                    if !hit_objects_minimal.hits[i] {
                        hit_objects_minimal.hits[i] = true;
                        hit_errors.push(frame.time - h.start_time);
                    }
                }
                OsuObjectKind::Spinner { .. } => {}
            },
            ClickAction::Ignore | ClickAction::Shake => {}
        }
    }

    let stats = ErrorStatistics::new(&hit_errors);

    stats.unstable_rate
}

#[derive(Debug, Default)]
#[allow(unused)]
struct ErrorStatistics {
    minus_avg: f64,
    plus_avg: f64,
    minus_max: f64,
    plus_max: f64,
    unstable_rate: f64,
}

impl ErrorStatistics {
    fn new(list: &[f64]) -> Self {
        let mut total = 0.0;
        let mut total_ = 0.0;
        let mut total_all = 0.0;
        let mut count = 0;
        let mut count_ = 0;
        let mut max = 0.0;
        let mut min = f64::MAX;

        for &curr in list {
            if curr > max {
                max = curr;
            }

            if curr < min {
                min = curr;
            }

            total_all += curr;

            if curr >= 0.0 {
                total += curr;
                count += 1;
            } else {
                total_ += curr;
                count_ += 1;
            }
        }

        let avg = total_all / list.len() as f64;
        let mut variance = 0.0;

        for curr in list {
            variance += (curr - avg) * (curr - avg);
        }

        variance /= list.len() as f64;

        Self {
            minus_avg: if count_ == 0 {
                0.0
            } else {
                total_ / count_ as f64
            },
            plus_avg: if count == 0 {
                0.0
            } else {
                total / count as f64
            },
            minus_max: min,
            plus_max: max,
            unstable_rate: variance.sqrt() * 10.0,
        }
    }
}

enum ClickAction {
    Ignore,
    Shake,
    Hit,
}

const HITTABLE_RANGE: f64 = 400.0;

fn check_click_action(
    h: &OsuObject,
    index: usize,
    minimal: &HitObjectsMinimal<'_>,
    hit_window_50: f64,
    frame: HitFrame,
) -> ClickAction {
    if h.is_circle()
        && index > 0
        && minimal.hit_objects[index - 1].stack_height > 0.0
        && !minimal.hits[index - 1]
    {
        return ClickAction::Ignore;
    }

    let mut is_next_circle = true;
    let iter = minimal.hit_objects.iter().zip(minimal.hits.iter());

    for (j, (t, is_hit)) in iter.enumerate() {
        if t.start_time + hit_window_50 <= frame.time || *is_hit {
            continue;
        }

        if t.start_time < h.start_time && index != j {
            is_next_circle = false;
        }

        break;
    }

    if is_next_circle && (h.start_time - frame.time).abs() < HITTABLE_RANGE {
        ClickAction::Hit
    } else {
        ClickAction::Shake
    }
}

fn hit_test(
    h: &OsuObject,
    is_hit: bool,
    preempt: f64,
    hit_window_50: f64,
    frame: HitFrame,
    radius_sq: f32,
) -> bool {
    let matches_time = h.start_time - preempt <= frame.time
        && h.start_time + hit_window_50 >= frame.time
        && !is_hit;

    let pos = h.stacked_pos();

    let matches_pos =
        (frame.x - pos.x) * (frame.x - pos.x) + (frame.y - pos.y) * (frame.y - pos.y) < radius_sq;

    matches_time && matches_pos
}

struct HitObjects {
    hit_objects: Vec<OsuObject>,
    hits: Vec<bool>,
    preempt: f64,
}

impl HitObjects {
    fn new(hit_objects: Vec<OsuObject>, preempt: f64) -> Self {
        // hit_objects.retain(|h| !h.is_spinner());
        let hits = vec![false; hit_objects.len()];

        Self {
            hit_objects,
            hits,
            preempt,
        }
    }

    fn minimal(&mut self, time: f64) -> HitObjectsMinimal<'_> {
        let min_left = time - self.preempt;
        let min_right = time + self.preempt;

        let start_idx = self
            .hit_objects
            .partition_point(|h| h.start_time < min_left);

        let end_idx = self.hit_objects[start_idx..].partition_point(|h| h.start_time < min_right);
        let hit_objects = &self.hit_objects[start_idx..start_idx + end_idx];
        let hits = &mut self.hits[start_idx..start_idx + end_idx];

        HitObjectsMinimal { hit_objects, hits }
    }
}

struct HitObjectsMinimal<'h> {
    hit_objects: &'h [OsuObject],
    hits: &'h mut [bool],
}

fn parse_hit_frames(replay: &Replay) -> Vec<HitFrame> {
    #[derive(Default)]
    struct ScanState {
        time_elapsed: i64,
        prev_keys: Buttons,
    }

    replay
        .replay_data
        .as_ref()
        .unwrap()
        .iter()
        .enumerate()
        .filter(|(_, action)| action.delta != -12345)
        .scan(ScanState::default(), |state, (i, action)| {
            state.time_elapsed += action.delta;

            let skip = i < 2
                && (action.x - 256.0).abs() <= f32::EPSILON
                && (action.y - 500.0).abs() <= f32::EPSILON;

            let keys = Buttons::from_f32(action.z);
            let prev_keys = mem::replace(&mut state.prev_keys, keys);
            let new_press = keys.is_new_press(prev_keys);

            let frame = (action.delta >= 0 && new_press && !skip).then_some(HitFrame {
                time: state.time_elapsed as f64,
                x: action.x,
                y: action.y,
            });

            Some(frame)
        })
        .flatten()
        .collect()
}

#[derive(Copy, Clone, Debug)]
struct HitFrame {
    time: f64,
    x: f32,
    y: f32,
}

const EZ: u32 = 2;
const HR: u32 = 16;

fn map_difficulty_range(difficulty: f32, min: f32, mid: f32, max: f32, mods: u32) -> f32 {
    let difficulty = if (mods & HR) > 0 {
        (difficulty * 1.4).min(10.0)
    } else if (mods & EZ) > 0 {
        (difficulty / 2.0).max(0.0)
    } else {
        difficulty
    };

    if difficulty > 5.0 {
        mid + (max - mid) * (difficulty - 5.0) / 5.0
    } else if difficulty < 5.0 {
        mid - (mid - min) * (5.0 - difficulty) / 5.0
    } else {
        mid
    }
}
