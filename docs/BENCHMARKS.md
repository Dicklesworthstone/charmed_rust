# Benchmark Suite Documentation

This document describes the benchmark infrastructure for charmed_rust, including how to run benchmarks, interpret results, and add new benchmarks.

---

## Overview

The charmed_rust benchmark suite provides comprehensive performance testing for all major crates:

| Crate | Benchmark File | Categories |
|-------|---------------|------------|
| lipgloss | `lipgloss_benchmarks.rs` | Style creation, colors, rendering, layout, borders |
| bubbletea | `bubbletea_benchmarks.rs` | Message dispatch, view rendering, commands, key parsing |
| glamour | `glamour_benchmarks.rs` | Markdown parsing, element rendering, config impact |
| bubbles | `bubbles_benchmarks.rs` | List, table, viewport, textinput, paginator, spinner, progress |

Benchmarks use [Criterion.rs](https://github.com/bheisler/criterion.rs), the de-facto standard for Rust benchmarking, providing statistical rigor and regression detection.

---

## Methodology

### Why Criterion?

Criterion was chosen for several reasons:

1. **Statistical analysis**: Automatically calculates confidence intervals and detects statistically significant changes
2. **Warm-up handling**: Runs warm-up iterations before measurement to ensure stable results
3. **Baseline comparison**: Stores historical results for regression detection
4. **HTML reports**: Generates detailed HTML reports with graphs and analysis
5. **Industry standard**: Used by Rust compiler, Tokio, and most performance-critical Rust projects

### Measurement Settings

Default Criterion settings used across all benchmarks:

| Setting | Value | Rationale |
|---------|-------|-----------|
| Warm-up | 3 seconds | Ensures JIT, caches, and branch predictors are primed |
| Measurement time | 5 seconds | Sufficient samples for statistical significance |
| Sample size | 100 | Default; adjusted automatically by Criterion |
| Confidence level | 95% | Standard for detecting regressions |
| Noise threshold | 2% | Changes below this are considered noise |

### Regression Thresholds

The CI pipeline uses these thresholds:

| Level | Threshold | Action |
|-------|-----------|--------|
| Noise | < 2% | Ignored |
| Minor | 2-10% | Logged, no action |
| Warning | 10-20% | Warning in PR comment |
| Critical | > 20% | CI fails, requires investigation |

### Known Measurement Limitations

1. **CPU frequency scaling**: Results may vary with CPU frequency. CI uses dedicated runners for consistency.
2. **Memory allocator**: Benchmark results include allocator overhead. We use the system allocator for consistency.
3. **Micro-benchmarks**: Very fast operations (<10ns) have higher measurement noise.
4. **First-run effects**: Some operations (lazy initialization, file I/O) have different first-run characteristics.

---

## Running Benchmarks Locally

### Full Benchmark Suite

Run all benchmarks across all crates:

```bash
cargo bench --workspace
```

This takes approximately 5-10 minutes and generates HTML reports in `target/criterion/`.

### Single Crate Benchmarks

Run benchmarks for a specific crate:

```bash
# Lipgloss (style and layout)
cargo bench -p lipgloss

# Bubbletea (runtime and messages)
cargo bench -p bubbletea

# Glamour (markdown rendering)
cargo bench -p glamour

# Bubbles (TUI components)
cargo bench -p bubbles
```

### Specific Benchmark

Run only benchmarks matching a pattern:

```bash
# All rendering benchmarks
cargo bench -- rendering

# All lipgloss benchmarks
cargo bench -- lipgloss

# Specific benchmark by name
cargo bench -- "lipgloss/rendering/render/short/simple"
```

### Quick Benchmarks (No Plots)

For faster iteration without HTML report generation:

```bash
cargo bench --workspace -- --noplot
```

### Baseline Comparison

Save a baseline for later comparison:

```bash
# Save current results as "main" baseline
cargo bench --workspace -- --save-baseline main

# Make changes, then compare
cargo bench --workspace -- --baseline main
```

Example output showing comparison:
```
lipgloss/rendering/render/short/simple
  time:   [1.2345 µs 1.2456 µs 1.2567 µs]
  change: [-2.1234% -1.0000% +0.1234%] (p = 0.12 > 0.05)
  No change in performance detected.
```

### Viewing HTML Reports

After running benchmarks, open the HTML report:

```bash
# macOS
open target/criterion/report/index.html

# Linux
xdg-open target/criterion/report/index.html
```

The report includes:
- Summary table of all benchmarks
- Time distribution graphs
- Comparison with previous runs
- Detailed statistics for each benchmark

---

## Go Comparison Benchmarks

To compare charmed_rust performance against the original Go Charm libraries:

### Prerequisites

- Go 1.21+ installed
- Python 3.8+ installed

### Running Comparison

```bash
# Full comparison (runs both Go and Rust benchmarks)
./scripts/compare_benchmarks.sh

# JSON output (for programmatic use)
./scripts/compare_benchmarks.sh --json
```

### Manual Comparison

1. Run Go benchmarks:
```bash
cd tests/conformance/go_reference
go test -bench=. -benchmem -benchtime=1s ./bench/...
```

2. Run Rust benchmarks:
```bash
cargo bench --workspace -- --noplot
```

3. Compare results:
```bash
python3 scripts/compare_results.py /tmp/charmed_bench/go_bench.txt /tmp/charmed_bench/rust_bench.txt
```

### Interpreting Comparison Results

The comparison script outputs a table with:

| Column | Description |
|--------|-------------|
| Benchmark | Normalized benchmark name |
| Go | Go execution time |
| Rust | Rust execution time |
| Ratio | Rust time / Go time (lower is better for Rust) |
| Status | Performance category |

Status categories:
- `++` excellent: Rust is faster or equal (ratio <= 1.0x)
- `+` good: Rust within 2x of Go (ratio <= 2.0x)
- `~` acceptable: Rust within 5x of Go (ratio <= 5.0x)
- `!!` needs_work: Rust more than 5x slower

**Note**: Some benchmarks may not have direct equivalents due to architectural differences.

---

## Performance Summary

### Lipgloss (Style & Layout)

| Operation | Typical Time | Throughput |
|-----------|-------------|------------|
| Style::new() | ~5 ns | - |
| Style with all props | ~100-200 ns | - |
| Render short text | ~1-2 µs | - |
| Render paragraph | ~5-10 µs | ~10-20 MB/s |
| join_horizontal (10 items) | ~3-5 µs | - |
| Border rendering | ~2-4 µs | - |

### Bubbletea (Runtime)

| Operation | Typical Time | Notes |
|-----------|-------------|-------|
| Single message dispatch | ~5-10 ns | Type-based routing |
| 1000 messages | ~10-15 µs | Batched processing |
| Simple view render | ~80-100 ns | Minimal state |
| Frame cycle (update + view) | ~100-200 ns | Typical game loop |
| KeyMsg creation | <1 ns | Zero-allocation |

### Glamour (Markdown)

| Operation | Typical Time | Throughput |
|-----------|-------------|------------|
| Small doc (~100 bytes) | ~50-100 µs | ~1-2 MB/s |
| Medium doc (~500 bytes) | ~200-500 µs | ~1-2 MB/s |
| Large doc (~10KB) | ~5-15 ms | ~0.5-1 MB/s |
| Code block (no highlighting) | ~10-30 µs | - |
| Table rendering | ~20-50 µs | - |

### Bubbles (Components)

| Operation | Typical Time | Notes |
|-----------|-------------|-------|
| List create (100 items) | ~50-100 µs | With delegate |
| List view (100 items) | ~50-100 µs | Visible items only |
| Table view (100 rows) | ~50-100 µs | With styling |
| Viewport render (1000 lines) | ~50-100 µs | Scrollable content |
| TextInput view | ~3-5 µs | With cursor |
| Spinner view | ~500 ns | Animation frame |
| Progress view (50%) | ~20-30 µs | With gradient |

**Note**: These are approximate values. Run benchmarks locally for exact measurements on your hardware.

---

## Contributing Benchmarks

### Adding a New Benchmark

1. **Locate the benchmark file**: `crates/<crate>/benches/<crate>_benchmarks.rs`

2. **Add a new benchmark function**:
```rust
fn bench_my_operation(c: &mut Criterion) {
    let mut group = c.benchmark_group("crate/category");

    group.bench_function("operation_name", |b| {
        b.iter(|| {
            black_box(my_operation())
        });
    });

    group.finish();
}
```

3. **Add to criterion_group**:
```rust
criterion_group!(
    benches,
    bench_existing,
    bench_my_operation,  // Add here
);
```

### Naming Conventions

Benchmark names follow a hierarchical pattern:

```
{crate}/{category}/{operation}[/{variant}]
```

Examples:
- `lipgloss/rendering/render/short/simple`
- `bubbletea/message_dispatch/single_message`
- `glamour/elements/code_blocks`
- `bubbles/list/view_100`

### What to Benchmark

**Do benchmark**:
- Public API operations users will call frequently
- Operations that are performance-critical (rendering, parsing)
- Operations where performance may regress (complex logic)
- Different input sizes to understand scaling behavior

**Don't benchmark**:
- Internal implementation details that may change
- One-time initialization (unless it's user-facing)
- Error paths (unless error handling is a concern)
- Operations that delegate entirely to external crates

### Validating Benchmark Correctness

Ensure your benchmark actually measures what you intend:

1. **Use `black_box`**: Prevents compiler from optimizing away the result
2. **Avoid setup in the loop**: Move setup outside `b.iter()`
3. **Check for consistent results**: Run multiple times, results should be similar
4. **Verify the operation runs**: Add a sanity check outside the benchmark

Example with setup:
```rust
fn bench_with_setup(c: &mut Criterion) {
    // Setup OUTSIDE the benchmark loop
    let data = prepare_large_dataset();

    c.bench_function("process_data", |b| {
        b.iter(|| {
            black_box(process(&data))
        });
    });
}
```

### Throughput Benchmarks

For operations processing data, add throughput measurement:

```rust
fn bench_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("rendering");

    let input = generate_input(1000);
    group.throughput(Throughput::Bytes(input.len() as u64));

    group.bench_function("render_1000", |b| {
        b.iter(|| black_box(render(&input)));
    });

    group.finish();
}
```

---

## CI Integration

### Automatic Benchmark Runs

The `.github/workflows/benchmarks.yml` workflow:

1. **On push to main**: Runs benchmarks and saves baseline
2. **On PRs**: Compares against main baseline and posts results as a comment

### PR Comment Format

PR benchmark comments include:
- Summary of regressions (if any)
- Full benchmark results in a collapsible section
- Warning/error status for regression detection

### Failing Builds

A PR will fail if any benchmark regresses more than 20%. To fix:

1. Review the benchmark results in the PR comment
2. Identify the regressed benchmark
3. Profile the code to find the cause
4. Fix the regression or justify the change
5. Re-push to trigger new benchmark run

### Local Pre-Push Check

Before pushing, run comparison against your baseline:

```bash
# Save baseline before changes
cargo bench --workspace -- --save-baseline before

# Make changes, then compare
cargo bench --workspace -- --baseline before
```

---

## Troubleshooting

### Benchmarks Take Too Long

- Use `--noplot` to skip HTML report generation
- Run specific benchmarks with `-- <pattern>`
- Reduce sample size: `-- --sample-size 50`

### Results Are Inconsistent

- Close other applications to reduce system noise
- Disable CPU frequency scaling (if possible)
- Run benchmarks multiple times and compare
- Consider using `cargo bench -- --measurement-time 10` for more samples

### Baseline Not Found

If comparison fails with "baseline not found":
- Save a baseline first: `cargo bench -- --save-baseline main`
- Or use `--save-baseline` on first run

### Out of Memory

For large benchmarks:
- Run crates individually: `cargo bench -p <crate>`
- Reduce benchmark input sizes temporarily
- Increase system swap space

---

## References

- [Criterion.rs User Guide](https://bheisler.github.io/criterion.rs/book/)
- [Criterion.rs API Docs](https://docs.rs/criterion)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Go testing package benchmark docs](https://pkg.go.dev/testing#hdr-Benchmarks)

---

*Document created: 2026-01-19*
*Bead: charmed_rust-58o*
