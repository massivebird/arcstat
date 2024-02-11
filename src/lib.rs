use arcconfig::{read_config, system::System};
use colored::{Colorize, ColoredString};
use self::config::Config;
use std::{
    collections::HashMap,
    sync::{Mutex, Arc},
    thread::{JoinHandle, self}, cmp::max, path::Path
};

pub mod config;

type ArcMutexHashmap<K, V> = Arc<Mutex<HashMap<K, V>>>;

fn create_thread(
    config: Arc<Config>,
    system: Arc<System>,
    systems_map: ArcMutexHashmap<System, (u32, Vec<u64>)>
) -> JoinHandle<()> {
    thread::spawn(move || {
        // initialize this system's data
        systems_map.lock().unwrap()
            .entry((*system).clone())
            .or_insert((0,Vec::new()));

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

        let add_to_file_sizes = |n: u64| {
            // add to this system's total file size
            systems_map.lock().unwrap()
                .entry((*system).clone())
                .and_modify(|(_, sizes)| sizes.push(n));
        };

        let increment_total_system_games = || {
            // add to this system's total game count
            systems_map.lock().unwrap()
                .entry((*system).clone())
                .and_modify(|v| v.0 += 1);
        };

        for entry in walk_archive() {
            // don't add directory sizes to totals, they're weirdos
            if !entry.path().is_dir() {
                let file_size = entry.metadata().unwrap().len();
                add_to_file_sizes(file_size);
            }

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
                    add_to_file_sizes(file_size);
                }
            }

            increment_total_system_games();
        }
    })
}

pub fn run() {
    let config = Config::new();

    let systems: Vec<Arc<System>> = read_config(&config.archive_root)
        .into_iter()
        .filter(|s| config.desired_systems.clone().map_or(
            true,
            |labels| labels.contains(&s.label)
        ))
        .map(Arc::new)
        .collect();

    let config = Arc::new(config);

    // track (game_count, bytes) for each system
    let systems_stats: ArcMutexHashmap<System, (u32, Vec<u64>)> = Arc::new(
        Mutex::new(HashMap::new())
    );

    let mut all_system_threads = Vec::with_capacity(systems.len());

    for system in &systems {
        all_system_threads.push(
            create_thread(
                Arc::clone(&config),
                Arc::clone(system),
                Arc::clone(&systems_stats)
            )
        );
    }

    for thread in all_system_threads {
        thread.join().expect("Child thread has panicked");
    }

    let mut totals: (u32, u64) = (0, 0);

    let mut add_to_totals = |(game_count, file_size): (u32, u64)| {
        totals.0 += game_count;
        totals.1 += file_size;
    };

    let headers = ("System", "Games", "Size", "Median Size");

    let (col_1_width, col_2_width, col_3_width) = {
        let col_1 = systems.iter()
            .map(|s| s.pretty_string.len())
            .max().unwrap();

        let col_2 = systems_stats.lock().unwrap()
            .values()
            .map(|(game_count, _)| game_count.to_string().len())
            .max().unwrap();

        let col_3 = systems_stats.lock().unwrap()
            .values()
            .map(|(_, file_size)| {
                bytes_to_human(file_size.iter().sum::<u64>()).len()
            })
            .max().unwrap();

        let padding = 2;

        // column space must be no less than length of header
        (
            max(col_1, headers.0.len()) + padding,
            max(col_2, headers.1.len()) + padding,
            max(col_3, headers.2.len()) + padding,
        )
    };

    let styled_header = |text: &str| -> ColoredString {
        text.underline().white()
    };

    println!("{: <col_1_width$}{: <col_2_width$}{: <col_3_width$}{}",
    styled_header(headers.0),
    styled_header(headers.1),
    styled_header(headers.2),
    styled_header(headers.3));

    let all_systems_stats = || {
        systems
            .iter()
            .filter(|s| systems_stats.lock().unwrap().contains_key(s.as_ref()))
    };

    for system in all_systems_stats() {
        let mut unlocked = systems_stats.lock().unwrap();

        let (game_count, sizes) = unlocked
            .get_mut(system.as_ref()).unwrap();

        sizes.sort();

        let total_system_size = sizes.iter().sum::<u64>();
        let median_size = *sizes.iter().nth(sizes.len()/2).unwrap();

        add_to_totals((*game_count, total_system_size));

        let file_size_string = bytes_to_human(total_system_size);

        println!("{: <col_1_width$}{game_count: <col_2_width$}{file_size_string: <col_3_width$}{}",
        system.pretty_string,
        bytes_to_human(median_size));
    }

    println!("{: <col_1_width$}{: <col_2_width$}{}", " ",
        totals.0,
        bytes_to_human(totals.1),
    );
}

fn bytes_to_human(bytes: u64) -> String {
    let bytes = bytes as f32;
    let bytes_per_megabyte = 1_048_576.0;
    let bytes_per_gigabyte = 1_073_741_824.0;
    if bytes > bytes_per_gigabyte {
        return format!("{:.2}G", bytes / bytes_per_gigabyte)
    } else {
        format!("{:.2}M", bytes / bytes_per_megabyte)
    }
}

