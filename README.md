# wxlistener

A fast, standalone command-line tool written in Rust to read live data from GW1000/GW2000/Ecowitt Gateway weather stations.

> [!WARNING]
> This code isn't very good. I don't recommend using it–I built this largely for myself and myself alone.
>
> Do with that information what you will.

## Features

- **Standalone binary** - No runtime dependencies, runs anywhere
- **Fast & efficient** - Written in Rust for maximum performance
- **Config file or CLI args** - Flexible configuration options
- **JSON or text output** - Machine-readable or human-friendly
- **Continuous monitoring** - Poll at regular intervals (default: 5 seconds)
- **Web interface** - Real-time browser dashboard with WebSocket updates
- **Database support** - Store data in PostgreSQL or MySQL databases
- **Supports all GW1000/GW2000 devices** - Compatible with Ecowitt Gateway API
- **Docker support** - Run in containers for easy deployment

## TODO

- [ ] Support sending metrics to a third party API endpoint
- [ ] Prescribe the format for said metrics-sending

## Installation

### Pre-built Binary

Download the compiled `wxlistener` binary from the releases page (coming soon) or build from source.

### Build from Source

```bash
# Clone the repository
git clone <your-repo>
cd listener

# Build release binary
cargo build --release

# Binary will be at: ./target/release/wxlistener
# Copy it anywhere in your PATH
sudo cp target/release/wxlistener /usr/local/bin/
```

### Docker

**Runs in continuous mode by default** - just set your device IP and go!

```bash
# Build the image
bin/docker-build

# Set your device IP
export WXLISTENER_IP=10.31.100.42

# Run (continuous mode by default)
docker run --network host -e WXLISTENER_IP wxlistener:latest

# Or use docker-compose
cp .env.example .env  # Edit with your IP
docker-compose up
```

**Environment Variables:**

- `WXLISTENER_IP` - Your weather station IP (required)
- `WXLISTENER_PORT` - Port (default: 45000)
- `WXLISTENER_INTERVAL` - Polling interval in seconds (default: 60)
- `WXLISTENER_FORMAT` - Output format: `text` or `json` (default: text)

See [docs/docker.md](docs/docker.md) for detailed Docker documentation.

## Usage

### Command Line Arguments

```bash
# Using IP address directly
wxlistener --ip 10.31.100.42

# Using custom port
wxlistener --ip 10.31.100.42 --port 45000

# Using config file
wxlistener --config wxlistener.toml

# JSON output
wxlistener --ip 10.31.100.42 --format json

# Continuous monitoring (poll every 30 seconds)
wxlistener --ip 10.31.100.42 --continuous 30

# Web interface mode (default port 18888)
wxlistener --ip 10.31.100.42 --web

# Web interface with custom port
wxlistener --ip 10.31.100.42 --web --web-port 8080

# Web interface with custom host binding
wxlistener --ip 10.31.100.42 --web --web-host 127.0.0.1
```

### Web Interface

The web interface provides a real-time dashboard that automatically updates every 5 seconds via WebSocket:

```bash
# Start the web server
wxlistener --ip 10.31.100.42 --web

# Open your browser to http://localhost:18888
```

Features:

- **Real-time updates** - Data refreshes automatically every 5 seconds
- **WebSocket connection** - Efficient, low-latency updates
- **Auto-reconnect** - Automatically reconnects if connection is lost
- **Dark theme** - Easy on the eyes for 24/7 monitoring
- **Plain text display** - Simple, readable format with formatted units

### Configuration File

Create a `wxlistener.toml` file:

```toml
# WXListener Configuration File
ip = "10.31.100.42"
port = 45000

# Optional: Database configuration
[database]
# Option 1: Use a connection string
connection_string = "postgres://username:password@localhost:5432/weather"

# Option 2: Use individual fields (if connection_string is not provided)
# db_type = "postgres"  # or "mysql"
# host = "localhost"
# port = 5432           # 5432 for postgres, 3306 for mysql
# username = "myuser"
# password = "mypass"
# database = "weather"

# Table name (optional, defaults to "wx_records")
table_name = "wx_records"
```

Then run:

