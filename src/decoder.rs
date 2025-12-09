/// Decoding functions for GW1000 binary data
pub fn decode_temp(data: &[u8]) -> f64 {
    let value = ((data[0] as u16) << 8) | (data[1] as u16);
    let value = if value > 32767 {
        value as i32 - 65536
    } else {
        value as i32
    };
    value as f64 / 10.0
}

pub fn decode_short(data: &[u8]) -> f64 {
    (((data[0] as u16) << 8) | (data[1] as u16)) as f64
}

pub fn decode_int(data: &[u8]) -> f64 {
    (((data[0] as u32) << 24)
        | ((data[1] as u32) << 16)
        | ((data[2] as u32) << 8)
        | (data[3] as u32)) as f64
}

pub fn decode_wind(data: &[u8]) -> f64 {
    let value = ((data[0] as u16) << 8) | (data[1] as u16);
    value as f64 / 10.0
}

pub fn decode_rain(data: &[u8]) -> f64 {
    let value = ((data[0] as u16) << 8) | (data[1] as u16);
    value as f64 / 10.0
}

pub fn decode_pressure(data: &[u8]) -> f64 {
    let value = ((data[0] as u16) << 8) | (data[1] as u16);
    value as f64 / 10.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_temp_positive() {
        // 25.5°C = 255 = 0x00FF
        let data = [0x00, 0xFF];
        assert_eq!(decode_temp(&data), 25.5);
    }

    #[test]
    fn test_decode_temp_negative() {
        // -10.5°C = -105 = 0xFF97 (two's complement)
        let data = [0xFF, 0x97];
        assert_eq!(decode_temp(&data), -10.5);
    }

    #[test]
    fn test_decode_temp_zero() {
        let data = [0x00, 0x00];
        assert_eq!(decode_temp(&data), 0.0);
    }

    #[test]
    fn test_decode_short() {
        // 360 = 0x0168
        let data = [0x01, 0x68];
        assert_eq!(decode_short(&data), 360.0);
    }

    #[test]
    fn test_decode_int() {
        // 1000000 = 0x000F4240
        let data = [0x00, 0x0F, 0x42, 0x40];
        assert_eq!(decode_int(&data), 1000000.0);
    }

    #[test]
    fn test_decode_wind() {
        // 12.5 m/s = 125 = 0x007D
        let data = [0x00, 0x7D];
        assert_eq!(decode_wind(&data), 12.5);
    }

    #[test]
    fn test_decode_rain() {
        // 45.3 mm = 453 = 0x01C5
        let data = [0x01, 0xC5];
        assert_eq!(decode_rain(&data), 45.3);
    }

    #[test]
    fn test_decode_pressure() {
        // 1013.2 hPa = 10132 = 0x2794
        let data = [0x27, 0x94];
        assert_eq!(decode_pressure(&data), 1013.2);
    }

    // Property-based tests
    mod proptests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn prop_decode_temp_roundtrip(temp in -500..500i16) {
                // Test that encoding and decoding temperature is consistent
                let encoded = ((temp as i32 * 10) as u16).to_be_bytes();
                let decoded = decode_temp(&encoded);
                let expected = temp as f64 / 10.0 * 10.0; // Original temp * 10 / 10
                prop_assert!((decoded - expected).abs() < 0.1);
            }

            #[test]
            fn prop_decode_short_always_positive(high: u8, low: u8) {
                // decode_short should always return a positive number
                let data = [high, low];
                let result = decode_short(&data);
                prop_assert!(result >= 0.0);
                prop_assert!(result <= 65535.0);
            }

            #[test]
            fn prop_decode_wind_in_range(high: u8, low: u8) {
                // Wind speed should be in reasonable range after decoding
                let data = [high, low];
                let result = decode_wind(&data);
                prop_assert!(result >= 0.0);
                prop_assert!(result <= 6553.5); // Max u16 / 10
            }

            #[test]
            fn prop_decode_rain_in_range(high: u8, low: u8) {
                // Rain should be in reasonable range after decoding
                let data = [high, low];
                let result = decode_rain(&data);
                prop_assert!(result >= 0.0);
                prop_assert!(result <= 6553.5); // Max u16 / 10
            }

            #[test]
            fn prop_decode_pressure_in_range(high: u8, low: u8) {
                // Pressure should be in reasonable range
                let data = [high, low];
                let result = decode_pressure(&data);
                prop_assert!(result >= 0.0);
                prop_assert!(result <= 6553.5); // Max u16 / 10
            }

            #[test]
            fn prop_decode_int_always_positive(b0: u8, b1: u8, b2: u8, b3: u8) {
                // decode_int should always return a positive number
                let data = [b0, b1, b2, b3];
                let result = decode_int(&data);
                prop_assert!(result >= 0.0);
                prop_assert!(result <= 4294967295.0); // Max u32
            }
        }
    }
}
