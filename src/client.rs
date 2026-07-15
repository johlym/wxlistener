use anyhow::{Context, Result};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

use crate::decoder::*;
use crate::protocol::{build_cmd_packet, verify_response};

// API Command codes
const CMD_READ_FIRMWARE_VERSION: u8 = 0x50;
const CMD_READ_STATION_MAC: u8 = 0x26;
const CMD_GW1000_LIVEDATA: u8 = 0x27;
const CMD_READ_SENSOR_ID_NEW: u8 = 0x3C;

// Protocol constants
const SOCKET_TIMEOUT: Duration = Duration::from_secs(16);

/// SENSOR_IDT indices for WH51 soil moisture channels 1–8
const WH51_SENSOR_CH1: u8 = 14;
const WH51_SENSOR_CH8: u8 = 21;

/// SENSOR_IDT indices for WN34/WH34 soil temperature channels 1–8
const WH34_SENSOR_CH1: u8 = 31;
const WH34_SENSOR_CH8: u8 = 38;

/// Disabled / re-learn sentinel IDs from the protocol manual
const SENSOR_ID_DISABLED: u32 = 0xFFFF_FFFE;
const SENSOR_ID_RELEARN: u32 = 0xFFFF_FFFF;

pub struct GW1000Client {
    ip: String,
    port: u16,
}

impl GW1000Client {
    pub fn new(ip: String, port: u16) -> Self {
        Self { ip, port }
    }

    fn build_cmd_packet(&self, cmd_code: u8, payload: &[u8]) -> Vec<u8> {
        build_cmd_packet(cmd_code, payload)
    }

    fn send_cmd(&self, packet: &[u8]) -> Result<Vec<u8>> {
        let addr = format!("{}:{}", self.ip, self.port);
        let mut stream = TcpStream::connect_timeout(&addr.parse()?, SOCKET_TIMEOUT)
            .context("Failed to connect to device")?;

        stream.set_read_timeout(Some(SOCKET_TIMEOUT))?;
        stream.set_write_timeout(Some(SOCKET_TIMEOUT))?;

        stream.write_all(packet)?;

        let mut response = vec![0u8; 1024];
        let n = stream.read(&mut response)?;
        response.truncate(n);

        Ok(response)
    }

    fn check_response(&self, response: &[u8], expected_cmd: u8) -> bool {
        verify_response(response, expected_cmd)
    }

    pub fn get_firmware_version(&self) -> Result<String> {
        let packet = self.build_cmd_packet(CMD_READ_FIRMWARE_VERSION, &[]);
        let response = self.send_cmd(&packet)?;

        if self.check_response(&response, CMD_READ_FIRMWARE_VERSION) {
            let size = response[3] as usize;
            let data = &response[4..4 + size - 3];
            Ok(String::from_utf8_lossy(data).to_string())
        } else {
            anyhow::bail!("Invalid firmware version response")
        }
    }

    pub fn get_mac_address(&self) -> Result<String> {
        let packet = self.build_cmd_packet(CMD_READ_STATION_MAC, &[]);
        let response = self.send_cmd(&packet)?;

        if self.check_response(&response, CMD_READ_STATION_MAC) {
            let size = response[3] as usize;
            let data = &response[4..4 + size - 3];
            let mac = data
                .iter()
                .map(|b| format!("{:02X}", b))
                .collect::<Vec<_>>()
                .join(":");
            Ok(mac)
        } else {
            anyhow::bail!("Invalid MAC address response")
        }
    }

    pub fn get_livedata(&self) -> Result<HashMap<String, f64>> {
        let packet = self.build_cmd_packet(CMD_GW1000_LIVEDATA, &[]);
        let response = self.send_cmd(&packet)?;

        if self.check_response(&response, CMD_GW1000_LIVEDATA) {
            // CMD_GW1000_LIVEDATA uses 2-byte size field (big-endian)
            let size = ((response[3] as usize) << 8) | (response[4] as usize);
            let data = &response[5..5 + size - 4];
            let mut result = self.parse_livedata(data)?;

            // Battery status for soil sensors lives in CMD_READ_SENSOR_ID_NEW (0x3C),
            // not in the livedata packet. Best-effort merge — livedata still succeeds
            // if the sensor-ID call fails.
            match self.get_soil_battery_data() {
                Ok(battery) => result.extend(battery),
                Err(e) => {
                    eprintln!("[WARN] Failed to read soil sensor battery status: {}", e);
                }
            }

            Ok(result)
        } else {
            anyhow::bail!("Invalid live data response")
        }
    }

