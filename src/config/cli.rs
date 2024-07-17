use clap::{command, Arg, Command, ValueHint};

pub fn build_args() -> Command {
    command!()
        .arg(
            Arg::new("desired_systems")
                .short('s')
                .long("systems")
                .help("Comma-separated system labels to analyze exclusively")
                .long_help(
                    "\
        Query specific systems via comma-seperated system labels.\
",
                )
                .value_name("labels"),
        )
        .arg(
            Arg::new("archive_root")
                .short('r')
                .long("archive-root")
                .alias("archive-path")
                .help("The root of your game archive")
                .long_help("\
        Provide the path to your archive root, overriding the VG_ARCHIVE environment variable if it exists.\
")
                .value_name("PATH")
                .value_hint(ValueHint::DirPath),
        )
}
