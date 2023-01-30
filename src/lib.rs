mod impls;
mod models;

pub use self::impls::{
    baseline::calculate_ur as calculate_ur_baseline, iters::calculate_ur as calculate_ur_iters,
    stable::calculate_ur as calculate_ur_stable,
};
