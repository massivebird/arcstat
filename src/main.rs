use self::config::Config;
use arcconfig::{read_config, system::System};
use colored::{ColoredString, Colorize};
use std::{
    cmp::max,
    collections::HashMap,
    path::Path,
    sync::{Arc, Mutex},
};
use tokio::task::JoinSet;

mod config;

type ArcMutexHashmap<K, V> = Arc<Mutex<HashMap<K, V>>>;

#[tokio::main]
async fn main() {
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

    // track (game_count, bytes) for each system
    let systems_stats: ArcMutexHashmap<System, (u32, u64)> = Arc::new(Mutex::new(HashMap::new()));

    let mut join_set: JoinSet<()> = JoinSet::new();

    for system in &systems {
        let system = system.clone();
        let config = config.clone();
        let systems_stats = Arc::clone(&systems_stats);
        join_set.spawn(async move { analyze_system(&config, &system, &systems_stats) });
    }

    join_set.join_all().await;

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

    // Prints totals!
    println!(
        "{: <col_1_width$}{: <col_2_width$}{:.2}G",
        " ",
        total_game_count,
        bytes_to_gigabytes(total_file_size),
    );
}

// async fn analyze_systems(
//     systems: &Vec<System>,
//     config: &Config,
//     systems_map: &ArcMutexHashmap<System, (u32, u64)>,
// ) {
//     let mut join_set: JoinSet<()> = JoinSet::new();

//     for system in systems {
//         join_set.spawn(async { analyze_system(config, system, systems_map) });
//     }

//     while let Some(_) = join_set.join_next().await {}
// }

fn analyze_system(
    config: &Config,
    system: &System,
    systems_map: &ArcMutexHashmap<System, (u32, u64)>,
) {
    // Initialize this system's data
    systems_map
        .lock()
        .unwrap()
        .entry((system).clone())
        .or_insert((0, 0));

    // I'll need this in multiple places later
    let add_to_total_system_size = |n: u64| {
        systems_map
            .lock()
            .unwrap()
            .entry((system).clone())
            .and_modify(|v| v.1 += n);
    };

    let system_path = format!(
        "{}/{}",
        config.archive_root.clone(),
        system.directory.as_str()
    );

    for entry in Path::new(&system_path)
        .read_dir()
        .unwrap()
        .filter_map(Result::ok) // silently skip errorful entries
        .filter(|e| !e.path().to_string_lossy().contains("!bios"))
    {
        let file_size = entry.metadata().unwrap().len();

        add_to_total_system_size(file_size);

        // If games are represented as directories,
        // prevent normal files from incrementing game count.
        if system.games_are_directories && entry.path().is_file() {
            continue;
        }

        if system.games_are_directories && entry.path().is_dir() {
            // Iterate over this game's multiple parts
            for part in Path::new(&entry.path())
                .read_dir()
                .unwrap()
                .filter_map(Result::ok)
            // skip errorful entries
            {
                let file_size = part.metadata().unwrap().len();
                add_to_total_system_size(file_size);
            }
        }

        // Increment this system's game count
        systems_map
            .lock()
            .unwrap()
            .entry(system.clone())
            .and_modify(|v| v.0 += 1);
    }
}

fn bytes_to_gigabytes(bytes: u64) -> f32 {
    bytes as f32 / 1_073_741_824.0
}
