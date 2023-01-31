mod impls;
mod models;

use osu_db::Replay;
use rosu_pp::Beatmap;

pub use impls::baseline::calculate_ur as calculate_ur_baseline;

macro_rules! export_fns {
    ( $( $feature:literal -> $module:ident ,)* ) => {
        $(
            #[cfg(feature = $feature)]
            pub fn calculate_ur(map: &Beatmap, replay: &Replay) -> f64 {
                impls::$module::calculate_ur(map, replay)
            }
        )*

        #[cfg(not(any($( feature = $feature, )*)))]
        pub fn calculate_ur(map: &Beatmap, replay: &Replay) -> f64 {
            impls::custom1::calculate_ur(map, replay)
        }
    }
}

export_fns! {
    "baseline" -> baseline,
    "iters" -> iters,
    "stable" -> stable,
    "circleguard" -> circleguard,
    "custom1" -> custom1,
}
