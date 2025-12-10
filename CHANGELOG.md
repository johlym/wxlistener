# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.2] - 2025-12-10

### Added

- Database support for PostgreSQL and MySQL
- Interactive table creation prompt when table doesn't exist
- `--db-create-table` flag for non-interactive table creation
- Configurable table name (defaults to `wx_records`)
- Support for both connection string and individual field database configuration

### Changed

- Updated sqlx from 0.7 to 0.8
- Default table name changed from `weather_data` to `wx_records`
- Database table is no longer created automatically; user is prompted

### Fixed

- Removed future Rust compatibility warnings from sqlx-postgres

## [0.1.1] - 2025-12-10

### Added

- Initial database configuration support
- Table creation functionality

## [0.1.0] - 2025-12-09

### Added

- Initial release
- GW1000/Ecowitt Gateway weather station support
- Text and JSON output formats
- Continuous monitoring mode
- Web interface with WebSocket updates
- Docker support
- Configuration file support
