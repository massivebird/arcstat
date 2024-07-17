use self::config::Config;
use arcconfig::{read_config, system::System};
use colored::{ColoredString, Colorize};
use std::{
    cmp::max,
    collections::HashMap,
    path::Path,
    sync::{Arc, Mutex},
    thread,
};

pub mod config;

type ArcMutexHashmap<K, V> = Arc<Mutex<HashMap<K, V>>>;

pub fn run() {
    let config = Config::new();

    let systems: Vec<System> = read_config(&config.archive_root)
        .into_iter()
        .filter(|s| {
            config
                .desired_systems
                .clone()
                .map_or(true, |labels| labels.contains(&s.label))
        })
        .collect();

    // track (game_count, bytes) for each system
    let systems_stats: ArcMutexHashmap<System, (u32, u64)> = Arc::new(Mutex::new(HashMap::new()));

    // This scope will wait to terminate until all child threads terminate.
    thread::scope(|scope| {
        // Must create this ref outside of thread::spawn, else the thread will attempt to
        // move config into itself
        let config_ref = &config;

        for system_ref in &systems {
            let systems_stats = Arc::clone(&systems_stats);

            scope.spawn(move || {
                analyze_system(config_ref, system_ref, &systems_stats);
            });
        }
    });

    let mut totals: (u32, u64) = (0, 0);

    let mut add_to_totals = |(game_count, file_size): (u32, u64)| {
        totals.0 += game_count;
        totals.1 += file_size;
    };

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

    // Prints header row
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

        add_to_totals((game_count, file_size));

        println!(
            "{: <col_1_width$}{game_count: <col_2_width$}{:.2}G",
            system.pretty_string,
            bytes_to_gigabytes(file_size)
        );
    }

    // Prints totals!
    println!(
        "{: <col_1_width$}{: <col_2_width$}{:.2}G",
        " ",
        totals.0,
        bytes_to_gigabytes(totals.1),
    );
}

fn analyze_system(
    config: &Config,
    system: &System,
    systems_map: &ArcMutexHashmap<System, (u32, u64)>,
) {
    // initialize this system's data
    systems_map
        .lock()
        .unwrap()
        .entry((*system).clone())
        .or_insert((0, 0));

    let system_path = format!(
        "{}/{}",
        config.archive_root.clone(),
        system.directory.as_str()
    );

    let walk_archive = || {
        Path::new(&system_path)
            .read_dir()
            .unwrap()
            .filter_map(Result::ok) // silently skip errorful entries
            .filter(|e| !e.path().to_string_lossy().contains("!bios"))
    };

    let add_to_total_system_size = |n: u64| {
        // add to this system's total file size
        systems_map
            .lock()
            .unwrap()
            .entry((*system).clone())
            .and_modify(|v| v.1 += n);
    };

    let increment_total_system_games = || {
        // add to this system's total game count
        systems_map
            .lock()
            .unwrap()
            .entry((*system).clone())
            .and_modify(|v| v.0 += 1);
    };

    for entry in walk_archive() {
        let file_size = entry.metadata().unwrap().len();

        add_to_total_system_size(file_size);

        // if games are represented as directories,
        // increment game count only once per directory
        if system.games_are_directories && entry.path().is_file() {
            continue;
        }

        if system.games_are_directories && entry.path().is_dir() {
            // game is split into one or more files inside this directory
            let game_parts = || {
                Path::new(&entry.path())
                    .read_dir()
                    .unwrap()
                    .filter_map(Result::ok)
            };

            for part in game_parts() {
                let file_size = part.metadata().unwrap().len();
                add_to_total_system_size(file_size);
            }
        }

        increment_total_system_games();
    }
}

fn bytes_to_gigabytes(bytes: u64) -> f32 {
    bytes as f32 / 1_073_741_824.0
}
