use std::collections::HashMap;
use walkdir::WalkDir;
use regex::Regex;
use std::env;
use systems::System;

mod systems;

pub struct Config {
    archive_root: String,
}

impl Config {
    pub fn new(args: &[String]) -> Config {
        let archive_root = args.get(2)
            .unwrap_or(&env::var("VG_ARCHIVE")
                .unwrap_or_else(
                    |_| panic!("Neither provided path nor VG_ARCHIVE are valid")
                )
            )
            .to_owned();

        Config { archive_root }
    }
}

fn bytes_to_megabytes(bytes: u64) -> f32 {
    bytes as f32 / 1_000_000.0
}

pub fn run(config: Config) {
    let systems = systems::generate_systems();

    // each system has (game_count, bytes)
    let mut systems_map: HashMap<&System, (u32, u64)> = HashMap::new();

    // let increment_game_count = |s: &System| {
    //     systems_map.entry(s).and_modify(|v| v.0 += 1);
    // };

    // silently skip error entries
    for entry in WalkDir::new(&config.archive_root)
        .into_iter().filter_map(|e| e.ok())
        {

            // "snes/Shadowrun.sfc"
            let relative_pathname = entry.path()
                .strip_prefix(&config.archive_root).unwrap()
                .to_string_lossy();
            // "snes"
            let base_dir = relative_pathname
                [..relative_pathname.find("/").unwrap_or(0)]
                .to_string();
            // "Shadowrun"
            let game_name = entry.path().file_stem()
                .unwrap().to_string_lossy().into_owned();

            let Some(system) = systems.iter()
                .filter(|s| s.directory == base_dir).next()
            else {
                continue;
            };

            // increment game count for current system
            systems_map.entry(&system).and_modify(|v| v.0 += 1).or_insert((1,0));

            let file_size = entry.metadata().unwrap().len();
            systems_map.entry(&system).and_modify(|v| v.1 += file_size);
        }

    for (system, (game_count, file_size)) in systems_map {
        if game_count == 0 { continue }
        println!("{: <5} {game_count: <4} {:.2}M",
            system.pretty_string,
            bytes_to_megabytes(file_size));
    }
}
