# charmed_conformance

Conformance testing harness for the charmed_rust ecosystem.

## TL;DR

**The Problem:** Ports are only as good as their behavioral parity with the
original Go implementations.

**The Solution:** `charmed_conformance` runs fixture-driven tests to ensure that
Rust outputs match expected reference behavior.

**Why Conformance Tests**

- **Behavioral parity**: validates output against known fixtures.
- **Regression safety**: catches subtle changes in rendering and logic.
- **Cross-crate coverage**: exercises harmonica, lipgloss, bubbletea, bubbles,
  glamour, huh, glow, and wish.

## Role in the charmed_rust (FrankenTUI) stack

This crate is the verification layer for the entire stack. It is not published
and should be run from the workspace during development or CI.

## How to Run

From the workspace root:

```bash
cargo test -p charmed_conformance
```

Run the CLI harness:

```bash
cargo run -p charmed_conformance --bin run-conformance
```

Generate a report:

```bash
cargo run -p charmed_conformance --bin generate-report
```

## Feature Flags

- `wish` (default): includes SSH-related conformance tests.

Disable wish tests:

```bash
cargo test -p charmed_conformance --no-default-features
```

## Test Inputs and Fixtures

Fixtures are stored under `tests/conformance/` and include reference outputs
captured from the Go implementations. The harness normalizes Unicode and produces
diffs for mismatches.

## Troubleshooting

- **Mismatch diffs**: check fixture normalization and ensure your changes are
  intentional.
- **Slow runs**: disable `wish` if you’re not working on SSH features.

## Limitations

- Fixture-based tests only cover what has been captured.
- SSH tests may be environment-sensitive depending on host capabilities.

## About Contributions

Please don't take this the wrong way, but I do not accept outside contributions for any of my projects. I simply don't have the mental bandwidth to review anything, and it's my name on the thing, so I'm responsible for any problems it causes; thus, the risk-reward is highly asymmetric from my perspective. I'd also have to worry about other "stakeholders," which seems unwise for tools I mostly make for myself for free. Feel free to submit issues, and even PRs if you want to illustrate a proposed fix, but know I won't merge them directly. Instead, I'll have Claude or Codex review submissions via `gh` and independently decide whether and how to address them. Bug reports in particular are welcome. Sorry if this offends, but I want to avoid wasted time and hurt feelings. I understand this isn't in sync with the prevailing open-source ethos that seeks community contributions, but it's the only way I can move at this velocity and keep my sanity.

## License

MIT. See `LICENSE` at the repository root.
