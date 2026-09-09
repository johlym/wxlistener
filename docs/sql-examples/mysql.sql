-- MySQL table creation script for wxlistener
-- This creates the wx_records table for storing weather station data

-- Create the database (if needed)
-- CREATE DATABASE weather;
-- USE weather;

-- Create the wx_records table
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
    -- Soil moisture (WH51) channels 1–8, percent
    soil_moisture_ch1 DOUBLE,
    soil_moisture_ch2 DOUBLE,
    soil_moisture_ch3 DOUBLE,
    soil_moisture_ch4 DOUBLE,
    soil_moisture_ch5 DOUBLE,
    soil_moisture_ch6 DOUBLE,
    soil_moisture_ch7 DOUBLE,
    soil_moisture_ch8 DOUBLE,
    -- Legacy ITEM_SOILTEMP channels 1–8, °C (optional)
    soil_temp_ch1 DOUBLE,
    soil_temp_ch2 DOUBLE,
    soil_temp_ch3 DOUBLE,
    soil_temp_ch4 DOUBLE,
    soil_temp_ch5 DOUBLE,
    soil_temp_ch6 DOUBLE,
    soil_temp_ch7 DOUBLE,
    soil_temp_ch8 DOUBLE,
    -- WH51 battery voltage (volts)
    soil_battery_ch1 DOUBLE,
    soil_battery_ch2 DOUBLE,
    soil_battery_ch3 DOUBLE,
    soil_battery_ch4 DOUBLE,
    soil_battery_ch5 DOUBLE,
    soil_battery_ch6 DOUBLE,
    soil_battery_ch7 DOUBLE,
    soil_battery_ch8 DOUBLE,
    -- Legacy soil_temp battery columns (unused by current WN34S path)
    soil_temp_battery_ch1 DOUBLE,
    soil_temp_battery_ch2 DOUBLE,
    soil_temp_battery_ch3 DOUBLE,
    soil_temp_battery_ch4 DOUBLE,
    soil_temp_battery_ch5 DOUBLE,
    soil_temp_battery_ch6 DOUBLE,
    soil_temp_battery_ch7 DOUBLE,
    soil_temp_battery_ch8 DOUBLE,
    -- Multi-channel temp probes (WN34/WN34S via ITEM_TF_USR), °C — same scale as outtemp
    tf_temp_ch1 DOUBLE,
    tf_temp_ch2 DOUBLE,
    tf_temp_ch3 DOUBLE,
    tf_temp_ch4 DOUBLE,
    tf_temp_ch5 DOUBLE,
    tf_temp_ch6 DOUBLE,
    tf_temp_ch7 DOUBLE,
    tf_temp_ch8 DOUBLE,
    -- WN34/WN34S battery voltage (volts)
    tf_battery_ch1 DOUBLE,
    tf_battery_ch2 DOUBLE,
    tf_battery_ch3 DOUBLE,
    tf_battery_ch4 DOUBLE,
    tf_battery_ch5 DOUBLE,
    tf_battery_ch6 DOUBLE,
    tf_battery_ch7 DOUBLE,
    tf_battery_ch8 DOUBLE,
    -- WH31/WN31 dip-switch temp+humidity channels 1–8 (°C / %)
    th_temp_ch1 DOUBLE,
    th_temp_ch2 DOUBLE,
    th_temp_ch3 DOUBLE,
    th_temp_ch4 DOUBLE,
    th_temp_ch5 DOUBLE,
    th_temp_ch6 DOUBLE,
    th_temp_ch7 DOUBLE,
    th_temp_ch8 DOUBLE,
    th_humid_ch1 DOUBLE,
    th_humid_ch2 DOUBLE,
    th_humid_ch3 DOUBLE,
    th_humid_ch4 DOUBLE,
    th_humid_ch5 DOUBLE,
    th_humid_ch6 DOUBLE,
    th_humid_ch7 DOUBLE,
    th_humid_ch8 DOUBLE,
    -- WH31/WN31 battery: 1 = low, 0 = normal
    th_battery_low_ch1 DOUBLE,
    th_battery_low_ch2 DOUBLE,
    th_battery_low_ch3 DOUBLE,
    th_battery_low_ch4 DOUBLE,
    th_battery_low_ch5 DOUBLE,
    th_battery_low_ch6 DOUBLE,
    th_battery_low_ch7 DOUBLE,
    th_battery_low_ch8 DOUBLE,
    INDEX idx_timestamp (timestamp DESC)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Optional: Create a user for wxlistener (replace 'your_password' with a secure password)
-- CREATE USER 'wxlistener'@'localhost' IDENTIFIED BY 'your_password';
-- GRANT SELECT, INSERT ON weather.wx_records TO 'wxlistener'@'localhost';
-- FLUSH PRIVILEGES;

-- Verify the table was created
DESCRIBE wx_records;

-- Example query to view recent data
-- SELECT * FROM wx_records ORDER BY timestamp DESC LIMIT 10;
