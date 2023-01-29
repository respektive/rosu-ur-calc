use std::{collections::HashSet, iter};

use osu_db::{Mod, Replay};
use rosu_pp::{Beatmap, BeatmapExt};

use self::models::{Buttons, ReplayData};

mod models;

pub fn calculate_ur_baseline(map: &Beatmap, replay: &Replay) -> f64 {
    let mods = replay
        .mods
        .without(Mod::DoubleTime)
        .without(Mod::HalfTime)
        .without(Mod::Nightcore)
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
                false => Buttons::default(),
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

pub fn calculate_ur(map: &Beatmap, replay: &Replay) -> f64 {
    let mods = replay
        .mods
        .without(Mod::DoubleTime)
        .without(Mod::HalfTime)
        .without(Mod::Nightcore)
        .bits();

    // https://github.com/ppy/osu/blob/master/osu.Game/Scoring/Legacy/LegacyScoreDecoder.cs#L263-L307
    let replay_data: Vec<_> = replay
        .replay_data
        .as_ref()
        .unwrap()
        .iter()
        .enumerate()
        .filter(|(_, action)| action.delta != -12345)
        .scan(0, |time_elapsed, (i, action)| {
            *time_elapsed += action.delta;

            let skip = i < 2
                && (action.x - 256.0).abs() <= f32::EPSILON
                && (action.y - 500.0).abs() <= f32::EPSILON;

            let frame = (!skip && action.delta >= 0).then_some(ReplayData {
                timestamp: *time_elapsed as f64,
                x: action.x,
                y: action.y,
                keys: Buttons::from_f32(action.z),
            });

            Some(frame)
        })
        .flatten()
        .collect();

    let attrs = map.attributes().mods(mods).build();

    //let radius = 64.0 * (1.0 - 7.0 * (attrs.cs as f32 - 5.0) / 5.0) / 2.0;
    //let scale = (1.0 - 7.0 * (attrs.cs as f32 - 5.0) / 5.0) / 2.0;
    let radius = 23.05 - (attrs.cs as f32 - 7.0) * 4.4825;
    let radius_sq = radius * radius;
    let hit_window_50 = 199.5 - attrs.od * 10.0;
    //let hit_window_50 = (150.0 + 50.0 * (5.0 - attrs.od) / 5.0) - 0.5;

    let mut used_frames: HashSet<u64> = HashSet::new();

    let hit_objects = map.osu_hitobjects(mods);

    let hit_errors: Vec<_> = iter::once(None)
        .chain(hit_objects.iter().map(Some))
        .zip(hit_objects.iter())
        .filter(|(_, h)| !h.is_spinner())
        .scan(false, |prev_hit, (prev, obj)| {
            let latest_hit = match obj.is_slider() {
                false => obj.start_time + hit_window_50,
                true => (obj.start_time + hit_window_50).min(obj.end_time().round()),
            };

            // cannot do the same for start_idx since the previous keys are required
            // which come before the start_idx timestamp
            let end_idx = replay_data.partition_point(|frame| frame.timestamp <= latest_hit);
            let frames = &replay_data[..end_idx];

            let hit_error = iter::once(Buttons::default()) // start with no keys
                .chain(frames.iter().map(|frame| frame.keys)) // followed by frame keys
                .zip(frames) // zip keys with successing frame
                .skip_while(|(_, frame)| frame.timestamp < obj.start_time - hit_window_50)
                .filter(|(_, frame)| !used_frames.contains(&frame.timestamp.to_bits()))
                .find_map(|(prev_frame_keys, frame)| {
                    // calculate in_circle, press, and notelock
                    let in_circle = (frame.x - obj.stacked_pos().x)
                        * (frame.x - obj.stacked_pos().x)
                        + (frame.y - obj.stacked_pos().y) * (frame.y - obj.stacked_pos().y)
                        < radius_sq;

                    let m1 = frame.keys.m1() && !prev_frame_keys.m1();
                    let m2 = frame.keys.m2() && !prev_frame_keys.m2();
                    let k1 = frame.keys.k1() && !prev_frame_keys.k1();
                    let k2 = frame.keys.k2() && !prev_frame_keys.k2();
                    let press = m1 || m2 || k1 || k2;

                    let notelock = prev.map_or(false, |prev| {
                        let mut notelock =
                            !*prev_hit && frame.timestamp < prev.start_time + hit_window_50;

                        if prev.is_slider() {
                            let in_prev_cirle = (frame.x - prev.stacked_pos().x)
                                * (frame.x - prev.stacked_pos().x)
                                + (frame.y - prev.stacked_pos().y)
                                    * (frame.y - prev.stacked_pos().y)
                                < radius_sq;
                            let sliderlock =
                                press && in_prev_cirle && frame.timestamp < prev.end_time();
                            notelock |= sliderlock;
                        }

                        notelock
                    });

                    (in_circle && press && !notelock).then_some(frame.timestamp)
                })
                .map(|frame_timestamp| {
                    used_frames.insert(frame_timestamp.to_bits());

                    frame_timestamp - obj.start_time
                });

            *prev_hit = hit_error.is_some();

            Some(hit_error)
        })
        .flatten()
        .collect();

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
