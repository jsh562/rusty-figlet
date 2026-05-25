# Bundled Fonts — rusty-figlet

This directory contains the FIGfont 2.0 (`.flf`) and TheLetter (`.tlf`) font
files that are compiled into the `rusty-figlet` binary via `include_bytes!`
so no runtime IO is required to render with any bundled font.

## FIGfont `.flf` files (12 — bundled since v0.1.0)

| File | Status |
|---|---|
| `standard.flf` | Placeholder (see `THIRD_PARTY.md` §Pragmatic-Path Note) |
| `slant.flf` | Placeholder |
| `small.flf` | Placeholder |
| `big.flf` | Placeholder |
| `mini.flf` | Placeholder |
| `banner.flf` | Placeholder |
| `block.flf` | Placeholder |
| `bubble.flf` | Placeholder |
| `digital.flf` | Placeholder |
| `lean.flf` | Placeholder |
| `script.flf` | Placeholder |
| `shadow.flf` | Placeholder |

Each `.flf` carries a `[PLACEHOLDER]` marker inside its FIGfont 2.0 comment
header. See `THIRD_PARTY.md` for the per-font attribution and the v0.1.0
closure decision.

## TheLetter `.tlf` files (3 — bundled since v0.3.0, all placeholders)

| File | Status | Generator |
|---|---|---|
| `mono9.tlf` | Placeholder | `tools/gen-placeholder-tlf.py` |
| `future.tlf` | Placeholder | `tools/gen-placeholder-tlf.py` |
| `pagga.tlf` | Placeholder | `tools/gen-placeholder-tlf.py` |

These are syntactically-valid `tlf2a` placeholders generated during E012
Phase 3 (T013). Per AD-006 they mirror the v0.1.0 FLF placeholder approach:
height=1, hardblank=`$`, endmark=`@`, 95 ASCII glyphs (codepoints 32..=126)
each rendered as the literal codepoint character on a single line ending in
`@@` (doubled endmark since the single row is also the final row).

Every glyph carries a `[PLACEHOLDER]` marker inside the TLF comment block
(preserved verbatim by the parser per FIGfont 2.0 §1). The real parser,
header reader, and bundled-asset pipeline are exercised end-to-end — only
the glyph art is placeholder.

### Why placeholder TLFs?

The clean-room policy (`AGENTS.md` and `docs/tlf-derivation.md`) forbids
copying upstream toilet `.tlf` files verbatim. Replacement of these
placeholders with original-art `.tlf` fonts (or with cleanly-licensed
third-party `.tlf` fonts whose redistribution terms align with MIT OR
Apache-2.0) is tracked as a follow-up beyond v0.3.0.

### Regenerating

```sh
python tools/gen-placeholder-tlf.py assets/fonts
```

Re-running overwrites the existing `.tlf` files. The script is deterministic,
so a clean re-run produces byte-identical output.
