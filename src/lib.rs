use std::{
    collections::HashMap,
    sync::{Mutex, Arc},
    thread::{JoinHandle, self},
    env, process};
use colored::*;
use walkdir::WalkDir;
use arcconfig::{System, read_config};

type ArcMutexHashmap<K, V> = Arc<Mutex<HashMap<K, V>>>;

pub struct Config {
    archive_root: String,
}

impl Config {
    pub fn new(args: &[String]) -> Self {
        let archive_root = args.get(2)
            .unwrap_or(&env::var("VG_ARCHIVE")
                .unwrap_or_else(|_| {
                    eprintln!("Neither provided path nor VG_ARCHIVE are valid");
                    process::exit(1);
                }
                )
            )
            .clone();

        Self { archive_root }
    }
}

fn bytes_to_gigabytes(bytes: u64) -> f32 {
    bytes as f32 / 1_000_000_000.0
}

fn create_thread(
    config: Arc<Config>,
    system: Arc<System>,
    systems_map: ArcMutexHashmap<System, (u32, u64)>
) -> JoinHandle<()> {
    thread::spawn(move || {
        for entry in WalkDir::new(config.archive_root.clone() + "/" + system.directory.as_str()).into_iter()
            .filter_map(Result::ok) // silently skip errorful entries
            .filter(|e| !e.path().to_string_lossy().contains("!bios"))
            .skip(1) // skip directory itself
            {
                let file_size = entry.metadata().unwrap().len();

                // add to system's total file size
                systems_map.lock().unwrap()
                    .entry((*system).clone())
                    .and_modify(|v| v.1 += file_size);

                if system.games_are_directories && entry.path().is_file() {
                    continue;
                }

                // add to system's game count
                systems_map.lock().unwrap()
                    .entry((*system).clone())
                    .and_modify(|v| v.0 += 1)
                    .or_insert((1,0));
            }
    })
}

pub fn run(config: Config) {
    let systems: Vec<Arc<System>> = read_config(&config.archive_root)
        .into_iter()
        .map(Arc::new)
        .collect();

    let config = Arc::new(config);

    // track (game_count, bytes) for each system
    let systems_map: ArcMutexHashmap<System, (u32, u64)> = Arc::new(Mutex::new(HashMap::new()));

    let mut children_threads: Vec<JoinHandle<()>> = Vec::with_capacity(systems.len());

    for system in &systems {
        children_threads.push(
            create_thread(Arc::clone(&config), Arc::clone(system), Arc::clone(&systems_map))
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

    let padding = 4;
    let col_1_width = padding + systems.iter()
        .map(|s| s.pretty_string.len())
        .max().unwrap();
    let col_2_width = padding + systems_map.lock().unwrap()
        .values()
        .map(|(game_count, _)| game_count.to_string().len())
        .max().unwrap();

    let column_header = |text: &str| -> ColoredString {
        text.underline().white()
    };

    println!("{: <col_1_width$}{: <col_2_width$}{}",
    column_header("System"),
    column_header("Games"),
    column_header("Size"));

    // iterates systems instead of systems_map to guarantee
    // display (alphabetical) order
    for system in systems
        .iter()
        .filter(|s| systems_map.lock().unwrap().contains_key(s.as_ref()))
        {
            let systems_map = systems_map.lock().unwrap();
            let (game_count, file_size) = systems_map.get(system.as_ref()).unwrap();
            add_to_totals((*game_count, *file_size));
            println!("{: <col_1_width$}{game_count: <col_2_width$}{:.2}G",
                system.pretty_string,
                bytes_to_gigabytes(*file_size),
            );
        }

    println!("{: <col_1_width$}{: <col_2_width$}{:.2}G", " ",
        totals.0,
        bytes_to_gigabytes(totals.1),
    );
}
