//! Local Ecowitt HTTP API client (GW3000 and the same API on GW1100/GW2000).
//!
//! Readings are converted into the metric `HashMap<String, f64>` the TCP decoder
//! already produces. Battery levels are separate keys from battery voltage.

use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::time::Duration;

use crate::config::RainSource;

/// Ecowitt display conversion: `W/m² = lux * 0.0079`. Reverse it to recover lux.
pub const LUX_PER_WM2_DIVISOR: f64 = 0.0079;
/// Foot-candles to lux.
pub const LUX_PER_FC: f64 = 10.76391;

const INHG_TO_HPA: f64 = 33.8639;
const MMHG_TO_HPA: f64 = 1.33322;
const MPH_TO_MS: f64 = 0.44704;
const KNOTS_TO_MS: f64 = 0.514444;
const INCH_TO_MM: f64 = 25.4;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(16);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ChannelKind {
    Aisle,
    Soil,
    Temp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Quantity {
    Temperature,
    Pressure,
    Wind,
    Rain,
    Light,
    Humidity,
    Direction,
    Index,
    Voltage,
    Level,
    Plain,
}

/// Unit indexes from `get_units_info`. Unknown codes stay `None` and bare values are skipped.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UnitPrefs {
    pub temperature: Option<u8>,
    pub pressure: Option<u8>,
    pub wind: Option<u8>,
    pub rain: Option<u8>,
    pub light: Option<u8>,
}

impl UnitPrefs {
    pub fn from_json(value: &Value) -> Self {
        Self {
            temperature: json_code(value.get("temperature")),
            pressure: json_code(value.get("pressure")),
            wind: json_code(value.get("wind")),
            rain: json_code(value.get("rain")),
            light: json_code(value.get("light")),
        }
    }
}

/// One row from `get_sensors_info`.
#[derive(Debug, Clone, PartialEq)]
pub struct HttpSensor {
    pub type_code: u16,
    pub id: String,
    pub name: String,
    pub battery: Option<f64>,
    pub signal: Option<u32>,
    pub idst: Option<u32>,
}

impl HttpSensor {
    /// Registered sensors have a real id, a non-zero signal, and `idst` other than 0.
    pub fn is_registered(&self) -> bool {
        let id = self.id.trim();
        if id.is_empty() {
            return false;
        }
        let hex = id.trim_start_matches("0x").trim_start_matches("0X");
        if hex.is_empty() || hex.chars().all(|c| c == '0') {
            return false;
        }
        if hex.eq_ignore_ascii_case("FFFFFFFE") || hex.eq_ignore_ascii_case("FFFFFFFF") {
            return false;
        }
        if self.signal.unwrap_or(0) == 0 {
            return false;
        }
        if self.idst == Some(0) {
            return false;
        }
        true
    }

    fn channel_slot(&self) -> Option<(ChannelKind, u8)> {
        let channel = |base: u16| -> Option<u8> {
            let ch = self.type_code.checked_sub(base)? + 1;
            if (1..=8).contains(&ch) {
                Some(ch as u8)
            } else {
                None
            }
        };
        match self.type_code {
            6..=13 => Some((ChannelKind::Aisle, channel(6)?)),
            14..=21 => Some((ChannelKind::Soil, channel(14)?)),
            31..=38 => Some((ChannelKind::Temp, channel(31)?)),
            _ => None,
        }
    }
}

fn parse_sensor(value: &Value) -> Option<HttpSensor> {
    let type_code = json_u32(value.get("type"))? as u16;
    let id = json_text(value.get("id")).unwrap_or_default();
    let name = json_text(value.get("name")).unwrap_or_default();
    Some(HttpSensor {
        type_code,
        id,
        name,
        battery: json_text(value.get("batt")).and_then(|s| s.parse().ok()),
        signal: json_u32(value.get("signal")),
        idst: json_u32(value.get("idst")),
    })
}

