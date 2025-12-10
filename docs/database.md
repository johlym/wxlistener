# Database Support

wxlistener now supports storing weather data in PostgreSQL or MySQL databases. Each weather reading is automatically stored as a new row with a timestamp.

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
- Create the table if it doesn't exist (unless already created)
- Start collecting and storing weather data
- Continue running even if database writes fail (errors are logged)

### Manual Table Creation

To create the database table without starting the listener, use the `--db-create-table` flag:

```bash
wxlistener --config wxlistener.toml --db-create-table
```

This will:

- Connect to the configured database
- Create the table with the appropriate schema
- Exit immediately

This is useful for:

- Pre-creating tables with specific permissions
- Verifying database connectivity
- Setting up the schema before running the listener

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
