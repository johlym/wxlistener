-- Add HTTP API battery-level columns to an existing MySQL table.
-- Voltage stays in soil_battery_ch* and tf_battery_ch*. These columns store the gateway level.
-- Usage: mysql -u root -p weather < docs/sql-examples/alter-add-battery-level-mysql.sql

ALTER TABLE wx_records ADD COLUMN soil_battery_level_ch1 DOUBLE;
ALTER TABLE wx_records ADD COLUMN soil_battery_level_ch2 DOUBLE;
ALTER TABLE wx_records ADD COLUMN soil_battery_level_ch3 DOUBLE;
ALTER TABLE wx_records ADD COLUMN soil_battery_level_ch4 DOUBLE;
ALTER TABLE wx_records ADD COLUMN soil_battery_level_ch5 DOUBLE;
ALTER TABLE wx_records ADD COLUMN soil_battery_level_ch6 DOUBLE;
ALTER TABLE wx_records ADD COLUMN soil_battery_level_ch7 DOUBLE;
ALTER TABLE wx_records ADD COLUMN soil_battery_level_ch8 DOUBLE;
ALTER TABLE wx_records ADD COLUMN tf_battery_level_ch1 DOUBLE;
ALTER TABLE wx_records ADD COLUMN tf_battery_level_ch2 DOUBLE;
ALTER TABLE wx_records ADD COLUMN tf_battery_level_ch3 DOUBLE;
ALTER TABLE wx_records ADD COLUMN tf_battery_level_ch4 DOUBLE;
ALTER TABLE wx_records ADD COLUMN tf_battery_level_ch5 DOUBLE;
ALTER TABLE wx_records ADD COLUMN tf_battery_level_ch6 DOUBLE;
ALTER TABLE wx_records ADD COLUMN tf_battery_level_ch7 DOUBLE;
ALTER TABLE wx_records ADD COLUMN tf_battery_level_ch8 DOUBLE;