    /// Read CMD_READ_SENSOR_ID_NEW and extract WH51 / WH34 soil battery values.
    ///
    /// WH51 (moisture): battery byte is voltage × 10 (volts = val × 0.1).
    /// WH34 (soil temp): battery byte is voltage × 50 (volts = val × 0.02).
    fn get_soil_battery_data(&self) -> Result<HashMap<String, f64>> {
        let packet = self.build_cmd_packet(CMD_READ_SENSOR_ID_NEW, &[]);
        let response = self.send_cmd(&packet)?;

        if !self.check_response(&response, CMD_READ_SENSOR_ID_NEW) {
            anyhow::bail!("Invalid sensor ID response");
        }

        // Response size is 2 bytes (same as livedata)
        if response.len() < 6 {
            anyhow::bail!("Sensor ID response too short");
        }
        let size = ((response[3] as usize) << 8) | (response[4] as usize);
        let end = 5 + size.saturating_sub(4);
        if end > response.len() {
            anyhow::bail!("Sensor ID response size exceeds packet length");
        }
        let data = &response[5..end];
        Ok(Self::parse_soil_battery(data))
    }

    /// Parse sensor-ID payload entries: type(1) + id(4) + battery(1) + signal(1).
    fn parse_soil_battery(data: &[u8]) -> HashMap<String, f64> {
        let mut result = HashMap::new();
        let mut index = 0;

        while index + 7 <= data.len() {
            let sensor_type = data[index];
            let sensor_id = u32::from_be_bytes([
                data[index + 1],
                data[index + 2],
                data[index + 3],
                data[index + 4],
            ]);
            let battery = data[index + 5];
            let signal = data[index + 6];
            index += 7;

            // Skip disabled / unregistered / no-signal sensors
            if sensor_id == SENSOR_ID_DISABLED
                || sensor_id == SENSOR_ID_RELEARN
                || sensor_id == 0
                || signal == 0
            {
                continue;
            }

            if (WH51_SENSOR_CH1..=WH51_SENSOR_CH8).contains(&sensor_type) {
                let ch = (sensor_type - WH51_SENSOR_CH1) + 1;
                // WH51: volts = val × 0.1
                result.insert(format!("soil_battery_ch{}", ch), battery as f64 * 0.1);
            } else if (WH34_SENSOR_CH1..=WH34_SENSOR_CH8).contains(&sensor_type) {
                let ch = (sensor_type - WH34_SENSOR_CH1) + 1;
                // WH34: volts = val × 0.02 — store under soil_temp_battery so it
                // doesn't collide with WH51 moisture battery on the same channel.
                result.insert(format!("soil_temp_battery_ch{}", ch), battery as f64 * 0.02);
            }
        }

        result
    }

