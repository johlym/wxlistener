/// Protocol constants and packet building utilities
/// Separated for easier testing
pub const HEADER: [u8; 2] = [0xFF, 0xFF];

pub fn build_cmd_packet(cmd_code: u8, payload: &[u8]) -> Vec<u8> {
    let size = 1 + 1 + payload.len() + 1;
    let mut body = vec![cmd_code, size as u8];
    body.extend_from_slice(payload);

    let checksum = calc_checksum(&body);

    let mut packet = HEADER.to_vec();
    packet.extend(body);
    packet.push(checksum);
    packet
}

pub fn calc_checksum(data: &[u8]) -> u8 {
    (data.iter().map(|&b| b as u32).sum::<u32>() % 256) as u8
}

pub fn verify_response(response: &[u8], expected_cmd: u8) -> bool {
    if response.len() < 5 {
        return false;
    }

    // Check header
    if response[0] != HEADER[0] || response[1] != HEADER[1] {
        return false;
    }

    // Check command code
    if response[2] != expected_cmd {
        return false;
    }

    // Verify checksum
    let calc_checksum = calc_checksum(&response[2..response.len() - 1]);
    let resp_checksum = response[response.len() - 1];

    calc_checksum == resp_checksum
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calc_checksum() {
        let data = vec![0x50, 0x03, 0x00];
        let checksum = calc_checksum(&data);
        assert_eq!(checksum, 0x53);
    }

    #[test]
    fn test_build_cmd_packet_no_payload() {
        // CMD_READ_FIRMWARE_VERSION = 0x50
        let packet = build_cmd_packet(0x50, &[]);

        assert_eq!(packet[0], 0xFF); // Header
        assert_eq!(packet[1], 0xFF); // Header
        assert_eq!(packet[2], 0x50); // Command
        assert_eq!(packet[3], 0x03); // Size (cmd + size + checksum)
        assert_eq!(packet[4], 0x53); // Checksum
    }

    #[test]
    fn test_build_cmd_packet_with_payload() {
        let payload = vec![0x01, 0x02];
        let packet = build_cmd_packet(0x27, &payload);

        assert_eq!(packet[0], 0xFF);
        assert_eq!(packet[1], 0xFF);
        assert_eq!(packet[2], 0x27); // Command
        assert_eq!(packet[3], 0x05); // Size (cmd + size + 2 payload + checksum)
        assert_eq!(packet[4], 0x01); // Payload
        assert_eq!(packet[5], 0x02); // Payload
                                     // Last byte is checksum
    }

    #[test]
    fn test_verify_response_valid() {
        // Valid response: FF FF 50 03 00 53
        let response = vec![0xFF, 0xFF, 0x50, 0x03, 0x00, 0x53];
        assert!(verify_response(&response, 0x50));
    }

    #[test]
    fn test_verify_response_invalid_header() {
        let response = vec![0xAA, 0xFF, 0x50, 0x03, 0x00, 0x53];
        assert!(!verify_response(&response, 0x50));
    }

    #[test]
    fn test_verify_response_wrong_command() {
        let response = vec![0xFF, 0xFF, 0x50, 0x03, 0x00, 0x53];
        assert!(!verify_response(&response, 0x27));
    }

    #[test]
    fn test_verify_response_bad_checksum() {
        let response = vec![0xFF, 0xFF, 0x50, 0x03, 0x00, 0xFF];
        assert!(!verify_response(&response, 0x50));
    }

    #[test]
    fn test_verify_response_too_short() {
        let response = vec![0xFF, 0xFF, 0x50];
        assert!(!verify_response(&response, 0x50));
    }

    // Property-based tests
    mod proptests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn prop_build_packet_has_header(cmd: u8, payload in prop::collection::vec(any::<u8>(), 0..20)) {
                // Every packet should start with the header
                let packet = build_cmd_packet(cmd, &payload);
                prop_assert_eq!(packet[0], HEADER[0]);
                prop_assert_eq!(packet[1], HEADER[1]);
            }

            #[test]
            fn prop_build_packet_has_command(cmd: u8, payload in prop::collection::vec(any::<u8>(), 0..20)) {
                // Command should be at position 2
                let packet = build_cmd_packet(cmd, &payload);
                prop_assert_eq!(packet[2], cmd);
            }

            #[test]
            fn prop_build_packet_correct_size(cmd: u8, payload in prop::collection::vec(any::<u8>(), 0..20)) {
                // Packet size should be: header(2) + cmd(1) + size(1) + payload + checksum(1)
                let packet = build_cmd_packet(cmd, &payload);
                let expected_size = 2 + 1 + 1 + payload.len() + 1;
                prop_assert_eq!(packet.len(), expected_size);
            }

            #[test]
            fn prop_checksum_deterministic(data in prop::collection::vec(any::<u8>(), 1..100)) {
                // Same data should always produce same checksum
                let checksum1 = calc_checksum(&data);
                let checksum2 = calc_checksum(&data);
                prop_assert_eq!(checksum1, checksum2);
            }

            #[test]
            fn prop_checksum_in_range(data in prop::collection::vec(any::<u8>(), 1..100)) {
                // Checksum should always be a valid u8
                let _checksum = calc_checksum(&data);
                // This test verifies the function doesn't panic
                prop_assert!(true);
            }

            #[test]
            fn prop_verify_response_rejects_wrong_command(
                cmd: u8,
                wrong_cmd: u8,
                payload in prop::collection::vec(any::<u8>(), 0..20)
            ) {
                // Build a valid packet but check with wrong command
                let packet = build_cmd_packet(cmd, &payload);
                if cmd != wrong_cmd {
                    prop_assert!(!verify_response(&packet, wrong_cmd));
                }
            }

            #[test]
            fn prop_build_and_verify_roundtrip(cmd: u8, payload in prop::collection::vec(any::<u8>(), 0..20)) {
                // A packet we build should verify correctly
                let packet = build_cmd_packet(cmd, &payload);
                prop_assert!(verify_response(&packet, cmd));
            }
        }
    }
}
