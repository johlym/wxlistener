# wxlistener Architecture

## Table of Contents

- [Project Structure](#project-structure)
- [Module Responsibilities](#module-responsibilities)
  - [`main.rs`](#mainrs)
  - [`client.rs`](#clientrs)
  - [`config.rs`](#configrs)
  - [`decoder.rs`](#decoderrs)
  - [`output.rs`](#outputrs)
- [Data Flow](#data-flow)
- [Protocol Overview](#protocol-overview)
  - [Packet Structure](#packet-structure)
  - [Response Structure](#response-structure)
- [Key Design Decisions](#key-design-decisions)
- [Adding New Features](#adding-new-features)
  - [Adding a New Measurement Field](#adding-a-new-measurement-field)
  - [Adding a New Command](#adding-a-new-command)
  - [Adding Output Format](#adding-output-format)
- [Testing](#testing)
- [Dependencies](#dependencies)

## Project Structure

```
src/
├── main.rs         - Entry point and main application logic
├── gateway.rs      - HTTP or legacy TCP selection
├── http_gateway.rs - Local HTTP API client and unit conversion
├── client.rs       - GW1000Client implementation (legacy TCP)
├── config.rs       - Command-line arguments and config file parsing
├── decoder.rs      - Binary data decoding functions
└── output.rs       - Output formatting (text and JSON)
```

## Module Responsibilities

### `main.rs`

- Application entry point
- Orchestrates the flow: parse args → connect → fetch data → display
- Handles continuous mode polling loop
- Minimal logic, delegates to other modules

### `gateway.rs` and `http_gateway.rs`

- Default transport is the local HTTP API (`GET /get_livedata_info?` on port 80)
- `--legacy`, TOML `legacy`, or `WXLISTENER_LEGACY` selects `GW1000Client`
- Port 45000 without legacy, or port 80 with legacy, fails at startup
- Display units are converted back to the TCP metric keys. `W/m²` uses `lux = (W/m²) / 0.0079`
- `rain_source` chooses the `rain` or `piezoRain` array
- Channel rows are emitted only when `get_sensors_info` shows the sensor registered

### `client.rs`

- `GW1000Client` struct - legacy TCP connection to the weather station
- Protocol implementation:
  - `build_cmd_packet()` - constructs binary command packets
  - `send_cmd()` - TCP socket communication
  - `check_response()` - validates responses (header, checksum)
- API methods:
  - `get_firmware_version()` - device firmware info
  - `get_mac_address()` - device MAC address
  - `get_livedata()` - retrieves all weather measurements
- `parse_livedata()` - parses binary response into HashMap

### `config.rs`

- `Args` struct - CLI argument definitions using clap
- `Config` struct - TOML configuration file structure
- `get_connection_info()` - resolves IP/port from args or config file
- Handles both command-line and file-based configuration

### `decoder.rs`

- Pure functions for decoding binary data:
  - `decode_temp()` - signed 16-bit temperature (÷10)
  - `decode_short()` - unsigned 16-bit integer
  - `decode_int()` - unsigned 32-bit integer
  - `decode_wind()` - wind speed (÷10)
  - `decode_rain()` - rainfall (÷10)
  - `decode_pressure()` - barometric pressure (÷10)
- No state, easily testable

### `output.rs`

- `print_livedata()` - formats and displays weather data
- `format_value()` - applies units and formatting per field type
- Handles human-readable text output
- JSON output handled in main.rs using serde_json

## Data Flow

```
CLI Args/Config File
        ↓
    main.rs (parse)
        ↓
    config.rs (resolve IP, port, legacy, rain source)
        ↓
    gateway.rs (HTTP API, or legacy TCP client.rs)
        ↓
    http_gateway.rs or decoder.rs (metric HashMap)
        ↓
    output.rs (format for display)
        ↓
    Terminal Output
```

## Protocol Overview

### Packet Structure

```
[Header] [Command] [Size] [Payload] [Checksum]
  2B        1B       1B     0-N B       1B
```

- **Header**: Always `0xFF 0xFF`
- **Command**: API command code (e.g., `0x27` for live data)
- **Size**: Total size of command + size + payload + checksum
- **Payload**: Command-specific data (often empty for queries)
- **Checksum**: LSB of sum of all bytes from command through payload

### Response Structure

```
[Header] [Command] [Size] [Data...] [Checksum]
  2B        1B      1-2B    N bytes      1B
```

- Live data (`0x27`) uses 2-byte size field (big-endian)
- Other commands use 1-byte size field
- Data section contains address-value pairs for measurements

## Key Design Decisions

1. **Modular Structure**: Each module has a single, clear responsibility
2. **Error Handling**: Uses `anyhow::Result` for propagating errors
3. **No Global State**: All state contained in structs
4. **Pure Decoders**: Decoding functions are stateless and testable
5. **Flexible Config**: Supports both CLI args and TOML config files
6. **Zero Dependencies at Runtime**: Compiles to standalone binary

## Adding New Features

### Adding a New Measurement Field

1. Add field address constant to `client.rs`
2. Add match arm in `parse_livedata()`
3. Add decoder function to `decoder.rs` if needed
4. Add formatting rule to `output.rs::format_value()`

### Adding a New Command

1. Add command constant to `client.rs`
2. Create method like `get_X()` following existing pattern
3. Implement response parsing
4. Call from `main.rs` as needed

### Adding Output Format

1. Add format option to `config.rs::Args`
2. Handle format in `main.rs` continuous/single read sections
3. Create formatter in `output.rs` or inline in main

## Testing

```bash
# Build
cargo build --release

# Run with config file
./target/release/wxlistener --config wxlistener.toml

# Run with CLI args
./target/release/wxlistener --ip 10.31.100.42
./target/release/wxlistener --ip 10.31.100.42 --legacy

# JSON output
./target/release/wxlistener --ip 10.31.100.42 --format json

# Continuous mode
./target/release/wxlistener --ip 10.31.100.42 --continuous 30

# One-shot sensor inventory (WN34S / ITEM_TF_USR validation)
./target/release/wxlistener --ip 10.31.100.42 --debug-sensors
```

`--debug-sensors` on the legacy TCP client dumps `CMD_READ_SENSOR_ID_NEW` entries (with type names), walks livedata for `ITEM_TF_USR*` / legacy `ITEM_SOILTEMP*`, prints listener-decoded `soil_*` and `tf_*` keys, and reports whether multi-channel WN34/WN34S temp probes are present. On the HTTP API it prints `get_sensors_info` and the mapped channel keys. Livedata temperatures come from `ITEM_TF_USR1`–`8` or HTTP `ch_temp` (`tf_temp_chN` / `tf_battery_chN` volts, plus `tf_battery_level_chN`); WH51 remains moisture-only under `soil_*`, with `soil_battery_level_chN` for the HTTP battery level.

## Dependencies

- `clap` - Command-line argument parsing
- `serde` / `serde_json` - Serialization for JSON output
- `toml` - TOML config file parsing
- `chrono` - Timestamp formatting
- `anyhow` - Error handling

All dependencies are compile-time only. The resulting binary has no runtime dependencies.
