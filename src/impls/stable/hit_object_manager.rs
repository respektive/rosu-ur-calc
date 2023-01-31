use rosu_pp::Beatmap;

use super::{
    frames::HitFrame,
    hit_object::{HitObject, HitObjectExt},
};

pub struct HitObjectManager<'h> {
    pub hit_window_50: i32,
    pub preempt: i32,
    pub radius_sq: f32,
    minimal_start: usize,
    minimal_end: usize,
    hit_objects: Vec<HitObject<'h>>,
}

impl HitObjectManager<'_> {
    pub const HITTABLE_RANGE: i32 = 400;
    pub const FADE_IN: i32 = 400;
    pub const FADE_OUT: i32 = 240;
}

impl<'h> HitObjectManager<'h> {
    pub fn new<O: HitObjectExt>(hit_objects: &'h [O], map: &Beatmap, mods: u32) -> Self {
        let attrs = map.attributes().mods(mods).build();

        let adjusted_cs = ((attrs.cs - 5.0) / 5.0) as f32;
        let sprite_display_size = 512.0 / 8.0 * (1.0 - 0.7 * adjusted_cs);
        const BROKEN_GAMEFIELD_ROUNDING_ALLOWANCE: f32 = 1.00041;
        let radius = sprite_display_size / 2.0 / 1.0 * BROKEN_GAMEFIELD_ROUNDING_ALLOWANCE;
        let radius_sq = radius * radius;

        let hit_window_50 = map_difficulty_range(map.od, 200.0, 150.0, 100.0, mods) as i32;
        let preempt = map_difficulty_range(map.ar, 1800.0, 1200.0, 450.0, mods) as i32;

        let hit_objects = hit_objects.iter().map(|h| HitObject::new(h)).collect();

        Self {
            hit_window_50,
            preempt,
            radius_sq,
            hit_objects,
            minimal_start: 0,
            minimal_end: 0,
        }
    }
}

impl<'h> HitObjectManager<'h> {
    pub fn update(&mut self, time: i32) {
        let min_left = time - self.preempt;
        let min_right = time + self.preempt;

        self.minimal_start = self
            .hit_objects
            .partition_point(|h| h.start_time() < min_left);

        self.minimal_end = self.minimal_start
            + self.hit_objects[self.minimal_start..]
                .partition_point(|h| h.start_time() < min_right);
    }

    pub fn hit_objects_minimal(&self) -> &[HitObject<'h>] {
        &self.hit_objects[self.minimal_start..self.minimal_end]
    }

    pub fn find_circle_at(&self, frame: &HitFrame) -> Option<(usize, &HitObject<'h>)> {
        /*
            Vector2 v = new Vector2(x, y);

            foreach (HitObject h in hitObjectsMinimal)
                if ((!checkScorable || h.IsScorable) && h.HitTest(v, hittableRangeOnly, radius))
                    return h;
            return null;
        */

        self.hit_objects_minimal()
            .iter()
            .enumerate()
            .find(|(_, h)| h.hit_test(frame, self))
    }

    pub fn hit(&mut self, index: usize) {
        self.hit_objects[self.minimal_start + index].hit()
    }
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
