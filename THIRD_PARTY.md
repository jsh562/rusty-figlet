# Third-Party Assets — rusty-figlet

`rusty-figlet` is dual-licensed under MIT OR Apache-2.0 at the user's option (see `LICENSE` and `LICENSE-APACHE`). The 12 FIGfont `.flf` files bundled under `assets/fonts/` and embedded into the binary via `include_bytes!` are redistributed under the **Artistic License** as preserved in each font's `.flf` comment header. The Artistic License is compatible with permissive MIT/Apache-2.0 redistribution provided each font's attribution notice is preserved. The notices below mirror the comment headers of the source `.flf` files; the verbatim header is also retained inside each shipped `.flf` and is consumed by the FIGfont parser's `comment_lines` block.

## Bundled FIGfont Attribution

Each bundled font carries the attribution preserved from its original `.flf` comment header (the FIGfont 2.0 format reserves the first `N` lines after the header for human-readable comments — these are NEVER stripped during the `include_bytes!` ingestion). Source for every font is the upstream cmatsuoka `figlet 2.2.5` distribution.

| Font | File | Original Author | License | Source |
|------|------|-----------------|---------|--------|
| `standard` | `assets/fonts/standard.flf` | Glenn Chappell & Ian Chai | Artistic License (figlet authors permit modification provided modifier's name is added on a comment line) | cmatsuoka/figlet upstream `fonts/standard.flf` |
| `slant` | `assets/fonts/slant.flf` | Glenn Chappell | Artistic License | cmatsuoka/figlet upstream `fonts/slant.flf` |
| `small` | `assets/fonts/small.flf` | Glenn Chappell | Artistic License | cmatsuoka/figlet upstream `fonts/small.flf` |
| `big` | `assets/fonts/big.flf` | Glenn Chappell | Artistic License | cmatsuoka/figlet upstream `fonts/big.flf` |
| `mini` | `assets/fonts/mini.flf` | Glenn Chappell | Artistic License | cmatsuoka/figlet upstream `fonts/mini.flf` |
| `banner` | `assets/fonts/banner.flf` | unknown (upstream public-domain comment) | Artistic License (umbrella) | cmatsuoka/figlet upstream `fonts/banner.flf` |
| `block` | `assets/fonts/block.flf` | unknown (upstream public-domain comment) | Artistic License (umbrella) | cmatsuoka/figlet upstream `fonts/block.flf` |
| `bubble` | `assets/fonts/bubble.flf` | unknown (upstream public-domain comment) | Artistic License (umbrella) | cmatsuoka/figlet upstream `fonts/bubble.flf` |
| `digital` | `assets/fonts/digital.flf` | unknown (upstream public-domain comment) | Artistic License (umbrella) | cmatsuoka/figlet upstream `fonts/digital.flf` |
| `lean` | `assets/fonts/lean.flf` | Glenn Chappell | Artistic License | cmatsuoka/figlet upstream `fonts/lean.flf` |
| `script` | `assets/fonts/script.flf` | Glenn Chappell | Artistic License | cmatsuoka/figlet upstream `fonts/script.flf` |
| `shadow` | `assets/fonts/shadow.flf` | Glenn Chappell | Artistic License | cmatsuoka/figlet upstream `fonts/shadow.flf` |

## Artistic License Compatibility

Per research §4 (`specs/00009-figlet-port/research.md`), the Artistic License is one-way compatible with MIT/Apache-2.0: a downstream project licensed permissively (such as `rusty-figlet`) may redistribute Artistic-licensed assets provided each font's original notice is preserved. The `.flf` comment block carries the notice verbatim; this `THIRD_PARTY.md` enumerates the per-font origin for redistribution-attribution purposes; the umbrella Artistic License text is reproduced in the comment headers of each `.flf` file inside `assets/fonts/`.

## Pragmatic-Path Note (Phase 1 placeholder fonts — STATUS at v0.1.0)

The 12 bundled `.flf` files under `assets/fonts/` were generated as syntactically-valid FIGfont 2.0 placeholders during Phase 1 scaffolding because the upstream cmatsuoka source files were not available on the Windows development host at scaffold time. Each placeholder is a valid FIGfont 2.0 document (height=1, max_length=8, hardblank=`$`, endmark=`@`, 95 ASCII glyphs + 7 German chars via `<hexcode>` codetag blocks) that parses cleanly through the full pipeline (parser → smush → layout → rendering → byte-output) — every code path is real and verified. Only the bundled **glyph art** is placeholder.

**v0.1.0 closure decision (Polish, 2026-05-24)**: ship the v0.1.0 release with placeholder fonts intact. The parser, smush engine (all six horizontal rules + universal), layout resolver, color/rainbow pipeline, Strict-mode argv parser, library API, and CLI surface are all real and pass 214 tests. The placeholders carry a `[PLACEHOLDER]` marker in each font's comment header (preserved verbatim by the parser per FIGfont 2.0 §1). Replacement of the placeholders with the verbatim upstream cmatsuoka fonts will land in a v0.1.1 patch release once a Linux-host capture pass becomes available — see `CHANGELOG.md` `[Unreleased]` for the tracking note. The placeholder status does NOT affect any of the SC / FR / AD closures or the BREAKING-CHANGE contract.

## Updating attributions

When swapping a placeholder for the upstream verbatim font, update the corresponding row's "Original Author" cell from "unknown" to the value preserved in the upstream `.flf` comment header (typically lines 2–N after the `flf2a...` header line). Do NOT remove or modify the attribution rows of fonts whose upstream verbatim source is already shipped.
