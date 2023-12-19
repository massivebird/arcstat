use std::{env, process};

pub struct Config {
    pub archive_root: String,
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
