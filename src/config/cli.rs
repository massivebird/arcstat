use clap::{Arg, ValueHint, Command, command};

pub fn build_args() -> Command {
    let desired_systems_long_help = "\
        Query specific systems via comma-seperated system labels.\
";

    let archive_root_long_help = "\
        Provide the path to your archive root, overriding the VG_ARCHIVE environment variable if it exists.\
";

    command!()
        .arg(Arg::new("desired_systems")
            .short('s')
            .long("systems")
            .help("Comma-separated system labels to analyze exclusively")
            .long_help(desired_systems_long_help)
            .value_name("labels")
        )
        .arg(Arg::new("archive_root")
            .short('r')
            .long("archive-root")
            .alias("archive-path")
            .help("The root of your game archive")
            .long_help(archive_root_long_help)
            .value_name("PATH")
            .value_hint(ValueHint::DirPath)
        )
}
