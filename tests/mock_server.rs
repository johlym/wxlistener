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
