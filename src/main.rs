use self::config::Config;
use arcconfig::{read_config, system::System};
use colored::ColoredString;
use tabled::settings::Style;
use std::{collections::VecDeque, path::Path};
use tabled::{Table, Tabled};
use tokio::spawn;

mod config;

#[derive(Default)]
struct Analysis {
    num_games: u32,
    file_size: u64,
}

#[derive(Tabled)]
struct TableRow {
    #[tabled(rename = "System")]
    system_str: String,
    #[tabled(rename = "Num games")]
    num_games: u32,
    #[tabled(rename = "Size")]
    file_size: u64,
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
                .map_or(true, |labels| labels.contains(&s.label))
        })
        .collect();

    let mut handles = VecDeque::new();

    for system in systems.clone() {
        let config = config.clone();
        handles.push_back(spawn(async { analyze_system(config, system) }));
    }

    let mut table_rows: Vec<TableRow> = Vec::new();

    for system in systems {
        let analysis = handles.pop_front().unwrap().await.unwrap();

        table_rows.push(TableRow {
            system_str: system.label,
            num_games: analysis.num_games,
            file_size: analysis.file_size,
        });
    }

    let mut table = Table::new(table_rows);

    table.with(Style::psql());

    println!("{table}");
}

fn analyze_system(config: Config, system: System) -> Analysis {
    let mut analysis = Analysis::default();

    let system_path = format!(
        "{}/{}",
        config.archive_root.clone(),
        system.directory.as_str()
    );

    // Call this for each file.

    for entry in Path::new(&system_path)
        .read_dir()
        .unwrap()
        .filter_map(Result::ok)
        .filter(|e| !e.path().to_string_lossy().contains("!bios"))
    {
        let file_size = entry.metadata().unwrap().len();

        analysis.file_size += file_size;

        // If games are represented as directories,
        // prevent normal files from incrementing game count.
        if system.games_are_directories && entry.path().is_file() {
            continue;
        }

        if system.games_are_directories && entry.path().is_dir() {
            // Iterate over this game's multiple parts.
            for part in Path::new(&entry.path())
                .read_dir()
                .unwrap()
                .filter_map(Result::ok)
            {
                let file_size = part.metadata().unwrap().len();
                analysis.file_size += file_size;
            }
        }

        // Increment this system's game count.
        analysis.num_games += 1;
    }

    analysis
}

fn bytes_to_gigabytes(bytes: u64) -> f32 {
    bytes as f32 / 1_073_741_824.0
}
