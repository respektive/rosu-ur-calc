use super::{frames::HitFrame, hit_object::HitObject, hit_object_manager::HitObjectManager};

pub struct Ruleset;

impl Ruleset {
    pub fn check_click_action(
        h: &HitObject<'_>,
        index: usize,
        frame: &HitFrame,
        manager: &HitObjectManager<'_>,
    ) -> ClickAction {
        /*
            if (h.IsType(HitObjectType.Normal))
            {
                //check stack - ignore clicks on circles lower in stack.
                int index = hitObjectManager.hitObjectsMinimal.IndexOf(h);

                if (index > 0 && hitObjectManager.hitObjectsMinimal[index - 1].StackCount > 0 &&
                    hitObjectManager.hitObjectsMinimal[index - 1].IsVisible &&
                    !hitObjectManager.hitObjectsMinimal[index - 1].IsHit)
                    return ClickAction.Ignore;
            }
        */

        if h.is_normal() && index > 0 {
            let prev = &manager.hit_objects_minimal()[index - 1];

            if prev.stack_count() > 0 && prev.is_visible(frame.time, manager) && !prev.is_hit {
                return ClickAction::Ignore;
            }
        }

        /*
            bool isNextCircle = true;
            foreach (HitObject t in hitObjectManager.hitObjectsMinimal)
            {
                if (t.StartTime + hitObjectManager.HitWindow50 <= AudioEngine.Time || t.IsHit) continue;

                if (t.StartTime < h.StartTime && t != h)
                    isNextCircle = false;
                break;
            }
        */

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

        /*
            int apLeniency = Player.Relaxing2 ? 200 : 0;
            return (isNextCircle && Math.Abs(h.StartTime - AudioEngine.Time) < HitObjectManager.HITTABLE_RANGE - apLeniency)
                       ? ClickAction.Hit
                       : ClickAction.Shake;
        */

        if is_next_circle && (h.start_time() - frame.time).abs() < HitObjectManager::HITTABLE_RANGE
        {
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
