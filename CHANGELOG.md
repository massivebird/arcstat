# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.9] - 2025-04-27

### Updated

+ Updated arcconfig dependency => 0.4.0

## [0.1.8] - 2025-01-25

### Added

+ Added [rayon](https://github.com/rayon-rs/rayon) crate for performance improvements.

## [0.1.7] - 2024-01-23

### Changed

+ Updated arcconfig dependency => 0.3.x

## [0.1.6] - 2024-01-20

### Added

+ Added GitHub Actions for simple automated testing
+ Added long help messages invoked with `--help`
+ Added some short option invocations
  + `-r` for `--archive-root`
  + `-s` for `--systems`

### Changed

+ Changed file navigation strategy for improved performance (#14)

### Fixed

+ Fixed byte to gigabyte conversion to be more accurate
+ Fixed incorrect dates in changelog

## [0.1.5] - 2024-01-17

### Changed

+ Updated arcconfig => 0.2.1

## [0.1.4] - 2024-01-16

### Fixed

+ Fixed fatal bug on finding zero games in an archive (#6)

### Documentation

+ Updated demo image
+ Updated README's "Building" and "Usage" sections
+ Added valid archive example to README
+ Added warning about nested system directories
+ Updated project synopsis semantics

## [0.1.3] - 2023-12-19

### Fixed

+ Fixed lifetime management as per `significant_drop_tightening` clippy warning

## [0.1.2] - 2023-12-19

### Changed

+ Changes `--systems` value parsing, now splits by commas and spaces

## [0.1.1] - 2023-12-18

### Fixed

+ Fixes column header padding, now accomodates for length of column header
