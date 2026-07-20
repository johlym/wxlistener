//! One-shot sensor inventory probe for validating WN34S / WH34 mapping.
use anyhow::Result;
use std::collections::HashMap;

use crate::client::GW1000Client;
use crate::decoder::decode_temp;
use crate::output::format_value;

/// Disabled / re-learn sentinel IDs from the protocol manual
const SENSOR_ID_DISABLED: u32 = 0xFFFF_FFFE;
const SENSOR_ID_RELEARN: u32 = 0xFFFF_FFFF;

const WH51_SENSOR_CH1: u8 = 14;
const WH51_SENSOR_CH8: u8 = 21;
const WH34_SENSOR_CH1: u8 = 31;
const WH34_SENSOR_CH8: u8 = 38;
const WH35_SENSOR_CH1: u8 = 40;
const WH35_SENSOR_CH8: u8 = 47;

#[derive(Debug, Clone, PartialEq)]
pub struct SensorIdEntry {
    pub sensor_type: u8,
    pub sensor_id: u32,
    pub battery_raw: u8,
    pub signal: u8,
}

impl SensorIdEntry {
    pub fn is_active(&self) -> bool {
        self.sensor_id != SENSOR_ID_DISABLED
            && self.sensor_id != SENSOR_ID_RELEARN
            && self.sensor_id != 0
            && self.signal != 0
    }

    pub fn type_label(&self) -> String {
        sensor_type_label(self.sensor_type)
    }

    /// Battery voltage when the protocol documents a voltage scale for this type.
    pub fn battery_volts(&self) -> Option<f64> {
        if (WH51_SENSOR_CH1..=WH51_SENSOR_CH8).contains(&self.sensor_type) {
            Some(self.battery_raw as f64 * 0.1)
        } else if (WH34_SENSOR_CH1..=WH34_SENSOR_CH8).contains(&self.sensor_type)
            || (WH35_SENSOR_CH1..=WH35_SENSOR_CH8).contains(&self.sensor_type)
        {
            Some(self.battery_raw as f64 * 0.02)
        } else {
            None
        }
    }

    pub fn listener_battery_key(&self) -> Option<String> {
        if (WH51_SENSOR_CH1..=WH51_SENSOR_CH8).contains(&self.sensor_type) {
            let ch = self.sensor_type - WH51_SENSOR_CH1 + 1;
            Some(format!("soil_battery_ch{}", ch))
        } else if (WH34_SENSOR_CH1..=WH34_SENSOR_CH8).contains(&self.sensor_type) {
            let ch = self.sensor_type - WH34_SENSOR_CH1 + 1;
            Some(format!("soil_temp_battery_ch{}", ch))
        } else {
            None
        }
    }
}

pub fn sensor_type_label(sensor_type: u8) -> String {
    match sensor_type {
        0 => "WH65 outdoor array".to_string(),
        1 => "WH68 anemometer".to_string(),
        2 => "WH80 ultrasonic".to_string(),
        3 => "WH40 rain".to_string(),
        4 => "WH25 indoor".to_string(),
        5 => "WH26 temp/humid".to_string(),
        t if (6..=13).contains(&t) => format!("WH31 temp/humid ch{}", t - 5),
        t if (WH51_SENSOR_CH1..=WH51_SENSOR_CH8).contains(&t) => {
            format!("WH51 soil moisture ch{}", t - WH51_SENSOR_CH1 + 1)
        }
        t if (22..=25).contains(&t) => format!("WH41 PM2.5 ch{}", t - 21),
        26 => "WH57 lightning".to_string(),
        t if (27..=30).contains(&t) => format!("WH55 leak ch{}", t - 26),
        t if (WH34_SENSOR_CH1..=WH34_SENSOR_CH8).contains(&t) => {
            format!("WH34/WN34(S/L) temp ch{}", t - WH34_SENSOR_CH1 + 1)
        }
        39 => "WH45 CO2".to_string(),
        t if (WH35_SENSOR_CH1..=WH35_SENSOR_CH8).contains(&t) => {
            format!("WH35/WN35 TF temp ch{}", t - WH35_SENSOR_CH1 + 1)
        }
        48 => "WH90 array".to_string(),
        t => format!("unknown(type={})", t),
    }
}

/// Parse SENSOR_ID_NEW payload entries: type(1) + id(4) + battery(1) + signal(1).
pub fn parse_sensor_id_entries(data: &[u8]) -> Vec<SensorIdEntry> {
    let mut entries = Vec::new();
    let mut index = 0;
    while index + 7 <= data.len() {
        entries.push(SensorIdEntry {
            sensor_type: data[index],
            sensor_id: u32::from_be_bytes([
                data[index + 1],
                data[index + 2],
                data[index + 3],
                data[index + 4],
            ]),
            battery_raw: data[index + 5],
            signal: data[index + 6],
        });
        index += 7;
    }
    entries
}

