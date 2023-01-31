use osu_db::{Mod, Replay};
use rosu_pp::{Beatmap, BeatmapExt};

use self::{
    error_stats::ErrorStatistics,
    frames::HitFrames,
    hit_object_manager::HitObjectManager,
    ruleset::{ClickAction, Ruleset},
};

mod error_stats;
mod frames;
mod hit_object;
mod hit_object_manager;
mod ruleset;

#[cfg_attr(not(feature = "stable"), allow(unused))]
pub fn calculate_ur(map: &Beatmap, replay: &Replay) -> f64 {
    let mods = replay
        .mods
        .without(Mod::DoubleTime)
        .without(Mod::HalfTime)
        .without(Mod::Nightcore)
        .bits();

    let frames = HitFrames::from_replay(replay);
    let hit_objects = map.osu_hitobjects(mods);
    let mut hit_errors = Vec::with_capacity(hit_objects.len());
    let mut manager = HitObjectManager::new(&hit_objects, map, mods);

    for frame in frames.iter() {
        manager.update(frame.time);

        let Some((i, h)) = manager.find_circle_at(frame) else {
            continue;
        };

        match Ruleset::check_click_action(h, i, frame, &manager) {
            ClickAction::Hit => {
                if h.is_normal() {
                    let accuracy = (frame.time - h.start_time()).abs();

                    if accuracy < manager.hit_window_50 {
                        hit_errors.push(frame.time - h.start_time());
                    }

                    manager.hit(i);
                } else if h.is_slider() && !h.is_hit {
                    hit_errors.push(frame.time - h.start_time());
                    manager.hit(i);
                }
            }
            ClickAction::Ignore | ClickAction::Shake => {}
        }
    }

    let stats = ErrorStatistics::new(&hit_errors);

    stats.unstable_rate
}