/// Map one `get_livedata_info` body onto listener keys.
///
/// `sensors == None` means the inventory request failed, so channel arrays are skipped.
/// `sensors == Some([])` means nothing is registered.
pub fn map_livedata(
    body: &Value,
    prefs: &UnitPrefs,
    sensors: Option<&[HttpSensor]>,
    rain_source: RainSource,
) -> HashMap<String, f64> {
    let mut data = HashMap::new();

    if let Some(items) = body.get("common_list").and_then(Value::as_array) {
        for item in items {
            let Some(id) = item.get("id").and_then(Value::as_str) else {
                continue;
            };
            let Some((key, quantity)) = common_field(id) else {
                continue;
            };
            let Some(raw) = json_text(item.get("val")) else {
                continue;
            };
            insert_reading(
                &mut data,
                key,
                quantity,
                &raw,
                item.get("unit").and_then(Value::as_str),
                prefs,
            );
        }
    }

    if let Some(wh25) = body
        .get("wh25")
        .and_then(Value::as_array)
        .and_then(|rows| rows.first())
    {
        let temp_unit = wh25.get("unit").and_then(Value::as_str);
        if let Some(raw) = json_text(wh25.get("intemp")) {
            if !data.contains_key("intemp") {
                insert_reading(
                    &mut data,
                    "intemp",
                    Quantity::Temperature,
                    &raw,
                    temp_unit,
                    prefs,
                );
            }
        }
        if let Some(raw) = json_text(wh25.get("inhumi")) {
            if !data.contains_key("inhumid") {
                insert_reading(&mut data, "inhumid", Quantity::Humidity, &raw, None, prefs);
            }
        }
        if let Some(raw) = json_text(wh25.get("abs")) {
            if !data.contains_key("absbarometer") {
                insert_reading(
                    &mut data,
                    "absbarometer",
                    Quantity::Pressure,
                    &raw,
                    None,
                    prefs,
                );
            }
        }
        if let Some(raw) = json_text(wh25.get("rel")) {
            if !data.contains_key("relbarometer") {
                insert_reading(
                    &mut data,
                    "relbarometer",
                    Quantity::Pressure,
                    &raw,
                    None,
                    prefs,
                );
            }
        }
    }

    let rain_key = match rain_source {
        RainSource::Rain => "rain",
        RainSource::Piezo => "piezoRain",
    };
    if let Some(items) = body.get(rain_key).and_then(Value::as_array) {
        for item in items {
            let Some(id) = item.get("id").and_then(Value::as_str) else {
                continue;
            };
            let Some(key) = rain_field(id) else {
                continue;
            };
            let Some(raw) = json_text(item.get("val")) else {
                continue;
            };
            insert_reading(
                &mut data,
                key,
                Quantity::Rain,
                &raw,
                item.get("unit").and_then(Value::as_str),
                prefs,
            );
        }
    }

    if let Some(sensors) = sensors {
        let allowed = registered_channels(sensors);
        map_aisle(body, &allowed, prefs, &mut data);
        map_soil(body, &allowed, prefs, &mut data);
        map_temp(body, &allowed, prefs, &mut data);
    }

    if let Some(raw) = body
        .get("debug")
        .and_then(Value::as_array)
        .and_then(|rows| rows.first())
        .and_then(|row| json_text(row.get("heap")))
    {
        insert_reading(&mut data, "heap_free", Quantity::Plain, &raw, None, prefs);
    }

    data
}

fn registered_channels(sensors: &[HttpSensor]) -> HashSet<(ChannelKind, u8)> {
    sensors
        .iter()
        .filter(|sensor| sensor.is_registered())
        .filter_map(|sensor| sensor.channel_slot())
        .collect()
}

fn map_aisle(
    body: &Value,
    allowed: &HashSet<(ChannelKind, u8)>,
    prefs: &UnitPrefs,
    data: &mut HashMap<String, f64>,
) {
    let Some(rows) = body.get("ch_aisle").and_then(Value::as_array) else {
        return;
    };
    for row in rows {
        let Some(channel) = json_channel(row.get("channel")) else {
            continue;
        };
        if !allowed.contains(&(ChannelKind::Aisle, channel)) {
            continue;
        }
        let unit = row.get("unit").and_then(Value::as_str);
        if let Some(raw) = json_text(row.get("temp")) {
            insert_reading(
                data,
                &format!("th_temp_ch{channel}"),
                Quantity::Temperature,
                &raw,
                unit,
                prefs,
            );
        }
        if let Some(raw) = json_text(row.get("humidity")) {
            insert_reading(
                data,
                &format!("th_humid_ch{channel}"),
                Quantity::Humidity,
                &raw,
                None,
                prefs,
            );
        }
    }
}

