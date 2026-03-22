# Changelog

All notable changes to **charmed_rust** are documented in this file.

Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) with live commit/tag links.
Versioning: [Semantic Versioning](https://semver.org/spec/v2.0.0.html) across all workspace crates.

Repository: <https://github.com/Dicklesworthstone/charmed_rust>

---

## [Unreleased] — after v0.2.0

[Unreleased]: https://github.com/Dicklesworthstone/charmed_rust/compare/v0.2.0...HEAD

10 commits since v0.2.0. HEAD is [`8a18ca8`](https://github.com/Dicklesworthstone/charmed_rust/commit/8a18ca87e3046c30ab50c1821c28a1b1a7c907b7).

### Added

- Non-consuming `try_downcast` method on `Message` in bubbletea ([`0ced628`](https://github.com/Dicklesworthstone/charmed_rust/commit/0ced6280e91c2cba69fdd4e397004c40c8bceced))

### Changed

- Reduced full-screen flicker by rendering only changed rows ([`17d9c9f`](https://github.com/Dicklesworthstone/charmed_rust/commit/17d9c9fcaf4f94a6874ba687ed01ab65e61a902d))
- Replaced crate-wide clippy allows with per-module lint suppression across all crates ([`cd98380`](https://github.com/Dicklesworthstone/charmed_rust/commit/cd98380971a8a60f451d707f4c1791ca0294e347), [`1ed50fa`](https://github.com/Dicklesworthstone/charmed_rust/commit/1ed50faf710a4a2a0778e62c5a5d17504f0c333b))
- Updated license to MIT with OpenAI/Anthropic Rider ([`e1e9403`](https://github.com/Dicklesworthstone/charmed_rust/commit/e1e940303c7ab6e1cbe14d67de65bcf9af19d4f9), [`99cf951`](https://github.com/Dicklesworthstone/charmed_rust/commit/99cf951ca4a91f5113c2bf23f18185e3e9676515))
- Upgraded dependencies and refreshed test snapshots/benchmarks ([`0e51a1c`](https://github.com/Dicklesworthstone/charmed_rust/commit/0e51a1c857d6ec8b81dd5b3ecea77281545bef30))

### Housekeeping

- Added GitHub social preview image ([`5d604d4`](https://github.com/Dicklesworthstone/charmed_rust/commit/5d604d4ee8d360ca83e35a50f504b824e7898ac4))
- Removed stale `a.out` compiler output ([`8a18ca8`](https://github.com/Dicklesworthstone/charmed_rust/commit/8a18ca87e3046c30ab50c1821c28a1b1a7c907b7))

---

## [v0.2.0] — 2026-02-15 (GitHub Release)

[v0.2.0]: https://github.com/Dicklesworthstone/charmed_rust/compare/v0.1.2...v0.2.0

92 commits since v0.1.2. Tagged at [`eed5f91`](https://github.com/Dicklesworthstone/charmed_rust/commit/eed5f91a9e661d9cc1490bf12b0ac67fe436b569). A major hardening and conformance release across the entire workspace.

**Published crates (all 0.2.0):** charmed-bubbletea, charmed-bubbletea-macros, charmed-lipgloss, charmed-bubbles, charmed-glamour, charmed-harmonica, charmed-huh, charmed-wish, charmed-glow, charmed-log, charmed-wasm, charmed-demo-showcase.

### Conformance Testing

- All crates now have real conformance tests (glow, wish, bubbles, textarea, textinput) — skipped tests now fail the suite ([`c9cb86b`](https://github.com/Dicklesworthstone/charmed_rust/commit/c9cb86ba21d324f66c6e56ffd9883179698f15ab), [`f861235`](https://github.com/Dicklesworthstone/charmed_rust/commit/f861235dd212e72c87ea062563f77238931d41c9))
- Unknown conformance fixtures must fail, not skip ([`362a131`](https://github.com/Dicklesworthstone/charmed_rust/commit/362a1313198e06cdffb87a905011a0688a7f212a))
- Made glow conformance real ([`9250abf`](https://github.com/Dicklesworthstone/charmed_rust/commit/9250abf826ff72528bd6b8b0dc85e9a708a0fa6f))
- Included glow + wish in conformance; fixed wish middleware conformance ([`3bfbd70`](https://github.com/Dicklesworthstone/charmed_rust/commit/3bfbd70eacd45dafa156b4a40a9bca261d9495f1))
- Added textarea conformance + fixed textarea view parity ([`5b6c43c`](https://github.com/Dicklesworthstone/charmed_rust/commit/5b6c43cbf86b4dc8ceecc9b8380a2aa098fa040b))
- Strict textinput echo_mode conformance parsing ([`9e7c5d8`](https://github.com/Dicklesworthstone/charmed_rust/commit/9e7c5d84ab925398a33553e211411c500408c5fe))
- Simplified UBS ignore globs and textinput echo mode matching ([`036b9ee`](https://github.com/Dicklesworthstone/charmed_rust/commit/036b9ee55f5d730909fcf3ab52442a7d6b89dc6d))

### wish (SSH Framework)

- Added shell-aware exec command parser with quote and escape handling ([`d54cdfc`](https://github.com/Dicklesworthstone/charmed_rust/commit/d54cdfc5ea5d571ab9773d64dd4f79393dfc8f71))
- Added typed max-session error for wish session manager ([`32d9bf5`](https://github.com/Dicklesworthstone/charmed_rust/commit/32d9bf57a62cc62023779e8dc21a8c08e7534ec0))
- Added `listen_with_listener` for race-free test server startup ([`6c9a7c8`](https://github.com/Dicklesworthstone/charmed_rust/commit/6c9a7c8cc86b5a138781fe6112b13ca7bbeffe71))
- Hardened SSH e2e probes and credentials ([`8eb4280`](https://github.com/Dicklesworthstone/charmed_rust/commit/8eb4280ae8c8466afeeb36f9f57f34f597a1bb3e))
- Removed unwrap/expect from ssh_e2e harness ([`b648181`](https://github.com/Dicklesworthstone/charmed_rust/commit/b64818107d3f3129948ee9d5cb99114b122a1157))
- Removed assert-false fallback in ssh_e2e ([`6ed9a04`](https://github.com/Dicklesworthstone/charmed_rust/commit/6ed9a04144ebe0e9a2821439354367da0fe70356))
- Hardened tea middleware join-error path ([`76ae48e`](https://github.com/Dicklesworthstone/charmed_rust/commit/76ae48e4b0265ff9987b5db09ce6a8e9256338f4))
- Hardened authorized_keys base64 padding validation ([`c347433`](https://github.com/Dicklesworthstone/charmed_rust/commit/c3474336477664978694cb9a2548b9bb36b8b1a5))
- Enforced strict base64 decode in github fetcher ([`d4bd7a7`](https://github.com/Dicklesworthstone/charmed_rust/commit/d4bd7a7138f3f13e12af18697ab63e824ddc2172))
- Windows SSH e2e + CI OpenSSH support ([`235341b`](https://github.com/Dicklesworthstone/charmed_rust/commit/235341b467d6df4a34fced98bb765afef294c696))

### bubbles (TUI Components)

- Refactored list pagination to use fixed chrome line counting for Go parity ([`ffbc7f7`](https://github.com/Dicklesworthstone/charmed_rust/commit/ffbc7f73ef215732f48d2619185b979e8db80ea2), [`8dc498b`](https://github.com/Dicklesworthstone/charmed_rust/commit/8dc498b9691209410cdf8cb85352552fef2e1b03), [`0483c36`](https://github.com/Dicklesworthstone/charmed_rust/commit/0483c367a80affc9235f5841932fce701fb0e013))
- Used display width for textarea prompt reservation ([`7dce044`](https://github.com/Dicklesworthstone/charmed_rust/commit/7dce0440c38b8de2fcf142aa9569be5ca8165f91))
- Matched Go unicode width via grapheme-aware `visible_width` ([`3bb0f04`](https://github.com/Dicklesworthstone/charmed_rust/commit/3bb0f045e0abd681b37d7952829636caa891a4ca))
- Validated mouse drag support and updated docs ([`5b565e1`](https://github.com/Dicklesworthstone/charmed_rust/commit/5b565e12788bd60a77ef475d1764e37a655d3e89))

### lipgloss (Terminal Styling)

- Hardened `truncate_line_ansi` against incomplete ANSI escapes ([`129eb4c`](https://github.com/Dicklesworthstone/charmed_rust/commit/129eb4c2b3d71f3fbec76efbf7e833672ed22976))
- Hardened CSI final-byte parsing in truncation ([`71caea5`](https://github.com/Dicklesworthstone/charmed_rust/commit/71caea53740156ec293714ba46a17c8d2c6f707f))
- Robust pager argv parsing and ANSI truncate escapes ([`e58c365`](https://github.com/Dicklesworthstone/charmed_rust/commit/e58c365f5e173815adba6b3d1854ff4c362c9988))

### glamour (Markdown Rendering)

- Fixed proportional shrink max-width overshoot ([`dd262ff`](https://github.com/Dicklesworthstone/charmed_rust/commit/dd262ffee708df4630fda2e23c35b629cdbd5f20))
- Saturated `StyleCache` zero capacity instead of panicking ([`e0d174f`](https://github.com/Dicklesworthstone/charmed_rust/commit/e0d174f61cc49c5f2297e9cbf055e8fdeb39396d))
- Removed `highlight_code` json unwrap ([`c89b2b7`](https://github.com/Dicklesworthstone/charmed_rust/commit/c89b2b7c3da8518fdb6c642a40f010e7d3b65b0a))

### huh (Interactive Forms)

- Hardened layout render path against row-part panics ([`c6dbe75`](https://github.com/Dicklesworthstone/charmed_rust/commit/c6dbe75c9cee734ffdc71f46295298e2ddc38282))
- Hardened fallback Enter handling in `shell_action` ([`996919c`](https://github.com/Dicklesworthstone/charmed_rust/commit/996919c827636c6e1eee25f29c1b4c53dc1c6957))

### bubbletea (Elm Architecture Framework)

- Hardened `ThemeContext` lock poisoning recovery ([`a25cb94`](https://github.com/Dicklesworthstone/charmed_rust/commit/a25cb9498dc5c6cc9e391af9050cd03d1285cf11))

### demo_showcase

- Added Services page, implemented export command, fixed `use_color` ([`b5310d9`](https://github.com/Dicklesworthstone/charmed_rust/commit/b5310d978fa6d73664f1044c5df9672138d36069))
- Handled OSC, DCS, and APC string-type escape sequences in `strip_ansi` ([`d02ce0e`](https://github.com/Dicklesworthstone/charmed_rust/commit/d02ce0eb329d906eb57052996cc855d21e857006))
- Fixed command palette ctrl navigation and pager resolution ([`9654169`](https://github.com/Dicklesworthstone/charmed_rust/commit/9654169295923fe8864b325b0a81efcff3257021))
- Hardened `truncate_line_ansi_aware` against incomplete escapes ([`968b9e4`](https://github.com/Dicklesworthstone/charmed_rust/commit/968b9e4baeae8293aed95e94079dbfebac8ac0a2))

### Examples

- Updated example run commands to use correct `charmed-*` packages ([`d47714e`](https://github.com/Dicklesworthstone/charmed_rust/commit/d47714e02f21bc285038cc003df98245356c556a))
- Integrated real bubbles `TextInput` in bubbletea textinput example ([`e01d80c`](https://github.com/Dicklesworthstone/charmed_rust/commit/e01d80ca5123ccb64d30f26d1ed4e1a06f3d1701))
- Integrated real bubbles `Spinner` in bubbletea spinner example ([`a45a759`](https://github.com/Dicklesworthstone/charmed_rust/commit/a45a75975b645cbeccb1ee74b9be9770aaf9ad27))

### Documentation

- Aligned README wish status with FEATURE_PARITY ([`07b61b5`](https://github.com/Dicklesworthstone/charmed_rust/commit/07b61b560b5722eec9a4b3302946473162c0e8df))
- Updated AGENTS.md with latest multi-agent conventions ([`0b91a88`](https://github.com/Dicklesworthstone/charmed_rust/commit/0b91a88e0c135b39da72614c81dc87d04fee3253))

### Dependencies

- Bumped GitHub Actions and Cargo dependencies ([`4344b91`](https://github.com/Dicklesworthstone/charmed_rust/commit/4344b91748019645a5d141e3e4ae83334965691e))

---

## [v0.1.2] — 2026-02-05 (GitHub Release)

[v0.1.2]: https://github.com/Dicklesworthstone/charmed_rust/compare/v0.1.1...v0.1.2

1 commit since v0.1.1 (documentation-only). Tagged at [`de60a63`](https://github.com/Dicklesworthstone/charmed_rust/commit/de60a635b3b632c39a7f837e73b0bae51df4f082). No API changes.

**Published crates (all 0.1.2):** charmed-bubbletea, charmed-bubbletea-macros, charmed-lipgloss, charmed-bubbles, charmed-glamour, charmed-harmonica, charmed-huh, charmed-wish, charmed-glow, charmed-log, charmed-wasm, charmed-demo-showcase.

### Documentation

- Expanded every crate README with TL;DR, install, usage, troubleshooting, limitations, and FAQ ([`de60a63`](https://github.com/Dicklesworthstone/charmed_rust/commit/de60a635b3b632c39a7f837e73b0bae51df4f082))
- Added conformance harness documentation in `tests/conformance/README.md`
- Version bump across the workspace and internal dependency pins to `0.1.2`
- Re-published all crates on crates.io so docs match the new READMEs

---

## [v0.1.1] — 2026-02-05 (Tag only, no GitHub Release)

[v0.1.1]: https://github.com/Dicklesworthstone/charmed_rust/compare/30286ed7b1d2c418d5b04c84bb9d88ecd5e53c39...v0.1.1

The initial public release. 462 commits, spanning 2026-01-17 to 2026-02-05. Tagged at [`3e33362`](https://github.com/Dicklesworthstone/charmed_rust/commit/3e3336271480844b1f27310e82a5169993882a6a). First commit: [`30286ed`](https://github.com/Dicklesworthstone/charmed_rust/commit/30286ed7b1d2c418d5b04c84bb9d88ecd5e53c39).

This release represents the complete port of Charm's Go TUI ecosystem to idiomatic Rust, with 9 library crates, 2 application crates, a WASM target, and a comprehensive conformance test suite.

### Core Framework — bubbletea

- Elm Architecture TUI framework with `Model` trait, `Message` passing, and `Cmd` effect system ([`30286ed`](https://github.com/Dicklesworthstone/charmed_rust/commit/30286ed7b1d2c418d5b04c84bb9d88ecd5e53c39))
- Tokio-based async command executor with `AsyncCmd` and `tick_async` ([`0b8cad9`](https://github.com/Dicklesworthstone/charmed_rust/commit/0b8cad9e54b9177fa76349a9b303c23ced95ff24), [`0faea06`](https://github.com/Dicklesworthstone/charmed_rust/commit/0faea06d9fcf71c5114e18e8727a25797ae1355a))
- Graceful shutdown coordination with `TaskTracker` and `CancellationToken` ([`141fe44`](https://github.com/Dicklesworthstone/charmed_rust/commit/141fe4476db1e7666f839ce0e3991de08a933434), [`49fb1ec`](https://github.com/Dicklesworthstone/charmed_rust/commit/49fb1eccf0ab941578cb320f436e0f2d5aff726b))
- `ProgramHandle` and `start()` for external event injection ([`8ece212`](https://github.com/Dicklesworthstone/charmed_rust/commit/8ece212392aa2011f1c14d668b627b5ca5d16e7e))
- Custom I/O mode for SSH and headless use ([`387cbb3`](https://github.com/Dicklesworthstone/charmed_rust/commit/387cbb36edb645dc07cdca25a1341ac8ce65e43a))
- Batch and sequence message handling in simulator ([`759e514`](https://github.com/Dicklesworthstone/charmed_rust/commit/759e514d9d3a21366dd9db79f68ba2f817dac101))
- Blocking command helpers, thiserror migration, derive macro integration ([`1856c78`](https://github.com/Dicklesworthstone/charmed_rust/commit/1856c78ea1a26b40f86b3cd1056341ab0c638ad5))
- Spinner and textinput examples ([`093f8a8`](https://github.com/Dicklesworthstone/charmed_rust/commit/093f8a8de4d0956f5813a62c2ff1e2cbb292f4b7))
- Improved ANSI sequence prefix detection for partial reads ([`df91368`](https://github.com/Dicklesworthstone/charmed_rust/commit/df91368df8b15b4b624f8e4c83471f37268ec4c5))
- Comprehensive modifier key tests for arrow keys ([`bbebfbc`](https://github.com/Dicklesworthstone/charmed_rust/commit/bbebfbca97349768c03c3848814fe3e3d4771974))

### bubbletea-macros

- `#[derive(Model)]` with `#[state]` attribute for render-triggering fields ([`ad140c3`](https://github.com/Dicklesworthstone/charmed_rust/commit/ad140c36d0b4845861125bea84720bd45a4eae4e))
- Extended `#[state]` support for generics and static lifetimes ([`2771a45`](https://github.com/Dicklesworthstone/charmed_rust/commit/2771a45ad2b25facfb3dff393c55e3fe8de01fb1))
- Comprehensive error messages for derive macro ([`f21d199`](https://github.com/Dicklesworthstone/charmed_rust/commit/f21d199c743efc816035afe659005a25252c8049))

### lipgloss — Terminal Styling

- CSS-like styling with colors, borders, padding, margins, alignment ([`30286ed`](https://github.com/Dicklesworthstone/charmed_rust/commit/30286ed7b1d2c418d5b04c84bb9d88ecd5e53c39))
- Corrected padding, margin, height, `join_vertical`, and `place` for Go conformance ([`9dde7cb`](https://github.com/Dicklesworthstone/charmed_rust/commit/9dde7cb2027a449876cdb3d44b99f5705af50aac))
- `StyleRanges` API and unset methods for style composition ([`6a3f618`](https://github.com/Dicklesworthstone/charmed_rust/commit/6a3f618486337b29d10fe5545a9f122ddde4138f))
- Per-side border color methods ([`df83ce0`](https://github.com/Dicklesworthstone/charmed_rust/commit/df83ce05e8e8576b63fdd105999f767d5c99a176))
- Partial border edges implementation ([`e3de168`](https://github.com/Dicklesworthstone/charmed_rust/commit/e3de16809a2e5c01053d045aad651afaf660f7f8))
- Public `visible_width` with comprehensive ANSI handling ([`78a24bb`](https://github.com/Dicklesworthstone/charmed_rust/commit/78a24bb95fae584ff20dcc3cd79e25b1400cd9c1))
- Output backend abstraction for WASM support ([`6649bde`](https://github.com/Dicklesworthstone/charmed_rust/commit/6649bde38689b07b1a897e56e3cfb517ea327861))
- Eliminated allocations in horizontal/vertical join functions ([`926e775`](https://github.com/Dicklesworthstone/charmed_rust/commit/926e77537579ff217b22a2d2b4713e6eb6585067))
- Optimized layout and style rendering to reduce allocations ([`3c07730`](https://github.com/Dicklesworthstone/charmed_rust/commit/3c07730e42a45f62cec2ee8b684e825c28dc44e5))
- `ColorSlot` and `ThemeRole` enums with slot-based color access ([`d4b05a0`](https://github.com/Dicklesworthstone/charmed_rust/commit/d4b05a09a66ccbb7b65409736dbd97e4a592213a), [`e9b410c`](https://github.com/Dicklesworthstone/charmed_rust/commit/e9b410ccefdfc36af2b67afa8c659f5de0453099))
- Theme integration helpers for Style API ([`cd274fc`](https://github.com/Dicklesworthstone/charmed_rust/commit/cd274fc3940c8b4636f346d1b391098773e4c327))

### bubbles — TUI Components (16 components)

- Model trait implementations for all components: Timer, List, Cursor, TextInput, TextArea, Progress, Paginator, Stopwatch, FilePicker, Table ([`db11cbb`](https://github.com/Dicklesworthstone/charmed_rust/commit/db11cbb62480f5a0ab9995ba738c411dad1539eb)...[`1adbcbb`](https://github.com/Dicklesworthstone/charmed_rust/commit/1adbcbb9e145f788d7cbc2088f002032b4c78eae))
- Mouse support for Table and List components ([`af25d5d`](https://github.com/Dicklesworthstone/charmed_rust/commit/af25d5d43db704a491cbc98a49634e6046bb1964))
- Horizontal scrolling and unicode-aware line width ([`a8d3375`](https://github.com/Dicklesworthstone/charmed_rust/commit/a8d3375d3800db144782aa8eea342f3f3c3e21a3))
- Go-parity duration formatting for stopwatch and timer ([`4c2f65b`](https://github.com/Dicklesworthstone/charmed_rust/commit/4c2f65b75f668cf4c51f0951ab4700f3cb75d6db), [`7f3de0d`](https://github.com/Dicklesworthstone/charmed_rust/commit/7f3de0d13ad51e4d8154b8d6963d3d42ca7324c4))
- Comprehensive bracketed paste tests ([`931ae7e`](https://github.com/Dicklesworthstone/charmed_rust/commit/931ae7e207c928c6ceb3392b1f4b1bfef8a1b576))
- Property-based tests for bubbles components ([`62cd570`](https://github.com/Dicklesworthstone/charmed_rust/commit/62cd570aa241ebc2a734ddb44e826280aa7910a3))
- NaN handling in `Progress::set_percent` and viewport/progress property tests ([`8b29daa`](https://github.com/Dicklesworthstone/charmed_rust/commit/8b29daa70ad3e7d6099a8357d3712176ebaa0566))

### glamour — Markdown Rendering

- Syntect-to-lipgloss theme mapping ([`ebaed10`](https://github.com/Dicklesworthstone/charmed_rust/commit/ebaed10fc4c4d6e5670b09da964d88eef920f925))
- Comprehensive syntax highlighting tests and table parsing module ([`cdce084`](https://github.com/Dicklesworthstone/charmed_rust/commit/cdce0842617ac34efeb893c4ce79b32711d67ff5))
- Table rendering utilities ([`4aafa42`](https://github.com/Dicklesworthstone/charmed_rust/commit/4aafa420d735d4f1f3102380de69f37c673b2b7c))
- Fixed table spacing to match Go implementation ([`eb89f8e`](https://github.com/Dicklesworthstone/charmed_rust/commit/eb89f8eb563132a601157c0b1698d96970d28db9))
- Fixed blockquote multi-paragraph and nested separator depth rendering ([`9c5587c`](https://github.com/Dicklesworthstone/charmed_rust/commit/9c5587cadd6f592bb3aacbd54864814f034e227f), [`c25ad4d`](https://github.com/Dicklesworthstone/charmed_rust/commit/c25ad4d9722e6de9f596fedab9c72533686fc3d2))
- ANSI-aware width calculation in word wrapping and table columns ([`c8bc8d6`](https://github.com/Dicklesworthstone/charmed_rust/commit/c8bc8d65bd2bd26482bb32bdcf8e3fa96f78091c), [`3d87cd0`](https://github.com/Dicklesworthstone/charmed_rust/commit/3d87cd0507dd24057f9aecabc63132e5dcfefbfa))
- LRU cache limit for `StyleCache` ([`cafaf45`](https://github.com/Dicklesworthstone/charmed_rust/commit/cafaf450b5a78ae125974e8be215df67dc2fa531))
- 81/84 (96%) conformance with Go implementation

### harmonica — Spring Physics Animations

- Spring physics, projectile motion, frame timing ([`30286ed`](https://github.com/Dicklesworthstone/charmed_rust/commit/30286ed7b1d2c418d5b04c84bb9d88ecd5e53c39))
- Complete conformance test suite with all 24 tests passing ([`2077d80`](https://github.com/Dicklesworthstone/charmed_rust/commit/2077d80691fdbf1e6b216505d5759df5a969e874))
- Spring physics improvements ([`4cfdfa8`](https://github.com/Dicklesworthstone/charmed_rust/commit/4cfdfa888091e6b3823839771ed590105a3c581c))

### wish — SSH Application Framework

- SSH server implementation with russh integration ([`514d456`](https://github.com/Dicklesworthstone/charmed_rust/commit/514d4566dc145fc735b077a70707604dd1d4ca5b))
- SSH e2e tests ([`9b9313e`](https://github.com/Dicklesworthstone/charmed_rust/commit/9b9313ee16886833c91e036e278c558ecbf96edd))
- Fixed memory leak and output channel type ([`8ac3be7`](https://github.com/Dicklesworthstone/charmed_rust/commit/8ac3be726dfcb19dbc13b998ce63758de5ea8ab2))
- Cleared `keyboard_interactive` state after auth success ([`ade2fd1`](https://github.com/Dicklesworthstone/charmed_rust/commit/ade2fd1e443a444a8fe45400eae49274a42d3098))
- Prevented split escape sequences in SSH handler ([`7bd1207`](https://github.com/Dicklesworthstone/charmed_rust/commit/7bd1207226a023ba978b6589f80554dda08cff9d))
- SSH stability audit documented ([`33155b6`](https://github.com/Dicklesworthstone/charmed_rust/commit/33155b689f9f1ab4247fd17e4fd78f7b759f2917))
- Hardened SSH server security ([`3384e0d`](https://github.com/Dicklesworthstone/charmed_rust/commit/3384e0d3920d88654d63d18309672b21759bed24))

### huh — Interactive Forms

- Catppuccin theme (Mocha variant) ([`7a45e4a`](https://github.com/Dicklesworthstone/charmed_rust/commit/7a45e4ad866c8ea7b449954f0b960a17dce1b580))
- Text editing keybindings for word/char manipulation ([`5860755`](https://github.com/Dicklesworthstone/charmed_rust/commit/5860755093929d87583d37b99792a58973e13503))
- Textarea word transformation operations ([`987b304`](https://github.com/Dicklesworthstone/charmed_rust/commit/987b304c7c323f0178bf0c2d5441516f78c4ae4e))
- `next()` method alias for Go API compatibility ([`45d3566`](https://github.com/Dicklesworthstone/charmed_rust/commit/45d3566cdceb6359a519a411915d46b49732713f))
- Migrated `FormError` to thiserror ([`7e0b47e`](https://github.com/Dicklesworthstone/charmed_rust/commit/7e0b47eab6895989feb3948ca0b6b1e2668714e0))
- Fixed MultiSelect cursor position mismatch with filtering ([`5d4711d`](https://github.com/Dicklesworthstone/charmed_rust/commit/5d4711d247b4c9f6a7efa4ac4c8803ce9759de24))

### charmed_log — Structured Logging

- Text/JSON output with styled output and levels ([`30286ed`](https://github.com/Dicklesworthstone/charmed_rust/commit/30286ed7b1d2c418d5b04c84bb9d88ecd5e53c39))
- Eliminated double-lock race window in `log()` ([`6d9d1e0`](https://github.com/Dicklesworthstone/charmed_rust/commit/6d9d1e0ce14dd6c22120fc6a423fbf0d44a1465a))
- Handled `RwLock` poisoning gracefully ([`c0c5d94`](https://github.com/Dicklesworthstone/charmed_rust/commit/c0c5d94d0bf6afc7eeaccf99021b4a9c0eb8b47c))

### glow — Markdown Reader CLI

- TUI pager mode with viewport scrolling ([`7976c1c`](https://github.com/Dicklesworthstone/charmed_rust/commit/7976c1cec01f74221313f4f3711134543aa355c6))
- Keyboard navigation and search ([`2b5a4df`](https://github.com/Dicklesworthstone/charmed_rust/commit/2b5a4df34494166f6452ed428fff6b891f8ded66))
- Browser and github modules with proper exports ([`19a17b8`](https://github.com/Dicklesworthstone/charmed_rust/commit/19a17b8dd05b769d0ffe112ae51d12f17b8c69ed))
- Improved file browser symlink handling and custom extensions ([`bf34eb8`](https://github.com/Dicklesworthstone/charmed_rust/commit/bf34eb8a23d4aa1f4134e730dc652c700d21f6c5))
- Migrated error types to thiserror ([`38387b9`](https://github.com/Dicklesworthstone/charmed_rust/commit/38387b98dfad20a0b1da29eb92a1231718c2220d))
- Conformance test infrastructure ([`a4f1e37`](https://github.com/Dicklesworthstone/charmed_rust/commit/a4f1e37aa6279294e3d24540532fb55983e1087a))

### charmed_wasm — WebAssembly Target

- WASM support with output backend abstraction ([`f283ec3`](https://github.com/Dicklesworthstone/charmed_rust/commit/f283ec3e6e47256c98504e2bb2114672f57ee796))
- `HtmlBackend` for browser rendering ([`6737532`](https://github.com/Dicklesworthstone/charmed_rust/commit/6737532ae001094e4afb6f4139c537d59a531573))
- WASM CI workflow ([`1b3651d`](https://github.com/Dicklesworthstone/charmed_rust/commit/1b3651d5b9dc0b5990c2c036e5194498e837a58f))

### demo_showcase — Flagship Demo Application

- Multi-page app: Dashboard, Services, Jobs, Logs, Docs, Files, Wizard, Settings ([`b97509c`](https://github.com/Dicklesworthstone/charmed_rust/commit/b97509c9edfeee5f27c691c3f781de3afcb84125)...[`c81bd90`](https://github.com/Dicklesworthstone/charmed_rust/commit/c81bd90e61f0fefb08e0375d8ec95de77ff14edd))
- Live theme switching with all built-in presets ([`38a8f33`](https://github.com/Dicklesworthstone/charmed_rust/commit/38a8f33ed3b84a253afb8b28c5a8f24af4529baa))
- Spring physics animations subsystem ([`4a708b9`](https://github.com/Dicklesworthstone/charmed_rust/commit/4a708b902ee8690ee39a6e761ae7bbd54f242cad))
- SSH server mode ([`9db11d9`](https://github.com/Dicklesworthstone/charmed_rust/commit/9db11d92d75325224f8d9c66a95e13f12348670c))
- Command palette, notes modal, and guided tour ([`a3fb793`](https://github.com/Dicklesworthstone/charmed_rust/commit/a3fb79398f8ac6bc8c4150e88440671a8ec5c005))
- Wizard multi-step deployment workflow with error states and recovery ([`059f49f`](https://github.com/Dicklesworthstone/charmed_rust/commit/059f49f157e5bf2aee6e21c9584502953704508b), [`046b780`](https://github.com/Dicklesworthstone/charmed_rust/commit/046b780a96441c62e3bb1049236d7585b7c52217))
- DocsPage with glamour markdown rendering and split-view navigation ([`291be2d`](https://github.com/Dicklesworthstone/charmed_rust/commit/291be2d26f027e59027e731c624185459f039927), [`36d2a8c`](https://github.com/Dicklesworthstone/charmed_rust/commit/36d2a8cfa9e2c387e625dab4cf62fbfa3d1847e0))
- CLI contract with clap, runtime config validation ([`d4eb9a1`](https://github.com/Dicklesworthstone/charmed_rust/commit/d4eb9a106fa25818637d4b323d1a9504c9125ead), [`eaafc86`](https://github.com/Dicklesworthstone/charmed_rust/commit/eaafc8642e474c1f3e7bd03448b2f50a7883c9f5))
- E2E headless runner and test suites ([`9802d9d`](https://github.com/Dicklesworthstone/charmed_rust/commit/9802d9d5160caf3ae521ca59e024c4105c064de8))
- Custom theme loading from JSON files ([`1f5609c`](https://github.com/Dicklesworthstone/charmed_rust/commit/1f5609cde069c17f25b13acd3c1f2d2da33226fe))
- Responsive window resize handling ([`8c23682`](https://github.com/Dicklesworthstone/charmed_rust/commit/8c236823934e9cc2d0344a8eb9552b4675426ca6))
- Max-width cap for ultra-wide terminals ([`41dfc80`](https://github.com/Dicklesworthstone/charmed_rust/commit/41dfc8052b90c8041c4dc25244dd5b7ff78dcce1))

### Conformance Testing Infrastructure

- Generated 397 Go reference fixtures for 6 crates ([`ac7859d`](https://github.com/Dicklesworthstone/charmed_rust/commit/ac7859d4321b187a566555f15fd6d5d5b7e16412))
- Completed Go reference capture for all 8 crates ([`c7f68eb`](https://github.com/Dicklesworthstone/charmed_rust/commit/c7f68eb9e4f9b6d871a911b313cd37f514f2b569))
- Test suites for: harmonica, lipgloss, bubbles (list, table, filepicker, cursor, keybinding), glamour (style, theme, syntax highlighting), huh, wish, mouse parsing (X10, SGR), cross-crate integration, E2E ([`2077d80`](https://github.com/Dicklesworthstone/charmed_rust/commit/2077d80691fdbf1e6b216505d5759df5a969e874)...[`db34604`](https://github.com/Dicklesworthstone/charmed_rust/commit/db34604cee191793e9a195b430bc46bde4c40386))
- Report generation and CI integration ([`94dccba`](https://github.com/Dicklesworthstone/charmed_rust/commit/94dccba9597926306b809f60cfd8e59a0fa1876c))
- Overhauled test harness for parallel execution and better reporting ([`3e6272b`](https://github.com/Dicklesworthstone/charmed_rust/commit/3e6272bb5609156a0832b2182c323c47734c6d78))
- Go comparison benchmarks ([`d788dba`](https://github.com/Dicklesworthstone/charmed_rust/commit/d788dba07c05ba722f2c5aacaa04657253fb7a59))

### Examples

- Comprehensive examples workspace with basic, intermediate, and advanced demos ([`93dd0d7`](https://github.com/Dicklesworthstone/charmed_rust/commit/93dd0d753f63c2c727b10b1e62811a93dfc20a66), [`14ae0f5`](https://github.com/Dicklesworthstone/charmed_rust/commit/14ae0f5e650bc0bb01dd3f8194ef259be0561dea))
- `TestTerminal` framework for end-to-end testing ([`7468bfb`](https://github.com/Dicklesworthstone/charmed_rust/commit/7468bfb62f71e5708b33028ed11dbb563ceeb51a))
- Proptest property-based testing across crates ([`97ce4e3`](https://github.com/Dicklesworthstone/charmed_rust/commit/97ce4e3c184ce56bbd4dcfd21c6ff7e981bee5fa))
- Theme examples: switcher, preview, themed counter

### CI/CD

- GitHub Actions CI/CD workflows ([`2696e6e`](https://github.com/Dicklesworthstone/charmed_rust/commit/2696e6eaa26b9fbf89af5cecc1a91e4375143f99))
- Dependabot for actions and cargo ([`ef414f9`](https://github.com/Dicklesworthstone/charmed_rust/commit/ef414f91ec3e2d22b205adff358d159cfdd57087))
- WASM CI workflow ([`1b3651d`](https://github.com/Dicklesworthstone/charmed_rust/commit/1b3651d5b9dc0b5990c2c036e5194498e837a58f))
- Hardened workflows and cache keys ([`61ee0d6`](https://github.com/Dicklesworthstone/charmed_rust/commit/61ee0d6090f504007c9158c3b528856ff841e5b3))

### Documentation

- Full project README with architecture diagrams, API reference, Quick Start, FAQ ([`d2d89f0`](https://github.com/Dicklesworthstone/charmed_rust/commit/d2d89f0923b30500b223e987eda28e3b8d7d75ca))
- CHARM_SPEC.md specification document ([`c6cad1f`](https://github.com/Dicklesworthstone/charmed_rust/commit/c6cad1f08f2b7c34d8a24a0819e80d5fd9a6f094))
- FEATURE_PARITY.md with conformance results ([`aa3c3fc`](https://github.com/Dicklesworthstone/charmed_rust/commit/aa3c3fc5aac7f351ce6c404ffdadc7012c9daa49))
- Async migration guide ([`a22aa44`](https://github.com/Dicklesworthstone/charmed_rust/commit/a22aa44f1d5e552e29c969ba4488e777dea6f398))
- Error handling guide and pattern documentation ([`cf3d8ee`](https://github.com/Dicklesworthstone/charmed_rust/commit/cf3d8ee116181a2496621aca7e466162e9474711))
- MIT License ([`83fea61`](https://github.com/Dicklesworthstone/charmed_rust/commit/83fea613c0d32ef64e0a15573cfdef8c96b244b7))
- Full Go API parity achieved across charmed_log, glamour, huh, and glow ([`44b228c`](https://github.com/Dicklesworthstone/charmed_rust/commit/44b228c1ce711d4cb698bf5adaa0581083101e47))

---

*Generated from git history. Excludes beads sync, chore, and merge-bookkeeping commits for readability.*
