use arcconfig::{System, read_config};
use colored::{Colorize, ColoredString};
use self::config::Config;
use std::{
    collections::HashMap,
    sync::{Mutex, Arc},
    thread::{JoinHandle, self}, cmp::max
};
use walkdir::WalkDir;

pub mod config;

type ArcMutexHashmap<K, V> = Arc<Mutex<HashMap<K, V>>>;

fn bytes_to_gigabytes(bytes: u64) -> f32 {
    bytes as f32 / 1_000_000_000.0
}

fn create_thread(
    config: Arc<Config>,
    system: Arc<System>,
    data: ArcMutexHashmap<System, (u32, u64)>
) -> JoinHandle<()> {
    thread::spawn(move || {
        // initialize this system's data
        data.lock().unwrap()
            .entry((*system).clone())
            .or_insert((0,0));

        let walk_archive = || {
            WalkDir::new(config.archive_root.clone() + "/" + system.directory.as_str()).into_iter()
                .filter_map(Result::ok) // silently skip errorful entries
                .filter(|e| !e.path().to_string_lossy().contains("!bios"))
                .skip(1) // skip archive root entry
        };

        for entry in walk_archive() {
            let file_size = entry.metadata().unwrap().len();

            // add to this system's total file size
            data.lock().unwrap()
                .entry((*system).clone())
                .and_modify(|v| v.1 += file_size);

            // if games are represented as directories,
            // increment game count only once per directory
            if system.games_are_directories && entry.path().is_file() {
                continue;
            }

            // add to this system's total game count
            data.lock().unwrap()
                .entry((*system).clone())
                .and_modify(|v| v.0 += 1);
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
    let data: ArcMutexHashmap<System, (u32, u64)> = Arc::new(Mutex::new(HashMap::new()));

    let mut children_threads: Vec<JoinHandle<()>> = Vec::with_capacity(systems.len());

    for system in &systems {
        children_threads.push(
            create_thread(Arc::clone(&config), Arc::clone(system), Arc::clone(&data))
        );
    }

    for thread in children_threads {
        thread.join().expect("Child thread has panicked");
    }

    let mut totals: (u32, u64) = (0, 0);

    let mut add_to_totals = |(game_count, file_size): (u32, u64)| {
        totals.0 += game_count;
        totals.1 += file_size;
    };

    let headers = ("System", "Games", "Size");

    let (col_1_width, col_2_width) = {
        let col_1 = systems.iter()
            .map(|s| s.pretty_string.len())
            .max().unwrap();

        let col_2 = data.lock().unwrap()
            .values()
            .map(|(game_count, _)| game_count.to_string().len())
            .max().unwrap();

        let padding = 2;

        // column space must be no less than length of header
        (
            max(col_1, headers.0.len()) + padding,
            max(col_2, headers.1.len()) + padding,
        )
    };

    let styled_header = |text: &str| -> ColoredString {
        text.underline().white()
    };

    println!("{: <col_1_width$}{: <col_2_width$}{}",
    styled_header(headers.0),
    styled_header(headers.1),
    styled_header(headers.2));

    let all_systems_stats = || {
        systems
            .iter()
            .filter(|s| data.lock().unwrap().contains_key(s.as_ref()))
    };

    for system in all_systems_stats() {
        let (game_count, file_size) = *data.lock().unwrap()
            .get(system.as_ref()).unwrap();
        add_to_totals((game_count, file_size));

        println!("{: <col_1_width$}{game_count: <col_2_width$}{:.2}G",
        system.pretty_string,
        bytes_to_gigabytes(file_size));
    }

    println!("{: <col_1_width$}{: <col_2_width$}{:.2}G", " ",
    totals.0,
    bytes_to_gigabytes(totals.1),
);
}
