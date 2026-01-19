#!/usr/bin/env python3
"""
Compare Go and Rust benchmark results.

Parses benchmark output from both Go's testing package and Rust's criterion
and produces a comparison table with performance ratios.

Usage:
    python3 compare_results.py go_bench.txt rust_bench.txt [--json]
"""

import argparse
import json
import re
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Optional


@dataclass
class BenchResult:
    """A single benchmark result."""
    name: str
    ns_per_op: float
    bytes_per_op: Optional[int] = None
    allocs_per_op: Optional[int] = None
    source: str = ""


def normalize_name(name: str) -> str:
    """Normalize benchmark names for matching between Go and Rust."""
    # Remove common prefixes
    name = re.sub(r'^Benchmark', '', name)
    name = re.sub(r'^bench_', '', name)

    # Normalize separators
    name = name.replace('_', '').replace('/', '').lower()

    # Remove trailing numbers (like -8 for GOMAXPROCS)
    name = re.sub(r'-\d+$', '', name)

    return name


def parse_go_bench(path: Path) -> dict[str, BenchResult]:
    """Parse Go benchmark output."""
    results = {}

    # Pattern: BenchmarkName-8    1000    1234 ns/op    56 B/op    2 allocs/op
    pattern = re.compile(
        r'^Benchmark(\S+?)(?:-\d+)?\s+'  # Name with optional -N suffix
        r'(\d+)\s+'                       # Iterations
        r'([\d.]+)\s+ns/op'              # Time per op
        r'(?:\s+(\d+)\s+B/op)?'          # Optional bytes per op
        r'(?:\s+(\d+)\s+allocs/op)?'     # Optional allocs per op
    )

    with open(path) as f:
        for line in f:
            match = pattern.match(line.strip())
            if match:
                name = match.group(1)
                ns = float(match.group(3))
                bytes_op = int(match.group(4)) if match.group(4) else None
                allocs_op = int(match.group(5)) if match.group(5) else None

                results[normalize_name(name)] = BenchResult(
                    name=f"Benchmark{name}",
                    ns_per_op=ns,
                    bytes_per_op=bytes_op,
                    allocs_per_op=allocs_op,
                    source="go"
                )

    return results


def parse_rust_bench(path: Path) -> dict[str, BenchResult]:
    """Parse Rust criterion benchmark output."""
    results = {}
    current_name = None

    # Criterion output patterns
    # Group header: lipgloss/rendering
    # Bench name line: render/short/simple
    # Time line: time:   [1.2345 µs 1.2456 µs 1.2567 µs]

    name_pattern = re.compile(r'^(\S+/\S+)\s*$|^(\S+)\s+time:')
    time_pattern = re.compile(
        r'time:\s+\[([\d.]+)\s+(\w+)\s+([\d.]+)\s+(\w+)\s+([\d.]+)\s+(\w+)\]'
    )

    # Also match simpler criterion output format
    simple_pattern = re.compile(
        r'^(\S+)\s+time:\s+\[([\d.]+)\s+(\w+)'
    )

    with open(path) as f:
        for line in f:
            line = line.strip()

            # Try to match benchmark name
            name_match = name_pattern.match(line)
            if name_match:
                current_name = name_match.group(1) or name_match.group(2)
                if current_name:
                    current_name = current_name.replace('/', '_')

            # Try to match time line
            time_match = time_pattern.search(line)
            if time_match and current_name:
                # Use the middle (median) value
                median_val = float(time_match.group(3))
                unit = time_match.group(4)

                # Convert to nanoseconds
                ns = convert_to_ns(median_val, unit)

                results[normalize_name(current_name)] = BenchResult(
                    name=current_name,
                    ns_per_op=ns,
                    source="rust"
                )
                current_name = None

            # Try simpler format
            simple_match = simple_pattern.match(line)
            if simple_match:
                name = simple_match.group(1)
                val = float(simple_match.group(2))
                unit = simple_match.group(3)

                ns = convert_to_ns(val, unit)

                results[normalize_name(name)] = BenchResult(
                    name=name,
                    ns_per_op=ns,
                    source="rust"
                )

    return results


def convert_to_ns(value: float, unit: str) -> float:
    """Convert time value to nanoseconds."""
    unit = unit.lower()
    if unit in ('ns', 'nanoseconds'):
        return value
    elif unit in ('µs', 'us', 'microseconds'):
        return value * 1000
    elif unit in ('ms', 'milliseconds'):
        return value * 1_000_000
    elif unit in ('s', 'seconds'):
        return value * 1_000_000_000
    else:
        # Assume nanoseconds
        return value


