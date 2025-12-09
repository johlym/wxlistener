use criterion::{black_box, criterion_group, criterion_main, Criterion};
use wxlistener::protocol::*;

fn benchmark_calc_checksum_small(c: &mut Criterion) {
    let data = vec![0x50, 0x03, 0x00];
    c.bench_function("calc_checksum_small", |b| {
        b.iter(|| calc_checksum(black_box(&data)))
    });
}

fn benchmark_calc_checksum_medium(c: &mut Criterion) {
    let data = vec![0x27; 50]; // 50 bytes
    c.bench_function("calc_checksum_medium", |b| {
        b.iter(|| calc_checksum(black_box(&data)))
    });
}

fn benchmark_calc_checksum_large(c: &mut Criterion) {
    let data = vec![0x27; 200]; // 200 bytes
    c.bench_function("calc_checksum_large", |b| {
        b.iter(|| calc_checksum(black_box(&data)))
    });
}

fn benchmark_build_cmd_packet_no_payload(c: &mut Criterion) {
    c.bench_function("build_cmd_packet_no_payload", |b| {
        b.iter(|| build_cmd_packet(black_box(0x50), black_box(&[])))
    });
}

fn benchmark_build_cmd_packet_small_payload(c: &mut Criterion) {
    let payload = vec![0x01, 0x02, 0x03, 0x04];
    c.bench_function("build_cmd_packet_small_payload", |b| {
        b.iter(|| build_cmd_packet(black_box(0x27), black_box(&payload)))
    });
}

fn benchmark_build_cmd_packet_large_payload(c: &mut Criterion) {
    let payload = vec![0x42; 100];
    c.bench_function("build_cmd_packet_large_payload", |b| {
        b.iter(|| build_cmd_packet(black_box(0x27), black_box(&payload)))
    });
}

fn benchmark_verify_response_valid(c: &mut Criterion) {
    let response = vec![0xFF, 0xFF, 0x50, 0x03, 0x00, 0x53];
    c.bench_function("verify_response_valid", |b| {
        b.iter(|| verify_response(black_box(&response), black_box(0x50)))
    });
}

fn benchmark_verify_response_invalid(c: &mut Criterion) {
    let response = vec![0xFF, 0xFF, 0x50, 0x03, 0x00, 0xFF];
    c.bench_function("verify_response_invalid", |b| {
        b.iter(|| verify_response(black_box(&response), black_box(0x50)))
    });
}

fn benchmark_full_packet_roundtrip(c: &mut Criterion) {
    let payload = vec![0x01, 0x02, 0x03];
    c.bench_function("full_packet_roundtrip", |b| {
        b.iter(|| {
            let packet = build_cmd_packet(black_box(0x27), black_box(&payload));
            verify_response(black_box(&packet), black_box(0x27))
        })
    });
}

criterion_group!(
    benches,
    benchmark_calc_checksum_small,
    benchmark_calc_checksum_medium,
    benchmark_calc_checksum_large,
    benchmark_build_cmd_packet_no_payload,
    benchmark_build_cmd_packet_small_payload,
    benchmark_build_cmd_packet_large_payload,
    benchmark_verify_response_valid,
    benchmark_verify_response_invalid,
    benchmark_full_packet_roundtrip
);
criterion_main!(benches);
