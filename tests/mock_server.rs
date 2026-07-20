/// Mock TCP server for testing GW1000 client
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// Mock GW1000 device server
pub struct MockGW1000Server {
    listener: TcpListener,
    responses: Arc<Mutex<Vec<Vec<u8>>>>,
}

impl MockGW1000Server {
    /// Create a new mock server on a random available port
    pub fn new() -> std::io::Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        listener.set_nonblocking(false)?;

        Ok(Self {
            listener,
            responses: Arc::new(Mutex::new(Vec::new())),
        })
    }

    /// Get the address the server is listening on
    #[allow(dead_code)]
    pub fn addr(&self) -> String {
        format!("127.0.0.1:{}", self.listener.local_addr().unwrap().port())
    }

    /// Get the port the server is listening on
    pub fn port(&self) -> u16 {
        self.listener.local_addr().unwrap().port()
    }

    /// Add a canned response for the next request
    pub fn add_response(&self, response: Vec<u8>) {
        self.responses.lock().unwrap().push(response);
    }

    /// Start the server in a background thread
    pub fn start(self) -> ServerHandle {
        let responses = Arc::clone(&self.responses);
        let listener = self.listener;

        let handle = thread::spawn(move || {
            for stream in listener.incoming() {
                match stream {
                    Ok(mut stream) => {
                        // Set a short timeout to avoid hanging
                        stream.set_read_timeout(Some(Duration::from_secs(1))).ok();
                        stream.set_write_timeout(Some(Duration::from_secs(1))).ok();

                        // Read the request
                        let mut buffer = vec![0u8; 1024];
                        if let Ok(n) = stream.read(&mut buffer) {
                            if n > 0 {
                                // Get the next canned response
                                let response = responses.lock().unwrap().pop();

                                if let Some(resp) = response {
                                    // Send the response
                                    stream.write_all(&resp).ok();
                                    stream.flush().ok();
                                }
                            }
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        ServerHandle { handle }
    }
}

/// Handle to a running server
#[allow(dead_code)]
pub struct ServerHandle {
    handle: thread::JoinHandle<()>,
}

impl ServerHandle {
    /// Wait for the server to finish (it won't unless the listener is dropped)
    #[allow(dead_code)]
    pub fn join(self) {
        self.handle.join().ok();
    }
}

/// Helper function to create a valid firmware version response
pub fn mock_firmware_response(version: &str) -> Vec<u8> {
    let mut response = vec![
        0xFF, 0xFF, // Header
        0x50, // Command (CMD_READ_FIRMWARE_VERSION)
    ];

    let version_bytes = version.as_bytes();
    let size = 1 + 1 + version_bytes.len() + 1; // cmd + size + data + checksum
    response.push(size as u8);
    response.extend_from_slice(version_bytes);

    // Calculate checksum
    let checksum: u8 = response[2..].iter().map(|&b| b as u32).sum::<u32>() as u8;
    response.push(checksum);

    response
}

/// Helper function to create a valid MAC address response
pub fn mock_mac_response(mac: &[u8; 6]) -> Vec<u8> {
    let mut response = vec![
        0xFF, 0xFF, // Header
        0x26, // Command (CMD_READ_STATION_MAC)
        0x09, // Size
    ];

    response.extend_from_slice(mac);

    // Calculate checksum
    let checksum: u8 = response[2..].iter().map(|&b| b as u32).sum::<u32>() as u8;
    response.push(checksum);

    response
}

/// Helper function to create a minimal live data response
pub fn mock_livedata_response() -> Vec<u8> {
    let mut response = vec![
        0xFF, 0xFF, // Header
        0x27, // Command (CMD_GW1000_LIVEDATA)
    ];

    // Build the data payload
    let mut data = Vec::new();

    // 0x02: outtemp = 25.5°C (255 = 0x00FF)
    data.extend_from_slice(&[0x02, 0x00, 0xFF]);

    // 0x07: outhumid = 65%
    data.extend_from_slice(&[0x07, 0x41]);

    // 0x2C: soil_moisture_ch1 = 78%
    data.extend_from_slice(&[0x2C, 0x4E]);

    // 0x63: TF_USR1 = 23.1°C (231 = 0x00E7), battery 81 → 1.62V
    data.extend_from_slice(&[0x63, 0x00, 0xE7, 81]);

    // 0x64: TF_USR2 = 19.5°C (195 = 0x00C3), battery 80 → 1.60V
    data.extend_from_slice(&[0x64, 0x00, 0xC3, 80]);

    // Calculate size: cmd(1) + size(2) + data + checksum(1)
    let size = 1 + 2 + data.len() + 1;

    // Add size as big-endian u16
    response.push(((size >> 8) & 0xFF) as u8);
    response.push((size & 0xFF) as u8);

    // Add data
    response.extend_from_slice(&data);

    // Calculate checksum (from command onwards, excluding header and checksum itself)
    let checksum: u8 = response[2..].iter().map(|&b| b as u32).sum::<u32>() as u8;
    response.push(checksum);

    response
}

/// Helper for CMD_READ_SENSOR_ID_NEW (0x3C) with WH51 moisture + WH34 temp probes.
/// Entry layout: type(1) + id(4) + battery(1) + signal(1).
/// WH51 ch1 type = 14; battery 15 → 1.5 V; signal = 4.
/// WH34 ch1 type = 31; battery present but livedata TF_USR battery should win on merge.
pub fn mock_sensor_id_response() -> Vec<u8> {
    let mut response = vec![
        0xFF, 0xFF, // Header
        0x3C, // Command (CMD_READ_SENSOR_ID_NEW)
    ];

    let mut data = Vec::new();
    // WH51 channel 1: type=14, id=0x12345678, battery=15 (1.5V), signal=4
    data.extend_from_slice(&[14, 0x12, 0x34, 0x56, 0x78, 15, 4]);
    // Disabled WH51 ch2: type=15, id=0xFFFFFFFE, battery=0, signal=0
    data.extend_from_slice(&[15, 0xFF, 0xFF, 0xFF, 0xFE, 0, 0]);
    // WH34 ch1: type=31, id=0xAABBCCDD, battery=62 (1.24V), signal=3
    // Livedata TF_USR1 carries 1.62V — that value must win after merge.
    data.extend_from_slice(&[31, 0xAA, 0xBB, 0xCC, 0xDD, 62, 3]);

    let size = 1 + 2 + data.len() + 1;
    response.push(((size >> 8) & 0xFF) as u8);
    response.push((size & 0xFF) as u8);
    response.extend_from_slice(&data);

    let checksum: u8 = response[2..].iter().map(|&b| b as u32).sum::<u32>() as u8;
    response.push(checksum);

    response
}

/// Queue livedata + sensor-ID responses in the order get_livedata() will consume them.
/// Mock server pops LIFO, so sensor-ID is pushed first.
pub fn queue_livedata_with_battery(server: &MockGW1000Server) {
    server.add_response(mock_sensor_id_response());
    server.add_response(mock_livedata_response());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_server_creation() {
        let server = MockGW1000Server::new().unwrap();
        assert!(server.port() > 0);
    }

    #[test]
    fn test_mock_firmware_response_structure() {
        let response = mock_firmware_response("GW2000B_V3.1.4");
        assert_eq!(response[0], 0xFF);
        assert_eq!(response[1], 0xFF);
        assert_eq!(response[2], 0x50);
    }

    #[test]
    fn test_mock_mac_response_structure() {
        let mac = [0xEC, 0x62, 0x60, 0xE0, 0x6E, 0x6F];
        let response = mock_mac_response(&mac);
        assert_eq!(response[0], 0xFF);
        assert_eq!(response[1], 0xFF);
        assert_eq!(response[2], 0x26);
        assert_eq!(response[3], 0x09);
    }

    #[test]
    fn test_mock_livedata_response_structure() {
        let response = mock_livedata_response();
        assert_eq!(response[0], 0xFF);
        assert_eq!(response[1], 0xFF);
        assert_eq!(response[2], 0x27);
    }
}
