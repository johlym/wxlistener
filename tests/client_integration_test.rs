/// Integration tests with mock TCP server
mod mock_server;

use mock_server::{
    mock_firmware_response, mock_livedata_response, mock_mac_response, MockGW1000Server,
};
use wxlistener::client::GW1000Client;

#[test]
fn test_client_get_firmware_version() {
    // Create mock server
    let server = MockGW1000Server::new().unwrap();
    let port = server.port();

    // Add canned response
    server.add_response(mock_firmware_response("GW2000B_V3.1.4"));

    // Start server in background
    let _handle = server.start();

    // Give server time to start
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Create client and test
    let client = GW1000Client::new("127.0.0.1".to_string(), port);
    let result = client.get_firmware_version();

    assert!(result.is_ok());
    let version = result.unwrap();
    assert_eq!(version, "GW2000B_V3.1.4");
}

#[test]
fn test_client_get_mac_address() {
    // Create mock server
    let server = MockGW1000Server::new().unwrap();
    let port = server.port();

    // Add canned response
    let mac = [0xEC, 0x62, 0x60, 0xE0, 0x6E, 0x6F];
    server.add_response(mock_mac_response(&mac));

    // Start server in background
    let _handle = server.start();

    // Give server time to start
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Create client and test
    let client = GW1000Client::new("127.0.0.1".to_string(), port);
    let result = client.get_mac_address();

    assert!(result.is_ok());
    let mac_str = result.unwrap();
    assert_eq!(mac_str, "EC:62:60:E0:6E:6F");
}

#[test]
fn test_client_get_livedata() {
    // Create mock server
    let server = MockGW1000Server::new().unwrap();
    let port = server.port();

    // Add canned response
    server.add_response(mock_livedata_response());

    // Start server in background
    let _handle = server.start();

    // Give server time to start
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Create client and test
    let client = GW1000Client::new("127.0.0.1".to_string(), port);
    let result = client.get_livedata();

    assert!(result.is_ok());
    let data = result.unwrap();

    // Check that we got some data
    assert!(!data.is_empty());

    // Check specific fields we know are in the mock response
    assert!(data.contains_key("outtemp"));
    assert!(data.contains_key("outhumid"));

    // Verify values
    assert_eq!(data.get("outtemp"), Some(&25.5));
    assert_eq!(data.get("outhumid"), Some(&65.0));
}

#[test]
fn test_client_connection_refused() {
    // Try to connect to a port that's not listening
    let client = GW1000Client::new("127.0.0.1".to_string(), 1); // Port 1 requires root
    let result = client.get_firmware_version();

    assert!(result.is_err());
}

#[test]
fn test_client_multiple_requests() {
    // Create mock server
    let server = MockGW1000Server::new().unwrap();
    let port = server.port();

    // Add multiple responses (in reverse order since we pop from the vec)
    server.add_response(mock_livedata_response());
    server.add_response(mock_mac_response(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]));
    server.add_response(mock_firmware_response("TEST_V1.0.0"));

    // Start server in background
    let _handle = server.start();

    // Give server time to start
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Create client
    let client = GW1000Client::new("127.0.0.1".to_string(), port);

    // Make multiple requests
    let fw = client.get_firmware_version();
    assert!(fw.is_ok());
    assert_eq!(fw.unwrap(), "TEST_V1.0.0");

    let mac = client.get_mac_address();
    assert!(mac.is_ok());
    assert_eq!(mac.unwrap(), "AA:BB:CC:DD:EE:FF");

    let data = client.get_livedata();
    assert!(data.is_ok());
}
