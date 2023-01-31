use osu_db::Replay;
use rosu_pp::Beatmap;
use rosu_ur_calc::calculate_ur;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let map_path = args[1].to_string();
    let replay_path = args[2].to_string();

    let map = Beatmap::from_path(map_path).unwrap();
    let replay = Replay::from_file(replay_path).unwrap();

    let unstable_rate = calculate_ur(&map, &replay);

    println!("UR: {unstable_rate:#?}");
}
