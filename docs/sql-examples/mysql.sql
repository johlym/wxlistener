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

-- Optional: Create a user for wxlistener (replace 'your_password' with a secure password)
-- CREATE USER 'wxlistener'@'localhost' IDENTIFIED BY 'your_password';
-- GRANT SELECT, INSERT ON weather.wx_records TO 'wxlistener'@'localhost';
-- FLUSH PRIVILEGES;

-- Verify the table was created
DESCRIBE wx_records;

-- Example query to view recent data
-- SELECT * FROM wx_records ORDER BY timestamp DESC LIMIT 10;
