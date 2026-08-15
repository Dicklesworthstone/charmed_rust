# Changelog

All notable changes to **charmed_rust** are documented in this file.

Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) with live commit links.
Versioning: [Semantic Versioning](https://semver.org/spec/v2.0.0.html) across all workspace crates.

Repository: <https://github.com/Dicklesworthstone/charmed_rust>

---

## [Unreleased] — after v0.2.0

[Unreleased]: https://github.com/Dicklesworthstone/charmed_rust/compare/v0.2.0...HEAD

### Rendering

- Reduced full-screen flicker by rendering only changed rows ([`17d9c9f`](https://github.com/Dicklesworthstone/charmed_rust/commit/17d9c9fcaf4f94a6874ba687ed01ab65e61a902d))

### bubbletea API

- Added non-consuming `try_downcast` method to `Message` for borrow-friendly downcasting ([`0ced628`](https://github.com/Dicklesworthstone/charmed_rust/commit/0ced6280e91c2cba69fdd4e397004c40c8bceced))

### Code Quality

- Replaced crate-wide clippy allows with per-module lint suppression across all crates ([`cd98380`](https://github.com/Dicklesworthstone/charmed_rust/commit/cd98380971a8a60f451d707f4c1791ca0294e347), [`1ed50fa`](https://github.com/Dicklesworthstone/charmed_rust/commit/1ed50faf710a4a2a0778e62c5a5d17504f0c333b))

### Licensing & Branding

- Updated license to MIT with OpenAI/Anthropic Rider ([`e1e9403`](https://github.com/Dicklesworthstone/charmed_rust/commit/e1e940303c7ab6e1cbe14d67de65bcf9af19d4f9), [`99cf951`](https://github.com/Dicklesworthstone/charmed_rust/commit/99cf951ca4a91f5113c2bf23f18185e3e9676515))
- Added GitHub social preview image ([`5d604d4`](https://github.com/Dicklesworthstone/charmed_rust/commit/5d604d4ee8d360ca83e35a50f504b824e7898ac4))

### Dependencies

- Upgraded dependencies, refreshed benchmarks and test snapshots ([`0e51a1c`](https://github.com/Dicklesworthstone/charmed_rust/commit/0e51a1c857d6ec8b81dd5b3ecea77281545bef30), [`c283ad0`](https://github.com/Dicklesworthstone/charmed_rust/commit/c283ad0d73904682daeb3da3954bedc4f1f3330d))
- **Security:** bumped `lru` 0.12 → 0.18 in charmed-glamour and the workspace, resolving RUSTSEC-2026-0002 and RUSTSEC-2026-0253 (both fixed in `lru` >= 0.17) ([`9af126b`](https://github.com/Dicklesworthstone/charmed_rust/commit/9af126bbaae0b9ef41c9f48d6e29e690f0af1c5d)). The fix is unpublished: crates.io still serves charmed-glamour 0.2.0 with `lru` 0.12.5, so downstream consumers (e.g. pi_agent_rust) keep failing `cargo audit` until a new charmed-* release is published.

### Housekeeping

- Removed stale `a.out` compiler output ([`8a18ca8`](https://github.com/Dicklesworthstone/charmed_rust/commit/8a18ca87e3046c30ab50c1821c28a1b1a7c907b7))

---

## [v0.2.0] — 2026-02-15 (GitHub Release)

[v0.2.0]: https://github.com/Dicklesworthstone/charmed_rust/compare/v0.1.2...v0.2.0

A major hardening and conformance release with 91 substantive commits since v0.1.2. Tagged at [`eed5f91`](https://github.com/Dicklesworthstone/charmed_rust/commit/eed5f91a9e661d9cc1490bf12b0ac67fe436b569).

**Published crates (all 0.2.0):** charmed-bubbletea, charmed-bubbletea-macros, charmed-lipgloss, charmed-bubbles, charmed-glamour, charmed-harmonica, charmed-huh, charmed-wish, charmed-glow, charmed-log, charmed-wasm, charmed-demo-showcase.

### Conformance Testing

- All crates now have real conformance tests -- skipped tests now fail the suite ([`c9cb86b`](https://github.com/Dicklesworthstone/charmed_rust/commit/c9cb86ba21d324f66c6e56ffd9883179698f15ab), [`f861235`](https://github.com/Dicklesworthstone/charmed_rust/commit/f861235dd212e72c87ea062563f77238931d41c9))
- Unknown conformance fixtures must fail, not skip ([`362a131`](https://github.com/Dicklesworthstone/charmed_rust/commit/362a1313198e06cdffb87a905011a0688a7f212a))
- Made glow conformance real and included glow + wish in the suite ([`9250abf`](https://github.com/Dicklesworthstone/charmed_rust/commit/9250abf826ff72528bd6b8b0dc85e9a708a0fa6f), [`3bfbd70`](https://github.com/Dicklesworthstone/charmed_rust/commit/3bfbd70eacd45dafa156b4a40a9bca261d9495f1))
- Added textarea conformance with view parity fixes ([`5b6c43c`](https://github.com/Dicklesworthstone/charmed_rust/commit/5b6c43cbf86b4dc8ceecc9b8380a2aa098fa040b))
- Made textinput echo_mode conformance parsing strict ([`9e7c5d8`](https://github.com/Dicklesworthstone/charmed_rust/commit/9e7c5d84ab925398a33553e211411c500408c5fe))
- Used renamed crate packages and real Catppuccin theme in conformance ([`8287031`](https://github.com/Dicklesworthstone/charmed_rust/commit/82870316861c2aed4f38877045cfd87e440c45d3))
- Removed bubbles conformance UBS ignore and false-positive triggers ([`44dfb59`](https://github.com/Dicklesworthstone/charmed_rust/commit/44dfb594deb94a20587e10d7dfba7cd6adcc734a))
- Simplified UBS ignore globs and textinput echo mode matching ([`036b9ee`](https://github.com/Dicklesworthstone/charmed_rust/commit/036b9ee55f5d730909fcf3ab52442a7d6b89dc6d))

### Unicode & Go Parity

- Matched Go unicode width via grapheme-aware `visible_width` ([`3bb0f04`](https://github.com/Dicklesworthstone/charmed_rust/commit/3bb0f045e0abd681b37d7952829636caa891a4ca))
- Used display width for textarea prompt reservation ([`7dce044`](https://github.com/Dicklesworthstone/charmed_rust/commit/7dce0440c38b8de2fcf142aa9569be5ca8165f91))
- Fixed bubbles list pagination tests for Go parity ([`8dc498b`](https://github.com/Dicklesworthstone/charmed_rust/commit/8dc498b9691209410cdf8cb85352552fef2e1b03))

### ANSI Escape Handling

- Hardened `truncate_line_ansi` against incomplete ANSI escapes in lipgloss ([`129eb4c`](https://github.com/Dicklesworthstone/charmed_rust/commit/129eb4c2b3d71f3fbec76efbf7e833672ed22976))
- Hardened CSI final-byte parsing in `truncate_line_ansi` ([`71caea5`](https://github.com/Dicklesworthstone/charmed_rust/commit/71caea53740156ec293714ba46a17c8d2c6f707f))
- Handled OSC, DCS, and APC string-type escape sequences in `strip_ansi` ([`d02ce0e`](https://github.com/Dicklesworthstone/charmed_rust/commit/d02ce0eb329d906eb57052996cc855d21e857006))
- Hardened `truncate_line_ansi_aware` against incomplete escapes in demo_showcase ([`968b9e4`](https://github.com/Dicklesworthstone/charmed_rust/commit/968b9e4baeae8293aed95e94079dbfebac8ac0a2))
- Robust pager argv parsing and ANSI truncate escapes ([`e58c365`](https://github.com/Dicklesworthstone/charmed_rust/commit/e58c365f5e173815adba6b3d1854ff4c362c9988))

### wish SSH Framework

- Added shell-aware exec command parser with quote and escape handling ([`d54cdfc`](https://github.com/Dicklesworthstone/charmed_rust/commit/d54cdfc5ea5d571ab9773d64dd4f79393dfc8f71))
- Added typed max-session error for wish session manager ([`32d9bf5`](https://github.com/Dicklesworthstone/charmed_rust/commit/32d9bf57a62cc62023779e8dc21a8c08e7534ec0))
- Hardened SSH e2e probes and credentials ([`8eb4280`](https://github.com/Dicklesworthstone/charmed_rust/commit/8eb4280ae8c8466afeeb36f9f57f34f597a1bb3e))
- Removed unwrap/expect from ssh_e2e harness ([`b648181`](https://github.com/Dicklesworthstone/charmed_rust/commit/b64818107d3f3129948ee9d5cb99114b122a1157))
- Removed assert-false fallback in ssh_e2e ([`6ed9a04`](https://github.com/Dicklesworthstone/charmed_rust/commit/6ed9a04144ebe0e9a2821439354367da0fe70356))
- Hardened wish tea middleware join-error path ([`76ae48e`](https://github.com/Dicklesworthstone/charmed_rust/commit/76ae48e4b0265ff9987b5db09ce6a8e9256338f4))
- Added `listen_with_listener` for race-free test server startup ([`6c9a7c8`](https://github.com/Dicklesworthstone/charmed_rust/commit/6c9a7c8cc86b5a138781fe6112b13ca7bbeffe71))
- Windows SSH e2e + CI OpenSSH support ([`235341b`](https://github.com/Dicklesworthstone/charmed_rust/commit/235341b467d6df4a34fced98bb765afef294c696))
- Hardened authorized_keys base64 padding validation ([`c347433`](https://github.com/Dicklesworthstone/charmed_rust/commit/c3474336477664978694cb9a2548b9bb36b8b1a5))
- Enforced strict base64 decode in GitHub fetcher ([`d4bd7a7`](https://github.com/Dicklesworthstone/charmed_rust/commit/d4bd7a7138f3f13e12af18697ab63e824ddc2172))

### bubbles Components

- Refactored list pagination to use fixed chrome line counting for Go parity ([`ffbc7f7`](https://github.com/Dicklesworthstone/charmed_rust/commit/ffbc7f73ef215732f48d2619185b979e8db80ea2), [`0483c36`](https://github.com/Dicklesworthstone/charmed_rust/commit/0483c367a80affc9235f5841932fce701fb0e013))
- Dynamically compute list pagination based on visible chrome ([`23d5f40`](https://github.com/Dicklesworthstone/charmed_rust/commit/23d5f40f844ea43eb6145906d85e7edf7233d5b2))

### glamour Markdown

- Fixed proportional shrink max-width overshoot ([`dd262ff`](https://github.com/Dicklesworthstone/charmed_rust/commit/dd262ffee708df4630fda2e23c35b629cdbd5f20))
- Saturated `StyleCache` zero capacity instead of panicking ([`e0d174f`](https://github.com/Dicklesworthstone/charmed_rust/commit/e0d174f61cc49c5f2297e9cbf055e8fdeb39396d))
- Removed `highlight_code` JSON unwrap ([`c89b2b7`](https://github.com/Dicklesworthstone/charmed_rust/commit/c89b2b7c3da8518fdb6c642a40f010e7d3b65b0a))

### huh Interactive Forms

- Hardened layout render path against row-part panics ([`c6dbe75`](https://github.com/Dicklesworthstone/charmed_rust/commit/c6dbe75c9cee734ffdc71f46295298e2ddc38282))

### lipgloss Styling

- Hardened ThemeContext lock poisoning recovery ([`a25cb94`](https://github.com/Dicklesworthstone/charmed_rust/commit/a25cb9498dc5c6cc9e391af9050cd03d1285cf11))
- Hardened fallback Enter handling in shell_action ([`996919c`](https://github.com/Dicklesworthstone/charmed_rust/commit/996919c827636c6e1eee25f29c1b4c53dc1c6957))

### demo_showcase

- Added Services page, implemented export command, fixed `use_color` ([`b5310d9`](https://github.com/Dicklesworthstone/charmed_rust/commit/b5310d978fa6d73664f1044c5df9672138d36069))
- Fixed command palette ctrl navigation and pager resolution ([`9654169`](https://github.com/Dicklesworthstone/charmed_rust/commit/9654169295923fe8864b325b0a81efcff3257021))

### Examples

- Integrated real bubbles TextInput and Spinner into bubbletea examples ([`e01d80c`](https://github.com/Dicklesworthstone/charmed_rust/commit/e01d80ca5123ccb64d30f26d1ed4e1a06f3d1701), [`a45a759`](https://github.com/Dicklesworthstone/charmed_rust/commit/a45a75975b645cbeccb1ee74b9be9770aaf9ad27))
- Fixed example run commands to use `charmed-*` packages ([`d47714e`](https://github.com/Dicklesworthstone/charmed_rust/commit/d47714e02f21bc285038cc003df98245356c556a))
- Validated mouse drag support and updated docs ([`5b565e1`](https://github.com/Dicklesworthstone/charmed_rust/commit/5b565e12788bd60a77ef475d1764e37a655d3e89))
- Aligned README wish status with FEATURE_PARITY ([`07b61b5`](https://github.com/Dicklesworthstone/charmed_rust/commit/07b61b560b5722eec9a4b3302946473162c0e8df))

### Dependencies & CI

- Bumped GitHub Actions and Cargo dependencies ([`4344b91`](https://github.com/Dicklesworthstone/charmed_rust/commit/4344b91748019645a5d141e3e4ae83334965691e))
- Updated UI test expected stderr line numbers for bubbletea-macros ([`880b0d8`](https://github.com/Dicklesworthstone/charmed_rust/commit/880b0d8a4df102613d85b886c0cbdd60eb953eb2))

---

## [v0.1.2] — 2026-02-05 (GitHub Release)

[v0.1.2]: https://github.com/Dicklesworthstone/charmed_rust/compare/v0.1.1...v0.1.2

A documentation-focused release with no API changes. Tagged at [`de60a63`](https://github.com/Dicklesworthstone/charmed_rust/commit/de60a635b3b632c39a7f837e73b0bae51df4f082).

**Published crates (all 0.1.2):** charmed-bubbletea, charmed-bubbletea-macros, charmed-lipgloss, charmed-bubbles, charmed-glamour, charmed-harmonica, charmed-huh, charmed-wish, charmed-glow, charmed-log, charmed-wasm, charmed-demo-showcase.

### Documentation

- Expanded every crate README with TL;DR, installation, usage, troubleshooting, limitations, and FAQ sections ([`de60a63`](https://github.com/Dicklesworthstone/charmed_rust/commit/de60a635b3b632c39a7f837e73b0bae51df4f082))
- Added conformance harness documentation in `tests/conformance/README.md`
- Re-published all crates on crates.io so docs match the new READMEs
- Version bump across the workspace and internal dependency pins to 0.1.2

---

## [v0.1.1] — 2026-02-05 (Tag only, no GitHub Release)

[v0.1.1]: https://github.com/Dicklesworthstone/charmed_rust/compare/30286ed...v0.1.1

The initial published release, representing the full build-out from first commit to crates.io readiness. 460 commits from project inception on 2026-01-17 to tag on 2026-02-05. Tagged at [`3e33362`](https://github.com/Dicklesworthstone/charmed_rust/commit/3e3336271480844b1f27310e82a5169993882a6a).

### Core Framework (bubbletea)

- Elm Architecture TUI framework: `Model` trait with `init`, `update`, `view` lifecycle ([`30286ed`](https://github.com/Dicklesworthstone/charmed_rust/commit/30286ed7b1d2c418d5b04c84bb9d88ecd5e53c39))
- Message passing with `Any`-based downcasting and typed `KeyMsg`, `MouseMsg`, `WindowSizeMsg` ([`292c99d`](https://github.com/Dicklesworthstone/charmed_rust/commit/292c99de918b4c24f53e7ec0e2a785646a84c90f))
- Command system: `Cmd::quit()`, `Cmd::none()`, `Cmd::batch()`, blocking command helpers ([`1856c78`](https://github.com/Dicklesworthstone/charmed_rust/commit/1856c78ea1a26b40f86b3cd1056341ab0c638ad5), [`8ba6aca`](https://github.com/Dicklesworthstone/charmed_rust/commit/8ba6acab661ca9c3ae8fb2940455779e7bd4635b))
- Tokio-based async command executor with `AsyncCmd` and `run_async` ([`0b8cad9`](https://github.com/Dicklesworthstone/charmed_rust/commit/0b8cad9e54b9177fa76349a9b303c23ced95ff24))
- Graceful shutdown coordination with thread tracking and join-on-exit ([`141fe44`](https://github.com/Dicklesworthstone/charmed_rust/commit/141fe4476db1e7666f839ce0e3991de08a933434), [`cb18435`](https://github.com/Dicklesworthstone/charmed_rust/commit/cb1843505f512ff3cdf923d3513b41589c2a3e9f), [`9555217`](https://github.com/Dicklesworthstone/charmed_rust/commit/9555217c7f9749acd6a669886a62c44be42dcb89))
- Graceful shutdown for external forwarder thread ([`aea92e3`](https://github.com/Dicklesworthstone/charmed_rust/commit/aea92e3669c907fd44c5ca94f21f0c0bf03f3739))
- `ProgramHandle` and `start()` for external event injection ([`8ece212`](https://github.com/Dicklesworthstone/charmed_rust/commit/8ece212392aa2011f1c14d668b627b5ca5d16e7e))
- Custom I/O mode for SSH and headless use ([`387cbb3`](https://github.com/Dicklesworthstone/charmed_rust/commit/387cbb36edb645dc07cdca25a1341ac8ce65e43a))
- Batch and sequence message handling in simulator ([`759e514`](https://github.com/Dicklesworthstone/charmed_rust/commit/759e514d9d3a21366dd9db79f68ba2f817dac101))
- TaskTracker and CancellationToken integration tests ([`49fb1ec`](https://github.com/Dicklesworthstone/charmed_rust/commit/49fb1eccf0ab941578cb320f436e0f2d5aff726b))
- Improved quit handling and thread handle naming ([`9d1b2ab`](https://github.com/Dicklesworthstone/charmed_rust/commit/9d1b2abd8c4047240b196bb0278198a5e344699e))
- Improved ANSI sequence prefix detection for partial reads ([`df91368`](https://github.com/Dicklesworthstone/charmed_rust/commit/df91368df8b15b4b624f8e4c83471f37268ec4c5))
- Simplified event polling with let-else chains ([`6280d99`](https://github.com/Dicklesworthstone/charmed_rust/commit/6280d9919a1ed4e6cd388a9d4b960c04affe54dc))
- `ShiftEnter`, `CtrlEnter`, and `CtrlShiftEnter` key types ([`5cd256f`](https://github.com/Dicklesworthstone/charmed_rust/commit/5cd256f9260c89427f6081417e0febd86dabea6d))
- Spinner and textinput examples ([`093f8a8`](https://github.com/Dicklesworthstone/charmed_rust/commit/093f8a8de4d0956f5813a62c2ff1e2cbb292f4b7))
- Comprehensive unit tests for core API ([`a035d38`](https://github.com/Dicklesworthstone/charmed_rust/commit/a035d384c3eb865959af7652063b77f71eb5016d), [`4001957`](https://github.com/Dicklesworthstone/charmed_rust/commit/4001957ca003031945780914d277c3d4f982b2bd))

### bubbletea-macros

- `#[derive(Model)]` derive macro with comprehensive error messages ([`f21d199`](https://github.com/Dicklesworthstone/charmed_rust/commit/f21d199c743efc816035afe659005a25252c8049))
- `#[state]` attribute for render-triggering fields with generics and static lifetime support ([`ad140c3`](https://github.com/Dicklesworthstone/charmed_rust/commit/ad140c36d0b4845861125bea84720bd45a4eae4e), [`2771a45`](https://github.com/Dicklesworthstone/charmed_rust/commit/2771a45ad2b25facfb3dff393c55e3fe8de01fb1))
- Comprehensive unit tests for macro code generation ([`3bead84`](https://github.com/Dicklesworthstone/charmed_rust/commit/3bead8476ff9a06c3cfaa3b75c539404c8e1c6b1))
- Fixed publish stubs ([`6ecd871`](https://github.com/Dicklesworthstone/charmed_rust/commit/6ecd871bd1c3b8b6be79f9187e878f7532153f3b))

### lipgloss Terminal Styling

- CSS-like styling: colors, borders, padding, margins, alignment ([`30286ed`](https://github.com/Dicklesworthstone/charmed_rust/commit/30286ed7b1d2c418d5b04c84bb9d88ecd5e53c39))
- Corrected padding, margin, height, `join_vertical`, and `place` for Go conformance ([`9dde7cb`](https://github.com/Dicklesworthstone/charmed_rust/commit/9dde7cb2027a449876cdb3d44b99f5705af50aac))
- Theming system: `ThemePreset` (Dark, Light, Dracula, Nord, Catppuccin), `ThemedStyle`, `ColorSlot`, `ThemeRole` ([`cd274fc`](https://github.com/Dicklesworthstone/charmed_rust/commit/cd274fc3940c8b4636f346d1b391098773e4c327), [`d4b05a0`](https://github.com/Dicklesworthstone/charmed_rust/commit/d4b05a09a66ccbb7b65409736dbd97e4a592213a), [`e9b410c`](https://github.com/Dicklesworthstone/charmed_rust/commit/e9b410ccefdfc36af2b67afa8c659f5de0453099))
- Custom theme loading from JSON/TOML/YAML files ([`1f5609c`](https://github.com/Dicklesworthstone/charmed_rust/commit/1f5609cde069c17f25b13acd3c1f2d2da33226fe))
- Per-side border color methods ([`df83ce0`](https://github.com/Dicklesworthstone/charmed_rust/commit/df83ce05e8e8576b63fdd105999f767d5c99a176))
- Partial border edges support ([`e3de168`](https://github.com/Dicklesworthstone/charmed_rust/commit/e3de16809a2e5c01053d045aad651afaf660f7f8))
- `StyleRanges` API and unset methods for style composition ([`6a3f618`](https://github.com/Dicklesworthstone/charmed_rust/commit/6a3f618486337b29d10fe5545a9f122ddde4138f))
- `visible_width` made public with comprehensive ANSI handling ([`78a24bb`](https://github.com/Dicklesworthstone/charmed_rust/commit/78a24bb95fae584ff20dcc3cd79e25b1400cd9c1))
- Output backend abstraction for WASM support ([`6649bde`](https://github.com/Dicklesworthstone/charmed_rust/commit/6649bde38689b07b1a897e56e3cfb517ea327861))
- Eliminated allocations in horizontal/vertical join functions ([`926e775`](https://github.com/Dicklesworthstone/charmed_rust/commit/926e77537579ff217b22a2d2b4713e6eb6585067))
- Optimized layout and style rendering to reduce allocations ([`3c07730`](https://github.com/Dicklesworthstone/charmed_rust/commit/3c07730e42a45f62cec2ee8b684e825c28dc44e5))
- Used `round()` for center alignment to match Go behavior ([`c7a6791`](https://github.com/Dicklesworthstone/charmed_rust/commit/c7a67915226a747812f9eca02dd05bef5fb469aa))
- Complete ANSI escape sequence handling and Go-compatible alignment ([`c4a8b48`](https://github.com/Dicklesworthstone/charmed_rust/commit/c4a8b4806168fb204eb9521522caa938871cb6cb))
- Comprehensive unit tests for styling ([`34f6871`](https://github.com/Dicklesworthstone/charmed_rust/commit/34f6871d8c54fcac63b8006e08e710f2ae73d18c))

### bubbles TUI Components (16 Components)

- Model trait implementations for all components: Timer, List, Cursor, TextInput, TextArea, Progress, Paginator, Stopwatch, FilePicker, Table ([`db11cbb`](https://github.com/Dicklesworthstone/charmed_rust/commit/db11cbb62480f5a0ab9995ba738c411dad1539eb), [`0f6ba2c`](https://github.com/Dicklesworthstone/charmed_rust/commit/0f6ba2c9d4268fcb47854d4abc79f151c8c5a4cd), [`9d8eb86`](https://github.com/Dicklesworthstone/charmed_rust/commit/9d8eb869cd79f571084abe279f04ad13ceea35a2), [`6bb40d3`](https://github.com/Dicklesworthstone/charmed_rust/commit/6bb40d3626e038caed177bf463bb243518df6956))
- Model::update tests for all components ([`0e8305c`](https://github.com/Dicklesworthstone/charmed_rust/commit/0e8305cc8f833d37e347a2e64ab37971eb3d240f), [`5ec4252`](https://github.com/Dicklesworthstone/charmed_rust/commit/5ec4252b071c2f8aa15829ae650bfc779f3cc167), [`1adbcbb`](https://github.com/Dicklesworthstone/charmed_rust/commit/1adbcbb9e145f788d7cbc2088f002032b4c78eae))
- Mouse support for Table and List components ([`af25d5d`](https://github.com/Dicklesworthstone/charmed_rust/commit/af25d5d43db704a491cbc98a49634e6046bb1964))
- Comprehensive mouse support test suite for Table ([`a3344d0`](https://github.com/Dicklesworthstone/charmed_rust/commit/a3344d0767004230f6dbdf21f0bd054af3c73fdc))
- Horizontal scrolling and unicode-aware line width ([`a8d3375`](https://github.com/Dicklesworthstone/charmed_rust/commit/a8d3375d3800db144782aa8eea342f3f3c3e21a3))
- Go-parity duration formatting for stopwatch and timer ([`4c2f65b`](https://github.com/Dicklesworthstone/charmed_rust/commit/4c2f65b75f668cf4c51f0951ab4700f3cb75d6db), [`7f3de0d`](https://github.com/Dicklesworthstone/charmed_rust/commit/7f3de0d13ad51e4d8154b8d6963d3d42ca7324c4))
- Help parity audit tests ([`1bde0cf`](https://github.com/Dicklesworthstone/charmed_rust/commit/1bde0cf4a5dfe63a40ffa8f5f9442e2d63244c87))
- TextArea cursor position accessors and byte offset methods ([`924db1f`](https://github.com/Dicklesworthstone/charmed_rust/commit/924db1fea10bf0643d9236b278fb56d789ea8ff2))
- Fixed NaN handling in `Progress::set_percent` with property tests ([`8b29daa`](https://github.com/Dicklesworthstone/charmed_rust/commit/8b29daa70ad3e7d6099a8357d3712176ebaa0566))
- Used absolute velocity in progress animation check ([`8613ecb`](https://github.com/Dicklesworthstone/charmed_rust/commit/8613ecb95439ff284ce2e91369916d5afbc0cff7))
- Fixed filepicker symlink mode and unicode input ([`b8faa56`](https://github.com/Dicklesworthstone/charmed_rust/commit/b8faa561b1b79b9a789f0a99ae6796c19e04cac6))
- Fixed Table::update signature mismatch ([`4c943de`](https://github.com/Dicklesworthstone/charmed_rust/commit/4c943de3f965698c4fec9640586355c1206ff4b6))
- Fixed textinput validation order (validate after truncation) ([`d9bce78`](https://github.com/Dicklesworthstone/charmed_rust/commit/d9bce78604157dd25f645bb67590ed7e6dcb96e7))
- Property-based tests for bubbles components ([`62cd570`](https://github.com/Dicklesworthstone/charmed_rust/commit/62cd570aa241ebc2a734ddb44e826280aa7910a3))
- Bracketed paste tests ([`931ae7e`](https://github.com/Dicklesworthstone/charmed_rust/commit/931ae7e207c928c6ceb3392b1f4b1bfef8a1b576))

### glamour Markdown Rendering

- Markdown rendering with themes (dark, light, dracula, ascii, notty) ([`30286ed`](https://github.com/Dicklesworthstone/charmed_rust/commit/30286ed7b1d2c418d5b04c84bb9d88ecd5e53c39))
- Syntect-to-lipgloss theme mapping ([`ebaed10`](https://github.com/Dicklesworthstone/charmed_rust/commit/ebaed10fc4c4d6e5670b09da964d88eef920f925))
- Table rendering utilities with documentation ([`4aafa42`](https://github.com/Dicklesworthstone/charmed_rust/commit/4aafa420d735d4f1f3102380de69f37c673b2b7c), [`d1a8dfa`](https://github.com/Dicklesworthstone/charmed_rust/commit/d1a8dfa1329947661237ced765537ba5773bf03a))
- Fixed table spacing to match Go implementation ([`eb89f8e`](https://github.com/Dicklesworthstone/charmed_rust/commit/eb89f8eb563132a601157c0b1698d96970d28db9))
- Fixed table rendering width calculation for unicode ([`fc0e8de`](https://github.com/Dicklesworthstone/charmed_rust/commit/fc0e8de8f1a564c603d747845f75c41113a329eb), [`4b7cfb5`](https://github.com/Dicklesworthstone/charmed_rust/commit/4b7cfb58d144bd6461641b8f3cd5b7fa6e6af6b6))
- ANSI-aware width for table column calculations and word wrapping ([`3d87cd0`](https://github.com/Dicklesworthstone/charmed_rust/commit/3d87cd0507dd24057f9aecabc63132e5dcfefbfa), [`c8bc8d6`](https://github.com/Dicklesworthstone/charmed_rust/commit/c8bc8d65bd2bd26482bb32bdcf8e3fa96f78091c))
- Fixed blockquote multi-paragraph and nested separator depth rendering ([`9c5587c`](https://github.com/Dicklesworthstone/charmed_rust/commit/9c5587cadd6f592bb3aacbd54864814f034e227f), [`c25ad4d`](https://github.com/Dicklesworthstone/charmed_rust/commit/c25ad4d9722e6de9f596fedab9c72533686fc3d2))
- LRU cache limit for StyleCache ([`cafaf45`](https://github.com/Dicklesworthstone/charmed_rust/commit/cafaf450b5a78ae125974e8be215df67dc2fa531))
- Added backtick delimiters to ASCII style inline code ([`5a06a44`](https://github.com/Dicklesworthstone/charmed_rust/commit/5a06a44091ccc70a6279aaa37476ffca695fb658))
- Conformance at 81/84 (96%) with Go reference ([`5768568`](https://github.com/Dicklesworthstone/charmed_rust/commit/57685686b90c67e17dead1d91cb5281f02e6e25d))
- Comprehensive syntax highlighting tests and table parsing module ([`cdce084`](https://github.com/Dicklesworthstone/charmed_rust/commit/cdce0842617ac34efeb893c4ce79b32711d67ff5))

### harmonica Spring Physics

- Spring physics, projectile motion, and frame timing ([`30286ed`](https://github.com/Dicklesworthstone/charmed_rust/commit/30286ed7b1d2c418d5b04c84bb9d88ecd5e53c39))
- Full conformance test suite with all 24 tests passing ([`2077d80`](https://github.com/Dicklesworthstone/charmed_rust/commit/2077d80691fdbf1e6b216505d5759df5a969e874))
- Spring physics improvements ([`4cfdfa8`](https://github.com/Dicklesworthstone/charmed_rust/commit/4cfdfa888091e6b3823839771ed590105a3c581c))
- Comprehensive unit tests for physics ([`34f6871`](https://github.com/Dicklesworthstone/charmed_rust/commit/34f6871d8c54fcac63b8006e08e710f2ae73d18c))

### wish SSH Framework

- SSH server built on `russh` with middleware patterns, session management, and PTY support ([`514d456`](https://github.com/Dicklesworthstone/charmed_rust/commit/514d4566dc145fc735b077a70707604dd1d4ca5b))
- Cleared keyboard_interactive state after auth success ([`ade2fd1`](https://github.com/Dicklesworthstone/charmed_rust/commit/ade2fd1e443a444a8fe45400eae49274a42d3098))
- Fixed memory leak and output channel type ([`8ac3be7`](https://github.com/Dicklesworthstone/charmed_rust/commit/8ac3be726dfcb19dbc13b998ce63758de5ea8ab2))
- Prevented split escape sequences in SSH handler ([`7bd1207`](https://github.com/Dicklesworthstone/charmed_rust/commit/7bd1207226a023ba978b6589f80554dda08cff9d))
- Hardened SSH server security and fixed char-boundary panics ([`3384e0d`](https://github.com/Dicklesworthstone/charmed_rust/commit/3384e0d3920d88654d63d18309672b21759bed24))
- Underflow prevention and TOCTOU race in session cleanup ([`15c9509`](https://github.com/Dicklesworthstone/charmed_rust/commit/15c9509926a2ed1f9df365e58c37c7a3b540cf16))
- Fixed u8 truncation in constant_time_eq length comparison ([`70ac227`](https://github.com/Dicklesworthstone/charmed_rust/commit/70ac2279191434082be50d686919316ef6481adf))
- Correct fingerprint prefix in conformance test ([`a0a6f0a`](https://github.com/Dicklesworthstone/charmed_rust/commit/a0a6f0a672c322cfb7fafb20af5f35defcb2951d))
- SSH e2e tests ([`9b9313e`](https://github.com/Dicklesworthstone/charmed_rust/commit/9b9313ee16886833c91e036e278c558ecbf96edd))
- SSH stability audit documented ([`33155b6`](https://github.com/Dicklesworthstone/charmed_rust/commit/33155b689f9f1ab4247fd17e4fd78f7b759f2917))
- Enhanced SSH handler and style improvements ([`c79ecb9`](https://github.com/Dicklesworthstone/charmed_rust/commit/c79ecb9819df844439e6d03be32ffb9e54a42dd0))

### huh Interactive Forms

- Validation framework conformance tests ([`cdb04f8`](https://github.com/Dicklesworthstone/charmed_rust/commit/cdb04f840e674f8f804aacac7a6fc7a2afb37d2d))
- MultiSelect conformance tests enabled ([`b30ea55`](https://github.com/Dicklesworthstone/charmed_rust/commit/b30ea5563106f774fae6a425b91b17adb439f5b0))
- Catppuccin theme (Mocha variant) ([`7a45e4a`](https://github.com/Dicklesworthstone/charmed_rust/commit/7a45e4ad866c8ea7b449954f0b960a17dce1b580))
- Text editing keybindings for word/char manipulation ([`5860755`](https://github.com/Dicklesworthstone/charmed_rust/commit/5860755093929d87583d37b99792a58973e13503))
- Textarea word transformation operations ([`987b304`](https://github.com/Dicklesworthstone/charmed_rust/commit/987b304c7c323f0178bf0c2d5441516f78c4ae4e))
- `next()` method alias for Go API compatibility ([`45d3566`](https://github.com/Dicklesworthstone/charmed_rust/commit/45d3566cdceb6359a519a411915d46b49732713f))
- Migrated FormError to thiserror ([`7e0b47e`](https://github.com/Dicklesworthstone/charmed_rust/commit/7e0b47eab6895989feb3948ca0b6b1e2668714e0))
- Fixed MultiSelect cursor position mismatch with filtering ([`5d4711d`](https://github.com/Dicklesworthstone/charmed_rust/commit/5d4711d247b4c9f6a7efa4ac4c8803ce9759de24))
- Fixed unsafe cast truncation warnings ([`90880f8`](https://github.com/Dicklesworthstone/charmed_rust/commit/90880f8c553ac9acea541d98f286beeeaea74bf3))

### charmed_log Structured Logging

- Text and JSON output with styled terminal rendering ([`30286ed`](https://github.com/Dicklesworthstone/charmed_rust/commit/30286ed7b1d2c418d5b04c84bb9d88ecd5e53c39))
- Handled RwLock poisoning gracefully ([`c0c5d94`](https://github.com/Dicklesworthstone/charmed_rust/commit/c0c5d94d0bf6afc7eeaccf99021b4a9c0eb8b47c))
- Eliminated double-lock race window in `log()` ([`6d9d1e0`](https://github.com/Dicklesworthstone/charmed_rust/commit/6d9d1e0ce14dd6c22120fc6a423fbf0d44a1465a))
- Performance documentation and runtime warning for caller reporting ([`88d5cab`](https://github.com/Dicklesworthstone/charmed_rust/commit/88d5cabde03332b276e4bc35f1b147c7cb22df33))

### glow Markdown Reader CLI

- Terminal Markdown reader with file and stdin support ([`dd52c20`](https://github.com/Dicklesworthstone/charmed_rust/commit/dd52c203d0261a74ebea07c8379419aa248be05a))
- TUI pager mode with viewport scrolling ([`7976c1c`](https://github.com/Dicklesworthstone/charmed_rust/commit/7976c1cec01f74221313f4f3711134543aa355c6))
- Keyboard navigation and search ([`2b5a4df`](https://github.com/Dicklesworthstone/charmed_rust/commit/2b5a4df34494166f6452ed428fff6b891f8ded66))
- Browser and GitHub modules with proper exports ([`19a17b8`](https://github.com/Dicklesworthstone/charmed_rust/commit/19a17b8dd05b769d0ffe112ae51d12f17b8c69ed))
- Improved file browser symlink handling and custom extensions ([`bf34eb8`](https://github.com/Dicklesworthstone/charmed_rust/commit/bf34eb8a23d4aa1f4134e730dc652c700d21f6c5))
- Fixed backward search position ([`fa24fc4`](https://github.com/Dicklesworthstone/charmed_rust/commit/fa24fc4e95124aa8d065ef5a7e6bb14e67f62068))
- Migrated error types to thiserror with comprehensive test coverage ([`38387b9`](https://github.com/Dicklesworthstone/charmed_rust/commit/38387b98dfad20a0b1da29eb92a1231718c2220d))
- Comprehensive documentation ([`720bdb4`](https://github.com/Dicklesworthstone/charmed_rust/commit/720bdb467898eedf15a766c5389e0de582eb59df))

### demo_showcase Flagship Demo

- Full 8-page demo application: Dashboard, Services, Jobs, Logs, Docs, Files, Wizard, Settings ([`b97509c`](https://github.com/Dicklesworthstone/charmed_rust/commit/b97509c9edfeee5f27c691c3f781de3afcb84125))
- Dashboard with metrics, sparklines, uptime/SLA widgets, animated counters ([`91d50f4`](https://github.com/Dicklesworthstone/charmed_rust/commit/91d50f42d2e3cbedc5cad6787081bcab5735825d), [`373fab4`](https://github.com/Dicklesworthstone/charmed_rust/commit/373fab4639a2fe7871ebc96744486428ea8857cd), [`77c19d0`](https://github.com/Dicklesworthstone/charmed_rust/commit/77c19d0aaea2f69fa4879240f2226cebaa2043ca))
- Jobs page with table navigation and detailed info pane ([`ae7178e`](https://github.com/Dicklesworthstone/charmed_rust/commit/ae7178ea4ef5b340e5ea64ed72c1b7bf3ce2d935), [`16936bb`](https://github.com/Dicklesworthstone/charmed_rust/commit/16936bbc72790f6ccdd19b59616ce575c7da0c62))
- DocsPage with glamour markdown rendering and split-view navigation ([`291be2d`](https://github.com/Dicklesworthstone/charmed_rust/commit/291be2d26f027e59027e731c624185459f039927), [`36d2a8c`](https://github.com/Dicklesworthstone/charmed_rust/commit/36d2a8cfa9e2c387e625dab4cf62fbfa3d1847e0))
- Files page with syntax highlighting and line numbers toggle ([`f513c96`](https://github.com/Dicklesworthstone/charmed_rust/commit/f513c969000a45c7d2ff3436bd880740f5da1df2), [`5f97f2b`](https://github.com/Dicklesworthstone/charmed_rust/commit/5f97f2b4c52f737880e0605c490a6e898fe6191a))
- Wizard multi-step deployment workflow with error states and recovery ([`059f49f`](https://github.com/Dicklesworthstone/charmed_rust/commit/059f49f157e5bf2aee6e21c9584502953704508b), [`046b780`](https://github.com/Dicklesworthstone/charmed_rust/commit/046b780a96441c62e3bb1049236d7585b7c52217))
- Live theme switching at runtime ([`38a8f33`](https://github.com/Dicklesworthstone/charmed_rust/commit/38a8f33ed3b84a253afb8b28c5a8f24af4529baa))
- Animation subsystem with spring physics via harmonica ([`4a708b9`](https://github.com/Dicklesworthstone/charmed_rust/commit/4a708b902ee8690ee39a6e761ae7bbd54f242cad))
- Log viewer with follow mode and structured filtering ([`d8442b2`](https://github.com/Dicklesworthstone/charmed_rust/commit/d8442b2a6a5c6314bc1a91da4f26e5e6a620bc24), [`2ae6ed3`](https://github.com/Dicklesworthstone/charmed_rust/commit/2ae6ed3a1bca569814a72778a77855ac27763443))
- Command palette, notes modal, and guided tour ([`a3fb793`](https://github.com/Dicklesworthstone/charmed_rust/commit/a3fb79398f8ac6bc8c4150e88440671a8ec5c005))
- In-doc search with match navigation ([`c316686`](https://github.com/Dicklesworthstone/charmed_rust/commit/c316686ca84bcc634e48169d7e2ee3e7c0244506))
- SSH server mode ([`9db11d9`](https://github.com/Dicklesworthstone/charmed_rust/commit/9db11d92d75325224f8d9c66a95e13f12348670c))
- Notifications/toasts system ([`85d339a`](https://github.com/Dicklesworthstone/charmed_rust/commit/85d339a46c06c2f83c811941c0538cb29be7f336))
- Shell-out action for terminal release/restore ([`8d6c9e7`](https://github.com/Dicklesworthstone/charmed_rust/commit/8d6c9e7855fac4a3c64736f76c9cbcb7e2ad1f82))
- Deterministic data generator with seed support ([`e6e258d`](https://github.com/Dicklesworthstone/charmed_rust/commit/e6e258d7f4bbb0f90655c84829b5c75f6f1a1569))
- CLI contract with clap, runtime config validation ([`d4eb9a1`](https://github.com/Dicklesworthstone/charmed_rust/commit/d4eb9a106fa25818637d4b323d1a9504c9125ead), [`eaafc86`](https://github.com/Dicklesworthstone/charmed_rust/commit/eaafc8642e474c1f3e7bd03448b2f50a7883c9f5))
- Responsive window resize handling ([`8c23682`](https://github.com/Dicklesworthstone/charmed_rust/commit/8c236823934e9cc2d0344a8eb9552b4675426ca6))
- E2E headless runner and comprehensive E2E test suites ([`9802d9d`](https://github.com/Dicklesworthstone/charmed_rust/commit/9802d9d5160caf3ae521ca59e024c4105c064de8), [`f2e0745`](https://github.com/Dicklesworthstone/charmed_rust/commit/f2e07455e23c889a37c61d12bca7e7db2ae8c705))
- RwLock to parking_lot migration ([`76dae36`](https://github.com/Dicklesworthstone/charmed_rust/commit/76dae369eac2f7f115065621ec9a052befd9c922))
- Auto-cap ultra-wide terminals and ANSI-aware truncation ([`41dfc80`](https://github.com/Dicklesworthstone/charmed_rust/commit/41dfc8052b90c8041c4dc25244dd5b7ff78dcce1), [`d822898`](https://github.com/Dicklesworthstone/charmed_rust/commit/d822898ca1e7e1d8cf26f4bbb46d0b7e86ab6bbc))
- Unicode-safe text truncation throughout ([`37f353f`](https://github.com/Dicklesworthstone/charmed_rust/commit/37f353f6825149c59cbaf14198adc7c9a2a26f02), [`1e00877`](https://github.com/Dicklesworthstone/charmed_rust/commit/1e008771e2184d3921f0ed239deef12585bc425d))
- ANSI CSI escape sequence handling for non-SGR sequences ([`f53358c`](https://github.com/Dicklesworthstone/charmed_rust/commit/f53358cec8b891612122e255c330ccf591ca082a), [`67b47f1`](https://github.com/Dicklesworthstone/charmed_rust/commit/67b47f1fdeb565eeede53ec1552b3e99013a516c))

### WASM Support

- `charmed_wasm` crate with `HtmlBackend` for browser rendering ([`f283ec3`](https://github.com/Dicklesworthstone/charmed_rust/commit/f283ec3e6e47256c98504e2bb2114672f57ee796), [`6737532`](https://github.com/Dicklesworthstone/charmed_rust/commit/6737532ae001094e4afb6f4139c537d59a531573))
- WASM CI workflow ([`1b3651d`](https://github.com/Dicklesworthstone/charmed_rust/commit/1b3651d5b9dc0b5990c2c036e5194498e837a58f))

### Conformance Testing Infrastructure

- Generated 397 Go reference fixtures for 6 crates ([`ac7859d`](https://github.com/Dicklesworthstone/charmed_rust/commit/ac7859d4321b187a566555f15fd6d5d5b7e16412))
- Complete Go reference capture for all 8 crates ([`c7f68eb`](https://github.com/Dicklesworthstone/charmed_rust/commit/c7f68eb9e4f9b6d871a911b313cd37f514f2b569))
- Conformance tests for: harmonica, lipgloss/bubbles (list, table), glamour (style/theme, syntax highlighting), bubbletea (mouse parsing for X10 and SGR), bubbles (filepicker, cursor, keybinding), huh, wish ([`2077d80`](https://github.com/Dicklesworthstone/charmed_rust/commit/2077d80691fdbf1e6b216505d5759df5a969e874), [`dc74370`](https://github.com/Dicklesworthstone/charmed_rust/commit/dc743707f22060c1883e0fbcdbf3e5172b7d70dd), [`93d83b0`](https://github.com/Dicklesworthstone/charmed_rust/commit/93d83b06bd1fd8c959be1f89c6537ee1247b7934), [`84ebc4f`](https://github.com/Dicklesworthstone/charmed_rust/commit/84ebc4fb41d25603cc8f5708bd2f388065ebbcf9), [`91ef36a`](https://github.com/Dicklesworthstone/charmed_rust/commit/91ef36a966951b34e9cd0bcc8067c6fc31b491b6), [`252e310`](https://github.com/Dicklesworthstone/charmed_rust/commit/252e310b608dbf09ec395c8cb726c4f70b6f5087), [`ee41a34`](https://github.com/Dicklesworthstone/charmed_rust/commit/ee41a34453e4888cf1695fbf5fbad5010e47b2f6), [`357fa4c`](https://github.com/Dicklesworthstone/charmed_rust/commit/357fa4c15ea344512845e2e49dd3aa51f14a616d))
- Cross-crate integration and E2E tests ([`db34604`](https://github.com/Dicklesworthstone/charmed_rust/commit/db34604cee191793e9a195b430bc46bde4c40386))
- Report generation and CI integration ([`94dccba`](https://github.com/Dicklesworthstone/charmed_rust/commit/94dccba9597926306b809f60cfd8e59a0fa1876c))
- Overhauled conformance test harness for parallel execution and better reporting ([`3e6272b`](https://github.com/Dicklesworthstone/charmed_rust/commit/3e6272bb5609156a0832b2182c323c47734c6d78))
- Glow CLI conformance test infrastructure ([`a4f1e37`](https://github.com/Dicklesworthstone/charmed_rust/commit/a4f1e37aa6279294e3d24540532fb55983e1087a))
- Comprehensive Unicode width parity tests ([`fe85084`](https://github.com/Dicklesworthstone/charmed_rust/commit/fe85084fa3ad10a5876158a3f4bb8e209f4fc0e9))

### Go API Parity

- Full Go API parity across charmed_log, glamour, huh, and glow ([`44b228c`](https://github.com/Dicklesworthstone/charmed_rust/commit/44b228c1ce711d4cb698bf5adaa0581083101e47))
- Tokyo Night theme and improved Unicode handling ([`ed3eb44`](https://github.com/Dicklesworthstone/charmed_rust/commit/ed3eb44d01c453468fcf9b94332e4cbe58bda970))
- Catppuccin theme parity verified ([`3780424`](https://github.com/Dicklesworthstone/charmed_rust/commit/37804241c7a9be7126b84fe09327ce3fef4461ef))
- Mouse drag support verified ([`f7316b6`](https://github.com/Dicklesworthstone/charmed_rust/commit/f7316b6e7544746e4fa10f88eafd178a9e9aa65b))
- Stopwatch and timer parity audits completed ([`43815ed`](https://github.com/Dicklesworthstone/charmed_rust/commit/43815edd4dc3e1556ec4a77e5cdc85ed9ee5fdf5), [`f37396f`](https://github.com/Dicklesworthstone/charmed_rust/commit/f37396f144c2d940e5e4d0075b890bf1a201dca2))

### Testing Infrastructure

- TestTerminal framework for end-to-end testing ([`7468bfb`](https://github.com/Dicklesworthstone/charmed_rust/commit/7468bfb62f71e5708b33028ed11dbb563ceeb51a))
- Proptest property-based testing ([`97ce4e3`](https://github.com/Dicklesworthstone/charmed_rust/commit/97ce4e3c184ce56bbd4dcfd21c6ff7e981bee5fa))
- Go comparison benchmarks ([`d788dba`](https://github.com/Dicklesworthstone/charmed_rust/commit/d788dba07c05ba722f2c5aacaa04657253fb7a59))
- Comprehensive test suites for glamour, glow, harmonica, lipgloss, and wish ([`8492f4e`](https://github.com/Dicklesworthstone/charmed_rust/commit/8492f4e0212c040cec1983dfc125061549ebf3ae))
- Comprehensive unit tests for all examples ([`56cddef`](https://github.com/Dicklesworthstone/charmed_rust/commit/56cddef5181802f80eeafbb92c589573f2db27a2))

### Examples

- Comprehensive examples workspace: basic, intermediate, and advanced demos ([`93dd0d7`](https://github.com/Dicklesworthstone/charmed_rust/commit/93dd0d753f63c2c727b10b1e62811a93dfc20a66), [`14ae0f5`](https://github.com/Dicklesworthstone/charmed_rust/commit/14ae0f5e650bc0bb01dd3f8194ef259be0561dea))
- Form, markdown-viewer, and multi-component examples ([`14ae0f5`](https://github.com/Dicklesworthstone/charmed_rust/commit/14ae0f5e650bc0bb01dd3f8194ef259be0561dea))
- Examples updated with derive macro ([`5b0b091`](https://github.com/Dicklesworthstone/charmed_rust/commit/5b0b091484add54976c50023e514d155fb89db7b))
- Decoupled bubbletea examples from bubbles ([`9ad69e7`](https://github.com/Dicklesworthstone/charmed_rust/commit/9ad69e76c9c20614bf1498db254cde747812d4fb))
- Avoided bubbles/bubbletea example name collisions ([`72d555e`](https://github.com/Dicklesworthstone/charmed_rust/commit/72d555e862708a8e59f42f22a143fa1ffed5018e))

### Documentation

- README with architecture diagram, crate reference, and comprehensive guides ([`d2d89f0`](https://github.com/Dicklesworthstone/charmed_rust/commit/d2d89f0923b30500b223e987eda28e3b8d7d75ca))
- Async migration guide and example ([`a22aa44`](https://github.com/Dicklesworthstone/charmed_rust/commit/a22aa44f1d5e552e29c969ba4488e777dea6f398))
- Unified error handling guide ([`bbcd936`](https://github.com/Dicklesworthstone/charmed_rust/commit/bbcd9368fb2870667782a1275a126019715819b3), [`cf3d8ee`](https://github.com/Dicklesworthstone/charmed_rust/commit/cf3d8ee116181a2496621aca7e466162e9474711))
- CHARM_SPEC.md specification document ([`c6cad1f`](https://github.com/Dicklesworthstone/charmed_rust/commit/c6cad1f08f2b7c34d8a24a0819e80d5fd9a6f094))
- FEATURE_PARITY.md with conformance results ([`aa3c3fc`](https://github.com/Dicklesworthstone/charmed_rust/commit/aa3c3fc5aac7f351ce6c404ffdadc7012c9daa49))
- Comprehensive glow documentation ([`720bdb4`](https://github.com/Dicklesworthstone/charmed_rust/commit/720bdb467898eedf15a766c5389e0de582eb59df))
- bubbletea-macros documentation and examples ([`aaf5727`](https://github.com/Dicklesworthstone/charmed_rust/commit/aaf57277d297d2d0b4ea689387e026858f44df55))
- wish README updated to current API ([`a8876c2`](https://github.com/Dicklesworthstone/charmed_rust/commit/a8876c2dcf55784f306b69a4dc3542d11f213cc3))
- demo_showcase comprehensive architecture docs and README ([`4fe80df`](https://github.com/Dicklesworthstone/charmed_rust/commit/4fe80df0780c9fbd7ef45a3dd4e4fd3163f23ca1), [`97414b2`](https://github.com/Dicklesworthstone/charmed_rust/commit/97414b20d3993d80a9927492fc1a52ae8b4f47ad))
- Specification expanded with component details and known limitations ([`b0329fe`](https://github.com/Dicklesworthstone/charmed_rust/commit/b0329fee1f75892c56f5646d722b21c19a4df90f))
- Per-crate READMEs ([`f5648d3`](https://github.com/Dicklesworthstone/charmed_rust/commit/f5648d3092edf9046bf6c73c0c643f8f14975a5a))
- MIT License added ([`83fea61`](https://github.com/Dicklesworthstone/charmed_rust/commit/83fea613c0d32ef64e0a15573cfdef8c96b244b7))

### CI/CD

- GitHub Actions CI/CD workflows (test, bench, audit, WASM) ([`2696e6e`](https://github.com/Dicklesworthstone/charmed_rust/commit/2696e6eaa26b9fbf89af5cecc1a91e4375143f99))
- Updated GitHub Actions with 2025 best practices ([`90d54d3`](https://github.com/Dicklesworthstone/charmed_rust/commit/90d54d33b1f2db08d46830a8939d3348433c6f4d))
- Hardened workflows and cache keys ([`61ee0d6`](https://github.com/Dicklesworthstone/charmed_rust/commit/61ee0d6090f504007c9158c3b528856ff841e5b3))
- Added dependabot for actions and cargo ([`ef414f9`](https://github.com/Dicklesworthstone/charmed_rust/commit/ef414f91ec3e2d22b205adff358d159cfdd57087))
- Added audit.toml to ignore unfixable rsa vulnerability ([`4a92b33`](https://github.com/Dicklesworthstone/charmed_rust/commit/4a92b330fb509950a2512f2bac6581a068003e80))
- Fixed shellcheck warnings in workflows ([`ef1ea3e`](https://github.com/Dicklesworthstone/charmed_rust/commit/ef1ea3e9d702a3cb2cc0a37cca030e0efc9ed6fc))

### Publishing

- Prepared crates.io publish with `charmed-` package name prefix ([`0537cdf`](https://github.com/Dicklesworthstone/charmed_rust/commit/0537cdf611d92bdcca255389f6a0727052080e5c))
- Workspace version bumped to 0.1.1 ([`5870169`](https://github.com/Dicklesworthstone/charmed_rust/commit/58701693af2e58ce5ab365ff5c96e661026b23d4))

---

## [v0.0.0] — 2026-01-17 (Initial commit)

- Initialized Rust port of Charm TUI library with workspace structure for all crates ([`30286ed`](https://github.com/Dicklesworthstone/charmed_rust/commit/30286ed7b1d2c418d5b04c84bb9d88ecd5e53c39))
