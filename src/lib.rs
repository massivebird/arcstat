use std::collections::HashMap;
use walkdir::WalkDir;
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

    // silently skip error entries
    for entry in WalkDir::new(&config.archive_root)
        .into_iter().filter_map(|e| e.ok())
        {

            // "snes/Shadowrun.sfc"
            let relative_pathname = entry.path()
                .strip_prefix(&config.archive_root).unwrap()
                .to_string_lossy();

            if relative_pathname.contains("!bios") { continue }

            // "snes"
            let base_dir = relative_pathname
                [..relative_pathname.find("/").unwrap_or(0)]
                .to_string();

            let Some(system) = systems.iter()
                .filter(|s| s.directory == base_dir).next()
            else {
                continue;
            };

            let file_size = entry.metadata().unwrap().len();
            systems_map.entry(&system).and_modify(|v| v.1 += file_size);

            // if games are directories,
            // don't increment game count for every normal file
            if system.games_are_directories && entry.path().is_file() {
                continue;
            }

            // increment game count for current system
            systems_map.entry(&system).and_modify(|v| v.0 += 1).or_insert((1,0));
        }

    let mut totals: (u32, u64) = (0, 0);

    let mut add_to_totals = |a: (u32, u64)| {
        totals.0 += a.0;
        totals.1 += a.1;
    };

    // iterates systems instead of systems_map to guarantee
    // display (alphabetical) order
    for system in systems.iter() {
        if !systems_map.contains_key(&system) { continue }
        let (game_count, file_size) = systems_map.get(&system).unwrap();
        if *game_count == 0 { continue }
        add_to_totals((*game_count, *file_size));
        println!("{: <6}{game_count: <5}{:.2}M",
            system.pretty_string,
            bytes_to_megabytes(*file_size));
    }

    println!("{: <6}{: <5}{:.2}M", " ",
        totals.0,
        bytes_to_megabytes(totals.1));
}
