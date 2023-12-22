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

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
enum ColumnType {
    GameCount,
    FileSize,
}

#[derive(Debug, Clone)]
struct Column {
    header: String,
    col_type: ColumnType,
    data: HashMap<System, u64>,
}

impl Column {
    fn new(header: &str, col_type: ColumnType) -> Self {
        Column {
            header: header.to_string(),
            col_type,
            data: HashMap::new(),
        }
    }

    fn calc_width(&self) -> usize {
        let padding = 2;

        if self.data.len() == 0 {
            return self.header.len();
        }

        let max_data_length = self.data.values()
            .map(|&v| display_data(&self, v).to_string().len())
            .max().unwrap();

        max(
            self.header.len(),
            max_data_length
        ) + padding
    }
    
    fn add(&mut self, system: &System, value: u64) {
        self.data.entry(system.clone()).and_modify(|v| *v += value);
    }
}

fn display_bytes_as_gigabytes(bytes: u64) -> String {
    let gb = bytes as f32 / 1_000_000_000.0;
    format!("{gb:.2}GB")
}

fn create_thread(
    config: Arc<Config>,
    system: System,
    columns: Arc<Mutex<Vec<Column>>>
) -> JoinHandle<()> {
    thread::spawn(move || {
        let walk_archive = || {
            WalkDir::new(config.archive_root.clone() + "/" + system.directory.as_str()).into_iter()
                .filter_map(Result::ok) // silently skip errorful entries
                .filter(|e| !e.path().to_string_lossy().contains("!bios"))
                .skip(1) // skip archive root entry
        };

        let add_if_exists = |col_type: ColumnType, value: u64| {
            columns.lock().unwrap().iter_mut().find(|h| h.col_type == col_type).unwrap().add(&system, value);
        };

        for entry in walk_archive() {
            let file_size = entry.metadata().unwrap().len();

            add_if_exists(ColumnType::FileSize, file_size);

            // if games are represented as directories,
            // increment game count only once per directory
            if system.games_are_directories && entry.path().is_file() {
                continue;
            }

            add_if_exists(ColumnType::GameCount, 1);
        }
    })
}

fn display_data(column: &Column, raw_data: u64) -> String {
    match column.col_type {
        ColumnType::FileSize => display_bytes_as_gigabytes(raw_data),
        ColumnType::GameCount => raw_data.to_string(),
    }
}

pub fn run() {
    let config = Config::new();

    let systems: Vec<System> = read_config(&config.archive_root)
        .into_iter()
        .filter(|s| config.desired_systems.clone().map_or(
            true,
            |labels| labels.contains(&s.label)
        ))
        .collect();

    let config = Arc::new(config);

    let mut columns: Vec<Column> = {
        let mut v: Vec<Column> = Vec::new();
        v.push(Column::new("Games", ColumnType::GameCount));
        v.push(Column::new("Size", ColumnType::FileSize));
        v
    };

    for column in &mut columns {
        for system in &systems {
            column.data.entry(system.clone()).or_insert(0);
        }
    }

    let mut children_threads: Vec<JoinHandle<()>> = Vec::with_capacity(systems.len());

    let columns = Arc::new(Mutex::new(columns));

    for system in &systems {
        children_threads.push(
            create_thread(Arc::clone(&config), system.clone(), Arc::clone(&columns))
        );
    }

    for thread in children_threads {
        thread.join().expect("Child thread has panicked");
    }

    let columns = columns.lock().unwrap();

    let styled_header = |text: &str| -> ColoredString {
        text.underline().white()
    };

    let system_col_header = "System";
    let name_column_width = max(
        system_col_header.len(),
        systems.iter().map(|s| s.pretty_string.len()).max().unwrap(),
    ) + 2;

    {
        let header = styled_header(system_col_header);
        print!("{header}{}", " ".repeat(name_column_width - header.len()));
    }

    for col in columns.iter() {
        let header = styled_header(&col.header);
        print!("{header}{}", " ".repeat(col.calc_width() - header.len()));
    }
    println!();

    for system in &systems {
        print!("{: <name_column_width$}", system.pretty_string);
        for col in columns.iter() {
            let raw_data = *col.data.get(system).unwrap();
            let output = display_data(col, raw_data);
            let width = col.calc_width();
            print!("{output: <width$}");
        }
        println!();
    }
}