fn map_soil(
    body: &Value,
    allowed: &HashSet<(ChannelKind, u8)>,
    prefs: &UnitPrefs,
    data: &mut HashMap<String, f64>,
) {
    let Some(rows) = body.get("ch_soil").and_then(Value::as_array) else {
        return;
    };
    for row in rows {
        let Some(channel) = json_channel(row.get("channel")) else {
            continue;
        };
        if !allowed.contains(&(ChannelKind::Soil, channel)) {
            continue;
        }
        if let Some(raw) = json_text(row.get("humidity")) {
            insert_reading(
                data,
                &format!("soil_moisture_ch{channel}"),
                Quantity::Humidity,
                &raw,
                None,
                prefs,
            );
        }
        if let Some(raw) = json_text(row.get("voltage")) {
            insert_reading(
                data,
                &format!("soil_battery_ch{channel}"),
                Quantity::Voltage,
                &raw,
                None,
                prefs,
            );
        }
        if let Some(raw) = json_text(row.get("battery")) {
            insert_reading(
                data,
                &format!("soil_battery_level_ch{channel}"),
                Quantity::Level,
                &raw,
                None,
                prefs,
            );
        }
    }
}

fn map_temp(
    body: &Value,
    allowed: &HashSet<(ChannelKind, u8)>,
    prefs: &UnitPrefs,
    data: &mut HashMap<String, f64>,
) {
    let Some(rows) = body.get("ch_temp").and_then(Value::as_array) else {
        return;
    };
    for row in rows {
        let Some(channel) = json_channel(row.get("channel")) else {
            continue;
        };
        if !allowed.contains(&(ChannelKind::Temp, channel)) {
            continue;
        }
        let unit = row.get("unit").and_then(Value::as_str);
        if let Some(raw) = json_text(row.get("temp")) {
            insert_reading(
                data,
                &format!("tf_temp_ch{channel}"),
                Quantity::Temperature,
                &raw,
                unit,
                prefs,
            );
        }
        if let Some(raw) = json_text(row.get("voltage")) {
            insert_reading(
                data,
                &format!("tf_battery_ch{channel}"),
                Quantity::Voltage,
                &raw,
                None,
                prefs,
            );
        }
        if let Some(raw) = json_text(row.get("battery")) {
            insert_reading(
                data,
                &format!("tf_battery_level_ch{channel}"),
                Quantity::Level,
                &raw,
                None,
                prefs,
            );
        }
    }
}

fn insert_reading(
    data: &mut HashMap<String, f64>,
    key: &str,
    quantity: Quantity,
    raw: &str,
    sibling_unit: Option<&str>,
    prefs: &UnitPrefs,
) {
    if let Some(value) = convert_reading(quantity, raw, sibling_unit, prefs) {
        data.insert(key.to_string(), value);
    }
}

fn convert_reading(
    quantity: Quantity,
    raw: &str,
    sibling_unit: Option<&str>,
    prefs: &UnitPrefs,
) -> Option<f64> {
    let raw = raw.trim();
    if raw.eq_ignore_ascii_case("none") {
        return None;
    }
    let (number, embedded) = split_number_unit(raw)?;
    let unit = embedded.or_else(|| {
        sibling_unit
            .map(normalize_unit)
            .filter(|unit| !unit.is_empty())
    });

    match quantity {
        Quantity::Humidity
        | Quantity::Direction
        | Quantity::Index
        | Quantity::Voltage
        | Quantity::Level
        | Quantity::Plain => Some(number),
        Quantity::Temperature => convert_temperature(number, unit.as_deref(), prefs),
        Quantity::Pressure => convert_pressure(number, unit.as_deref(), prefs),
        Quantity::Wind => convert_wind(number, unit.as_deref(), prefs),
        Quantity::Rain => convert_rain(number, unit.as_deref(), prefs),
        Quantity::Light => convert_light(number, unit.as_deref(), prefs),
    }
}

fn convert_temperature(number: f64, unit: Option<&str>, prefs: &UnitPrefs) -> Option<f64> {
    let unit = unit.or(match prefs.temperature {
        Some(0) => Some("c"),
        Some(1) => Some("f"),
        _ => None,
    })?;
    let celsius = match unit {
        "c" | "celsius" => number,
        "f" | "fahrenheit" => (number - 32.0) * 5.0 / 9.0,
        _ => return None,
    };
    Some(round_tenth(celsius))
}

