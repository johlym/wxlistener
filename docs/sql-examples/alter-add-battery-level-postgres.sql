-- Add HTTP API battery-level columns to an existing PostgreSQL table.
-- Voltage stays in soil_battery_ch* and tf_battery_ch*. These columns store the gateway level.
-- Usage: psql -U postgres -d weather -f docs/sql-examples/alter-add-battery-level-postgres.sql

ALTER TABLE wx_records ADD COLUMN IF NOT EXISTS soil_battery_level_ch1 DOUBLE PRECISION;
ALTER TABLE wx_records ADD COLUMN IF NOT EXISTS soil_battery_level_ch2 DOUBLE PRECISION;
ALTER TABLE wx_records ADD COLUMN IF NOT EXISTS soil_battery_level_ch3 DOUBLE PRECISION;
ALTER TABLE wx_records ADD COLUMN IF NOT EXISTS soil_battery_level_ch4 DOUBLE PRECISION;
ALTER TABLE wx_records ADD COLUMN IF NOT EXISTS soil_battery_level_ch5 DOUBLE PRECISION;
ALTER TABLE wx_records ADD COLUMN IF NOT EXISTS soil_battery_level_ch6 DOUBLE PRECISION;
ALTER TABLE wx_records ADD COLUMN IF NOT EXISTS soil_battery_level_ch7 DOUBLE PRECISION;
ALTER TABLE wx_records ADD COLUMN IF NOT EXISTS soil_battery_level_ch8 DOUBLE PRECISION;
ALTER TABLE wx_records ADD COLUMN IF NOT EXISTS tf_battery_level_ch1 DOUBLE PRECISION;
ALTER TABLE wx_records ADD COLUMN IF NOT EXISTS tf_battery_level_ch2 DOUBLE PRECISION;
ALTER TABLE wx_records ADD COLUMN IF NOT EXISTS tf_battery_level_ch3 DOUBLE PRECISION;
ALTER TABLE wx_records ADD COLUMN IF NOT EXISTS tf_battery_level_ch4 DOUBLE PRECISION;
ALTER TABLE wx_records ADD COLUMN IF NOT EXISTS tf_battery_level_ch5 DOUBLE PRECISION;
ALTER TABLE wx_records ADD COLUMN IF NOT EXISTS tf_battery_level_ch6 DOUBLE PRECISION;
ALTER TABLE wx_records ADD COLUMN IF NOT EXISTS tf_battery_level_ch7 DOUBLE PRECISION;
ALTER TABLE wx_records ADD COLUMN IF NOT EXISTS tf_battery_level_ch8 DOUBLE PRECISION;