def compare_results(go_results: dict, rust_results: dict) -> list[dict]:
    """Compare Go and Rust benchmark results."""
    comparisons = []

    # Find matching benchmarks
    all_names = set(go_results.keys()) | set(rust_results.keys())

    for norm_name in sorted(all_names):
        go_res = go_results.get(norm_name)
        rust_res = rust_results.get(norm_name)

        comparison = {
            'normalized_name': norm_name,
            'go_name': go_res.name if go_res else None,
            'rust_name': rust_res.name if rust_res else None,
            'go_ns': go_res.ns_per_op if go_res else None,
            'rust_ns': rust_res.ns_per_op if rust_res else None,
            'ratio': None,
            'status': 'missing_both'
        }

        if go_res and rust_res:
            if go_res.ns_per_op > 0:
                ratio = rust_res.ns_per_op / go_res.ns_per_op
                comparison['ratio'] = ratio

                if ratio <= 1.0:
                    comparison['status'] = 'excellent'  # Rust is faster
                elif ratio <= 2.0:
                    comparison['status'] = 'good'
                elif ratio <= 5.0:
                    comparison['status'] = 'acceptable'
                else:
                    comparison['status'] = 'needs_work'
        elif go_res:
            comparison['status'] = 'missing_rust'
        elif rust_res:
            comparison['status'] = 'missing_go'

        comparisons.append(comparison)

    return comparisons


def format_time(ns: Optional[float]) -> str:
    """Format nanoseconds as human-readable time."""
    if ns is None:
        return "N/A"

    if ns < 1000:
        return f"{ns:.1f} ns"
    elif ns < 1_000_000:
        return f"{ns/1000:.1f} µs"
    elif ns < 1_000_000_000:
        return f"{ns/1_000_000:.1f} ms"
    else:
        return f"{ns/1_000_000_000:.2f} s"


def print_table(comparisons: list[dict]) -> None:
    """Print comparison results as a table."""
    # Calculate column widths
    name_width = max(len(c['normalized_name']) for c in comparisons)
    name_width = max(name_width, 20)

    # Header
    print(f"{'Benchmark':<{name_width}}  {'Go':<12}  {'Rust':<12}  {'Ratio':<8}  Status")
    print("-" * (name_width + 50))

    # Status symbols
    symbols = {
        'excellent': '++',
        'good': '+',
        'acceptable': '~',
        'needs_work': '!!',
        'missing_rust': '?R',
        'missing_go': '?G',
        'missing_both': '??'
    }

    matched_count = 0
    excellent_count = 0
    good_count = 0

    for comp in comparisons:
        name = comp['normalized_name'][:name_width]
        go_time = format_time(comp['go_ns'])
        rust_time = format_time(comp['rust_ns'])
        ratio = f"{comp['ratio']:.2f}x" if comp['ratio'] else "N/A"
        status = comp['status']
        symbol = symbols.get(status, '?')

        print(f"{name:<{name_width}}  {go_time:<12}  {rust_time:<12}  {ratio:<8}  {symbol} {status}")

        if comp['ratio'] is not None:
            matched_count += 1
            if status == 'excellent':
                excellent_count += 1
            elif status in ('excellent', 'good'):
                good_count += 1

    # Summary
    print()
    print("=" * 60)
    print("Summary:")
    print(f"  Total benchmarks:    {len(comparisons)}")
    print(f"  Matched (Go+Rust):   {matched_count}")
    print(f"  Excellent (<=1.0x):  {excellent_count}")
    print(f"  Good (<=2.0x):       {good_count}")
    print()
    print("Legend:")
    print("  ++ excellent (Rust faster or equal)")
    print("  +  good (Rust within 2x)")
    print("  ~  acceptable (Rust within 5x)")
    print("  !! needs_work (Rust >5x slower)")
    print("  ?R missing Rust benchmark")
    print("  ?G missing Go benchmark")


def main():
    parser = argparse.ArgumentParser(description='Compare Go and Rust benchmark results')
    parser.add_argument('go_bench', type=Path, help='Path to Go benchmark output')
    parser.add_argument('rust_bench', type=Path, help='Path to Rust benchmark output')
    parser.add_argument('--json', action='store_true', help='Output as JSON')

    args = parser.parse_args()

    if not args.go_bench.exists():
        print(f"Error: Go benchmark file not found: {args.go_bench}", file=sys.stderr)
        sys.exit(1)

    if not args.rust_bench.exists():
        print(f"Error: Rust benchmark file not found: {args.rust_bench}", file=sys.stderr)
        sys.exit(1)

    go_results = parse_go_bench(args.go_bench)
    rust_results = parse_rust_bench(args.rust_bench)

    comparisons = compare_results(go_results, rust_results)

    if args.json:
        output = {
            'comparisons': comparisons,
            'summary': {
                'total': len(comparisons),
                'matched': sum(1 for c in comparisons if c['ratio'] is not None),
                'excellent': sum(1 for c in comparisons if c['status'] == 'excellent'),
                'good': sum(1 for c in comparisons if c['status'] in ('excellent', 'good')),
                'acceptable': sum(1 for c in comparisons if c['status'] in ('excellent', 'good', 'acceptable')),
                'needs_work': sum(1 for c in comparisons if c['status'] == 'needs_work'),
            }
        }
        print(json.dumps(output, indent=2))
    else:
        print_table(comparisons)


if __name__ == '__main__':
    main()
