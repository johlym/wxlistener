# Changelog

## [0.5.1] - 2026-10-18

### Fixed

- HTTP POST always includes `uv`. The ingest API requires it, and GW3000 livedata often sends UVI without UV radiation. A real `uv` value is still sent when the station provides one.

## [0.5.0] - 2026-10-15

### Added

- Local Ecowitt HTTP API support for GW3000, GW1100, and GW2000. `get_livedata_info` is converted into the existing metric readings (°C, hPa, m/s, mm, lux).
- `rain_source` config (`rain` or `piezo`) to choose which rain array fills `rain_*`.
- Battery-level keys `soil_battery_level_chN` and `tf_battery_level_chN`, plus HTTP POST `battery_level` and database columns. Voltage stays on the existing `*_battery_chN` keys.
- `--debug-sensors` inventory output for the HTTP API via `get_sensors_info`.

### Changed

- The HTTP API is the default transport on port 80. `--legacy`, `legacy = true`, or `WXLISTENER_LEGACY=1` selects the old TCP client on port 45000.
- Port 45000 without legacy, or port 80 with legacy, fails at startup.
- `W/m²` light values convert with `lux = (W/m²) / 0.0079`.
- Channel readings are emitted only when `get_sensors_info` shows the sensor is registered (`id` + `idst`). `signal` is reception quality and does not hide a live channel.

### Upgrade notes

- Configs and environments that set port `45000` must set legacy mode or change the port to 80.
- Existing database tables need `docs/sql-examples/alter-add-battery-level-postgres.sql` or `docs/sql-examples/alter-add-battery-level-mysql.sql` before inserts that include the new level keys.