fn convert_pressure(number: f64, unit: Option<&str>, prefs: &UnitPrefs) -> Option<f64> {
    let unit = unit.or(match prefs.pressure {
        Some(0) => Some("hpa"),
        Some(1) => Some("inhg"),
        Some(2) => Some("mmhg"),
        _ => None,
    })?;
    let hpa = match unit {
        "hpa" | "mbar" | "mb" => number,
        "inhg" => number * INHG_TO_HPA,
        "mmhg" => number * MMHG_TO_HPA,
        _ => return None,
    };
    Some(round_tenth(hpa))
}

fn convert_wind(number: f64, unit: Option<&str>, prefs: &UnitPrefs) -> Option<f64> {
    let unit = unit.or(match prefs.wind {
        Some(0) => Some("m/s"),
        Some(1) => Some("km/h"),
        Some(2) => Some("mph"),
        Some(3) => Some("knots"),
        _ => None,
    })?;
    let meters_per_second = match unit {
        "m/s" | "ms" => number,
        "km/h" | "kmh" | "kph" => number / 3.6,
        "mph" => number * MPH_TO_MS,
        "kn" | "kt" | "knot" | "knots" => number * KNOTS_TO_MS,
        _ => return None,
    };
    Some(round_tenth(meters_per_second))
}

fn convert_rain(number: f64, unit: Option<&str>, prefs: &UnitPrefs) -> Option<f64> {
    let unit = unit.or(match prefs.rain {
        Some(0) => Some("mm"),
        Some(1) => Some("in"),
        _ => None,
    })?;
    let mm = match unit {
        "mm" => number,
        "in" | "inch" | "inches" => number * INCH_TO_MM,
        _ => return None,
    };
    Some(round_tenth(mm))
}

fn convert_light(number: f64, unit: Option<&str>, prefs: &UnitPrefs) -> Option<f64> {
    let unit = unit.or(match prefs.light {
        // Code 1 is W/m². Codes 0 and 2 are not confirmed, so a bare value is skipped.
        Some(1) => Some("w/m2"),
        _ => None,
    })?;
    match unit {
        "lux" | "lx" => Some(number),
        "w/m2" | "wm2" => Some(number / LUX_PER_WM2_DIVISOR),
        "fc" | "ftc" => Some(number * LUX_PER_FC),
        _ => None,
    }
}

fn round_tenth(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}

/// `0x02` is dew-point's neighbor, outdoor temperature. Decimal `"3"` is feel-like, not `0x03`.
fn common_field(id: &str) -> Option<(&'static str, Quantity)> {
    let code = hex_id(id)?;
    Some(match code {
        0x01 => ("intemp", Quantity::Temperature),
        0x02 => ("outtemp", Quantity::Temperature),
        0x03 => ("dewpoint", Quantity::Temperature),
        0x04 => ("windchill", Quantity::Temperature),
        0x05 => ("heatindex", Quantity::Temperature),
        0x06 => ("inhumid", Quantity::Humidity),
        0x07 => ("outhumid", Quantity::Humidity),
        0x08 => ("absbarometer", Quantity::Pressure),
        0x09 => ("relbarometer", Quantity::Pressure),
        0x0A => ("wind_dir", Quantity::Direction),
        0x0B => ("wind_speed", Quantity::Wind),
        0x0C => ("gust_speed", Quantity::Wind),
        0x15 => ("light", Quantity::Light),
        0x16 => ("uv", Quantity::Index),
        0x17 => ("uvi", Quantity::Index),
        0x19 => ("day_max_wind", Quantity::Wind),
        _ => return None,
    })
}

fn rain_field(id: &str) -> Option<&'static str> {
    match hex_id(id)? {
        0x0D => Some("rain_event"),
        0x0E => Some("rain_rate"),
        0x10 => Some("rain_day"),
        0x11 => Some("rain_week"),
        0x12 => Some("rain_month"),
        0x13 => Some("rain_year"),
        _ => None,
    }
}

fn hex_id(id: &str) -> Option<u32> {
    let id = id.trim();
    let hex = id.strip_prefix("0x").or_else(|| id.strip_prefix("0X"))?;
    u32::from_str_radix(hex.trim(), 16).ok()
}

