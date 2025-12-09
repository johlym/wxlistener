#![no_main]

use libfuzzer_sys::fuzz_target;
use wxlistener::protocol::*;

fuzz_target!(|data: &[u8]| {
    // Test protocol functions with arbitrary input
    // They should never panic, regardless of input
    
    if data.len() >= 1 {
        let cmd = data[0];
        let payload = if data.len() > 1 { &data[1..] } else { &[] };
        
        // Test packet building
        let packet = build_cmd_packet(cmd, payload);
        
        // Test checksum calculation
        let _ = calc_checksum(&data);
        
        // Test response verification with the built packet
        let _ = verify_response(&packet, cmd);
        
        // Test verification with arbitrary data
        if data.len() >= 5 {
            let _ = verify_response(&data, cmd);
        }
    }
});
