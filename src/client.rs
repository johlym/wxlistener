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

// Protocol constants
const SOCKET_TIMEOUT: Duration = Duration::from_secs(5);

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
        let mut stream = TcpStream::connect_timeout(
            &addr.parse()?,
            SOCKET_TIMEOUT
        ).context("Failed to connect to device")?;

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
            let mac = data.iter()
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
            self.parse_livedata(data)
        } else {
            anyhow::bail!("Invalid live data response")
        }
    }

    fn parse_livedata(&self, data: &[u8]) -> Result<HashMap<String, f64>> {
        let mut result = HashMap::new();
        let mut index = 0;

        while index < data.len() {
            let field_addr = data[index];
            
            match field_addr {
                0x01 => { // intemp
                    if index + 2 < data.len() {
                        let val = decode_temp(&data[index+1..index+3]);
                        result.insert("intemp".to_string(), val);
                        index += 3;
                    } else { break; }
                }
                0x02 => { // outtemp
                    if index + 2 < data.len() {
                        let val = decode_temp(&data[index+1..index+3]);
                        result.insert("outtemp".to_string(), val);
                        index += 3;
                    } else { break; }
                }
                0x06 => { // inhumid
                    if index + 1 < data.len() {
                        result.insert("inhumid".to_string(), data[index+1] as f64);
                        index += 2;
                    } else { break; }
                }
                0x07 => { // outhumid
                    if index + 1 < data.len() {
                        result.insert("outhumid".to_string(), data[index+1] as f64);
                        index += 2;
                    } else { break; }
                }
                0x08 => { // absbarometer
                    if index + 2 < data.len() {
                        let val = decode_pressure(&data[index+1..index+3]);
                        result.insert("absbarometer".to_string(), val);
                        index += 3;
                    } else { break; }
                }
                0x09 => { // relbarometer
                    if index + 2 < data.len() {
                        let val = decode_pressure(&data[index+1..index+3]);
                        result.insert("relbarometer".to_string(), val);
                        index += 3;
                    } else { break; }
                }
                0x0A => { // wind_dir
                    if index + 2 < data.len() {
                        let val = decode_short(&data[index+1..index+3]);
                        result.insert("wind_dir".to_string(), val);
                        index += 3;
                    } else { break; }
                }
                0x0B => { // wind_speed
                    if index + 2 < data.len() {
                        let val = decode_wind(&data[index+1..index+3]);
                        result.insert("wind_speed".to_string(), val);
                        index += 3;
                    } else { break; }
                }
                0x0C => { // gust_speed
                    if index + 2 < data.len() {
                        let val = decode_wind(&data[index+1..index+3]);
                        result.insert("gust_speed".to_string(), val);
                        index += 3;
                    } else { break; }
                }
                0x0D => { // rain_event
                    if index + 2 < data.len() {
                        let val = decode_rain(&data[index+1..index+3]);
                        result.insert("rain_event".to_string(), val);
                        index += 3;
                    } else { break; }
                }
                0x0E => { // rain_rate
                    if index + 2 < data.len() {
                        let val = decode_rain(&data[index+1..index+3]);
                        result.insert("rain_rate".to_string(), val);
                        index += 3;
                    } else { break; }
                }
                0x10 => { // rain_day
                    if index + 2 < data.len() {
                        let val = decode_rain(&data[index+1..index+3]);
                        result.insert("rain_day".to_string(), val);
                        index += 3;
                    } else { break; }
                }
                0x11 => { // rain_week
                    if index + 2 < data.len() {
                        let val = decode_rain(&data[index+1..index+3]);
                        result.insert("rain_week".to_string(), val);
                        index += 3;
                    } else { break; }
                }
                0x12 => { // rain_month
                    if index + 4 < data.len() {
                        let val = decode_int(&data[index+1..index+5]) / 10.0;
                        result.insert("rain_month".to_string(), val);
                        index += 5;
                    } else { break; }
                }
                0x13 => { // rain_year
                    if index + 4 < data.len() {
                        let val = decode_int(&data[index+1..index+5]) / 10.0;
                        result.insert("rain_year".to_string(), val);
                        index += 5;
                    } else { break; }
                }
                0x15 => { // light
                    if index + 4 < data.len() {
                        let val = decode_int(&data[index+1..index+5]) / 10.0;
                        result.insert("light".to_string(), val);
                        index += 5;
                    } else { break; }
                }
                0x16 => { // uv
                    if index + 2 < data.len() {
                        let val = decode_short(&data[index+1..index+3]);
                        result.insert("uv".to_string(), val);
                        index += 3;
                    } else { break; }
                }
                0x17 => { // uvi
                    if index + 1 < data.len() {
                        result.insert("uvi".to_string(), data[index+1] as f64);
                        index += 2;
                    } else { break; }
                }
                0x19 => { // day_max_wind
                    if index + 2 < data.len() {
                        let val = decode_wind(&data[index+1..index+3]);
                        result.insert("day_max_wind".to_string(), val);
                        index += 3;
                    } else { break; }
                }
                0x6C => { // heap_free
                    if index + 4 < data.len() {
                        let val = decode_int(&data[index+1..index+5]);
                        result.insert("heap_free".to_string(), val);
                        index += 5;
                    } else { break; }
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
