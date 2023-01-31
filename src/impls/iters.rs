#![cfg_attr(not(feature = "iters"), allow(unused))]

use std::{collections::HashSet, iter};

use osu_db::{Mod, Replay};
use rosu_pp::{osu::OsuObject, Beatmap, BeatmapExt};

use crate::models::{Buttons, ReplayData};

pub fn calculate_ur(map: &Beatmap, replay: &Replay) -> f64 {
    let mods = replay
        .mods
        .without(Mod::DoubleTime)
        .without(Mod::HalfTime)
        .without(Mod::Nightcore)
        .bits();

    // https://github.com/ppy/osu/blob/bbeb62ea47f9a205a10964ac332cbe157d844a78/osu.Game/Scoring/Legacy/LegacyScoreDecoder.cs#L263-L307
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

    let hit_objects = map.osu_hitobjects(mods);
    let mut used_frames: HashSet<u64> = HashSet::with_capacity(hit_objects.len());

    // the first object has no predecessor
    let hit_errors: Vec<_> = iter::once(None)
        // followed by the hitobjects
        .chain(hit_objects.iter().map(Some))
        // zip each object with its predecessor
        .zip(hit_objects.iter())
        // filter out spinners
        .filter(|(_, h)| !h.is_spinner())
        // for each object, try to find its hit frame
        .scan(false, |prev_hit, (prev, obj)| {
            let latest_hit = match obj.is_slider() {
                false => obj.start_time + hit_window_50,
                true => (obj.start_time + hit_window_50).min(obj.end_time().round()),
            };

            let start_idx = replay_data
                .partition_point(|frame| frame.timestamp < obj.start_time - hit_window_50);
            let end_idx =
                replay_data[start_idx..].partition_point(|frame| frame.timestamp <= latest_hit);
            let frames = &replay_data[..start_idx + end_idx];

            // start with no keys
            let hit_error = iter::once(Buttons::default())
                // followed by frame keys
                .chain(frames.iter().map(|frame| frame.keys))
                // zip keys with successing frame
                .zip(frames)
                // skip frames that are before the object's hit window
                .skip(start_idx)
                // filter out frames that are not hits
                .filter_map(|(prev_frame_keys, frame)| {
                    let in_circle = is_in_circle(frame, obj, radius_sq);
                    let press = frame.keys.is_new_press(prev_frame_keys);

                    let notelock = prev.map_or(false, |prev| {
                        let mut notelock =
                            !*prev_hit && frame.timestamp < prev.start_time + hit_window_50;

                        if prev.is_slider() {
                            let in_prev_circle = is_in_circle(frame, prev, radius_sq);
                            let sliderlock =
                                press && in_prev_circle && frame.timestamp < prev.end_time();
                            notelock |= sliderlock;
                        }

                        notelock
                    });

                    (in_circle && press && !notelock).then_some(frame)
                })
                // take the first frame who's timestamp wasn't used for a previous object
                .find(|frame| used_frames.insert(frame.timestamp.to_bits()))
                .map(|frame| frame.timestamp - obj.start_time);

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

fn is_in_circle(frame: &ReplayData, obj: &OsuObject, radius_sq: f32) -> bool {
    (frame.x - obj.stacked_pos().x) * (frame.x - obj.stacked_pos().x)
        + (frame.y - obj.stacked_pos().y) * (frame.y - obj.stacked_pos().y)
        < radius_sq
}
