mod impls;
mod models;

use osu_db::Replay;
use rosu_pp::Beatmap;

pub use impls::{
    baseline::calculate_ur as calculate_ur_baseline,
    circleguard::calculate_ur as calculate_ur_circleguard,
    custom1::calculate_ur as calculate_ur_custom1, custom2::calculate_ur as calculate_ur_custom2,
    iters::calculate_ur as calculate_ur_iters, stable::calculate_ur as calculate_ur_stable,
};

macro_rules! default_fn {
    ( $( $feature:literal -> $module:ident ,)* ) => {
        $(
            #[cfg(feature = $feature)]
            pub fn calculate_ur(map: &Beatmap, replay: &Replay) -> f64 {
                impls::$module::calculate_ur(map, replay)
            }
        )*

        #[cfg(not(any($( feature = $feature, )*)))]
        pub fn calculate_ur(map: &Beatmap, replay: &Replay) -> f64 {
            impls::custom2::calculate_ur(map, replay)
        }
    }
}

default_fn! {
    "baseline" -> baseline,
    "iters" -> iters,
    "stable" -> stable,
    "circleguard" -> circleguard,
    "custom1" -> custom1,
    "custom2" -> custom2,
}
