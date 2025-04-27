use self::config::Config;
use arcconfig::{read_config, system::System};
use colored::ColoredString;
use rayon::prelude::*;
use std::{
    collections::VecDeque,
    path::Path,
    sync::{Arc, Mutex},
};
use tabled::{settings::Style, Table, Tabled};
use tokio::spawn;

mod config;

#[derive(Copy, Clone, Default, Debug)]
struct Analysis {
    num_games: u32,
    file_size: u64,
}

#[derive(Tabled)]
struct TableRow {
    #[tabled(rename = "System")]
    system_str: ColoredString,
    #[tabled(rename = "Games")]
    num_games: u32,
    #[tabled(rename = "Size (GB)")]
    file_size: String,
}

#[tokio::main]
async fn main() {
    let config = Config::generate();

    let systems: Vec<System> = read_config(&config.archive_root)
        .into_iter()
        .filter(|s| {
            config
                .desired_systems
                .clone()
                .is_none_or(|labels| labels.contains(&s.label))
        })
        .collect();

    let mut handles = VecDeque::new();

    for system in systems.clone() {
        let config = config.clone();
        handles.push_back(spawn(async move { analyze_system(&config, &system) }));
    }

    let mut table_rows: Vec<TableRow> = Vec::new();

    let mut total_num_games = 0;
    let mut total_file_size = 0;

    for system in systems.clone() {
        let analysis = handles.pop_front().unwrap().await.unwrap();

        total_num_games += analysis.num_games;
        total_file_size += analysis.file_size;

        table_rows.push(TableRow {
            system_str: system.pretty_string,
            num_games: analysis.num_games,
            file_size: format!("{:.02}", analysis.file_size as f32 / 1_073_741_824.0),
        });
    }

    table_rows.push(TableRow {
        system_str: String::new().into(),
        num_games: total_num_games,
        file_size: format!("{:.02}", total_file_size as f32 / 1_073_741_824.0),
    });

    let table = Table::new(table_rows)
        .with(Style::psql().remove_vertical())
        .to_string();

    println!("{table}");
}

fn analyze_system(config: &Config, system: &System) -> Analysis {
    let analysis = Arc::new(Mutex::new(Analysis::default()));

    let system_path = config.archive_root.join(system.directory.as_str());

    let analysis = Arc::clone(&analysis);

    Path::new(&system_path)
        .read_dir()
        .unwrap()
        .par_bridge()
        .filter_map(Result::ok)
        .filter(|e| !e.path().to_string_lossy().contains("!bios"))
        .for_each(|entry| {
            let file_size = entry.metadata().unwrap().len();

            analysis.lock().unwrap().file_size += file_size;

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
                    analysis.lock().unwrap().file_size += file_size;
                }
            }

            // Increment this system's game count.
            analysis.lock().unwrap().num_games += 1;
        });

    // Retrieve the wrapped `Analysis`.
    Arc::into_inner(analysis).unwrap().into_inner().unwrap()
}
