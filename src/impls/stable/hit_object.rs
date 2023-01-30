use std::fmt::{Debug, Formatter, Result as FmtResult};

use rosu_pp::{
    osu::{OsuObject, OsuObjectKind},
    parse::Pos2,
};

use super::{frames::HitFrame, hit_object_manager::HitObjectManager};

pub struct HitObject<'h> {
    pub is_hit: bool,
    h: &'h dyn HitObjectExt,
}

impl Debug for HitObject<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        #[derive(Debug)]
        enum HitObjectKind {
            Circle,
            Slider,
            Spinner,
        }

        #[derive(Debug)]
        #[allow(unused)]
        struct HitObject {
            time: i32,
            pos: Pos2,
            kind: HitObjectKind,
            is_hit: bool,
        }

        let h = HitObject {
            time: self.h.start_time(),
            pos: self.h.pos(),
            kind: if self.h.is_normal() {
                HitObjectKind::Circle
            } else if self.h.is_slider() {
                HitObjectKind::Slider
            } else {
                HitObjectKind::Spinner
            },
            is_hit: self.is_hit,
        };

        Debug::fmt(&h, f)
    }
}

impl<'h> HitObject<'h> {
    pub fn new(h: &'h dyn HitObjectExt) -> Self {
        Self { h, is_hit: false }
    }

    pub fn hit(&mut self) {
        self.is_hit = true;
    }
}

impl HitObject<'_> {
    pub fn hit_test(&self, frame: &HitFrame, manager: &HitObjectManager<'_>) -> bool {
        /*
            return ((!hittableRangeOnly && IsVisible) ||
                  (StartTime - hitObjectManager.PreEmpt <= AudioEngine.Time &&
                   StartTime + hitObjectManager.HitWindow50 >= AudioEngine.Time && !IsHit)) &&
                 (Vector2.DistanceSquared(testPosition, Position) <= radius * radius ||
                  (!hittableRangeOnly &&
                   Vector2.DistanceSquared(testPosition, Position2) <= radius * radius));

            simplified:

            return StartTime - hitObjectManager.PreEmpt <= AudioEngine.Time &&
                   StartTime + hitObjectManager.HitWindow50 >= AudioEngine.Time && !IsHit &&
                 Vector2.DistanceSquared(testPosition, Position) <= radius * radius;
        */

        let matches_time = self.start_time() - manager.preempt <= frame.time
            && self.start_time() + manager.hit_window_50 >= frame.time;

        let matches_pos = frame.pos.dist_sq(self.pos()) <= manager.radius_sq;

        matches_time && matches_pos && !self.is_hit
    }

    pub fn start_time(&self) -> i32 {
        self.h.start_time()
    }

    fn pos(&self) -> Pos2 {
        self.h.pos()
    }

    pub fn stack_count(&self) -> i32 {
        self.h.stack_count()
    }

    pub fn is_normal(&self) -> bool {
        self.h.is_normal()
    }

    pub fn is_slider(&self) -> bool {
        self.h.is_slider()
    }

    pub fn is_visible(&self, time: i32, manager: &HitObjectManager<'_>) -> bool {
        self.h.is_visible(time, manager)
    }
}

pub trait HitObjectExt {
    fn start_time(&self) -> i32;
    fn pos(&self) -> Pos2;
    fn stack_count(&self) -> i32;
    fn is_normal(&self) -> bool;
    fn is_slider(&self) -> bool;
    fn is_visible(&self, time: i32, manager: &HitObjectManager<'_>) -> bool;
}

impl HitObjectExt for OsuObject {
    #[inline]
    fn start_time(&self) -> i32 {
        self.start_time as i32
    }

    #[inline]
    fn pos(&self) -> Pos2 {
        self.stacked_pos()
    }

    #[inline]
    fn stack_count(&self) -> i32 {
        self.stack_height as i32
    }

    #[inline]
    fn is_normal(&self) -> bool {
        self.is_circle()
    }

    #[inline]
    fn is_slider(&self) -> bool {
        self.is_slider()
    }

    #[inline]
    fn is_visible(&self, time: i32, manager: &HitObjectManager<'_>) -> bool {
        match self.kind {
            OsuObjectKind::Circle => {
                time >= self.start_time as i32 - manager.preempt
                    && time <= self.start_time as i32 + HitObjectManager::FADE_OUT
            }
            OsuObjectKind::Slider(ref slider) => {
                time >= self.start_time as i32 - manager.preempt
                    && time <= slider.end_time as i32 + HitObjectManager::FADE_OUT
            }
            OsuObjectKind::Spinner { end_time } => {
                time >= self.start_time as i32 - HitObjectManager::FADE_IN
                    && time <= end_time as i32
            }
        }
    }
}
