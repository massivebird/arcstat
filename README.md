# arcstat

> short for "archive search"

My command line querying utility for summarizing video game archives!

🦀 written in Rust

<p align="center">
  <img height="75%" src="https://github.com/massivebird/arcstat/assets/33991373/fee88b87-a399-4d81-99d9-bca09e22f4b8" />
</p>

## What does arcstat do?

Arcstat provides a summary of each game system in your archive, which includes per system:

+ Number of games
+ Cumulative storage size

## Building the project

```bash
git clone https://github.com/massivebird/arcstat
cd arcstat
cargo run
```

## Usage

Arcstat finds the root of your archive using the environment variable `VG_ARCHIVE`. You can set this during testing like so:

```bash
VG_ARCHIVE="path/to/archive/root" cargo run
```

### Customization

Arcstat looks for a `config.yaml` file in the root of your archive. This configuration file tells arcstat where and how to look for games!

> For a quickstart on YAML syntax, click [here](https://docs.ansible.com/ansible/latest/reference_appendices/YAMLSyntax.html).

Here is an example configuration:

```yaml
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
