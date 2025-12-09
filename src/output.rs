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

fn format_value(key: &str, value: f64) -> String {
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