fn split_number_unit(raw: &str) -> Option<(f64, Option<String>)> {
    let raw = raw.trim();
    let mut end = 0usize;
    let bytes = raw.as_bytes();
    if bytes.first() == Some(&b'+') || bytes.first() == Some(&b'-') {
        end = 1;
    }
    let mut seen_dot = false;
    while end < bytes.len() {
        match bytes[end] {
            b'0'..=b'9' => end += 1,
            b'.' if !seen_dot => {
                seen_dot = true;
                end += 1;
            }
            _ => break,
        }
    }
    if end == 0 || (end == 1 && (bytes[0] == b'+' || bytes[0] == b'-')) {
        return None;
    }
    let number: f64 = raw[..end].parse().ok()?;
    let unit = normalize_unit(&raw[end..]);
    if unit.is_empty() {
        Some((number, None))
    } else {
        Some((number, Some(unit)))
    }
}

fn normalize_unit(raw: &str) -> String {
    let unit = raw
        .trim()
        .to_lowercase()
        .replace('²', "2")
        .replace('°', "")
        .replace(' ', "");
    if unit == "km/h" || unit == "m/s" || unit.starts_with("w/") {
        return unit;
    }
    unit.strip_suffix("/hr")
        .or_else(|| unit.strip_suffix("/h"))
        .unwrap_or(&unit)
        .to_string()
}

fn json_text(value: Option<&Value>) -> Option<String> {
    match value? {
        Value::String(text) => Some(text.clone()),
        Value::Number(number) => Some(number.to_string()),
        Value::Bool(flag) => Some(flag.to_string()),
        _ => None,
    }
}

fn json_code(value: Option<&Value>) -> Option<u8> {
    json_u32(value).and_then(|code| u8::try_from(code).ok())
}

fn json_u32(value: Option<&Value>) -> Option<u32> {
    match value? {
        Value::Number(number) => number.as_u64().and_then(|n| u32::try_from(n).ok()),
        Value::String(text) => text.trim().parse().ok(),
        _ => None,
    }
}

fn json_channel(value: Option<&Value>) -> Option<u8> {
    let channel = json_u32(value)? as u8;
    if (1..=8).contains(&channel) {
        Some(channel)
    } else {
        None
    }
}

/// Blocking client for the gateway's local HTTP API.
#[derive(Clone)]
pub struct HttpGateway {
    origin: String,
    client: reqwest::blocking::Client,
    rain_source: RainSource,
}

impl HttpGateway {
    pub fn new(ip: impl Into<String>, port: u16, rain_source: RainSource) -> Result<Self> {
        let ip = ip.into();
        let host = if ip.contains(':') && !ip.starts_with('[') {
            format!("[{ip}]")
        } else {
            ip
        };
        let client = reqwest::blocking::Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .context("Failed to build HTTP gateway client")?;
        Ok(Self {
            origin: format!("http://{host}:{port}"),
            client,
            rain_source,
        })
    }

    pub fn get_firmware_version(&self) -> Result<String> {
        let body = self.get_json("/get_version?")?;
        json_text(body.get("version")).context("get_version response has no version string")
    }

    pub fn get_mac_address(&self) -> Result<String> {
        let body = self.get_json("/get_network_info?")?;
        json_text(body.get("mac")).context("get_network_info response has no mac string")
    }

    pub fn fetch_sensors(&self) -> Result<Vec<HttpSensor>> {
        let mut all = Vec::new();
        let mut seen = HashSet::new();
        for page in 1..=8 {
            let body = self.get_json(&format!("/get_sensors_info?page={page}"))?;
            let items = body
                .as_array()
                .context("get_sensors_info did not return an array")?;
            if items.is_empty() {
                break;
            }
            let mut added = 0usize;
            for item in items {
                let Some(sensor) = parse_sensor(item) else {
                    continue;
                };
                let key = format!("{}:{}", sensor.type_code, sensor.id);
                if seen.insert(key) {
                    all.push(sensor);
                    added += 1;
                }
            }
            if added == 0 {
                break;
            }
        }
        Ok(all)
    }

    pub fn get_livedata(&self) -> Result<HashMap<String, f64>> {
        let body = self.get_json("/get_livedata_info?")?;
        let prefs = match self.get_json("/get_units_info?") {
            Ok(units) => UnitPrefs::from_json(&units),
            Err(err) => {
                eprintln!("[WARN] Failed to read unit settings: {err}");
                UnitPrefs::default()
            }
        };
        let sensors = match self.fetch_sensors() {
            Ok(sensors) => Some(sensors),
            Err(err) => {
                eprintln!(
                    "[WARN] Failed to read sensor inventory; skipping channel sensors: {err}"
                );
                None
            }
        };
        let data = map_livedata(&body, &prefs, sensors.as_deref(), self.rain_source);
        if data.is_empty() {
            anyhow::bail!("HTTP live data contained no recognized measurements");
        }
        Ok(data)
    }

