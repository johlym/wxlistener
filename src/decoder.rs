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
    (((data[0] as u32) << 24) | ((data[1] as u32) << 16) | 
     ((data[2] as u32) << 8) | (data[3] as u32)) as f64
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
