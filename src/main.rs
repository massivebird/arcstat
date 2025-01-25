use self::config::Config;
use arcconfig::{read_config, system::System};
use colored::{ColoredString, Colorize};
use rayon::prelude::*;
use std::{
    cmp::max,
    collections::HashMap,
    fs::DirEntry,
    path::Path,
    sync::{Arc, Mutex},
    thread,
};

mod config;

type ArcMutexHashmap<K, V> = Arc<Mutex<HashMap<K, V>>>;

fn main() {
    let config = Config::generate();

    let systems: Vec<System> = read_config(&config.archive_root)
        .into_iter()
        .filter(|s| {
            config
                .desired_systems
                .clone()
                .map_or(true, |labels| labels.contains(&s.label))
        })
        .collect();

    // Record (game_count, bytes) per system.
    let systems_stats: ArcMutexHashmap<System, (u32, u64)> = Arc::new(Mutex::new(HashMap::new()));

    thread::scope(|scope| {
        for system_ref in &systems {
            let systems_stats = Arc::clone(&systems_stats);

            scope.spawn({
                let config_ref = &config; // Borrow without moving config.
                move || {
                    analyze_system(config_ref, system_ref, &systems_stats);
                }
            });
        }
    });

    let headers = ("System", "Games", "Size");

    let (col_1_width, col_2_width) = {
        let col_1 = systems.iter().map(|s| s.pretty_string.len()).max().unwrap();

        let col_2 = systems_stats
            .lock()
            .unwrap()
            .values()
            .map(|(game_count, _)| game_count.to_string().len())
            .max()
            .unwrap();

        let padding = 2;

        // column space must be no less than length of header
        (
            max(col_1, headers.0.len()) + padding,
            max(col_2, headers.1.len()) + padding,
        )
    };

    let styled_header = |text: &str| -> ColoredString { text.underline().white() };

    // Print header row.
    println!(
        "{: <col_1_width$}{: <col_2_width$}{}",
        styled_header(headers.0),
        styled_header(headers.1),
        styled_header(headers.2)
    );

    // Iterates over all systems, or only user-specified systems if any.
    for system in systems
        .iter()
        .filter(|s| systems_stats.lock().unwrap().contains_key(s))
    {
        let (game_count, file_size) = *systems_stats.lock().unwrap().get(system).unwrap();

        println!(
            "{: <col_1_width$}{game_count: <col_2_width$}{:.2}G",
            system.pretty_string,
            bytes_to_gigabytes(file_size)
        );
    }

    let (total_game_count, total_file_size) = systems_stats
        .lock()
        .unwrap()
        .iter()
        .fold((0, 0), |acc, (_, (game_count, file_size))| {
            (acc.0 + game_count, acc.1 + file_size)
        });

    // Prints totals row.
    println!(
        "{: <col_1_width$}{: <col_2_width$}{:.2}G",
        " ",
        total_game_count,
        bytes_to_gigabytes(total_file_size),
    );
}

fn analyze_system(
    config: &Config,
    system: &System,
    systems_map: &ArcMutexHashmap<System, (u32, u64)>,
) {
    // Initialize this system's data.
    systems_map
        .lock()
        .unwrap()
        .entry((*system).clone())
        .or_insert((0, 0));

    let add_to_total_system_size = |n: u64| {
        systems_map
            .lock()
            .unwrap()
            .entry((*system).clone())
            .and_modify(|v| v.1 += n);
    };

    let system_path = format!(
        "{}/{}",
        config.archive_root.clone(),
        system.directory.as_str()
    );

    // Call this for each file.
    let analyze_entry = |entry: DirEntry| {
        let file_size = entry.metadata().unwrap().len();

        add_to_total_system_size(file_size);

        // If games are represented as directories,
        // prevent normal files from incrementing game count.
        if system.games_are_directories && entry.path().is_file() {
            return;
        }

        if system.games_are_directories && entry.path().is_dir() {
            // Iterate over this game's multiple parts.
            for part in Path::new(&entry.path())
                .read_dir()
                .unwrap()
                .filter_map(Result::ok)
            {
                let file_size = part.metadata().unwrap().len();
                add_to_total_system_size(file_size);
            }
        }

        // Increment this system's game count.
        systems_map
            .lock()
            .unwrap()
            .entry((*system).clone())
            .and_modify(|v| v.0 += 1);
    };

    Path::new(&system_path)
        .read_dir()
        .unwrap()
        .par_bridge() // Run these in parallel with the rayon crate.
        .filter_map(Result::ok)
        .filter(|e| !e.path().to_string_lossy().contains("!bios"))
        .for_each(analyze_entry);
}

fn bytes_to_gigabytes(bytes: u64) -> f32 {
    bytes as f32 / 1_073_741_824.0
}
