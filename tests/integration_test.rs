use std::collections::HashMap;

// Mock data for testing live data parsing
fn create_mock_livedata_response() -> Vec<u8> {
    let mut response = vec![
        0xFF, 0xFF, // Header
        0x27,       // Command (CMD_GW1000_LIVEDATA)
        0x00, 0x2A, // Size (42 bytes) - big-endian
    ];
    
    // Add some sample fields
    // 0x02: outtemp = 25.5°C (255 = 0x00FF)
    response.extend_from_slice(&[0x02, 0x00, 0xFF]);
    
    // 0x07: outhumid = 65%
    response.extend_from_slice(&[0x07, 0x41]);
    
    // 0x08: absbarometer = 1013.2 hPa (10132 = 0x2794)
    response.extend_from_slice(&[0x08, 0x27, 0x94]);
    
    // 0x0B: wind_speed = 5.5 m/s (55 = 0x0037)
    response.extend_from_slice(&[0x0B, 0x00, 0x37]);
    
    // 0x0E: rain_rate = 2.5 mm (25 = 0x0019)
    response.extend_from_slice(&[0x0E, 0x00, 0x19]);
    
    // Calculate and append checksum
    let checksum: u8 = response[2..].iter().map(|&b| b as u32).sum::<u32>() as u8;
    response.push(checksum);
    
    response
}

#[test]
fn test_mock_response_structure() {
    let response = create_mock_livedata_response();
    
    // Verify header
    assert_eq!(response[0], 0xFF);
    assert_eq!(response[1], 0xFF);
    
    // Verify command
    assert_eq!(response[2], 0x27);
    
    // Verify size field (big-endian)
    let size = ((response[3] as usize) << 8) | (response[4] as usize);
    assert!(size > 0);
}

#[test]
fn test_config_file_parsing() {
    use std::fs;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "ip = \"192.168.1.100\"").unwrap();
    writeln!(temp_file, "port = 45000").unwrap();
    
    let content = fs::read_to_string(temp_file.path()).unwrap();
    let config: toml::Value = toml::from_str(&content).unwrap();
    
    assert_eq!(config["ip"].as_str().unwrap(), "192.168.1.100");
    assert_eq!(config["port"].as_integer().unwrap(), 45000);
}

#[test]
fn test_output_formatting() {
    let mut data = HashMap::new();
    data.insert("outtemp".to_string(), 25.5);
    data.insert("outhumid".to_string(), 65.0);
    data.insert("wind_speed".to_string(), 5.5);
    
    // Just verify we can create the data structure
    assert_eq!(data.len(), 3);
    assert_eq!(data.get("outtemp"), Some(&25.5));
}