    fn parse_livedata(&self, data: &[u8]) -> Result<HashMap<String, f64>> {
        let mut result = HashMap::new();
        let mut index = 0;

        while index < data.len() {
            let field_addr = data[index];

            match field_addr {
                0x01 => {
                    // intemp
                    if index + 2 < data.len() {
                        let val = decode_temp(&data[index + 1..index + 3]);
                        result.insert("intemp".to_string(), val);
                        index += 3;
                    } else {
                        break;
                    }
                }
                0x02 => {
                    // outtemp
                    if index + 2 < data.len() {
                        let val = decode_temp(&data[index + 1..index + 3]);
                        result.insert("outtemp".to_string(), val);
                        index += 3;
                    } else {
                        break;
                    }
                }
                0x03 => {
                    // dew point
                    if index + 2 < data.len() {
                        let val = decode_temp(&data[index + 1..index + 3]);
                        result.insert("dewpoint".to_string(), val);
                        index += 3;
                    } else {
                        break;
                    }
                }
                0x04 => {
                    // wind chill
                    if index + 2 < data.len() {
                        let val = decode_temp(&data[index + 1..index + 3]);
                        result.insert("windchill".to_string(), val);
                        index += 3;
                    } else {
                        break;
                    }
                }
                0x05 => {
                    // heat index
                    if index + 2 < data.len() {
                        let val = decode_temp(&data[index + 1..index + 3]);
                        result.insert("heatindex".to_string(), val);
                        index += 3;
                    } else {
                        break;
                    }
                }
                0x06 => {
                    // inhumid
                    if index + 1 < data.len() {
                        result.insert("inhumid".to_string(), data[index + 1] as f64);
                        index += 2;
                    } else {
                        break;
                    }
                }
                0x07 => {
                    // outhumid
                    if index + 1 < data.len() {
                        result.insert("outhumid".to_string(), data[index + 1] as f64);
                        index += 2;
                    } else {
                        break;
                    }
                }
                0x08 => {
                    // absbarometer
                    if index + 2 < data.len() {
                        let val = decode_pressure(&data[index + 1..index + 3]);
                        result.insert("absbarometer".to_string(), val);
                        index += 3;
                    } else {
                        break;
                    }
                }
                0x09 => {
                    // relbarometer
                    if index + 2 < data.len() {
                        let val = decode_pressure(&data[index + 1..index + 3]);
                        result.insert("relbarometer".to_string(), val);
                        index += 3;
                    } else {
                        break;
                    }
                }
                0x0A => {
                    // wind_dir
                    if index + 2 < data.len() {
                        let val = decode_short(&data[index + 1..index + 3]);
                        result.insert("wind_dir".to_string(), val);
                        index += 3;
                    } else {
                        break;
                    }
                }
                0x0B => {
                    // wind_speed
                    if index + 2 < data.len() {
                        let val = decode_wind(&data[index + 1..index + 3]);
                        result.insert("wind_speed".to_string(), val);
                        index += 3;
                    } else {
                        break;
                    }
                }
                0x0C => {
                    // gust_speed
                    if index + 2 < data.len() {
                        let val = decode_wind(&data[index + 1..index + 3]);
                        result.insert("gust_speed".to_string(), val);
                        index += 3;
                    } else {
                        break;
                    }
                }
                0x0D => {
                    // rain_event
                    if index + 2 < data.len() {
                        let val = decode_rain(&data[index + 1..index + 3]);
                        result.insert("rain_event".to_string(), val);
                        index += 3;
                    } else {
                        break;
                    }
                }
                0x0E => {
                    // rain_rate
                    if index + 2 < data.len() {
                        let val = decode_rain(&data[index + 1..index + 3]);
                        result.insert("rain_rate".to_string(), val);
                        index += 3;
                    } else {
                        break;
                    }
                }
                0x10 => {
                    // rain_day
                    if index + 2 < data.len() {
                        let val = decode_rain(&data[index + 1..index + 3]);
                        result.insert("rain_day".to_string(), val);
                        index += 3;
                    } else {
                        break;
                    }
                }
                0x11 => {
                    // rain_week
                    if index + 2 < data.len() {
                        let val = decode_rain(&data[index + 1..index + 3]);
                        result.insert("rain_week".to_string(), val);
                        index += 3;
                    } else {
                        break;
                    }
                }
                0x12 => {
                    // rain_month
                    if index + 4 < data.len() {
                        let val = decode_int(&data[index + 1..index + 5]) / 10.0;
                        result.insert("rain_month".to_string(), val);
                        index += 5;
                    } else {
                        break;
                    }
                }
                0x13 => {
                    // rain_year
                    if index + 4 < data.len() {
                        let val = decode_int(&data[index + 1..index + 5]) / 10.0;
                        result.insert("rain_year".to_string(), val);
                        index += 5;
                    } else {
                        break;
                    }
                }
                0x15 => {
                    // light
                    if index + 4 < data.len() {
                        let val = decode_int(&data[index + 1..index + 5]) / 10.0;
                        result.insert("light".to_string(), val);
                        index += 5;
                    } else {
                        break;
                    }
                }
                0x16 => {
                    // uv
                    if index + 2 < data.len() {
                        let val = decode_short(&data[index + 1..index + 3]);
                        result.insert("uv".to_string(), val);
                        index += 3;
                    } else {
                        break;
                    }
                }
                0x17 => {
                    // uvi
                    if index + 1 < data.len() {
                        result.insert("uvi".to_string(), data[index + 1] as f64);
                        index += 2;
                    } else {
                        break;
                    }
                }
                0x19 => {
                    // day_max_wind
                    if index + 2 < data.len() {
                        let val = decode_wind(&data[index + 1..index + 3]);
                        result.insert("day_max_wind".to_string(), val);
                        index += 3;
                    } else {
                        break;
                    }
                }
                // Soil temperature channels 1–8 (WN34): 2-byte signed temp ×10
                0x2B | 0x2D | 0x2F | 0x31 | 0x33 | 0x35 | 0x37 | 0x39 => {
                    if let Some(ch) = soil_temp_channel(field_addr) {
                        if index + 2 < data.len() {
                            let val = decode_temp(&data[index + 1..index + 3]);
                            result.insert(format!("soil_temp_ch{}", ch), val);
                            index += 3;
                        } else {
                            break;
                        }
                    } else {
                        index += 1;
                    }
                }
                // Soil moisture channels 1–8 (WH51): 1-byte percent
                0x2C | 0x2E | 0x30 | 0x32 | 0x34 | 0x36 | 0x38 | 0x3A => {
                    if let Some(ch) = soil_moisture_channel(field_addr) {
                        if index + 1 < data.len() {
                            result
                                .insert(format!("soil_moisture_ch{}", ch), data[index + 1] as f64);
                            index += 2;
                        } else {
                            break;
                        }
                    } else {
                        index += 1;
                    }
                }
                0x6C => {
                    // heap_free
                    if index + 4 < data.len() {
                        let val = decode_int(&data[index + 1..index + 5]);
                        result.insert("heap_free".to_string(), val);
                        index += 5;
                    } else {
                        break;
                    }
                }
                _ => {
                    // Unknown field, skip it
                    index += 1;
                }
            }
        }

        Ok(result)
    }
}

