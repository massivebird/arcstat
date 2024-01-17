use clap::{Arg, ValueHint, Command};

pub fn build_args() -> Command {
    clap::command!()
        .arg(Arg::new("desired_systems")
            .long("systems")
            .help("Comma-separated system labels to analyze exclusively")
            .value_name("labels")
        )
        .arg(Arg::new("archive_root")
            .long("archive-root")
            .alias("archive-path")
            .help("The root of your game archive")
            .value_name("PATH")
            .value_hint(ValueHint::DirPath)
        )
}