#[derive(Debug, Clone, PartialEq)]
pub struct LivedataProbeField {
    pub addr: u8,
    pub label: String,
    pub detail: String,
}

/// Walk livedata payload looking for soil-temp and TF (WH35) fields.
pub fn probe_livedata_fields(data: &[u8]) -> Vec<LivedataProbeField> {
    let mut fields = Vec::new();
    let mut index = 0;

    while index < data.len() {
        let addr = data[index];
        match addr {
            a if soil_temp_addr_channel(a).is_some() => {
                let ch = soil_temp_addr_channel(a).unwrap();
                if index + 2 < data.len() {
                    let temp = decode_temp(&data[index + 1..index + 3]);
                    fields.push(LivedataProbeField {
                        addr,
                        label: format!("ITEM_SOILTEMP{} → soil_temp_ch{}", ch, ch),
                        detail: format!("{:.1}°C", temp),
                    });
                    index += 3;
                } else {
                    break;
                }
            }
            a if (0x63..=0x6A).contains(&a) => {
                let ch = a - 0x62;
                if index + 3 < data.len() {
                    let temp = decode_temp(&data[index + 1..index + 3]);
                    let batt = data[index + 3] as f64 * 0.02;
                    fields.push(LivedataProbeField {
                        addr,
                        label: format!("ITEM_TF_USR{} (WH35/WN35)", ch),
                        detail: format!("{:.1}°C, battery {:.2} V", temp, batt),
                    });
                    index += 4;
                } else {
                    break;
                }
            }
            _ => {
                // Skip known TLV value sizes; unknown addresses advance one byte
                // (same strategy as production parse_livedata).
                let value_len = livedata_value_size(addr).unwrap_or(0);
                if value_len > 0 && index + value_len >= data.len() {
                    break;
                }
                index += 1 + value_len;
            }
        }
    }

    fields
}

fn soil_temp_addr_channel(addr: u8) -> Option<u8> {
    match addr {
        0x2B => Some(1),
        0x2D => Some(2),
        0x2F => Some(3),
        0x31 => Some(4),
        0x33 => Some(5),
        0x35 => Some(6),
        0x37 => Some(7),
        0x39 => Some(8),
        _ => None,
    }
}

