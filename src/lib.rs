use std::{
    collections::HashMap,
    sync::{Mutex, Arc},
    thread::{JoinHandle, self},
    env, process};
use walkdir::WalkDir;
use archive_systems::{System, generate_systems};

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
    let systems = generate_systems().map(Arc::new);

    // For your archive, use the below definition for the `systems` variable.
    // The syntax is System::new(
    //   <ColoredString from "colored" crate: the system display name>,
    //   <String: system directory relative to archive root>,
    //   <bool: "games are stored as directories" as opposed to regular files>
    // );

    // use colored::Colorize;
    // let systems: Vec<Arc<System>> = vec![
    //     Arc::new(System::new("N64".truecolor(0,215,135), "n64", false)),
    // ];

    let config = Arc::new(config);

    // each system has (game_count, bytes)
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

    // iterates systems instead of systems_map to guarantee
    // display (alphabetical) order
    for system in systems
        .iter()
        .filter(|s| systems_map.lock().unwrap().contains_key(s.as_ref()))
        {
            let systems_map = systems_map.lock().unwrap();
            let (game_count, file_size) = systems_map.get(system.as_ref()).unwrap();
            add_to_totals((*game_count, *file_size));
            println!("{: <6}{game_count: <5}{:.2}G",
                system.pretty_string,
                bytes_to_gigabytes(*file_size));
        }

    println!("{: <6}{: <5}{:.2}G", " ",
        totals.0,
        bytes_to_gigabytes(totals.1));
}
