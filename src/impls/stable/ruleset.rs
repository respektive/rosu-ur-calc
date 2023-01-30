use super::{frames::HitFrame, hit_object::HitObject, hit_object_manager::HitObjectManager};

const HITTABLE_RANGE: i32 = 400;

pub struct Ruleset;

impl Ruleset {
    pub fn check_click_action(
        h: &HitObject<'_>,
        index: usize,
        frame: &HitFrame,
        manager: &HitObjectManager<'_>,
    ) -> ClickAction {
        if h.is_normal() && index > 0 {
            let prev = &manager.hit_objects_minimal()[index - 1];

            if prev.stack_count() > 0 && prev.is_visible(frame.time, manager) && !prev.is_hit {
                return ClickAction::Ignore;
            }
        }

        let mut is_next_circle = true;

        for (j, t) in manager.hit_objects_minimal().iter().enumerate() {
            if t.start_time() + manager.hit_window_50 <= frame.time || t.is_hit {
                continue;
            }

            if t.start_time() < h.start_time() && index != j {
                is_next_circle = false;
            }

            break;
        }

        if is_next_circle && (h.start_time() - frame.time).abs() < HITTABLE_RANGE {
            ClickAction::Hit
        } else {
            ClickAction::Shake
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum ClickAction {
    Ignore,
    Shake,
    Hit,
}
