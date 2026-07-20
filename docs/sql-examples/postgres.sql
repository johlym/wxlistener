-- PostgreSQL table creation script for wxlistener
-- This creates the wx_records table for storing weather station data

-- Create the database (if needed)
-- CREATE DATABASE weather;
-- \c weather;

-- Create the wx_records table
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
    day_max_wind DOUBLE PRECISION,
    -- Soil moisture (WH51) channels 1–8, percent
    soil_moisture_ch1 DOUBLE PRECISION,
    soil_moisture_ch2 DOUBLE PRECISION,
    soil_moisture_ch3 DOUBLE PRECISION,
    soil_moisture_ch4 DOUBLE PRECISION,
    soil_moisture_ch5 DOUBLE PRECISION,
    soil_moisture_ch6 DOUBLE PRECISION,
    soil_moisture_ch7 DOUBLE PRECISION,
    soil_moisture_ch8 DOUBLE PRECISION,
    -- Legacy ITEM_SOILTEMP channels 1–8, °C (optional)
    soil_temp_ch1 DOUBLE PRECISION,
    soil_temp_ch2 DOUBLE PRECISION,
    soil_temp_ch3 DOUBLE PRECISION,
    soil_temp_ch4 DOUBLE PRECISION,
    soil_temp_ch5 DOUBLE PRECISION,
    soil_temp_ch6 DOUBLE PRECISION,
    soil_temp_ch7 DOUBLE PRECISION,
    soil_temp_ch8 DOUBLE PRECISION,
    -- WH51 battery voltage (volts)
    soil_battery_ch1 DOUBLE PRECISION,
    soil_battery_ch2 DOUBLE PRECISION,
    soil_battery_ch3 DOUBLE PRECISION,
    soil_battery_ch4 DOUBLE PRECISION,
    soil_battery_ch5 DOUBLE PRECISION,
    soil_battery_ch6 DOUBLE PRECISION,
    soil_battery_ch7 DOUBLE PRECISION,
    soil_battery_ch8 DOUBLE PRECISION,
    -- Legacy soil_temp battery columns (unused by current WN34S path)
    soil_temp_battery_ch1 DOUBLE PRECISION,
    soil_temp_battery_ch2 DOUBLE PRECISION,
    soil_temp_battery_ch3 DOUBLE PRECISION,
    soil_temp_battery_ch4 DOUBLE PRECISION,
    soil_temp_battery_ch5 DOUBLE PRECISION,
    soil_temp_battery_ch6 DOUBLE PRECISION,
    soil_temp_battery_ch7 DOUBLE PRECISION,
    soil_temp_battery_ch8 DOUBLE PRECISION,
    -- Multi-channel temp probes (WN34/WN34S via ITEM_TF_USR), °C — same scale as outtemp
    tf_temp_ch1 DOUBLE PRECISION,
    tf_temp_ch2 DOUBLE PRECISION,
    tf_temp_ch3 DOUBLE PRECISION,
    tf_temp_ch4 DOUBLE PRECISION,
    tf_temp_ch5 DOUBLE PRECISION,
    tf_temp_ch6 DOUBLE PRECISION,
    tf_temp_ch7 DOUBLE PRECISION,
    tf_temp_ch8 DOUBLE PRECISION,
    -- WN34/WN34S battery voltage (volts)
    tf_battery_ch1 DOUBLE PRECISION,
    tf_battery_ch2 DOUBLE PRECISION,
    tf_battery_ch3 DOUBLE PRECISION,
    tf_battery_ch4 DOUBLE PRECISION,
    tf_battery_ch5 DOUBLE PRECISION,
    tf_battery_ch6 DOUBLE PRECISION,
    tf_battery_ch7 DOUBLE PRECISION,
    tf_battery_ch8 DOUBLE PRECISION
);

-- Create an index on timestamp for faster queries
CREATE INDEX IF NOT EXISTS idx_wx_records_timestamp ON wx_records(timestamp DESC);

-- Optional: Create a user for wxlistener (replace 'your_password' with a secure password)
-- CREATE USER wxlistener WITH PASSWORD 'your_password';
-- GRANT CONNECT ON DATABASE weather TO wxlistener;
-- GRANT SELECT, INSERT ON wx_records TO wxlistener;
-- GRANT USAGE, SELECT ON SEQUENCE wx_records_id_seq TO wxlistener;

-- Verify the table was created
\d wx_records;

-- Example query to view recent data
-- SELECT * FROM wx_records ORDER BY timestamp DESC LIMIT 10;
