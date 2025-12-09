use chrono::{DateTime, Utc};
use std::collections::HashMap;

pub fn print_livedata(data: &HashMap<String, f64>, timestamp: &DateTime<Utc>) {
    println!("============================================================");
    println!("LIVE DATA - {}", timestamp.format("%Y-%m-%d %H:%M:%S UTC"));
    println!("============================================================");

    let mut keys: Vec<_> = data.keys().collect();
    keys.sort();

    for key in keys {
        let value = data[key];
        let formatted = format_value(key, value);
        println!("{:<20} : {}", key, formatted);
    }

    println!("============================================================");
}

pub fn format_value(key: &str, value: f64) -> String {
    match key {
        k if k.contains("temp") => format!("{:.1}°C", value),
        k if k.contains("humid") => format!("{}%", value as i32),
        k if k.contains("barometer") => format!("{:.1} hPa", value),
        k if k.contains("wind") || k.contains("gust") => format!("{:.1} m/s", value),
        k if k.contains("rain") => format!("{:.1} mm", value),
        "light" => format!("{:.1} lux", value),
        "heap_free" => format!("{} bytes ({:.1} KB)", value as i32, value / 1024.0),
        _ => format!("{}", value),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_value_temperature() {
        assert_eq!(format_value("outtemp", 25.5), "25.5°C");
        assert_eq!(format_value("intemp", -10.3), "-10.3°C");
        assert_eq!(format_value("temp_sensor1", 0.0), "0.0°C");
    }

    #[test]
    fn test_format_value_humidity() {
        assert_eq!(format_value("outhumid", 65.7), "65%");
        assert_eq!(format_value("inhumid", 45.2), "45%");
        assert_eq!(format_value("humid", 99.9), "99%");
    }

    #[test]
    fn test_format_value_barometer() {
        assert_eq!(format_value("absbarometer", 1013.25), "1013.2 hPa");
        assert_eq!(format_value("relbarometer", 996.0), "996.0 hPa");
    }

    #[test]
    fn test_format_value_wind() {
        assert_eq!(format_value("wind_speed", 12.5), "12.5 m/s");
        assert_eq!(format_value("gust_speed", 25.0), "25.0 m/s");
        assert_eq!(format_value("day_max_wind", 30.5), "30.5 m/s");
    }

    #[test]
    fn test_format_value_rain() {
        assert_eq!(format_value("rain_rate", 5.5), "5.5 mm");
        assert_eq!(format_value("rain_day", 12.3), "12.3 mm");
        assert_eq!(format_value("rain_month", 100.0), "100.0 mm");
    }

    #[test]
    fn test_format_value_light() {
        assert_eq!(format_value("light", 50000.0), "50000.0 lux");
        assert_eq!(format_value("light", 0.0), "0.0 lux");
    }

    #[test]
    fn test_format_value_heap_free() {
        assert_eq!(format_value("heap_free", 149240.0), "149240 bytes (145.7 KB)");
        assert_eq!(format_value("heap_free", 1024.0), "1024 bytes (1.0 KB)");
    }

    #[test]
    fn test_format_value_unknown() {
        assert_eq!(format_value("unknown_field", 42.0), "42");
        assert_eq!(format_value("uv", 5.0), "5");
    }

    #[test]
    fn test_print_livedata() {
        let mut data = HashMap::new();
        data.insert("outtemp".to_string(), 25.5);
        data.insert("outhumid".to_string(), 65.0);
        data.insert("wind_speed".to_string(), 5.5);

        let timestamp = Utc::now();

        // This test just ensures the function doesn't panic
        // We can't easily test stdout without more complex mocking
        print_livedata(&data, &timestamp);
    }

    #[test]
    fn test_print_livedata_empty() {
        let data = HashMap::new();
        let timestamp = Utc::now();

        // Should handle empty data gracefully
        print_livedata(&data, &timestamp);
    }

    #[test]
    fn test_print_livedata_sorted() {
        let mut data = HashMap::new();
        data.insert("z_field".to_string(), 1.0);
        data.insert("a_field".to_string(), 2.0);
        data.insert("m_field".to_string(), 3.0);

        let timestamp = Utc::now();

        // Keys should be sorted alphabetically
        // This test ensures no panic with various keys
        print_livedata(&data, &timestamp);
    }
}
