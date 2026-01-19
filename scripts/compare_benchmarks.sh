#!/bin/bash
# Compare Go and Rust benchmark results
# Usage: ./scripts/compare_benchmarks.sh [--json]
#
# This script runs benchmarks for both Go (original Charm libraries) and
# Rust (charmed_rust) and compares the results.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
GO_REF_DIR="$PROJECT_ROOT/tests/conformance/go_reference"
OUTPUT_DIR="${OUTPUT_DIR:-/tmp/charmed_bench}"

JSON_OUTPUT=false
if [[ "${1:-}" == "--json" ]]; then
    JSON_OUTPUT=true
fi

mkdir -p "$OUTPUT_DIR"

log() {
    if [[ "$JSON_OUTPUT" == "false" ]]; then
        echo "$@"
    fi
}

log "=== charmed_rust Benchmark Comparison ==="
log "Output directory: $OUTPUT_DIR"
log ""

# Run Go benchmarks
log "=== Running Go Benchmarks ==="
cd "$GO_REF_DIR"

# Disable GC for fairer comparison
export GOGC=off

if ! go test -bench=. -benchmem -benchtime=1s ./bench/... > "$OUTPUT_DIR/go_bench.txt" 2>&1; then
    log "Warning: Some Go benchmarks may have failed"
    log "Check $OUTPUT_DIR/go_bench.txt for details"
fi

log "Go benchmarks written to $OUTPUT_DIR/go_bench.txt"

# Run Rust benchmarks
log ""
log "=== Running Rust Benchmarks ==="
cd "$PROJECT_ROOT"

# Run each crate's benchmarks separately and collect output
RUST_OUTPUT="$OUTPUT_DIR/rust_bench.txt"
> "$RUST_OUTPUT"  # Clear file

for crate in lipgloss bubbletea glamour bubbles; do
    log "Running $crate benchmarks..."
    if cargo bench -p "$crate" --bench "${crate}_benchmarks" -- --noplot 2>/dev/null >> "$RUST_OUTPUT"; then
        log "  $crate: OK"
    else
        log "  $crate: SKIPPED (benchmark may not exist)"
    fi
done

log "Rust benchmarks written to $RUST_OUTPUT"

# Compare results
log ""
log "=== Comparison Results ==="

if [[ "$JSON_OUTPUT" == "true" ]]; then
    python3 "$SCRIPT_DIR/compare_results.py" "$OUTPUT_DIR/go_bench.txt" "$OUTPUT_DIR/rust_bench.txt" --json
else
    python3 "$SCRIPT_DIR/compare_results.py" "$OUTPUT_DIR/go_bench.txt" "$OUTPUT_DIR/rust_bench.txt"
fi
