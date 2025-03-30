use self::config::Config;
use arcconfig::{read_config, system::System};
use rayon::prelude::*;
use regex::Regex;
use std::{
    collections::VecDeque,
    path::Path,
    sync::{Arc, Mutex},
};
use tabled::{
    settings::{object::Cell, Color, Style},
    Table, Tabled,
};
use tokio::spawn;

mod config;

#[derive(Copy, Clone, Default)]
struct Analysis {
    num_games: u32,
    file_size: u64,
}

#[derive(Tabled)]
struct TableRow {
    #[tabled(rename = "System")]
    system_str: String,
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
        handles.push_back(spawn(async { analyze_system(config, system) }));
    }

    let mut table_rows: Vec<TableRow> = Vec::new();

    for system in systems.clone() {
        let analysis = handles.pop_front().unwrap().await.unwrap();

        table_rows.push(TableRow {
            system_str: system.pretty_string.input,
            num_games: analysis.num_games,
            file_size: format!("{:.02}", analysis.file_size as f32 / 1_073_741_824.0),
        });
    }

    let mut table = Table::new(table_rows);

    table.with(Style::psql());

    // `colored::ColoredString` causes table formatting issues.
    // We have to style them manually through tabled's API.
    // This just means passing color data from colored to tabled.
    let re = Regex::new(r"38;2;(?<r>\d+);(?<g>\d+);(?<b>\d+)").unwrap();

    for (i, system) in systems.iter().enumerate() {
        let (r, g, b) = {
            let s = &system
                .pretty_string
                .fgcolor
                .unwrap()
                .to_fg_str()
                .into_owned();

            re.captures(s).map_or((255, 255, 255), |caps| {
                (
                    caps.name("r").unwrap().as_str().parse::<u8>().unwrap(),
                    caps.name("g").unwrap().as_str().parse::<u8>().unwrap(),
                    caps.name("b").unwrap().as_str().parse::<u8>().unwrap(),
                )
            })
        };

        table.modify(Cell::new(1 + i, 0), Color::rgb_fg(r, g, b));
    }

    println!("{table}");
}

fn analyze_system(config: Config, system: System) -> Analysis {
    let analysis = Arc::new(Mutex::new(Analysis::default()));

    let system_path = format!("{}/{}", config.archive_root, system.directory.as_str());

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

    let a = analysis.lock().unwrap();

    Analysis {
        num_games: a.num_games,
        file_size: a.file_size,
    }
}
