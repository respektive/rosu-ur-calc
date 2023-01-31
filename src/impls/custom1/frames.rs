use std::fmt::{Debug, Formatter, Result as FmtResult};

use osu_db::Replay;
use rosu_pp::parse::Pos2;

pub struct HitFrames;

impl HitFrames {
    pub fn from_replay(replay: &Replay) -> Vec<HitFrame> {
        #[derive(Default)]
        struct ScanState {
            time_elapsed: i32,
            prev_keys: u8,
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

                let keys = action.z as u8;
                let new_keys = keys & !state.prev_keys;
                let is_doubletap = (new_keys & (M1 | M2)) == (M1 | M2);
                state.prev_keys = keys;

                let frame = (new_keys > 0 && action.delta >= 0 && !skip).then_some(HitFrame {
                    time: state.time_elapsed,
                    pos: Pos {
                        x: action.x,
                        y: action.y,
                    },
                });

                let dup = frame.filter(|_| is_doubletap);

                Some([frame, dup])
            })
            .flatten()
            .flatten()
            .collect()
    }
}

const M1: u8 = 1 << 0;
const M2: u8 = 1 << 1;

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
