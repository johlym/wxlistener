#![no_main]

use libfuzzer_sys::fuzz_target;
use wxlistener::decoder::*;

fuzz_target!(|data: &[u8]| {
    // Test all decoder functions with arbitrary input
    // They should never panic, regardless of input
    
    if data.len() >= 2 {
        let _ = decode_temp(&data[0..2]);
        let _ = decode_short(&data[0..2]);
        let _ = decode_wind(&data[0..2]);
        let _ = decode_rain(&data[0..2]);
        let _ = decode_pressure(&data[0..2]);
    }
    
    if data.len() >= 4 {
        let _ = decode_int(&data[0..4]);
    }
});
