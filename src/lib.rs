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

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
enum ColumnType {
    GameCount,
    FileSize,
}

impl ColumnType {
    fn get_header(&self) -> String {
        match self {
            Self::GameCount => "Games".to_string(),
            Self::FileSize => "Size".to_string(),
        }
    }

    fn get_width(&self) -> usize {
        self.get_header().len()
    }
}

#[derive(Debug, Clone)]
struct Column {
    header: String,
    col_type: ColumnType,
    values: HashMap<System, u64>,
}

impl Column {
    fn new(header: &str, col_type: ColumnType) -> Self {
        Column {
            header: header.to_string(),
            col_type,
            values: HashMap::new(),
        }
    }

    fn calc_width(&self) -> usize {
        if self.values.len() == 0 {
            return 0;
        }

        max(
            self.header.len(),
            self.values.values().map(|v| v.to_string().len()).max().unwrap()
        )
    }

    fn get_total(&self) -> u64 {
        self.values.values().sum()
    }
    
    fn add(&mut self, system: System, value: u64) {
        self.values.entry(system.clone()).and_modify(|v| *v += value);
    }
}

fn calc_width(col_type: &ColumnType, values: ArcMutexHashmap<System, u64>) -> usize {
    max(
        col_type.get_width(),
        values.lock().unwrap().values().map(|v| v.to_string().len()).max().unwrap()
    )
}

fn bytes_to_gigabytes(bytes: u64) -> f32 {
    bytes as f32 / 1_000_000_000.0
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

        // let add_if_exists = |col_type: ColumnType, value: u64| {
        //     columns.lock().unwrap().iter_mut().find(|h| h.col_type == col_type).unwrap().add(system.clone(), value);
        // };

        for entry in walk_archive() {
            let file_size = entry.metadata().unwrap().len();

            columns.lock().unwrap().iter_mut().find(|h| h.col_type == ColumnType::FileSize).unwrap().add(system.clone(), file_size);

            // if games are represented as directories,
            // increment game count only once per directory
            if system.games_are_directories && entry.path().is_file() {
                continue;
            }

            // add_if_exists(ColumnType::GameCount, file_size);
        }
    })
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

    for column in columns.iter_mut() {
        for system in systems.iter() {
            column.values.entry(system.clone()).or_insert(0);
        }
    }

    let mut children_threads: Vec<JoinHandle<()>> = Vec::with_capacity(systems.len());

    let columns = Arc::new(Mutex::new(columns));

    for system in &systems {
        children_threads.push(
            create_thread(Arc::clone(&config), system.clone(), Arc::clone(&columns))
        );
    }

    let columns = columns.lock().unwrap();

    for thread in children_threads {
        thread.join().expect("Child thread has panicked");
    }

    let styled_header = |text: &str| -> ColoredString {
        text.underline().white()
    };

    {
        let width = systems.iter().map(|s| s.pretty_string.len()).max().unwrap() + 2;
        let header = styled_header("System");
        print!("{header}{}", " ".repeat(2 + width - header.len()));
    }

    for col in columns.iter() {
        let width = col.calc_width();
        let header = styled_header(&col.header);
        print!("{header}{}", " ".repeat(2));
    }
    println!();

    dbg!(&columns);

    let name_width = systems.iter().map(|s| s.pretty_string.len()).max().unwrap() + 2;
    for system in systems.iter() {
        print!("{: <name_width$}", system.pretty_string);
        for col in columns.iter() {
            let numero = col.values.get(system).unwrap();
            let width = col.calc_width();
            print!("{numero: <width$}");
        }
    }

    // for system in all_systems_stats() {
    //     let game_count = data.lock().unwrap()
    //         .get(system.as_ref()).unwrap()
    //         .get(&ColumnType::GameCount).unwrap().clone();

    //     let file_size = data.lock().unwrap()
    //         .get(system.as_ref()).unwrap()
    //         .get(&ColumnType::FileSize).unwrap().clone();

    //     println!("{: <col_1_width$}{game_count: <col_2_width$}{:.2}G",
    //     system.pretty_string,
    //     bytes_to_gigabytes(file_size));
    // }

    // println!("{: <col_1_width$}{: <col_2_width$}{:.2}G", " ",
    // totals.0,
    // bytes_to_gigabytes(totals.1),
// );
}