    fn get_json(&self, path: &str) -> Result<Value> {
        let url = format!("{}{path}", self.origin);
        let response = self
            .client
            .get(&url)
            .header(reqwest::header::CONNECTION, "close")
            .send()
            .with_context(|| format!("HTTP request failed: {url}"))?;
        let status = response.status();
        if status.is_redirection() {
            anyhow::bail!("HTTP {status} redirect from {url} was not followed");
        }
        let body = response
            .text()
            .with_context(|| format!("Failed to read HTTP body from {url}"))?;
        if !status.is_success() {
            anyhow::bail!("HTTP {status} from {url}: {body}");
        }
        serde_json::from_str(&body).with_context(|| format!("Invalid JSON from {url}: {body}"))
    }
}

impl std::fmt::Debug for HttpGateway {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HttpGateway")
            .field("origin", &self.origin)
            .field("rain_source", &self.rain_source)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const CAPTURED_LIVEDATA: &str = r#"{
        "common_list":[
            {"id":"0x02","val":"41.9","unit":"F"},
            {"id":"0x07","val":"97%"},
            {"id":"3","val":"41.9","unit":"F"},
            {"id":"5","val":"0.027 kPa"},
            {"id":"0x03","val":"41.2","unit":"F"},
            {"id":"0x0B","val":"0.00 mph"},
            {"id":"0x0C","val":"0.00 mph"},
            {"id":"0x19","val":"0.00 mph"},
            {"id":"0x15","val":"0.00 W/m2"},
            {"id":"0x17","val":"0"},
            {"id":"0x0A","val":"99"},
            {"id":"0x6D","val":"99"}
        ],
        "rain":[
            {"id":"0x0D","val":"0.00 in"},
            {"id":"0x0E","val":"0.00 in/Hr"},
            {"id":"0x7D","val":"0.00 in"},
            {"id":"0x7C","val":"0.00 in"},
            {"id":"0x10","val":"0.00 in"},
            {"id":"0x11","val":"0.01 in"},
            {"id":"0x12","val":"0.30 in"},
            {"id":"0x13","val":"0.30 in","battery":"0"},
            {"id":"0x14","val":"0.30 in"}
        ],
        "wh25":[{"intemp":"78.6","unit":"F","inhumi":"32%","abs":"29.51 inHg","rel":"29.51 inHg"}],
        "ch_aisle":[{"channel":"1","name":"","battery":"0","temp":"41.7","unit":"F","humidity":"63%"}],
        "ch_soil":[
            {"channel":"1","name":"","battery":"5","voltage":"1.50","humidity":"60%"},
            {"channel":"2","name":"","battery":"5","voltage":"1.50","humidity":"51%"},
            {"channel":"3","name":"","battery":"5","voltage":"1.50","humidity":"62%"}
        ],
        "ch_temp":[
            {"channel":"1","name":"","temp":"57.2","unit":"F","battery":"5","voltage":"1.52"},
            {"channel":"2","name":"","temp":"50.0","unit":"F","battery":"5","voltage":"1.52"}
        ],
        "debug":[{"heap":"99192","runtime":"719501","usr_interval":"--","is_cnip":false}]
    }"#;

    fn captured_body() -> Value {
        serde_json::from_str(CAPTURED_LIVEDATA).unwrap()
    }

    fn registered_fixture() -> Vec<HttpSensor> {
        [6u16, 14, 15, 16, 31, 32]
            .into_iter()
            .map(|type_code| HttpSensor {
                type_code,
                id: "AA".to_string(),
                name: String::new(),
                battery: None,
                signal: Some(4),
                idst: Some(1),
            })
            .collect()
    }

    fn assert_close(data: &HashMap<String, f64>, key: &str, expected: f64) {
        let actual = *data.get(key).unwrap_or_else(|| panic!("missing {key}"));
        assert!(
            (actual - expected).abs() < 1e-9,
            "{key}: got {actual}, expected {expected}"
        );
    }

    #[test]
    fn captured_payload_maps_to_metric_keys() {
        let data = map_livedata(
            &captured_body(),
            &UnitPrefs::default(),
            Some(&registered_fixture()),
            RainSource::Rain,
        );

        assert_close(&data, "outtemp", 5.5);
        assert_close(&data, "outhumid", 97.0);
        assert_close(&data, "dewpoint", 5.1);
        assert_close(&data, "wind_speed", 0.0);
        assert_close(&data, "gust_speed", 0.0);
        assert_close(&data, "day_max_wind", 0.0);
        assert_close(&data, "light", 0.0);
        assert_close(&data, "uvi", 0.0);
        assert_close(&data, "wind_dir", 99.0);
        assert_close(&data, "intemp", 25.9);
        assert_close(&data, "inhumid", 32.0);
        assert_close(&data, "absbarometer", 999.3);
        assert_close(&data, "relbarometer", 999.3);
        assert_close(&data, "rain_event", 0.0);
        assert_close(&data, "rain_rate", 0.0);
        assert_close(&data, "rain_day", 0.0);
        assert_close(&data, "rain_week", 0.3);
        assert_close(&data, "rain_month", 7.6);
        assert_close(&data, "rain_year", 7.6);
        assert_close(&data, "th_temp_ch1", 5.4);
        assert_close(&data, "th_humid_ch1", 63.0);
        assert!(!data.contains_key("th_battery_low_ch1"));
        assert_close(&data, "soil_moisture_ch1", 60.0);
        assert_close(&data, "soil_moisture_ch2", 51.0);
        assert_close(&data, "soil_moisture_ch3", 62.0);
        assert_close(&data, "soil_battery_ch1", 1.5);
        assert_close(&data, "soil_battery_level_ch1", 5.0);
        assert_close(&data, "tf_temp_ch1", 14.0);
        assert_close(&data, "tf_temp_ch2", 10.0);
        assert_close(&data, "tf_battery_ch1", 1.52);
        assert_close(&data, "tf_battery_level_ch2", 5.0);
        assert_close(&data, "heap_free", 99192.0);
        assert_eq!(data.len(), 37);
    }

    #[test]
    fn unregistered_channels_and_unknown_ids_are_omitted() {
        let sensors = vec![HttpSensor {
            type_code: 6,
            id: "FFFFFFFE".to_string(),
            name: String::new(),
            battery: None,
            signal: Some(0),
            idst: Some(0),
        }];
        let data = map_livedata(
            &captured_body(),
            &UnitPrefs::default(),
            Some(&sensors),
            RainSource::Rain,
        );
        assert!(!data.keys().any(|key| key.starts_with("th_")
            || key.starts_with("soil_")
            || key.starts_with("tf_")));
        assert!(data.contains_key("outtemp"));
        assert!(!data.contains_key("rain_totals"));
    }

    #[test]
    fn missing_inventory_skips_channel_arrays() {
        let data = map_livedata(
            &captured_body(),
            &UnitPrefs::default(),
            None,
            RainSource::Rain,
        );
        assert!(data.contains_key("outtemp"));
        assert!(!data.contains_key("th_temp_ch1"));
        assert!(!data.contains_key("soil_moisture_ch1"));
    }

    #[test]
    fn piezo_source_ignores_the_traditional_rain_array() {
        let mut body = captured_body();
        body["piezoRain"] = json!([
            {"id":"0x0D","val":"1.00 in"},
            {"id":"0x10","val":"2.00 in"}
        ]);
        let data = map_livedata(&body, &UnitPrefs::default(), None, RainSource::Piezo);
        assert_close(&data, "rain_event", 25.4);
        assert_close(&data, "rain_day", 50.8);
        assert!(!data.contains_key("rain_week"));
    }

    #[test]
    fn bare_values_use_unit_codes_and_skip_unknown_light_codes() {
        let prefs = UnitPrefs::from_json(&json!({
            "temperature": "1",
            "pressure": "1",
            "wind": "2",
            "rain": "1",
            "light": "0",
            "capacity": "0"
        }));
        let body = json!({
            "common_list": [
                {"id": "0x02", "val": "41.9"},
                {"id": "0x0B", "val": "10"},
                {"id": "0x15", "val": "10"},
                {"id": "0x08", "val": "29.51"}
            ]
        });
        let data = map_livedata(&body, &prefs, None, RainSource::Rain);
        assert_close(&data, "outtemp", 5.5);
        assert_close(&data, "wind_speed", 4.5);
        assert_close(&data, "absbarometer", 999.3);
        assert!(!data.contains_key("light"));
    }

    #[test]
    fn light_code_one_converts_bare_irradiance() {
        let prefs = UnitPrefs {
            light: Some(1),
            ..UnitPrefs::default()
        };
        let body = json!({"common_list":[{"id":"0x15","val":"10"}]});
        let data = map_livedata(&body, &prefs, None, RainSource::Rain);
        let lux = *data.get("light").unwrap();
        assert!((lux - (10.0 / LUX_PER_WM2_DIVISOR)).abs() < 1e-6);
    }

    #[test]
    fn humidity_none_is_omitted() {
        let body = json!({
            "ch_aisle":[{"channel":"1","temp":"20","unit":"C","humidity":"None","battery":"0"}]
        });
        let sensors = registered_fixture();
        let data = map_livedata(
            &body,
            &UnitPrefs::default(),
            Some(&sensors),
            RainSource::Rain,
        );
        assert_close(&data, "th_temp_ch1", 20.0);
        assert!(!data.contains_key("th_humid_ch1"));
    }

    #[test]
    fn wh25_temperature_unit_is_not_applied_to_pressure() {
        let body = json!({
            "wh25":[{"intemp":"32","unit":"F","abs":"29.92 inHg","rel":"1013.25 hPa","inhumi":"None"}]
        });
        let data = map_livedata(&body, &UnitPrefs::default(), None, RainSource::Rain);
        assert_close(&data, "intemp", 0.0);
        assert_close(&data, "absbarometer", round_tenth(29.92 * INHG_TO_HPA));
        assert_close(&data, "relbarometer", 1013.3);
        assert!(!data.contains_key("inhumid"));
    }

    #[test]
    fn http_client_reads_a_local_gateway() {
        use axum::extract::Query;
        use axum::{routing::get, Json, Router};
        use std::collections::HashMap as StdMap;

        let app = Router::new()
            .route(
                "/get_version",
                get(|| async { Json(json!({"version": "Version:GW3000_V1.2.3"})) }),
            )
            .route(
                "/get_network_info",
                get(|| async { Json(json!({"mac": "AA:BB:CC:DD:EE:FF"})) }),
            )
            .route(
                "/get_units_info",
                get(|| async {
                    Json(json!({
                        "temperature": "1",
                        "pressure": "1",
                        "wind": "2",
                        "rain": "1",
                        "light": "1"
                    }))
                }),
            )
            .route("/get_livedata_info", get(|| async { Json(captured_body()) }))
            .route(
                "/get_sensors_info",
                get(|Query(query): Query<StdMap<String, String>>| async move {
                    let page = query.get("page").and_then(|p| p.parse::<u32>().ok()).unwrap_or(1);
                    if page == 1 {
                        Json(json!([
                            {"type":"6","id":"AA","signal":"4","idst":"1","batt":"0","name":"TH1"},
                            {"type":"14","id":"BB","signal":"3","idst":"1","batt":"5","name":"Soil1"}
                        ]))
                    } else {
                        Json(json!([]))
                    }
                }),
            );

        let runtime = tokio::runtime::Runtime::new().unwrap();
        let listener = runtime
            .block_on(tokio::net::TcpListener::bind("127.0.0.1:0"))
            .unwrap();
        let port = listener.local_addr().unwrap().port();
        std::thread::spawn(move || {
            runtime.block_on(async move {
                axum::serve(listener, app).await.unwrap();
            });
        });

        let gateway = HttpGateway::new("127.0.0.1", port, RainSource::Rain).unwrap();
        assert_eq!(
            gateway.get_firmware_version().unwrap(),
            "Version:GW3000_V1.2.3"
        );
        assert_eq!(gateway.get_mac_address().unwrap(), "AA:BB:CC:DD:EE:FF");
        let data = gateway.get_livedata().unwrap();
        assert_close(&data, "outtemp", 5.5);
        assert_close(&data, "th_temp_ch1", 5.4);
        assert_close(&data, "soil_moisture_ch1", 60.0);
        assert!(!data.contains_key("soil_moisture_ch2"));
        assert!(!data.contains_key("tf_temp_ch1"));
    }

    #[test]
    fn disabled_sensor_ids_are_not_registered() {
        let disabled = HttpSensor {
            type_code: 14,
            id: "FFFFFFFE".into(),
            name: String::new(),
            battery: None,
            signal: Some(4),
            idst: Some(1),
        };
        assert!(!disabled.is_registered());
        let quiet = HttpSensor {
            type_code: 14,
            id: "10".into(),
            name: String::new(),
            battery: None,
            signal: Some(0),
            idst: Some(1),
        };
        assert!(!quiet.is_registered());
    }
}
