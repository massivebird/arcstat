# arcstat

> short for "archive stats"

My command line utility for getting statistics on my video game archive!

🦀 written in Rust

## why

My video game archive has a lot of games across a lot of systems. I wanted to compare each system by number of games and their cumulative file sizes.

## getting started

```cmd
git clone https://github.com/massivebird/arcstat
cd arcstat
```

## usage

### Your archive root

Arcstat finds the root of your archive using the environment variable `VG_ARCHIVE`. You can set this during testing like so:

```cmd
VG_ARCHIVE="path/to/archive/root" cargo run
```

### Your systems

Follow the instructions in the `run` function inside `src/lib.rs` to define your inner archive configuration.

> You can also fork the [archive_systems](https://github.com/massivebird/archive_systems) dependency and customize the `generate_systems` function (but don't forget to update this project's `Cargo.toml`!) I do this out of convenience since my project [arcsearch](https://github.com/massivebird/arcsearch) also depends on my inner archive configuration, so I don't have to define it in two separate projects.

## demos

![demo-arcstat](https://i.imgur.com/63hfWwK.png)
