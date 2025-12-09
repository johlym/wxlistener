# Benchmarking Guide

## Overview

We use [Criterion.rs](https://github.com/bheisler/criterion.rs) for performance benchmarking. Benchmarks help track performance over time and identify optimization opportunities.

## Running Benchmarks

### Run All Benchmarks

```bash
cargo bench
```

### Run Specific Benchmark Suite

```bash
# Decoder benchmarks only
cargo bench --bench decoder_bench

# Protocol benchmarks only
cargo bench --bench protocol_bench
```

### Run Specific Benchmark

```bash
# Run just one benchmark
cargo bench decode_temp

# Run benchmarks matching a pattern
cargo bench decode_
```

## Benchmark Suites

### 1. Decoder Benchmarks (`decoder_bench`)

Tests performance of binary data decoding functions:

- **decode_temp** - Temperature decoding (positive values)
- **decode_temp_negative** - Temperature decoding (negative values)
- **decode_short** - 16-bit unsigned integer
- **decode_int** - 32-bit unsigned integer
- **decode_wind** - Wind speed decoding
- **decode_rain** - Rainfall decoding
- **decode_pressure** - Barometric pressure
- **all_decoders** - All decoders in sequence

### 2. Protocol Benchmarks (`protocol_bench`)

Tests performance of protocol operations:

- **calc_checksum_small** - Checksum of 3 bytes
- **calc_checksum_medium** - Checksum of 50 bytes
- **calc_checksum_large** - Checksum of 200 bytes
- **build_cmd_packet_no_payload** - Empty packet
- **build_cmd_packet_small_payload** - 4-byte payload
- **build_cmd_packet_large_payload** - 100-byte payload
- **verify_response_valid** - Valid response verification
- **verify_response_invalid** - Invalid response verification
- **full_packet_roundtrip** - Build + verify cycle

## Understanding Output

### Sample Output

```
decode_temp             time:   [2.1234 ns 2.1456 ns 2.1678 ns]
                        change: [-1.2345% +0.5678% +2.3456%] (p = 0.23 > 0.05)
                        No change in performance detected.
```

**Explanation:**

- **time**: Mean execution time with confidence interval
- **change**: Performance change from previous run
- **p-value**: Statistical significance (< 0.05 = significant)

### Performance Status

- **Improved** 🎉: Faster than before (negative change %)
- **Regressed** ⚠️: Slower than before (positive change %)
- **No change**: Within noise threshold

## Viewing Results

### HTML Reports

Criterion generates detailed HTML reports:

```bash
# Open the report in your browser
open target/criterion/report/index.html
```

Reports include:

- Performance graphs
- Statistical analysis
- Comparison with previous runs
- Outlier detection

### Command Line Summary

```bash
# Show summary without running benchmarks
cargo bench --no-run
```

## Comparing Performance

### Baseline Comparison

```bash
# Save current performance as baseline
cargo bench -- --save-baseline main

# Make changes to code...

# Compare against baseline
cargo bench -- --baseline main
```

### Between Branches

```bash
# On main branch
git checkout main
cargo bench -- --save-baseline main

# On feature branch
git checkout feature-branch
cargo bench -- --baseline main
```

## Performance Targets

### Current Performance (Approximate)

| Operation           | Target   | Typical |
| ------------------- | -------- | ------- |
| decode_temp         | < 5 ns   | ~2 ns   |
| decode_short        | < 5 ns   | ~2 ns   |
| decode_int          | < 10 ns  | ~3 ns   |
| calc_checksum (50B) | < 50 ns  | ~20 ns  |
| build_cmd_packet    | < 100 ns | ~50 ns  |
| verify_response     | < 100 ns | ~40 ns  |

### Regression Threshold

We consider a performance regression significant if:

- Change > 10% AND p-value < 0.05
- Absolute time increase > 5ns for hot paths

## Optimization Tips

### 1. Profile Before Optimizing

```bash
# Use flamegraph for profiling
cargo install flamegraph
cargo flamegraph --bench decoder_bench
```

### 2. Check Assembly

```bash
# View generated assembly
cargo asm wxlistener::decoder::decode_temp
```

### 3. Use Release Mode

Always benchmark in release mode (cargo bench does this automatically).

### 4. Minimize Noise

- Close unnecessary applications
- Run on consistent hardware
- Disable CPU frequency scaling if possible

```bash
# Linux: Disable CPU frequency scaling
sudo cpupower frequency-set --governor performance
```

## Custom Benchmarks

### Adding a New Benchmark

1. **Edit benchmark file** (`benches/decoder_bench.rs`):

```rust
fn benchmark_my_function(c: &mut Criterion) {
    let data = [0x01, 0x02];
    c.bench_function("my_function", |b| {
        b.iter(|| my_function(black_box(&data)))
    });
}

criterion_group!(
    benches,
    // ... existing benchmarks
    benchmark_my_function
);
```

2. **Run the new benchmark**:

```bash
cargo bench my_function
```

### Parameterized Benchmarks

```rust
fn benchmark_with_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("checksum_sizes");

    for size in [10, 50, 100, 200].iter() {
        let data = vec![0x42; *size];
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &data,
            |b, data| b.iter(|| calc_checksum(black_box(data)))
        );
    }

    group.finish();
}
```

## CI Integration

### GitHub Actions

```yaml
- name: Run benchmarks
  run: cargo bench --no-fail-fast

- name: Store benchmark results
  uses: benchmark-action/github-action-benchmark@v1
  with:
    tool: "cargo"
    output-file-path: target/criterion/*/new/estimates.json
```

### Performance Tracking

Track performance over time:

```bash
# Export results
cargo bench -- --output-format bencher | tee output.txt

# Compare with historical data
# (Store output.txt in git or external service)
```

## Troubleshooting

### "Benchmark took too long"

Increase the measurement time:

```rust
c.bench_function("slow_function", |b| {
    b.iter(|| slow_function())
}).measurement_time(Duration::from_secs(10));
```

### High Variance

```bash
# Increase sample size
cargo bench -- --sample-size 1000
```

### Outliers

Criterion automatically detects and reports outliers. High outlier counts may indicate:

- System noise (other processes)
- Non-deterministic code
- Memory allocation patterns

## Best Practices

1. **Benchmark Hot Paths**: Focus on frequently called functions
2. **Use `black_box`**: Prevent compiler from optimizing away code
3. **Consistent Environment**: Run on same hardware
4. **Track Over Time**: Save baselines for comparison
5. **Document Changes**: Note why performance changed
6. **Set Thresholds**: Define acceptable performance ranges

## Resources

- [Criterion.rs User Guide](https://bheisler.github.io/criterion.rs/book/)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Benchmarking in Rust](https://doc.rust-lang.org/cargo/commands/cargo-bench.html)
