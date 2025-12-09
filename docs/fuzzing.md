# Fuzzing Guide

## Overview

Fuzzing is a testing technique that provides random/malformed input to find bugs, crashes, and security vulnerabilities. We use `cargo-fuzz` (libFuzzer) to fuzz test our binary parsing code.

## Setup

### Install cargo-fuzz

```bash
cargo install cargo-fuzz
```

### Verify Installation

```bash
cargo fuzz --version
```

## Fuzz Targets

We have two fuzz targets:

### 1. `fuzz_decoder` - Binary Data Decoders

Tests all decoder functions with arbitrary binary input:

- `decode_temp()` - Temperature decoding
- `decode_short()` - 16-bit integer decoding
- `decode_int()` - 32-bit integer decoding
- `decode_wind()` - Wind speed decoding
- `decode_rain()` - Rainfall decoding
- `decode_pressure()` - Pressure decoding

**Goal:** Ensure decoders never panic regardless of input.

### 2. `fuzz_protocol` - Protocol Functions

Tests protocol packet building and validation:

- `build_cmd_packet()` - Packet construction
- `calc_checksum()` - Checksum calculation
- `verify_response()` - Response validation

**Goal:** Ensure protocol functions handle malformed packets gracefully.

## Running Fuzz Tests

### Quick Fuzz (10 seconds)

```bash
# Fuzz decoders
cargo fuzz run fuzz_decoder -- -max_total_time=10

# Fuzz protocol
cargo fuzz run fuzz_protocol -- -max_total_time=10
```

### Extended Fuzz (1 minute)

```bash
cargo fuzz run fuzz_decoder -- -max_total_time=60
cargo fuzz run fuzz_protocol -- -max_total_time=60
```

### Continuous Fuzzing

```bash
# Run until manually stopped (Ctrl+C)
cargo fuzz run fuzz_decoder

# Run with specific number of runs
cargo fuzz run fuzz_decoder -- -runs=1000000
```

### Parallel Fuzzing

```bash
# Use multiple CPU cores
cargo fuzz run fuzz_decoder -- -jobs=4
```

## Understanding Output

### Successful Run

```
#1      INITED cov: 45 ft: 46 corp: 1/1b exec/s: 0 rss: 32Mb
#2      NEW    cov: 47 ft: 48 corp: 2/3b lim: 4 exec/s: 0 rss: 32Mb
...
Done 10000 runs in 10 seconds
```

- `cov`: Code coverage (edges covered)
- `ft`: Features (unique code paths)
- `corp`: Corpus size (interesting inputs found)
- `exec/s`: Executions per second

### Crash Found

If a crash is found, you'll see:

```
==12345==ERROR: AddressSanitizer: heap-buffer-overflow
...
artifact_prefix='./fuzz/artifacts/fuzz_decoder/'; Test unit written to ./fuzz/artifacts/fuzz_decoder/crash-...
```

The crashing input is saved to `fuzz/artifacts/` for reproduction.

## Reproducing Crashes

```bash
# Reproduce a specific crash
cargo fuzz run fuzz_decoder fuzz/artifacts/fuzz_decoder/crash-abc123

# Debug with more information
cargo fuzz run fuzz_decoder fuzz/artifacts/fuzz_decoder/crash-abc123 -- -verbosity=2
```

## Corpus Management

The corpus is a collection of interesting inputs discovered during fuzzing.

### View Corpus

```bash
ls fuzz/corpus/fuzz_decoder/
```

### Add Seed Inputs

```bash
# Add known interesting inputs
echo -n "\xFF\xFF\x50\x03\x00\x53" > fuzz/corpus/fuzz_decoder/firmware_response
```

### Minimize Corpus

```bash
# Remove redundant corpus entries
cargo fuzz cmin fuzz_decoder
```

## Coverage Analysis

### Generate Coverage Report

```bash
# Run with coverage
cargo fuzz coverage fuzz_decoder

# View coverage
cargo cov -- show target/*/release/fuzz_decoder \
    --format=html \
    --instr-profile=fuzz/coverage/fuzz_decoder/coverage.profdata \
    > coverage.html
```

## Best Practices

1. **Start Small**: Run short fuzzing sessions first (10-60 seconds)
2. **Run Regularly**: Integrate fuzzing into CI/CD
3. **Monitor Coverage**: Track code coverage over time
4. **Save Corpus**: Commit interesting corpus entries to git
5. **Fix Crashes**: Prioritize fixing any crashes found
6. **Fuzz After Changes**: Run fuzzing after modifying parsing code

## Integration with CI

### GitHub Actions Example

```yaml
- name: Fuzz test
  run: |
    cargo install cargo-fuzz
    cargo fuzz run fuzz_decoder -- -max_total_time=60
    cargo fuzz run fuzz_protocol -- -max_total_time=60
```

## Troubleshooting

### "No fuzz targets found"

Make sure you're in the project root directory.

### "Sanitizer not supported"

Fuzzing requires nightly Rust:

```bash
rustup install nightly
cargo +nightly fuzz run fuzz_decoder
```

### Out of Memory

Reduce memory usage:

```bash
cargo fuzz run fuzz_decoder -- -rss_limit_mb=2048
```

## Advanced Options

### Dictionary-Based Fuzzing

Create a dictionary file for protocol-specific values:

```bash
cat > fuzz/fuzz.dict << EOF
header="\xFF\xFF"
cmd_firmware="\x50"
cmd_mac="\x26"
cmd_livedata="\x27"
EOF

cargo fuzz run fuzz_protocol -- -dict=fuzz/fuzz.dict
```

### Structured Fuzzing

For more complex fuzzing, use `arbitrary` crate:

```rust
use arbitrary::Arbitrary;

#[derive(Arbitrary, Debug)]
struct FuzzInput {
    cmd: u8,
    payload: Vec<u8>,
}
```

## Resources

- [cargo-fuzz book](https://rust-fuzz.github.io/book/cargo-fuzz.html)
- [libFuzzer documentation](https://llvm.org/docs/LibFuzzer.html)
- [Rust Fuzz project](https://github.com/rust-fuzz)
