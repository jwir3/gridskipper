# Code Coverage Guide

This document explains how to generate and view code coverage reports for the gridskipper project.

## Quick Start

### Unix/Linux/macOS
```bash
./scripts/coverage.sh
```

### Windows
```cmd
scripts\coverage.bat
```

## What the Scripts Do

The coverage scripts will:

1. **Clean up** any previous coverage data
2. **Build and run tests** with coverage instrumentation enabled
3. **Merge profile data** from all test runs
4. **Generate reports** in both HTML and console formats
5. **Clean up temporary files** while keeping the final reports

## Output Files

After running the coverage script, you'll find:

- `coverage/index.html` - Interactive HTML coverage report
- `gridskipper.profdata` - Merged coverage profile data (for reuse)

## Understanding the Coverage Report

### Console Output
The console shows a summary table with:
- **Regions**: Code regions analyzed
- **Cover**: Percentage of regions/lines covered
- **Functions**: Function coverage statistics
- **Lines**: Line-by-line coverage statistics

### HTML Report
The HTML report provides:
- **Line-by-line coverage** with color coding
- **Red lines**: Uncovered code that should be tested
- **Green lines**: Code covered by tests
- **Interactive navigation** through source files

## Coverage Metrics

Current coverage levels:
- **Overall**: ~42% line coverage
- **environment.rs**: ~73% line coverage
- **lib.rs**: ~29% line coverage

## Improving Coverage

To improve coverage:

1. **Identify uncovered lines** in the HTML report (shown in red)
2. **Write tests** that exercise those code paths
3. **Run the coverage script** again to see improvements
4. **Aim for higher coverage** in critical code paths

## Troubleshooting

### Common Issues

**"No profile files generated"**
- Ensure tests are actually running: `cargo test`
- Check that the Rust toolchain is properly installed

**"Could not find LLVM tools"**
- Verify Rust installation: `rustc --version`
- Ensure you're using a complete Rust installation (not just rustc)

**"Could not find test binary"**
- Run `cargo clean` and try again
- Ensure the project builds successfully: `cargo build`

### Manual Coverage Generation

If the scripts don't work, you can generate coverage manually:

```bash
# 1. Clean and build with coverage
cargo clean
RUSTFLAGS="-C instrument-coverage" LLVM_PROFILE_FILE="gridskipper-%p-%m.profraw" cargo test

# 2. Merge profile data
~/.rustup/toolchains/stable-*/lib/rustlib/*/bin/llvm-profdata merge -sparse gridskipper-*.profraw -o gridskipper.profdata

# 3. Generate HTML report
~/.rustup/toolchains/stable-*/lib/rustlib/*/bin/llvm-cov show \
    --format=html \
    --instr-profile=gridskipper.profdata \
    --ignore-filename-regex='/.cargo/registry' \
    target/debug/deps/gridskipper-* \
    --sources src \
    --output-dir=coverage
```

## Git Integration

Coverage files are automatically ignored via `.gitignore`:
- `*.profraw` - Raw profile data
- `*.profdata` - Merged profile data  
- `/coverage/` - HTML reports

## Best Practices

1. **Run coverage regularly** during development
2. **Focus on critical paths** - aim for high coverage on important functionality
3. **Don't chase 100%** - some code paths (error handling, edge cases) may not need testing
4. **Use coverage to find gaps** in your test suite, not as the sole measure of test quality

## Integration with CI/CD

To integrate coverage into your CI/CD pipeline:

```yaml
# Example GitHub Actions step
- name: Generate Coverage
  run: |
    ./scripts/coverage.sh
    # Upload coverage reports to your preferred service
```

For more information about Rust code coverage, see the [official Rust documentation](https://doc.rust-lang.org/rustc/instrument-coverage.html).
