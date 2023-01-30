use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};
use osu_db::Replay;
use rosu_pp::Beatmap;
use rosu_ur_calc::{calculate_ur_baseline, calculate_ur_iters};

pub fn unstable_rate_bench(c: &mut Criterion) {
    let map_file = "Ayase Rie - Yuima-ruWorld TVver. (Fycho) [Extra]";
    let replay_file = "replay-osu_983680_2294262584";
    new_group(c, "respektive haitai", map_file, replay_file);

    let map_file = "sphere - HIGH POWERED (TV Size) (Azunyan-) [POWER OVERLOAD EXPERT]";
    let replay_file = "replay-osu_2779503_3916842208";
    new_group(c, "respektive high powered", map_file, replay_file);

    let map_file = "Euchaeta - Who's World (P_O) [Who Does This World Belong To]";
    let replay_file = "replay-osu_3312004_4205640222";
    new_group(c, "respektive whos world", map_file, replay_file);
}

fn new_group(c: &mut Criterion, name: &str, map_file: &str, replay_file: &str) {
    let (map, replay) = parse_map_replay(map_file, replay_file);
    let mut group = c.benchmark_group(name);

    group.bench_with_input("baseline", &(&map, &replay), |b, &(map, replay)| {
        b.iter(|| calculate_ur_baseline(black_box(map), black_box(replay)))
    });

    group.bench_with_input("iters", &(&map, &replay), |b, (map, replay)| {
        b.iter(|| calculate_ur_iters(black_box(map), black_box(replay)))
    });

    group.finish();
}

fn parse_map_replay(map_file: &str, replay_file: &str) -> (Beatmap, Replay) {
    let map_path = format!("./test-data/{map_file}.osu");
    let replay_path = format!("./test-data/{replay_file}.osr");

    let map = Beatmap::from_path(map_path).unwrap();
    let replay = Replay::from_file(replay_path).unwrap();

    (map, replay)
}

criterion_group!(benches, unstable_rate_bench);
criterion_main!(benches);
