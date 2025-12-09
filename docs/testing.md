# Testing Guide for wxlistener

## Test Structure

```
wxlistener/
├── src/
│   ├── decoder.rs      - Unit tests for binary decoding
│   └── protocol.rs     - Unit tests for packet building/validation
└── tests/
    └── integration_test.rs - Integration tests
```

## Running Tests

### Run All Tests

```bash
cargo test
```

### Run Unit Tests Only

```bash
cargo test --lib
```

### Run Integration Tests Only

```bash
cargo test --test integration_test
```

### Run Specific Test

```bash
cargo test test_decode_temp_positive
```

### Run Tests with Output

```bash
cargo test -- --nocapture
```

### Run Tests in Release Mode

```bash
cargo test --release
```

## Test Coverage

### Unit Tests

#### `decoder.rs` Tests

- ✅ `test_decode_temp_positive` - Positive temperature values
- ✅ `test_decode_temp_negative` - Negative temperature values (two's complement)
- ✅ `test_decode_temp_zero` - Zero temperature
- ✅ `test_decode_short` - 16-bit unsigned integers
- ✅ `test_decode_int` - 32-bit unsigned integers
- ✅ `test_decode_wind` - Wind speed decoding
- ✅ `test_decode_rain` - Rainfall decoding
- ✅ `test_decode_pressure` - Barometric pressure decoding

#### `protocol.rs` Tests

- ✅ `test_calc_checksum` - Checksum calculation
- ✅ `test_build_cmd_packet_no_payload` - Packet building without payload
- ✅ `test_build_cmd_packet_with_payload` - Packet building with payload
- ✅ `test_verify_response_valid` - Valid response verification
- ✅ `test_verify_response_invalid_header` - Invalid header detection
- ✅ `test_verify_response_wrong_command` - Wrong command code detection
- ✅ `test_verify_response_bad_checksum` - Bad checksum detection
- ✅ `test_verify_response_too_short` - Short response handling

### Integration Tests

#### `integration_test.rs`

- ✅ `test_mock_response_structure` - Mock response validation
- ✅ `test_config_file_parsing` - TOML config parsing
- ✅ `test_output_formatting` - Data structure formatting

## Manual Testing

### Test with Real Device

```bash
# Single read
./target/release/wxlistener --ip YOUR_IP --port 45000

# Continuous monitoring
./target/release/wxlistener --ip YOUR_IP --continuous 10

# JSON output
./target/release/wxlistener --ip YOUR_IP --format json

# Using config file
./target/release/wxlistener --config wxlistener.toml
```

### Test Config File

Create `test_config.toml`:

```toml
ip = "10.31.100.42"
port = 45000
```

Run:

```bash
./target/release/wxlistener --config test_config.toml
```

### Test Error Handling

```bash
# Invalid IP
./target/release/wxlistener --ip 192.168.1.999

# Unreachable device
./target/release/wxlistener --ip 192.168.1.1 --port 12345

# Missing config file
./target/release/wxlistener --config nonexistent.toml

# No arguments
./target/release/wxlistener
```

## Adding New Tests

### Adding a Unit Test

In the relevant module (e.g., `decoder.rs`):

```rust
#[test]
fn test_my_new_feature() {
    let input = [0x01, 0x02];
    let result = my_function(&input);
    assert_eq!(result, expected_value);
}
```

### Adding an Integration Test

In `tests/integration_test.rs`:

```rust
#[test]
fn test_my_integration() {
    // Setup
    let data = create_test_data();

    // Execute
    let result = process_data(data);

    // Verify
    assert!(result.is_ok());
}
```

## Test Data

### Sample Binary Responses

#### Firmware Version Response

```
FF FF 50 13 47 57 32 30 30 30 42 5F 56 33 2E 31 2E 34 00 C3
│  │  │  │  └─────────────────────────────────────────┘  └─ Checksum
│  │  │  └─ Size (19 bytes)
│  │  └─ Command (0x50)
│  └─ Header
└─ Header
```

#### MAC Address Response

```
FF FF 26 09 EC 62 60 E0 6E 6F 1E
│  │  │  │  └──────────────┘  └─ Checksum
│  │  │  └─ Size (9 bytes)
│  │  └─ Command (0x26)
│  └─ Header
└─ Header
```

### Expected Values

| Field                 | Hex Input | Expected Output |
| --------------------- | --------- | --------------- |
| Temperature (25.5°C)  | `00 FF`   | 25.5            |
| Temperature (-10.5°C) | `FF 97`   | -10.5           |
| Humidity (65%)        | `41`      | 65              |
| Pressure (1013.2 hPa) | `27 94`   | 1013.2          |
| Wind (12.5 m/s)       | `00 7D`   | 12.5            |
| Rain (45.3 mm)        | `01 C5`   | 45.3            |

## Continuous Integration

### GitHub Actions Example

```yaml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo test --verbose
      - run: cargo test --release
```

## Benchmarking

### Run Benchmarks

```bash
cargo bench
```

### Profile Performance

```bash
cargo build --release
time ./target/release/wxlistener --ip YOUR_IP
```

## Coverage

### Using tarpaulin (Linux/macOS)

```bash
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
```

### Using llvm-cov

```bash
cargo install cargo-llvm-cov
cargo llvm-cov --html
```

## Debugging Tests

### Run with backtrace

```bash
RUST_BACKTRACE=1 cargo test
```

### Run single test with output

```bash
cargo test test_decode_temp_positive -- --nocapture --exact
```

### Run tests in single thread

```bash
cargo test -- --test-threads=1
```

## Known Limitations

1. **Network Tests**: Integration tests don't actually connect to a device (requires mock server)
2. **Live Data Parsing**: Full parsing tests require complete mock responses
3. **Error Cases**: Some error conditions are hard to test without a real device

## Advanced Testing

### Property-Based Testing

We use `proptest` for property-based testing to verify invariants across many random inputs:

```bash
# Property tests run automatically with cargo test
cargo test prop_
```

**Property tests include:**

- Temperature encoding/decoding roundtrip
- Value range validation for all decoders
- Packet structure invariants
- Checksum determinism

### Mock TCP Server

Integration tests use a mock TCP server to test client behavior without a real device:

```bash
# Run integration tests with mock server
cargo test --test client_integration_test
```

**Mock server features:**

- Simulates GW1000/GW2000 device responses
- Tests firmware version retrieval
- Tests MAC address retrieval
- Tests live data parsing
- Tests error handling (connection refused, etc.)

## Fuzzing

We use `cargo-fuzz` (libFuzzer) to fuzz test binary parsing code. See [FUZZING.md](fuzzing.md) for details.

```bash
# Quick fuzz test (10 seconds)
cargo fuzz run fuzz_decoder -- -max_total_time=10
cargo fuzz run fuzz_protocol -- -max_total_time=10
```

**Fuzz targets:**

- `fuzz_decoder` - Tests all decoder functions with arbitrary input
- `fuzz_protocol` - Tests packet building and validation

## Performance Benchmarks

We use Criterion.rs for performance benchmarking. See [BENCHMARKING.md](benchmarking.md) for details.

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark suite
cargo bench --bench decoder_bench
cargo bench --bench protocol_bench
```

**Benchmark results (typical):**

- `decode_temp`: ~1.0 ns
- `decode_short`: ~0.76 ns
- `decode_int`: ~0.76 ns
- `calc_checksum` (50B): ~2.7 ns
- `build_cmd_packet`: ~63-170 ns (depending on payload)
- `verify_response`: ~1.7 ns
- `full_packet_roundtrip`: ~85 ns

## Future Testing Improvements

- [ ] Add test coverage reporting
- [ ] Add mutation testing
- [ ] Add end-to-end tests with Docker
