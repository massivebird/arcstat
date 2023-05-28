use std::collections::HashMap;
use walkdir::{WalkDir, DirEntry};
use std::env;
use archive_systems::{System, generate_systems};

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
    let systems = generate_systems();

    // each system has (game_count, bytes)
    let mut systems_map: HashMap<&System, (u32, u64)> = HashMap::new();

    let is_valid_system_dir = |e: &DirEntry| {
        systems.iter().any(|s| e.path().to_string_lossy().contains(&s.directory))
    };

    let is_not_bios_dir = |e: &DirEntry| {
        !e.path().to_string_lossy().contains("!bios")
    };

    // silently skip error entries
    for entry in WalkDir::new(&config.archive_root).into_iter()
        .filter_map(|e| e.ok()) // silently skip errorful entries
        .filter(|e| is_not_bios_dir(e) && is_valid_system_dir(e))
        {

            // "snes/Shadowrun.sfc"
            let relative_pathname = entry.path()
                .strip_prefix(&config.archive_root).unwrap()
                .to_string_lossy();

            // "snes"
            let base_dir = relative_pathname
                [..relative_pathname.find('/').unwrap_or(0)]
                .to_string();

            let Some(system) = systems.iter()
                .find(|s| s.directory == base_dir)
            else {
                continue;
            };

            let file_size = entry.metadata().unwrap().len();
            systems_map.entry(system).and_modify(|v| v.1 += file_size);

            // if games are directories,
            // don't increment game count for every normal file
            if system.games_are_directories && entry.path().is_file() {
                continue;
            }

            // increment game count for current system
            systems_map.entry(system).and_modify(|v| v.0 += 1).or_insert((1,0));
        }

    let mut totals: (u32, u64) = (0, 0);

    let mut add_to_totals = |(game_count, file_size): (u32, u64)| {
        totals.0 += game_count;
        totals.1 += file_size;
    };

    // iterates systems instead of systems_map to guarantee
    // display (alphabetical) order
    for system in systems
        .iter()
        .filter(|s| systems_map.contains_key(s))
        {
            let (game_count, file_size) = systems_map.get(&system).unwrap();
            add_to_totals((*game_count, *file_size));
            println!("{: <6}{game_count: <5}{:.2}M",
                system.pretty_string,
                bytes_to_megabytes(*file_size));
        }

    println!("{: <6}{: <5}{:.2}M", " ",
        totals.0,
        bytes_to_megabytes(totals.1));
}
