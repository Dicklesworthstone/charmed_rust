#!/usr/bin/env python3
"""Report and enforce Clippy allow policy across the repository."""

from __future__ import annotations

import argparse
import re
import sys
from collections import Counter, defaultdict
from pathlib import Path


ATTRIBUTE_RE = re.compile(r"#\s*!?\s*\[\s*allow\s*\((.*?)\)\s*\]", re.DOTALL)
LINT_RE = re.compile(r"clippy::([a-zA-Z0-9_]+)")
FORBIDDEN = {"all", "style", "pedantic", "nursery"}


def discover_rs_files(repo_root: Path) -> list[Path]:
    paths: list[Path] = []
    for root in (repo_root / "crates", repo_root / "tests"):
        if not root.exists():
            continue
        paths.extend(root.rglob("*.rs"))
    return sorted(paths)


def area_key(repo_root: Path, file_path: Path) -> str:
    rel = file_path.relative_to(repo_root)
    parts = rel.parts
    if len(parts) >= 2 and parts[0] == "crates":
        return f"crates/{parts[1]}"
    if len(parts) >= 2 and parts[0] == "tests" and parts[1] == "conformance":
        return "tests/conformance"
    if parts and parts[0] == "tests":
        return "tests"
    return "other"


def collect(
    repo_root: Path,
) -> tuple[Counter[str], Counter[str], list[tuple[str, int, str]]]:
    by_area: Counter[str] = Counter()
    by_lint: Counter[str] = Counter()
    forbidden_hits: list[tuple[str, int, str]] = []

    for path in discover_rs_files(repo_root):
        text = path.read_text(encoding="utf-8")
        rel = path.relative_to(repo_root)

        for match in ATTRIBUTE_RE.finditer(text):
            body = match.group(1)
            lints = LINT_RE.findall(body)
            if not lints:
                continue

            key = area_key(repo_root, path)
            for lint in lints:
                by_area[key] += 1
                by_lint[lint] += 1

                if lint in FORBIDDEN and match.group(0).lstrip().startswith("#!"):
                    line = text.count("\n", 0, match.start()) + 1
                    forbidden_hits.append((str(rel), line, lint))

    return by_area, by_lint, forbidden_hits


def render(
    by_area: Counter[str],
    by_lint: Counter[str],
    forbidden_hits: list[tuple[str, int, str]],
) -> str:
    total = sum(by_lint.values())
    lines: list[str] = []
    lines.append("## Clippy Allow Report")
    lines.append("")
    lines.append(f"- Total `allow(clippy::...)` entries: **{total}**")
    lines.append("")

    lines.append("### By Area")
    lines.append("")
    lines.append("| Area | Count |")
    lines.append("|---|---:|")
    for area, count in sorted(by_area.items(), key=lambda it: (-it[1], it[0])):
        lines.append(f"| `{area}` | {count} |")
    lines.append("")

    lines.append("### Top Lints")
    lines.append("")
    lines.append("| Lint | Count |")
    lines.append("|---|---:|")
    for lint, count in sorted(by_lint.items(), key=lambda it: (-it[1], it[0]))[:20]:
        lines.append(f"| `clippy::{lint}` | {count} |")
    lines.append("")

    lines.append("### Forbidden Inner-Attribute Check")
    lines.append("")
    if not forbidden_hits:
        lines.append(
            "- No forbidden `#![allow(clippy::all|style|pedantic|nursery)]` patterns found."
        )
    else:
        lines.append("- Forbidden patterns found:")
        for path, line, lint in forbidden_hits:
            lines.append(f"  - `{path}:{line}` (`clippy::{lint}`)")

    lines.append("")
    return "\n".join(lines)


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Clippy allow metrics and policy check"
    )
    parser.add_argument(
        "--fail-on-forbidden",
        action="store_true",
        help="Return non-zero if forbidden patterns exist",
    )
    parser.add_argument(
        "--summary-path", type=Path, help="Append markdown report to this file"
    )
    args = parser.parse_args()

    repo_root = Path(__file__).resolve().parents[1]
    by_area, by_lint, forbidden_hits = collect(repo_root)
    report = render(by_area, by_lint, forbidden_hits)

    print(report)

    if args.summary_path is not None:
        with args.summary_path.open("a", encoding="utf-8") as fh:
            fh.write(report)
            fh.write("\n")

    if args.fail_on_forbidden and forbidden_hits:
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
