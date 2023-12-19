# arcstat

> short for "archive search"

My command line utility for analyzing video game archives!

🦀 written in Rust

<p align="center">
  <img height="75%" src="https://github.com/massivebird/arcstat/assets/33991373/fee88b87-a399-4d81-99d9-bca09e22f4b8" />
</p>

## What does arcstat do?

Arcstat provides a summary of each game system in your archive, which includes per system:

+ Number of games
+ Cumulative storage size
## Building

To manually build the project, you must first [install Rust](https://www.rust-lang.org/tools/install).

Once you have Rust installed, run the following commands:

```bash
git clone https://github.com/massivebird/arcstat
cd arcstat
cargo run # runs unoptimized build
```

### Adding arcstat to your PATH

If you want to add arcstat to your PATH, I recommend building it in release mode for better optimization.

```bash
cd arcstat
# build release mode
cargo build --release
# add arcstat to your PATH
ln -rs ./target/release/arcstat <dir-in-PATH>/arcstat
# run arcstat
arcstat
```

## Usage

Basic arcstat syntax is simple! You can run it without any arguments:

```bash
arcstat
```

For information on all its optional arguments, run `arcstat --help`.

### Locating your archive

To find your archive, arcstat defaults to reading the environment variable `VG_ARCHIVE`.

You can also provide this path from the command line:

```bash
arcstat --archive-path /path/to/archive
```

### Customization

Arcstat looks for a `config.yaml` file in the root of your archive. This configuration file tells arcstat where and how to look for games!

> For a quickstart on YAML syntax, click [here](https://docs.ansible.com/ansible/latest/reference_appendices/YAMLSyntax.html).

Here is an example configuration:

```yaml
# config.yaml
systems:
  ds: # system "label" — call it whatever you want!
    display_name: "DS"
    color: [135,215,255]
    path: "ds" # path relative to archive root
    games_are_directories: false # are games stored as directories?
  snes:
    display_name: "SNES"
    color: [95,0,255]
    path: "snes"
    games_are_directories: false
  wii:
    display_name: "WII"
    color: [0,215,255]
    path: "wbfs"
    games_are_directories: true
```
