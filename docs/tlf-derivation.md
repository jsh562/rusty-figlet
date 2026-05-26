# TLF (`tlf2a`) Parser — Clean-Room Derivation Log

**Created**: 2026-05-25 (E012 Phase 3 — T014)
**Module**: `src/tlf.rs` + `src/header.rs`
**Authority**: spec `<umbrella>/specs/00012-e012-toilet-feature-parity-rusty-figlet\spec.md` FR-001, plan HINT-001, AGENTS.md clean-room policy.

## Attestation

**No upstream toilet source code was consulted during the derivation of the `rusty-figlet` TLF parser.**

The parser implementation in `src/tlf.rs` was derived exclusively from:

1. Observed on-disk `.tlf` font files (header bytes only — see §Observed Files below).
2. Publicly available documentation:
   - `https://manpages.ubuntu.com/manpages/jammy/man1/toilet.1.html` (toilet manpage).
   - `https://www.mankier.com/1/toilet` (toilet manpage, duplicate source).
   - `https://github.com/cacalabs/toilet/blob/master/fonts/mono9.tlf` (single file, header line only — see §Header Observation).
   - `http://caca.zoy.org/wiki/toilet` (toilet wiki).
3. FIGfont 2.0 specification — already documented in `src/figfont.rs` and `specs/00009-figlet-port/research.md`.
4. The `research.md` synthesis at `specs/00012-e012-toilet-feature-parity-rusty-figlet/research.md` §TLF Font Format.

`libcaca` C source, `toilet` C source, and existing language ports (`pyfiglet`, `figlet-rs`) were NOT inspected for algorithmic guidance. The implementation is independently derived from observable artifact bytes + manpage text.

## Observed Files (≥5)

The following `.tlf` files were inspected for header-only observation (first line of each file) to confirm the `tlf2a` magic prefix and the FIGfont-shaped numeric-field layout:

| Sample | Header (line 1, observed) | Origin |
|---|---|---|
| `mono9.tlf` | `tlf2a$ 9 6 11 0 0 0 0 0` | toilet 0.3-1 distribution |
| `future.tlf` | `tlf2a$ 4 3 9 0 13 0 0 0` | toilet 0.3-1 distribution |
| `pagga.tlf` | `tlf2a$ 4 3 8 0 16 0 64 0` | toilet 0.3-1 distribution |
| `smblock.tlf` | `tlf2a$ 2 2 4 0 14` | toilet 0.3-1 distribution |
| `wideterm.tlf` | `tlf2a$ 2 1 8 0 0` | toilet 0.3-1 distribution |

These header observations are public-knowledge derivations: the `tlf2a$` magic + 5-to-8 whitespace-separated decimal integers shape is visible at byte 0 of every file. **The parser source code does not embed any glyph data, comment block, or codetag stream from these files; only the header shape was inferred.**

## Header Field Derivation

The observed headers exhibit between 5 and 8 trailing integer fields after the hardblank. Cross-referencing with the FIGfont 2.0 specification (already documented in `src/figfont.rs`), the fields map as:

| Position | Name | Required? | Default if omitted |
|---|---|---|---|
| 1 | `height` | yes | — |
| 2 | `baseline` | yes | — |
| 3 | `max_length` | yes | — |
| 4 | `old_layout` (range `-1..=63`) | yes | — |
| 5 | `comment_lines` | yes | — |
| 6 | `print_direction` | no | `0` |
| 7 | `full_layout` | no | derived from `old_layout` |
| 8 | `codetag_count` | no | `0` |

This is identical to the FIGfont 2.0 numeric-field shape, which validates the AD-001 decision to **share** a numeric-field reader between FLF and TLF (`src/header.rs::parse_header_line` is invoked with `magic_len = 5` for both, differing only in the magic-byte verification step).

## Glyph Table Derivation

The glyph table format was derived from the FIGfont 2.0 specification:

1. After `comment_lines` rows of human-readable text, ASCII codepoints 32..=126 each occupy `height` rows.
2. Each row ends in a single endmark character (typically `@`) on rows 0..height-1, and a **doubled** endmark on the final row.
3. After the 95 required ASCII glyphs, optional codetag blocks supply additional codepoints. Each codetag block consists of a header line (`<hex-codepoint> [optional comment]`) followed by `height` glyph rows.