fn soil_temp_channel(addr: u8) -> Option<u8> {
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

fn soil_moisture_channel(addr: u8) -> Option<u8> {
    match addr {
        0x2C => Some(1),
        0x2E => Some(2),
        0x30 => Some(3),
        0x32 => Some(4),
        0x34 => Some(5),
        0x36 => Some(6),
        0x38 => Some(7),
        0x3A => Some(8),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_soil_moisture_and_temp() {
        let client = GW1000Client::new("127.0.0.1".to_string(), 45000);
        let data = [
            0x2C, 78, // soil_moisture_ch1 = 78%
            0x2B, 0x00, 0xB9, // soil_temp_ch1 = 18.5°C
            0x2E, 50, // soil_moisture_ch2 = 50%
            0x39, 0x00, 0xC8, // soil_temp_ch8 = 20.0°C
        ];
        let result = client.parse_livedata(&data).unwrap();

        assert_eq!(result.get("soil_moisture_ch1"), Some(&78.0));
        assert_eq!(result.get("soil_temp_ch1"), Some(&18.5));
        assert_eq!(result.get("soil_moisture_ch2"), Some(&50.0));
        assert_eq!(result.get("soil_temp_ch8"), Some(&20.0));
    }

    #[test]
    fn test_parse_soil_battery() {
        let data = [
            // WH51 ch1: type=14, id=0x12345678, battery=15 → 1.5V, signal=4
            14, 0x12, 0x34, 0x56, 0x78, 15, 4, // WH51 ch2 disabled
            15, 0xFF, 0xFF, 0xFF, 0xFE, 0, 0,
            // WH34 ch1: type=31, id=0xAABBCCDD, battery=62 → 1.24V, signal=3
            31, 0xAA, 0xBB, 0xCC, 0xDD, 62, 3,
        ];
        let result = GW1000Client::parse_soil_battery(&data);

        assert_eq!(result.get("soil_battery_ch1"), Some(&1.5));
        assert!(!result.contains_key("soil_battery_ch2"));
        assert!((result.get("soil_temp_battery_ch1").unwrap() - 1.24).abs() < 0.001);
    }

    #[test]
    fn test_soil_channel_helpers() {
        assert_eq!(soil_moisture_channel(0x2C), Some(1));
        assert_eq!(soil_moisture_channel(0x3A), Some(8));
        assert_eq!(soil_moisture_channel(0x2B), None);
        assert_eq!(soil_temp_channel(0x2B), Some(1));
        assert_eq!(soil_temp_channel(0x39), Some(8));
        assert_eq!(soil_temp_channel(0x2C), None);
    }
}
