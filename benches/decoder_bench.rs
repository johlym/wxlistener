use criterion::{black_box, criterion_group, criterion_main, Criterion};
use wxlistener::decoder::*;

fn benchmark_decode_temp(c: &mut Criterion) {
    let data = [0x00, 0xFF]; // 25.5°C
    c.bench_function("decode_temp", |b| b.iter(|| decode_temp(black_box(&data))));
}

fn benchmark_decode_temp_negative(c: &mut Criterion) {
    let data = [0xFF, 0x97]; // -10.5°C
    c.bench_function("decode_temp_negative", |b| {
        b.iter(|| decode_temp(black_box(&data)))
    });
}

fn benchmark_decode_short(c: &mut Criterion) {
    let data = [0x01, 0x68]; // 360
    c.bench_function("decode_short", |b| {
        b.iter(|| decode_short(black_box(&data)))
    });
}

fn benchmark_decode_int(c: &mut Criterion) {
    let data = [0x00, 0x0F, 0x42, 0x40]; // 1000000
    c.bench_function("decode_int", |b| b.iter(|| decode_int(black_box(&data))));
}

fn benchmark_decode_wind(c: &mut Criterion) {
    let data = [0x00, 0x7D]; // 12.5 m/s
    c.bench_function("decode_wind", |b| b.iter(|| decode_wind(black_box(&data))));
}

fn benchmark_decode_rain(c: &mut Criterion) {
    let data = [0x01, 0xC5]; // 45.3 mm
    c.bench_function("decode_rain", |b| b.iter(|| decode_rain(black_box(&data))));
}

fn benchmark_decode_pressure(c: &mut Criterion) {
    let data = [0x27, 0x94]; // 1013.2 hPa
    c.bench_function("decode_pressure", |b| {
        b.iter(|| decode_pressure(black_box(&data)))
    });
}

fn benchmark_all_decoders(c: &mut Criterion) {
    let data_2 = [0x00, 0xFF];
    let data_4 = [0x00, 0x0F, 0x42, 0x40];

    c.bench_function("all_decoders", |b| {
        b.iter(|| {
            black_box(decode_temp(&data_2));
            black_box(decode_short(&data_2));
            black_box(decode_wind(&data_2));
            black_box(decode_rain(&data_2));
            black_box(decode_pressure(&data_2));
            black_box(decode_int(&data_4));
        })
    });
}

criterion_group!(
    benches,
    benchmark_decode_temp,
    benchmark_decode_temp_negative,
    benchmark_decode_short,
    benchmark_decode_int,
    benchmark_decode_wind,
    benchmark_decode_rain,
    benchmark_decode_pressure,
    benchmark_all_decoders
);
criterion_main!(benches);