**TLF extensions to FIGfont 2.0** (per manpage + wiki observation, not source):

- **UTF-8 input encoding** — glyph rows may contain multi-byte UTF-8 sequences for multi-column Unicode display. The rusty-figlet TLF parser uses `std::str::from_utf8` on the input bytes and raises `FigletError::TlfParse` on invalid UTF-8.
- **Inline color markers** — research.md §TLF describes inline color/style markers per cell. Implementation derives the SO (`\x0E`) + attribute byte convention from libcaca's caca_attr documentation (publicly available at `http://caca.zoy.org/doxygen/libcaca/group__caca__attr.html` — the public API doc, not C source). The rusty-figlet parser sets a `multicolor: bool` flag when at least one cell carries a SO marker, and stores the attribute byte in `TlfCell::color_attr`.

## Error Variants Derivation

Per FR-016 + FR-028 the parser raises two distinct error variants:

- `FigletError::InvalidTlfHeader { found: Vec<u8> }` — magic prefix mismatch or structurally-invalid header. `found` is capped at 32 bytes per spec Security Posture (prevents log spam from adversarial inputs).
- `FigletError::TlfParse { reason: String, line: u32 }` — any later parse failure with a 1-indexed line number for byte-precise diagnostics.

Both variants are additive to the existing `#[non_exhaustive]` `FigletError` enum and do NOT break v0.2.x pattern matches.

## Bounded Resource Use (FR-026 / FR-028)

- **Working set bound** — the parser's allocations are bounded by source byte length:
  - One `String` is constructed via UTF-8 decode (`str::from_utf8` returns a borrowed `&str`, no copy).
  - Per-glyph allocations: `Vec<TlfRow>` of length `height`, each `TlfRow::cells` of length ≤ `max_length`.
  - The `HashMap<u32, TlfGlyph>` is sized at most by `comment_lines + 95 + codetag_count`.
- **Parse-error cost bound** — `header.rs::parse_header_line` carries the byte offset alongside every advance; error sites surface byte-precise locations without re-scanning the input (O(1) extra cost beyond the offending byte).
- **File-size cap** — files larger than 8 MiB are rejected with `FigletError::TlfParse` before any allocation (per spec Edge Cases). Zero-byte files are rejected with `FigletError::InvalidTlfHeader`.
- **Per-row cell cap** — files declaring `max_length > 65_536` are rejected up front (per spec Edge Cases — 64 KiB cell-per-row cap).

## Validation

The derived parser is validated against the placeholder `.tlf` files generated by `tools/gen-placeholder-tlf.py` (under `assets/fonts/`). Those placeholders are **original-to-rusty-figlet** — they do not embed any glyph art from the observed upstream samples. The validation exercises the full parse path (magic check → numeric header → comment skip → 95 ASCII glyphs → optional codetag stream → EOF) and the conversion from `TlfFont` → `FIGfont` shape that backs `Figlet::from_tlf_bytes`.

Tests:

- `src/tlf.rs` `#[cfg(test)] mod tests` — 11 unit tests covering: `valid_tlf_returns_ok`, `invalid_magic_returns_invalid_tlf_header`, `empty_input_returns_invalid_tlf_header`, `malformed_header_returns_invalid_tlf_header`, `zero_height_returns_invalid_tlf_header`, `truncated_glyph_table_returns_tlf_parse_with_line`, `file_size_exceeded_returns_tlf_parse`, `extended_metadata_header_form_accepted`, `multicolor_marker_is_observed`, `rejects_inconsistent_endmark`, `rejects_missing_doubled_endmark_final_row`, `unicode_glyph_cell_decodes`, `max_length_cap_enforced`.
- `tests/tlf_bundled_integration.rs` — 4 end-to-end tests exercising the bundled placeholder TLFs via `Figlet::from_tlf_bytes`.

## Re-attestation on future changes

Any change to `src/tlf.rs` MUST preserve the clean-room policy. Specifically:

- DO NOT copy code or comments from `cacalabs/toilet`, `libcaca`, `pyfiglet`, or `figlet-rs`.
- DO NOT embed verbatim `.tlf` font files from upstream toilet — bundle only placeholder or original-art fonts.
- IF additional `.tlf` sample observations are required (e.g., to support a TLF v2 magic prefix), record the observed sample bytes here under a new §Observed Files row.
- Update this log's `created` date when this section changes.
