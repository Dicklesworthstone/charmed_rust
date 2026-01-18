# Glamour Conformance Discrepancies

## Summary

With **semantic comparison** (comparing text content and style presence rather than exact byte output):
- **43/61 tests pass** (70% semantic conformance)
- **18/61 tests fail** (actual functional differences)

The original exact-match comparison showed 0/61 passing because Go glamour applies ANSI codes character-by-character with 80-character width padding, while Rust glamour produces cleaner output at the word/block level.

## Semantic Comparison Mode

The conformance harness now supports three comparison modes:

1. **Exact**: Byte-for-byte matching (0/61 pass - Go uses character-level ANSI)
2. **Semantic**: Text content + style attributes match (43/61 pass)
3. **TextOnly**: Plain text content matches, ignoring styles (similar results)

## Remaining Functional Differences (18 tests)

These represent actual behavior differences that need investigation:

### 1. Nested Lists (4 tests)
| Test | Issue |
|------|-------|
| `list_nested_unordered` | Missing "Item 1" - first item of outer list lost |
| `list_nested_ordered` | Missing "First" - first item of outer list lost |
| `list_mixed_nested` | First item lost, numbering becomes bullets |
| `list_task_list` | Adding bullet markers (•) when Go doesn't |

### 2. Links (5 tests)
| Test | Issue |
|------|-------|
| `link_inline` | Not appending URL after link text |
| `link_inline_title` | Not appending URL after link text |
| `link_reference` | Not appending URL after link text |
| `link_autolink_email` | Not appending mailto: prefix/URL |
| `link_image` / `link_image_title` | Arrow character (→ vs ->) |

### 3. Blockquotes (2 tests)
| Test | Issue |
|------|-------|
| `blockquote_multi_paragraph` | Missing empty line between paragraphs in quote |
| `blockquote_nested` | Missing nested quote markers (│ │) |

### 4. Style Presets (5 tests)
| Test | Issue |
|------|-------|
| `style_preset_dark` | Extra space before "code" missing |
| `style_preset_light` | Extra space before "code" missing |
| `style_preset_notty` | Backticks around inline code, asterisks for bullets |
| `style_preset_ascii` | Backticks around inline code, asterisks for bullets |
| `style_preset_dracula` | Missing "#" prefix on heading |

### 5. Formatting (1 test)
| Test | Issue |
|------|-------|
| `format_mixed` | Extra space before inline code missing |

## Test Categories Summary

| Category | Exact | Semantic | Notes |
|----------|-------|----------|-------|
| Basic text | 0/6 | 6/6 | All pass with semantic |
| Headings | 0/8 | 8/8 | All pass with semantic |
| Formatting | 0/9 | 8/9 | 1 fail: format_mixed |
| Lists | 0/9 | 4/9 | 5 fail: nested + task lists |
| Code blocks | 0/6 | 6/6 | All pass with semantic |
| Links | 0/7 | 2/7 | 5 fail: URL rendering |
| Blockquotes | 0/5 | 3/5 | 2 fail: multi-para, nested |
| Horizontal rules | 0/6 | 6/6 | All pass with semantic |
| Style presets | 0/5 | 0/5 | 5 fail: mode differences |
| **Total** | **0/61** | **43/61** | **70% semantic** |

## ANSI Styling Differences (Resolved by Semantic Mode)

These differences are handled by semantic comparison:

### Character-by-Character vs Word-Level
**Go glamour:**
- Applies ANSI codes per-character
- Example: `"\u001b[38;5;252mH\u001b[0m\u001b[38;5;252me\u001b[0m..."` for "Hello"

**Rust glamour:**
- Applies styling at word/block level (cleaner, more efficient)

### Fixed Width Padding
**Go glamour:** Pads all lines to 80 characters with styled spaces
**Rust glamour:** No fixed-width padding (output matches content)

## Syntax Highlighting Gap

**Critical conformance gap**: Go glamour implements syntax highlighting via chroma, while Rust glamour does NOT.

### Go Glamour Behavior
Go glamour uses [chroma](https://github.com/alecthomas/chroma) for syntax highlighting:
- Keywords (`fn`, `func`, `def`, `if`, `for`) → color 39 (blue)
- Function names → color 42 (green)
- Strings → color 173 (orange)
- Comments → color 246 (gray)
- Types → color 140 (purple)
- Regular text → color 251/252 (light gray)

Example Go output for Rust code:
```
\u001b[38;5;39mfn\u001b[0m \u001b[38;5;42mmain\u001b[0m() { \u001b[38;5;173m"Hello"\u001b[0m }
```

### Rust Glamour Behavior
Rust glamour outputs code blocks with uniform styling (no per-token colors):
- All code text gets the same color (typically 251/252)
- No language-specific token classification

### Test Coverage

The following tests verify syntax highlighting conformance:
- `test_syntax_highlight_rust_text_content` - Text preservation for Rust code
- `test_syntax_highlight_go_text_content` - Text preservation for Go code
- `test_syntax_highlight_python_text_content` - Text preservation for Python
- `test_syntax_highlight_json_text_content` - Text preservation for JSON
- `test_syntax_highlight_no_language` - Code blocks without language hints
- `test_syntax_highlight_rust_gap_detection` - Documents the highlighting gap
- `test_syntax_highlight_go_gap_detection` - Documents the highlighting gap
- `test_syntax_highlight_conformance` - Fixture-based conformance test

### Languages Tested
| Language | Text Preserved | Syntax Highlighting |
|----------|---------------|---------------------|
| Rust | ✓ | ✗ (gap) |
| Go | ✓ | ✗ (gap) |
| Python | ✓ | ✗ (gap) |
| JSON | ✓ | ✗ (gap) |
| No lang | ✓ | N/A |

## Priority Fixes

To improve conformance further:

1. **Syntax highlighting**: Implement chroma-like token classification and coloring
2. **Nested lists**: Debug first-item handling in nested list rendering
3. **Links**: Append URL text after link display text (Go behavior)
4. **Task lists**: Don't add bullet markers to task list items
5. **Style presets**: Match notty/ascii mode output more closely
6. **Blockquotes**: Handle multi-paragraph and nested quotes correctly

## Files

- `tests/conformance/crates/glamour/mod.rs`: Tests with CompareMode support
- `tests/conformance/src/harness/comparison.rs`: Semantic comparison utilities
- `tests/conformance/fixtures/go_outputs/glamour.json`: Go reference (61 tests)

---
*Updated: 2026-01-18*
*Semantic conformance: 43/61 (70%)*