/// Value byte count after the address byte for fields we skip over while probing.
fn livedata_value_size(addr: u8) -> Option<usize> {
    match addr {
        0x01 | 0x02 | 0x03 | 0x04 | 0x05 => Some(2), // temps
        0x06 | 0x07 => Some(1),                      // humidity
        0x08 | 0x09 => Some(2),                      // pressure
        0x0A | 0x0B | 0x0C => Some(2),               // wind
        0x0D | 0x0E | 0x0F | 0x10 | 0x11 => Some(2), // rain short
        0x12 | 0x13 | 0x14 | 0x15 => Some(4),        // rain int / light
        0x16 => Some(2),                             // uv
        0x17 => Some(1),                             // uvi
        0x18 => Some(6),                             // time
        0x19 => Some(2),                             // day max wind
        // soil moisture (WH51)
        0x2C | 0x2E | 0x30 | 0x32 | 0x34 | 0x36 | 0x38 | 0x3A => Some(1),
        0x6C => Some(4), // heap
        _ => None,
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum SensorPathVerdict {
    Wh34Confirmed,
    Wh35Path,
    NoneFound,
}

impl SensorPathVerdict {
    pub fn message(&self) -> &'static str {
        match self {
            Self::Wh34Confirmed => {
                "WN34/WH34 path confirmed (types 31–38 and/or soil_temp_ch* present)"
            }
            Self::Wh35Path => {
                "Data is on WH35/TF path (types 40–47 or ITEM_TF_USR); soil_temp parser will not see it"
            }
            Self::NoneFound => {
                "No temperature-probe sensors registered / no signal on WH34 or WH35 paths"
            }
        }
    }
}

pub fn verdict(
    entries: &[SensorIdEntry],
    livedata_fields: &[LivedataProbeField],
    parsed: &HashMap<String, f64>,
) -> SensorPathVerdict {
    let has_wh34_id = entries
        .iter()
        .any(|e| e.is_active() && (WH34_SENSOR_CH1..=WH34_SENSOR_CH8).contains(&e.sensor_type));
    // soil_temp_ch* only (not soil_temp_battery_ch* from SENSOR_ID).
    let has_soil_temp = parsed.keys().any(|k| k.starts_with("soil_temp_ch"))
        || livedata_fields.iter().any(|f| f.label.contains("SOILTEMP"));
    let has_wh35_id = entries
        .iter()
        .any(|e| e.is_active() && (WH35_SENSOR_CH1..=WH35_SENSOR_CH8).contains(&e.sensor_type));
    let has_tf = livedata_fields.iter().any(|f| f.label.contains("TF_USR"));

    // Livedata path evidence beats SENSOR_ID registration: a WH34 ID can remain
    // active while the gateway only emits ITEM_TF_USR* (WH35 path).
    if has_soil_temp {
        SensorPathVerdict::Wh34Confirmed
    } else if has_tf {
        SensorPathVerdict::Wh35Path
    } else if has_wh34_id {
        SensorPathVerdict::Wh34Confirmed
    } else if has_wh35_id {
        SensorPathVerdict::Wh35Path
    } else {
        SensorPathVerdict::NoneFound
    }
}

/// Connect, print the sensor inventory report, and return the verdict.
pub fn run_debug_sensors(client: &GW1000Client) -> Result<SensorPathVerdict> {
    println!("============================================================");
    println!("Sensor debug probe (WN34S / WH34 validation)");
    println!("============================================================");

    println!("\n--- Device Information ---");
    match client.get_firmware_version() {
        Ok(version) => println!("[OK] Firmware Version: {}", version),
        Err(e) => println!("[ERROR] Failed to get firmware: {}", e),
    }
    match client.get_mac_address() {
        Ok(mac) => println!("[OK] MAC Address: {}", mac),
        Err(e) => println!("[ERROR] Failed to get MAC: {}", e),
    }

    println!("\n--- SENSOR_ID_NEW (0x3C) ---");
    let sensor_payload = client.fetch_sensor_id_payload()?;
    let entries = parse_sensor_id_entries(&sensor_payload);
    if entries.is_empty() {
        println!("(no sensor ID entries)");
    } else {
        for e in &entries {
            let status = if e.is_active() { "active" } else { "inactive" };
            let batt = e
                .battery_volts()
                .map(|v| format!("{:.2} V", v))
                .unwrap_or_else(|| format!("raw={}", e.battery_raw));
            let key = e
                .listener_battery_key()
                .map(|k| format!(" → {}", k))
                .unwrap_or_default();
            println!(
                "  [{:>8}] type={:<3} {:<36} id={:08X}  batt={}  signal={}{}",
                status,
                e.sensor_type,
                e.type_label(),
                e.sensor_id,
                batt,
                e.signal,
                key
            );
        }
    }

    println!("\n--- Livedata soil / TF fields (0x27) ---");
    let livedata_payload = client.fetch_livedata_payload()?;
    let probe_fields = probe_livedata_fields(&livedata_payload);
    let parsed = client.parse_livedata_public(&livedata_payload)?;

    let soil_related: Vec<_> = probe_fields
        .iter()
        .filter(|f| f.label.contains("SOILTEMP") || f.label.contains("TF_USR"))
        .collect();
    if soil_related.is_empty() {
        println!("(no ITEM_SOILTEMP* or ITEM_TF_USR* fields found in payload walk)");
    } else {
        for f in &soil_related {
            println!("  0x{:02X}  {}  {}", f.addr, f.label, f.detail);
        }
    }

    println!("\n--- Listener-decoded soil keys ---");
    // Merge batteries from sensor-ID the same way get_livedata does.
    let mut merged = parsed;
    for e in entries.iter().filter(|e| e.is_active()) {
        if let (Some(key), Some(volts)) = (e.listener_battery_key(), e.battery_volts()) {
            if key.starts_with("soil_") {
                merged.insert(key, volts);
            }
        }
    }
    let mut merged_keys: Vec<_> = merged
        .iter()
        .filter(|(k, _)| {
            k.starts_with("soil_temp_ch")
                || k.starts_with("soil_temp_battery_ch")
                || k.starts_with("soil_moisture_ch")
                || k.starts_with("soil_battery_ch")
        })
        .collect();
    merged_keys.sort_by(|a, b| a.0.cmp(b.0));
    if merged_keys.is_empty() {
        println!("(none)");
    } else {
        for (k, v) in merged_keys {
            println!("  {}: {}", k, format_value(k, *v));
        }
    }

    println!("\n--- Payload hex (livedata) ---");
    println!("  {}", hex_preview(&livedata_payload, 64));

    let v = verdict(&entries, &probe_fields, &merged);
    println!("\n--- Verdict ---");
    println!("{}", v.message());
    println!();

    Ok(v)
}

fn hex_preview(data: &[u8], max: usize) -> String {
    let take = data.len().min(max);
    let mut s = data[..take]
        .iter()
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<_>>()
        .join(" ");
    if data.len() > max {
        s.push_str(&format!(" … ({} bytes total)", data.len()));
    } else {
        s.push_str(&format!(" ({} bytes)", data.len()));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sensor_type_label_wh34_wh35() {
        assert!(sensor_type_label(31).contains("WN34"));
        assert!(sensor_type_label(38).contains("ch8"));
        assert!(sensor_type_label(40).contains("WN35"));
        assert!(sensor_type_label(14).contains("WH51"));
    }

    #[test]
    fn test_parse_sensor_id_entries() {
        let data = [
            31, 0xAA, 0xBB, 0xCC, 0xDD, 62, 3, // WH34 active
            15, 0xFF, 0xFF, 0xFF, 0xFE, 0, 0, // WH51 disabled
            40, 0x11, 0x22, 0x33, 0x44, 50, 2, // WH35 active
        ];
        let entries = parse_sensor_id_entries(&data);
        assert_eq!(entries.len(), 3);
        assert!(entries[0].is_active());
        assert!(!entries[1].is_active());
        assert!(entries[2].is_active());
        assert!((entries[0].battery_volts().unwrap() - 1.24).abs() < 0.001);
        assert_eq!(
            entries[0].listener_battery_key().as_deref(),
            Some("soil_temp_battery_ch1")
        );
        assert!(entries[2].listener_battery_key().is_none());
    }

    #[test]
    fn test_probe_livedata_soil_and_tf() {
        let data = [
            0x02, 0x00, 0xC8, // outtemp — skipped by probe list filter but walked
            0x2B, 0x00, 0xB9, // soil_temp_ch1 = 18.5
            0x63, 0x00, 0xC8, 62, // TF_USR1 = 20.0°C, 1.24V
        ];
        let fields = probe_livedata_fields(&data);
        assert_eq!(fields.len(), 2);
        assert!(fields[0].label.contains("SOILTEMP"));
        assert!(fields[0].detail.contains("18.5"));
        assert!(fields[1].label.contains("TF_USR1"));
        assert!(fields[1].detail.contains("20.0"));
    }

    #[test]
    fn test_verdict_wh34() {
        let entries = parse_sensor_id_entries(&[31, 0xAA, 0xBB, 0xCC, 0xDD, 62, 3]);
        let mut parsed = HashMap::new();
        parsed.insert("soil_temp_ch1".to_string(), 18.5);
        assert_eq!(
            verdict(&entries, &[], &parsed),
            SensorPathVerdict::Wh34Confirmed
        );
    }

    #[test]
    fn test_verdict_wh35() {
        let entries = parse_sensor_id_entries(&[40, 0x11, 0x22, 0x33, 0x44, 50, 2]);
        let fields = probe_livedata_fields(&[0x63, 0x00, 0xC8, 50]);
        let parsed = HashMap::new();
        assert_eq!(
            verdict(&entries, &fields, &parsed),
            SensorPathVerdict::Wh35Path
        );
    }

    #[test]
    fn test_verdict_tf_livedata_beats_wh34_id() {
        // WH34 registered in SENSOR_ID, but livedata only has ITEM_TF_USR*.
        let entries = parse_sensor_id_entries(&[31, 0xAA, 0xBB, 0xCC, 0xDD, 62, 3]);
        let fields = probe_livedata_fields(&[0x63, 0x00, 0xC8, 50]);
        let mut parsed = HashMap::new();
        // Battery keys from SENSOR_ID must not count as WH34 temperature data.
        parsed.insert("soil_temp_battery_ch1".to_string(), 1.24);
        assert_eq!(
            verdict(&entries, &fields, &parsed),
            SensorPathVerdict::Wh35Path
        );
    }

    #[test]
    fn test_verdict_wh34_id_only() {
        let entries = parse_sensor_id_entries(&[31, 0xAA, 0xBB, 0xCC, 0xDD, 62, 3]);
        let parsed = HashMap::new();
        assert_eq!(
            verdict(&entries, &[], &parsed),
            SensorPathVerdict::Wh34Confirmed
        );
    }

    #[test]
    fn test_verdict_none() {
        let parsed = HashMap::new();
        assert_eq!(verdict(&[], &[], &parsed), SensorPathVerdict::NoneFound);
    }
}