```bash
wxlistener --config wxlistener.toml
```

#### Database Configuration

The tool supports both PostgreSQL and MySQL databases. You can configure the database in two ways:

**Option 1: Connection String**

```toml
[database]
connection_string = "postgres://user:pass@localhost:5432/weather"
# or
connection_string = "mysql://user:pass@localhost:3306/weather"
table_name = "wx_records"  # optional; this is the default table name
```

**Option 2: Individual Fields**

```toml
[database]
db_type = "postgres"  # or "mysql"
host = "localhost"
port = 5432           # optional, defaults: postgres=5432, mysql=3306
username = "myuser"
password = "mypass"
database = "weather"
table_name = "wx_records"  # optional; this is the default table name
```

The database table will be created automatically with columns matching the weather data fields. The `heap_free` field is excluded from database storage. Each weather reading is stored as a new row with a timestamp.

**Create Table Manually**

To create the database table without starting the listener:

```bash
wxlistener --config wxlistener.toml --db-create-table
```

This will connect to the database, create the table (if it doesn't exist), and exit.

## Output Example

### Text Format (default)

```
============================================================
GW1000/Ecowitt Gateway Weather Station Listener
============================================================
Target device: 10.31.100.42:45000

--- Device Information ---
✓ Firmware Version: GW2000B_V3.1.4
✓ MAC Address: EC:62:60:E0:6E:6F

--- Live Data ---
============================================================
LIVE DATA - 2025-12-09 06:25:48 UTC
============================================================
absbarometer         : 996.0 hPa
day_max_wind         : 6.6 m/s
gust_speed           : 0.5 m/s
heap_free            : 149240 bytes (145.7 KB)
inhumid              : 35%
intemp               : 29.3°C
light                : 0.0 lux
outhumid             : 99%
outtemp              : 12.2°C
rain_day             : 57.9 mm
rain_event           : 77.9 mm
rain_month           : 106.6 mm
rain_rate            : 7.2 mm
rain_week            : 77.9 mm
rain_year            : 882.4 mm
relbarometer         : 993.3 hPa
uv                   : 0
uvi                  : 0
wind_dir             : 109.0 m/s
wind_speed           : 0.1 m/s
============================================================
```

### JSON Format

```bash
wxlistener --ip 10.31.100.42 --format json
```

Returns structured JSON data perfect for parsing or piping to other tools.

## Supported Devices

- GW1000
- GW1100
- GW1200
- GW2000

### Alleged Support

I can't actually verify these.

- WH2650, WH2680, WN1900 (Wi-Fi weather stations)
- WS3800, WS3900, WS3910 (weather station consoles)

## Data Fields

The tool reads and displays:

- **Temperature**: Indoor, outdoor, dew point, wind chill, heat index
- **Humidity**: Indoor and outdoor
- **Pressure**: Absolute and relative barometer
- **Wind**: Speed, direction, gusts, daily max
- **Rain**: Rate, daily, weekly, monthly, yearly totals
- **Light**: UV index, UV radiation, luminosity
- **System**: Device memory usage

## Requirements

- Rust 1.82+ (for building from source)
- Network access to your weather station

## Development Setup

If you want to work on this code, there's a setup script that will configure everything:

```bash
# Clone the repository
git clone <your-repo>
cd listener

# Run the setup script
bin/setup
```

The setup script will:

- Check for Rust installation (and guide you if missing)
- Verify Rust version (>= 1.82)
- Install rustfmt and clippy if needed
- Fetch all dependencies
- Build the project
- Run tests to verify everything works
- Create an example config file
- Make scripts executable

After setup, update `wxlistener.toml` with your device's IP address and you're ready to go!

## Testing

Everything is wrapped in a single command:

```sh
$ bin/test
```

## License

As much as I wanted to license it under MIT, since this is a derivative work of the Python GW1000 driver for WeeWX by Gary Roderick, this work is licensed under the GNU General Public License v3.0. See [LICENSE.md](LICENSE.md) for details.

## Credits

- Original Python driver by Gary Roderick
- Further improved upon by [Ian Millard](https://github.com/Millardiang/weewx-gw1000)
