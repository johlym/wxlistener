# Test Coverage Guide

## Table of Contents

- [Overview](#overview)
- [Current Coverage](#current-coverage)
  - [By Module](#by-module)
- [Quick Start](#quick-start)
  - [Install Coverage Tool](#install-coverage-tool)
  - [Generate Coverage Report](#generate-coverage-report)
  - [View Report](#view-report)
- [Understanding Coverage Metrics](#understanding-coverage-metrics)
  - [Lines Coverage](#lines-coverage)
  - [Regions Coverage](#regions-coverage)
  - [Functions Coverage](#functions-coverage)
- [Improving Coverage](#improving-coverage)
  - [Priority Areas](#priority-areas)
  - [Example: Adding Config Tests](#example-adding-config-tests)
- [Coverage Targets](#coverage-targets)
  - [Current Status](#current-status)
  - [Goals](#goals)
- [Advanced Usage](#advanced-usage)
  - [Generate Different Formats](#generate-different-formats)
  - [Exclude Files from Coverage](#exclude-files-from-coverage)
  - [Coverage for Specific Tests](#coverage-for-specific-tests)
- [CI Integration](#ci-integration)
  - [GitHub Actions](#github-actions)
  - [Coverage Badges](#coverage-badges)
- [Interpreting Results](#interpreting-results)
  - [Good Coverage (>80%)](#good-coverage-80)
  - [Moderate Coverage (50-80%)](#moderate-coverage-50-80)
  - [Low Coverage (<50%)](#low-coverage-50)
- [Coverage vs. Test Quality](#coverage-vs-test-quality)
  - [Bad Test (100% coverage, poor quality)](#bad-test-100-coverage-poor-quality)
  - [Good Test (100% coverage, high quality)](#good-test-100-coverage-high-quality)
- [Best Practices](#best-practices)
- [Troubleshooting](#troubleshooting)
  - ["No coverage data generated"](#no-coverage-data-generated)
  - ["Coverage decreased"](#coverage-decreased)
  - [High variance in coverage](#high-variance-in-coverage)
- [Resources](#resources)

## Overview

Test coverage measures how much of your code is executed during tests. We use `cargo-llvm-cov` for comprehensive coverage reporting.

## Current Coverage

```
Overall Coverage: 72.93% lines, 70.80% regions
Functions: 98.36% (60/61)
```

### By Module

| Module          | Lines  | Regions | Functions | Status        |
| --------------- | ------ | ------- | --------- | ------------- |
| **decoder.rs**  | 100%   | 100%    | 100%      | ✅ Excellent  |
| **protocol.rs** | 100%   | 100%    | 100%      | ✅ Excellent  |
| **config.rs**   | 100%   | 100%    | 100%      | ✅ Excellent  |
| **output.rs**   | 100%   | 100%    | 100%      | ✅ Excellent  |
| **client.rs**   | 37.16% | 38.29%  | 100%      | ⚠️ Needs work |
| **main.rs**     | 0%     | 0%      | 0%        | ❌ Not tested |

## Quick Start

### Install Coverage Tool

```bash
cargo install cargo-llvm-cov
```

### Generate Coverage Report

```bash
# Using our script (recommended)
bin/coverage

# Or manually
cargo llvm-cov --all-features --workspace --html
```

### View Report

```bash
# Open HTML report
open target/llvm-cov/html/index.html
```

## Understanding Coverage Metrics

### Lines Coverage

**What it measures:** Percentage of code lines executed during tests.

```rust
fn example(x: i32) -> i32 {
    if x > 0 {        // Line 1: Covered
        return x * 2; // Line 2: Covered if x > 0
    }
    return 0;         // Line 3: Covered if x <= 0
}
```

### Regions Coverage

**What it measures:** Percentage of code regions (branches) executed.

```rust
fn example(x: i32) -> i32 {
    if x > 0 {        // Region 1: if branch
        return x * 2; // Region 2: true path
    }                 // Region 3: false path
    return 0;
}
```

### Functions Coverage

**What it measures:** Percentage of functions called during tests.

## Improving Coverage

### Priority Areas

1. **client.rs** (37% coverage)

   - Add tests for error paths
   - Test network failure scenarios
   - Test malformed responses

2. **config.rs** (0% coverage)

   - Test CLI argument parsing
   - Test config file loading
   - Test validation logic

3. **output.rs** (0% coverage)

   - Test text formatting
   - Test value formatting
   - Test edge cases

4. **main.rs** (0% coverage)
   - Integration tests for main flow
   - Test continuous mode
   - Test different output formats

### Example: Adding Config Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_connection_info_from_ip() {
        let args = Args {
            ip: Some("192.168.1.1".to_string()),
            port: Some(45000),
            config: None,
            format: "text".to_string(),
            continuous: None,
        };

        let (ip, port) = args.get_connection_info().unwrap();
        assert_eq!(ip, "192.168.1.1");
        assert_eq!(port, 45000);
    }
}
```

## Coverage Targets

### Current Status

- ✅ **Core Logic**: 100% (decoder, protocol)
- ⚠️ **Network Layer**: 37% (client)
- ❌ **CLI/Output**: 0% (config, output, main)

### Goals

| Component   | Current | Target | Priority |
| ----------- | ------- | ------ | -------- |
| decoder.rs  | 100%    | 100%   | ✅ Done  |
| protocol.rs | 100%    | 100%   | ✅ Done  |
| client.rs   | 37%     | 80%    | High     |
| config.rs   | 0%      | 90%    | High     |
| output.rs   | 0%      | 80%    | Medium   |
| main.rs     | 0%      | 60%    | Low      |

**Overall Target**: 75% line coverage

## Advanced Usage

### Generate Different Formats

```bash
# HTML report (default)
cargo llvm-cov --all-features --workspace --html

# Text summary
cargo llvm-cov --all-features --workspace --summary-only

# LCOV format (for CI tools)
cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info

# Cobertura XML (for GitLab, Jenkins)
cargo llvm-cov --all-features --workspace --cobertura --output-path cobertura.xml

# JSON format
cargo llvm-cov --all-features --workspace --json --output-path coverage.json
```

### Exclude Files from Coverage

```bash
# Exclude test files
cargo llvm-cov --all-features --workspace --html --ignore-filename-regex tests/

# Exclude specific modules
cargo llvm-cov --all-features --workspace --html --ignore-filename-regex 'main\.rs'
```

### Coverage for Specific Tests

```bash
# Only unit tests
cargo llvm-cov --lib --html

# Only integration tests
cargo llvm-cov --test client_integration_test --html

# Specific test
cargo llvm-cov --html -- test_decode_temp
```

## CI Integration

### GitHub Actions

```yaml
name: Coverage

on: [push, pull_request]

jobs:
  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          override: true

      - name: Install cargo-llvm-cov
        run: cargo install cargo-llvm-cov

      - name: Generate coverage
        run: cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info

      - name: Upload to Codecov
        uses: codecov/codecov-action@v3
        with:
          files: lcov.info
          fail_ci_if_error: true

      - name: Check coverage threshold
        run: |
          COVERAGE=$(cargo llvm-cov --all-features --workspace --summary-only | grep "TOTAL" | awk '{print $10}' | sed 's/%//')
          if (( $(echo "$COVERAGE < 70" | bc -l) )); then
            echo "Coverage $COVERAGE% is below 70% threshold"
            exit 1
          fi
```

### Coverage Badges

Add to your README.md:

```markdown
[![codecov](https://codecov.io/gh/username/wxlistener/branch/main/graph/badge.svg)](https://codecov.io/gh/username/wxlistener)
```

## Interpreting Results

### Good Coverage (>80%)

```
decoder.rs: 100.00% lines
```

- All code paths tested
- High confidence in correctness
- Safe to refactor

### Moderate Coverage (50-80%)

```
client.rs: 37.16% lines
```

- Core functionality tested
- Some edge cases missing
- Should add more tests

### Low Coverage (<50%)

```
config.rs: 0.00% lines
```

- Minimal or no testing
- High risk of bugs
- Priority for new tests

## Coverage vs. Test Quality

**Important**: High coverage ≠ good tests!

### Bad Test (100% coverage, poor quality)

```rust
#[test]
fn test_everything() {
    let _ = decode_temp(&[0, 0]);
    let _ = decode_short(&[0, 0]);
    // Covers code but doesn't verify correctness!
}
```

### Good Test (100% coverage, high quality)

```rust
#[test]
fn test_decode_temp_positive() {
    let data = [0x00, 0xFF]; // 25.5°C
    assert_eq!(decode_temp(&data), 25.5);
}

#[test]
fn test_decode_temp_negative() {
    let data = [0xFF, 0x97]; // -10.5°C
    assert_eq!(decode_temp(&data), -10.5);
}
```

## Best Practices

1. **Focus on Critical Code**: Prioritize coverage for core logic
2. **Test Edge Cases**: Don't just aim for line coverage
3. **Meaningful Assertions**: Verify behavior, not just execution
4. **Regular Monitoring**: Track coverage over time
5. **Set Thresholds**: Enforce minimum coverage in CI
6. **Review Reports**: Look for untested branches

## Troubleshooting

### "No coverage data generated"

Make sure you're running tests:

```bash
cargo llvm-cov test --all-features --workspace --html
```

### "Coverage decreased"

Check what changed:

```bash
# Compare with previous run
cargo llvm-cov --all-features --workspace --html --open
```

### High variance in coverage

Some reasons:

- Conditional compilation (`#[cfg(...)]`)
- Platform-specific code
- Error handling paths
- Unreachable code

## Resources

- [cargo-llvm-cov documentation](https://github.com/taiki-e/cargo-llvm-cov)
- [LLVM coverage mapping](https://llvm.org/docs/CoverageMappingFormat.html)
- [Codecov documentation](https://docs.codecov.com/)
