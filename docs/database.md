# Database Support

wxlistener now supports storing weather data in PostgreSQL or MySQL databases. Each weather reading is automatically stored as a new row with a timestamp.

## Table of Contents

- [Features](#features)
- [Configuration](#configuration)
  - [Option 1: Connection String](#option-1-connection-string)
  - [Option 2: Individual Fields](#option-2-individual-fields)
- [Database Schema](#database-schema)
- [Usage](#usage)
  - [Manual Table Creation](#manual-table-creation)
    - [Option 1: Using the --db-create-table flag](#option-1-using-the---db-create-table-flag)
    - [Option 2: Using SQL scripts directly](#option-2-using-sql-scripts-directly)
    - [Option 3: Manual SQL execution](#option-3-manual-sql-execution)
  - [Creating a Database User](#creating-a-database-user)
- [Example Queries](#example-queries)
  - [Get latest reading](#get-latest-reading)
  - [Get average temperature for today](#get-average-temperature-for-today)
  - [Get hourly rainfall](#get-hourly-rainfall)
- [Troubleshooting](#troubleshooting)
  - [Connection fails](#connection-fails)
  - [Table creation fails](#table-creation-fails)
  - [Data not being inserted](#data-not-being-inserted)
- [Performance](#performance)
- [Security Notes](#security-notes)

## Features

- **Automatic table creation** - The weather data table is created automatically on first run
- **PostgreSQL and MySQL support** - Works with both major database systems
- **Flexible configuration** - Configure via connection string or individual fields
- **Automatic filtering** - The `heap_free` field is automatically excluded from storage
- **Non-blocking** - Database writes happen asynchronously without blocking data collection

## Configuration

Add a `[database]` section to your `wxlistener.toml` configuration file.

### Option 1: Connection String

The simplest way to configure database access:

```toml
[database]
connection_string = "postgres://username:password@localhost:5432/weather"
table_name = "wx_records"  # optional, defaults to "wx_records"
```

For MySQL:

```toml
[database]
connection_string = "mysql://username:password@localhost:3306/weather"
table_name = "wx_records"
```

### Option 2: Individual Fields

For more explicit configuration:

```toml
[database]
db_type = "postgres"  # or "mysql"
host = "localhost"
port = 5432           # optional, defaults: postgres=5432, mysql=3306
username = "myuser"
password = "mypass"
database = "weather"
table_name = "wx_records"  # optional; this is the default table name (can be overridden)
```

## Database Schema

The table is created automatically with the following columns:

| Column         | Type                      | Description                        |
| -------------- | ------------------------- | ---------------------------------- |
| `id`           | SERIAL/INT AUTO_INCREMENT | Primary key                        |
| `timestamp`    | TIMESTAMP                 | When the reading was taken         |
| `intemp`       | DOUBLE                    | Indoor temperature (°C)            |
| `outtemp`      | DOUBLE                    | Outdoor temperature (°C)           |
| `dewpoint`     | DOUBLE                    | Dew point (°C)                     |
| `windchill`    | DOUBLE                    | Wind chill (°C)                    |
| `heatindex`    | DOUBLE                    | Heat index (°C)                    |
| `inhumid`      | DOUBLE                    | Indoor humidity (%)                |
| `outhumid`     | DOUBLE                    | Outdoor humidity (%)               |
| `absbarometer` | DOUBLE                    | Absolute barometric pressure (hPa) |
| `relbarometer` | DOUBLE                    | Relative barometric pressure (hPa) |
| `wind_dir`     | DOUBLE                    | Wind direction (degrees)           |
| `wind_speed`   | DOUBLE                    | Wind speed (m/s)                   |
| `gust_speed`   | DOUBLE                    | Gust speed (m/s)                   |
| `rain_event`   | DOUBLE                    | Rain event total (mm)              |
| `rain_rate`    | DOUBLE                    | Rain rate (mm)                     |
| `rain_day`     | DOUBLE                    | Daily rain total (mm)              |
| `rain_week`    | DOUBLE                    | Weekly rain total (mm)             |
| `rain_month`   | DOUBLE                    | Monthly rain total (mm)            |
| `rain_year`    | DOUBLE                    | Yearly rain total (mm)             |
| `light`        | DOUBLE                    | Light intensity (lux)              |
| `uv`           | DOUBLE                    | UV radiation                       |
| `uvi`          | DOUBLE                    | UV index                           |
| `day_max_wind` | DOUBLE                    | Daily maximum wind speed (m/s)     |

**Note:** The `heap_free` field from the weather station is not stored in the database.

## Usage

1. Set up your PostgreSQL or MySQL database
2. Create a database for weather data (e.g., `CREATE DATABASE weather;`)
3. Add database configuration to `wxlistener.toml`
4. (Optional) Create the table manually:
   ```bash
   wxlistener --config wxlistener.toml --db-create-table
   ```
5. Run wxlistener: `wxlistener --config wxlistener.toml`

The tool will:

- Connect to the database
- Check if the table exists
- If the table doesn't exist, prompt you to create it (Y/n)
- Start collecting and storing weather data
- Continue running even if database writes fail (errors are logged)

### Manual Table Creation

There are three ways to create the database table:

#### Option 1: Using the --db-create-table flag

```bash
wxlistener --config wxlistener.toml --db-create-table
```

This will connect to the configured database, create the table, and exit immediately.

#### Option 2: Using SQL scripts directly

Pre-made SQL scripts are available in `docs/sql-examples/`:

**PostgreSQL:**

```bash
# Using psql
psql -U postgres -d weather -f docs/sql-examples/postgres.sql

# Or interactively
psql -U postgres -d weather
\i docs/sql-examples/postgres.sql
```

**MySQL:**

```bash
# Using mysql client
mysql -u root -p weather < docs/sql-examples/mysql.sql

# Or interactively
mysql -u root -p weather
source docs/sql-examples/mysql.sql
```

#### Option 3: Manual SQL execution

**PostgreSQL:**

```sql
CREATE TABLE IF NOT EXISTS wx_records (
    id SERIAL PRIMARY KEY,
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL,
    intemp DOUBLE PRECISION,
    outtemp DOUBLE PRECISION,
    dewpoint DOUBLE PRECISION,
    windchill DOUBLE PRECISION,
    heatindex DOUBLE PRECISION,
    inhumid DOUBLE PRECISION,
    outhumid DOUBLE PRECISION,
    absbarometer DOUBLE PRECISION,
    relbarometer DOUBLE PRECISION,
    wind_dir DOUBLE PRECISION,
    wind_speed DOUBLE PRECISION,
    gust_speed DOUBLE PRECISION,
    rain_event DOUBLE PRECISION,
    rain_rate DOUBLE PRECISION,
    rain_day DOUBLE PRECISION,
    rain_week DOUBLE PRECISION,
    rain_month DOUBLE PRECISION,
    rain_year DOUBLE PRECISION,
    light DOUBLE PRECISION,
    uv DOUBLE PRECISION,
    uvi DOUBLE PRECISION,
    day_max_wind DOUBLE PRECISION
);

-- Create an index on timestamp for faster queries
CREATE INDEX IF NOT EXISTS idx_wx_records_timestamp ON wx_records(timestamp DESC);
```

**MySQL:**

```sql
CREATE TABLE IF NOT EXISTS wx_records (
    id INT AUTO_INCREMENT PRIMARY KEY,
    timestamp TIMESTAMP NOT NULL,
    intemp DOUBLE,
    outtemp DOUBLE,
    dewpoint DOUBLE,
    windchill DOUBLE,
    heatindex DOUBLE,
    inhumid DOUBLE,
    outhumid DOUBLE,
    absbarometer DOUBLE,
    relbarometer DOUBLE,
    wind_dir DOUBLE,
    wind_speed DOUBLE,
    gust_speed DOUBLE,
    rain_event DOUBLE,
    rain_rate DOUBLE,
    rain_day DOUBLE,
    rain_week DOUBLE,
    rain_month DOUBLE,
    rain_year DOUBLE,
    light DOUBLE,
    uv DOUBLE,
    uvi DOUBLE,
    day_max_wind DOUBLE,
    INDEX idx_timestamp (timestamp DESC)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
```

Manual table creation is useful for:

- Pre-creating tables with specific permissions
- Verifying database connectivity
- Setting up the schema before running the listener
- Non-interactive environments (scripts, Docker, etc.)
- Custom table modifications or additional indexes

### Creating a Database User

For security, create a dedicated database user for wxlistener with minimal permissions:

**PostgreSQL:**

```sql
-- Create user
CREATE USER wxlistener WITH PASSWORD 'your_secure_password';

-- Grant database connection
GRANT CONNECT ON DATABASE weather TO wxlistener;

-- Grant table permissions
GRANT SELECT, INSERT ON wx_records TO wxlistener;

-- Grant sequence permissions (for auto-increment)
GRANT USAGE, SELECT ON SEQUENCE wx_records_id_seq TO wxlistener;
```

**MySQL:**

```sql
-- Create user
CREATE USER 'wxlistener'@'localhost' IDENTIFIED BY 'your_secure_password';

-- Grant table permissions
GRANT SELECT, INSERT ON weather.wx_records TO 'wxlistener'@'localhost';

-- Apply changes
FLUSH PRIVILEGES;
```

Then update your configuration to use this user:

```toml
[database]
connection_string = "postgres://wxlistener:your_secure_password@localhost:5432/weather"
# or for MySQL:
# connection_string = "mysql://wxlistener:your_secure_password@localhost:3306/weather"
```

## Example Queries

### Get latest reading

```sql
SELECT * FROM wx_records ORDER BY timestamp DESC LIMIT 1;
```

### Get average temperature for today

```sql
SELECT AVG(outtemp) as avg_temp
FROM wx_records
WHERE timestamp >= CURRENT_DATE;
```

### Get hourly rainfall

```sql
SELECT
    DATE_TRUNC('hour', timestamp) as hour,
    MAX(rain_day) - MIN(rain_day) as rainfall
FROM wx_records
GROUP BY DATE_TRUNC('hour', timestamp)
ORDER BY hour DESC;
```

## Troubleshooting

### Connection fails

- Verify database credentials
- Check that the database exists
- Ensure the database server is running and accessible
- Check firewall rules

### Table creation fails

- Verify the user has CREATE TABLE permissions
- Check database logs for specific errors

### Data not being inserted

- Check console output for error messages
- Verify the table exists: `\dt` (PostgreSQL) or `SHOW TABLES;` (MySQL)
- Check database user has INSERT permissions

## Performance

- Database writes are asynchronous and non-blocking
- Failed writes are logged but don't stop data collection
- Connection pool size is set to 5 connections
- Each reading generates one INSERT statement

## Security Notes

- Store database credentials securely
- Use strong passwords
- Consider using environment variables for sensitive data
- Restrict database user permissions to only what's needed (CREATE, INSERT, SELECT)
- Use SSL/TLS connections in production environments
