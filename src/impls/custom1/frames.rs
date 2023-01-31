use std::{
    fmt::{Debug, Formatter, Result as FmtResult},
    mem,
};

use osu_db::Replay;
use rosu_pp::parse::Pos2;

use crate::models::Buttons;

pub struct HitFrames;

impl HitFrames {
    pub fn from_replay(replay: &Replay) -> Vec<HitFrame> {
        #[derive(Default)]
        struct ScanState {
            time_elapsed: i32,
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
                state.time_elapsed += action.delta as i32;

                let skip = i < 2
                    && (action.x - 256.0).abs() <= f32::EPSILON
                    && (action.y - 500.0).abs() <= f32::EPSILON;

                let keys = Buttons::from_f32(action.z);
                let prev_keys = mem::replace(&mut state.prev_keys, keys);
                let new_press = keys.is_new_press(prev_keys);

                let frame = (action.delta >= 0 && new_press && !skip).then_some(HitFrame {
                    time: state.time_elapsed,
                    pos: Pos {
                        x: action.x,
                        y: action.y,
                    },
                });

                Some(frame)
            })
            .flatten()
            .collect()
    }
}

#[derive(Copy, Clone, Debug)]
pub struct HitFrame {
    pub time: i32,
    pub pos: Pos,
}

#[derive(Copy, Clone)]
pub struct Pos {
    x: f32,
    y: f32,
}

impl Pos {
    pub fn dist_sq(&self, other: Pos2) -> f32 {
        (self.x - other.x) * (self.x - other.x) + (self.y - other.y) * (self.y - other.y)
    }
}

impl Debug for Pos {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "({}, {})", self.x, self.y)
    }
}
