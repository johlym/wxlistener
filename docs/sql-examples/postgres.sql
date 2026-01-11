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
    day_max_wind DOUBLE PRECISION
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
