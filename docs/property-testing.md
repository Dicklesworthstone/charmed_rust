# Property-Based Testing in charmed_rust

This guide covers property-based testing patterns using proptest for terminal input/output parsing in the charmed_rust ecosystem.

## Table of Contents

1. [Introduction](#introduction)
2. [Quick Start](#quick-start)
3. [Generator Patterns](#generator-patterns)
4. [Property Patterns](#property-patterns)
5. [Roundtrip Testing](#roundtrip-testing)
6. [Debugging Failures](#debugging-failures)
7. [Best Practices](#best-practices)

## Introduction

Property-based testing (PBT) complements traditional unit tests by:

- Testing with randomly generated inputs
- Automatically finding minimal failing cases (shrinking)
- Discovering edge cases humans might miss
- Verifying invariants across the entire input space

### When to Use Property Tests

| Scenario | Use Property Tests |
|----------|-------------------|
| Parser correctness | Yes - test all valid inputs |
| Roundtrip consistency | Yes - verify parse/serialize cycle |
| Edge case discovery | Yes - find boundary issues |
| Specific bug reproduction | No - use unit tests |
| Performance regression | No - use benchmarks |

## Quick Start

### Adding proptest

Add to your `Cargo.toml`:

```toml
[dev-dependencies]
proptest = "1.5"
proptest-derive = "0.5"
```

### Running Property Tests

```bash
# Default profile (256 cases)
cargo test

# Increase cases for more thorough testing
PROPTEST_CASES=1000 cargo test

# With logging for debugging
RUST_LOG=proptest=debug cargo test -- --nocapture

# Reproduce a specific failure
PROPTEST_SEED=0x1234567890abcdef cargo test
```

### Writing Your First Property Test

```rust
use proptest::prelude::*;

proptest! {
    /// Parser never panics on arbitrary input
    #[test]
    fn parser_never_panics(bytes in prop::collection::vec(any::<u8>(), 0..100)) {
        let _ = parse_input(&bytes);  // Must not panic
    }
}
```

## Generator Patterns

### Using the Arbitrary Trait

The `Arbitrary` trait auto-generates random instances:

```rust
use proptest::arbitrary::Arbitrary;
use proptest_derive::Arbitrary;

#[derive(Debug, Clone, Arbitrary)]
pub enum KeyModifier {
    None,
    Shift,
    Ctrl,
    Alt,
    #[proptest(weight = 0)]  // Exclude from generation
    Reserved,
}
```

### Using prop_compose!

For complex generators with dependencies:

```rust
use proptest::prelude::*;

prop_compose! {
    /// Generate a valid cursor position within terminal bounds
    [pub] fn arb_cursor_position(max_cols: u16, max_rows: u16)(
        col in 1..=max_cols,
        row in 1..=max_rows,
    ) -> (u16, u16) {
        (col, row)
    }
}

prop_compose! {
    /// Generate a mouse event at a valid position
    [pub] fn arb_mouse_event()(
        x in 1u16..=200,
        y in 1u16..=50,
        button in 0u8..=2,
    ) -> MouseEvent {
        MouseEvent { x, y, button }
    }
}
```

### Generator Selection with prop_oneof!

```rust
fn arb_key() -> impl Strategy<Value = Key> {
    prop_oneof![
        // Weight common cases higher
        10 => any::<char>()
            .prop_filter("printable", |c| c.is_ascii_graphic())
            .prop_map(Key::Char),
        5 => (0u8..=26).prop_map(|n| Key::Ctrl((b'a' + n) as char)),
        2 => (1u8..=12).prop_map(|n| Key::F(n)),
        1 => arb_special_key(),
    ]
}
```

### Combining Generators

```rust
prop_compose! {
    fn arb_styled_text()(
        text in "[a-zA-Z0-9 ]{1,50}",
        bold in any::<bool>(),
        color in 0u8..=255,
    ) -> StyledText {
        StyledText { text, bold, color }
    }
}
```

## Property Patterns

### Invariant Properties

Properties that must always hold:

```rust
proptest! {
    /// Parser never panics on any input
    #[test]
    fn never_panics(bytes in prop::collection::vec(any::<u8>(), 0..1000)) {
        let _ = Parser::parse(&bytes);  // Must not panic
    }

    /// Consumed bytes never exceeds input length
    #[test]
    fn consumed_bounded(bytes in prop::collection::vec(any::<u8>(), 1..100)) {
        if let Some((_, consumed)) = Parser::parse(&bytes) {
            prop_assert!(consumed <= bytes.len());
            prop_assert!(consumed >= 1);
        }
    }
}
```

### Oracle Properties

Compare against a reference implementation:

```rust
proptest! {
    /// Our parser matches the reference implementation
    #[test]
    fn matches_reference(input in arb_valid_input()) {
        let our_result = our_parser::parse(&input);
        let ref_result = reference_parser::parse(&input);

        prop_assert_eq!(our_result, ref_result);
    }
}
```

### Metamorphic Properties

Transform input and verify output relationship:

```rust
proptest! {
    /// Parsing prefix doesn't affect suffix parsing
    #[test]
    fn prefix_independent(
        prefix in arb_complete_sequence(),
        suffix in arb_complete_sequence(),
    ) {
        let mut combined = prefix.to_bytes();
        combined.extend(suffix.to_bytes());

        // Parse prefix
        let (_, consumed) = Parser::parse(&combined).unwrap();

        // Remaining should parse same as suffix alone
        let remaining = &combined[consumed..];
        let remaining_result = Parser::parse(remaining);
        let suffix_result = Parser::parse(&suffix.to_bytes());

        prop_assert_eq!(remaining_result, suffix_result);
    }
}
```

### Conditional Properties

```rust
proptest! {
    #[test]
    fn coordinates_preserved(event in arb_mouse_event()) {
        // Only test valid coordinate ranges
        prop_assume!(event.x > 0 && event.y > 0);
        prop_assume!(event.x <= 9999 && event.y <= 9999);

        let bytes = event.to_bytes();
        let (parsed, _) = MouseParser::parse(&bytes).unwrap();

        prop_assert_eq!(parsed.x, event.x);
        prop_assert_eq!(parsed.y, event.y);
    }
}
```

## Roundtrip Testing

Roundtrip tests verify that parse(serialize(x)) == x and vice versa.

### Basic Roundtrip Pattern

```rust
proptest! {
    /// Value survives serialization roundtrip
    #[test]
    fn roundtrip(value in arb_value()) {
        let serialized = value.serialize();
        let deserialized = Value::deserialize(&serialized);

        prop_assert_eq!(deserialized, Ok(value));
    }
}
```

### Canonical Form Roundtrip

```rust
proptest! {
    /// Non-canonical input normalizes to canonical form
    #[test]
    fn canonical_normalization(input in arb_non_canonical_input()) {
        let parsed = Parser::parse(&input).unwrap();
        let canonical = parsed.to_canonical_bytes();
        let reparsed = Parser::parse(&canonical).unwrap();

        // Values equal
        prop_assert_eq!(parsed, reparsed);

        // Canonical form is stable
        let recanonical = reparsed.to_canonical_bytes();
        prop_assert_eq!(canonical, recanonical);
    }
}
```

### Lossy Roundtrip

When some information is lost:

```rust
proptest! {
    /// X10 mouse loses modifier information
    #[test]
    fn x10_lossy_roundtrip(event in arb_sgr_mouse_event()) {
        // X10 has coordinate limits
        prop_assume!(event.x <= 223 && event.y <= 223);

        // Convert to X10 (loses modifiers)
        let x10_bytes = event.to_x10_bytes();
        let (x10_parsed, _) = MouseParser::parse(&x10_bytes).unwrap();

        // Core fields preserved
        prop_assert_eq!(x10_parsed.button, event.button);
        prop_assert_eq!(x10_parsed.x, event.x);
        prop_assert_eq!(x10_parsed.y, event.y);

        // Modifiers lost (X10 limitation) - don't assert
    }
}
```

## Debugging Failures

### Understanding Failure Output

When a property test fails, proptest outputs:

```
thread 'key_tests::ascii_roundtrip' panicked at 'Test failed:
assertion failed: `(left == right)`
  left: `Key::Char('A')`,
 right: `Key::Char('a')`

minimal failing input: key = Key::Char('A')
     successes: 42
     local rejects: 0
     global rejects: 0

To re-run this failing case, add this to your test:
    proptest!(@seed 0x1234567890abcdef)
```

### Reproducing Failures

```bash
# Use the seed from failure output
PROPTEST_SEED=0x1234567890abcdef cargo test key_tests::ascii_roundtrip

# Or add to test file for investigation
proptest! {
    #![proptest_config(ProptestConfig::with_seed(0x1234567890abcdef))]

    #[test]
    fn failing_test(input in arb_input()) {
        // ...
    }
}
```

### Regression Files

Proptest saves failing cases to `target/proptest-regressions/`:

```
target/proptest-regressions/
└── crates/bubbletea/src/key/proptest_tests.rs/
    └── ascii_roundtrip.txt
```

These files ensure the same failure is tested on every run until fixed.

### Adding Logging

```rust
proptest! {
    #[test]
    fn debuggable_test(input in arb_input()) {
        eprintln!("Testing input: {:?}", input);

        let result = process(&input);
        eprintln!("Result: {:?}", result);

        prop_assert!(result.is_valid());
    }
}
```

Run with:

```bash
cargo test -- --nocapture
```

## Best Practices

### DO

1. **Use prop_assert!** instead of assert! for better shrinking
2. **Document generators** with examples of generated values
3. **Keep generators simple** - compose from smaller pieces
4. **Test invariants** that should always hold
5. **Use prop_assume!** to filter invalid inputs
6. **Cache regression files** in CI
7. **Set reasonable case counts** (256 local, 1000 CI)

### DON'T

1. **Don't ignore failures** - they reveal real bugs
2. **Don't generate overly complex inputs** - shrinking becomes slow
3. **Don't test implementation details** - test observable behavior
4. **Don't use property tests for deterministic cases** - use unit tests
5. **Don't skip shrinking** - minimal cases are valuable

### Generator Guidelines

```rust
// Good: Simple, composable
prop_compose! {
    fn arb_point()(x in 0..100i32, y in 0..100i32) -> Point {
        Point { x, y }
    }
}

// Good: Clear constraints
fn arb_valid_utf8() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9 ]{1,100}"
}

// Bad: Too complex, slow to shrink
fn arb_complex_thing() -> impl Strategy<Value = Complex> {
    // Many nested generators - hard to debug
}
```

### Property Guidelines

```rust
// Good: Tests invariant
proptest! {
    #[test]
    fn length_preserved(s in ".*") {
        let processed = process(&s);
        prop_assert_eq!(processed.len(), s.len());
    }
}

// Good: Tests relationship
proptest! {
    #[test]
    fn sorted_output(v in prop::collection::vec(any::<i32>(), 0..100)) {
        let sorted = sort(&v);
        prop_assert!(sorted.windows(2).all(|w| w[0] <= w[1]));
    }
}

// Bad: Tests specific value - use unit test instead
proptest! {
    #[test]
    fn specific_case(x in Just(42)) {
        assert_eq!(f(x), 84);
    }
}
```

## CI Integration

### Running in CI

```yaml
# .github/workflows/ci.yml
test:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - name: Run tests
      run: cargo test --workspace --locked
      env:
        PROPTEST_CASES: 1000
```

### Caching Regressions

```yaml
- name: Cache proptest regressions
  uses: actions/cache@v4
  with:
    path: target/proptest-regressions
    key: proptest-regressions-${{ hashFiles('**/Cargo.lock') }}
    restore-keys: |
      proptest-regressions-
```

## Logging in Property Tests

Enable logging for debugging:

```bash
# Summary level (recommended for CI)
RUST_LOG=proptest=info cargo test

# Detailed level (for debugging failures)
RUST_LOG=proptest=debug cargo test -- --nocapture

# Specific test with full trace
RUST_LOG=trace cargo test key_roundtrip -- --nocapture
```

### Log Levels

| Level | Use Case |
|-------|----------|
| error | Test failures |
| warn  | Skipped tests, unusual conditions |
| info  | Test progress, case counts |
| debug | Generated values, intermediate results |
| trace | Full input/output details |

## Further Reading

- [proptest Book](https://proptest-rs.github.io/proptest/intro.html)
- [Hypothesis (Python equivalent)](https://hypothesis.works/)
- [QuickCheck (Haskell origin)](https://hackage.haskell.org/package/QuickCheck)
