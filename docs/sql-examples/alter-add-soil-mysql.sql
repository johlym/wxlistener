-- Migration: add soil sensor columns to an existing wx_records table (MySQL 8.0+)
-- Run once after upgrading wxlistener to a build that emits soil_* fields.
-- New installs should use mysql.sql instead (columns already included).
--
-- Note: MySQL has no ADD COLUMN IF NOT EXISTS. Ignore "Duplicate column" errors
-- if re-running, or check information_schema first.

ALTER TABLE wx_records
  ADD COLUMN soil_moisture_ch1 DOUBLE NULL,
  ADD COLUMN soil_moisture_ch2 DOUBLE NULL,
  ADD COLUMN soil_moisture_ch3 DOUBLE NULL,
  ADD COLUMN soil_moisture_ch4 DOUBLE NULL,
  ADD COLUMN soil_moisture_ch5 DOUBLE NULL,
  ADD COLUMN soil_moisture_ch6 DOUBLE NULL,
  ADD COLUMN soil_moisture_ch7 DOUBLE NULL,
  ADD COLUMN soil_moisture_ch8 DOUBLE NULL,
  ADD COLUMN soil_temp_ch1 DOUBLE NULL,
  ADD COLUMN soil_temp_ch2 DOUBLE NULL,
  ADD COLUMN soil_temp_ch3 DOUBLE NULL,
  ADD COLUMN soil_temp_ch4 DOUBLE NULL,
  ADD COLUMN soil_temp_ch5 DOUBLE NULL,
  ADD COLUMN soil_temp_ch6 DOUBLE NULL,
  ADD COLUMN soil_temp_ch7 DOUBLE NULL,
  ADD COLUMN soil_temp_ch8 DOUBLE NULL,
  ADD COLUMN soil_battery_ch1 DOUBLE NULL,
  ADD COLUMN soil_battery_ch2 DOUBLE NULL,
  ADD COLUMN soil_battery_ch3 DOUBLE NULL,
  ADD COLUMN soil_battery_ch4 DOUBLE NULL,
  ADD COLUMN soil_battery_ch5 DOUBLE NULL,
  ADD COLUMN soil_battery_ch6 DOUBLE NULL,
  ADD COLUMN soil_battery_ch7 DOUBLE NULL,
  ADD COLUMN soil_battery_ch8 DOUBLE NULL,
  ADD COLUMN soil_temp_battery_ch1 DOUBLE NULL,
  ADD COLUMN soil_temp_battery_ch2 DOUBLE NULL,
  ADD COLUMN soil_temp_battery_ch3 DOUBLE NULL,
  ADD COLUMN soil_temp_battery_ch4 DOUBLE NULL,
  ADD COLUMN soil_temp_battery_ch5 DOUBLE NULL,
  ADD COLUMN soil_temp_battery_ch6 DOUBLE NULL,
  ADD COLUMN soil_temp_battery_ch7 DOUBLE NULL,
  ADD COLUMN soil_temp_battery_ch8 DOUBLE NULL;
