# rusty-figlet — v0.2.0 Feature Layout

**Status**: implementation draft for the v0.2.0 Cargo features convention
backfill (spec 00011, Phase 2 reference port).

**Authority**:
- `specs/adrs/0006-cargo-features-convention-for-portfolio-ports.md` (why)
- `project-instructions.md` §Cargo Feature Surface (what)
- This document — the per-port carving + WHY for each leaf, per HINT-003
  + HINT-009 of spec 00011.

This is the canonical reference port per FR-040 + AD-001. The shape established
here (umbrella set, leaf carving criteria, preset bundle naming, CHANGELOG
migration table format, README "Cargo Features" section, CI matrix) is the
FROZEN format anchor copied by Phases 3..11.

## Source-tree walk

`src/` modules (v0.1.0, post-Phase-1 baseline):

| Module        | Always-on? | CLI-only deps                                 | Notes                                                |
|---------------|-----------:|-----------------------------------------------|------------------------------------------------------|
| `error.rs`    | yes        | (thiserror — always-on)                       | `FigletError` enum; library + binary need it.        |
| `figfont.rs`  | yes        | none                                          | In-house FIGfont 2.0 parser. Library core.           |
| `smush.rs`    | yes        | none                                          | All six horizontal smush rules. Library core.        |
| `layout.rs`   | yes        | none                                          | Layout flag resolver (`-k`/`-W`/`-S`/`-s`/`-o`/`-m`).|
| `mode.rs`     | yes        | none                                          | CompatibilityMode resolver.                          |
| `lib.rs`      | yes        | none                                          | Public API (`FigletBuilder`, `Figlet`, `Banner`).    |
| `cli.rs`      | no — `cli` | clap                                          | clap-derive `Cli` struct + `Completions` subcommand. |
| `main.rs`     | no — `cli` | clap, clap_complete, anstyle, termcolor       | Binary entry; gated by `required-features = ["cli"]`.|
| `color.rs`    | no — leaf  | anstyle, termcolor                            | ColorChoice, rainbow palette, per-char SGR writer.   |
| `output.rs`   | no — leaf  | termcolor                                     | Banner writer (drives Banner iterator → stdout).     |
| `width.rs`    | no — leaf  | terminal_size                                 | `-t` terminal-width detection.                       |
| `strict.rs`   | yes        | none                                          | Strict-mode hand-rolled argv parser (no deps).       |

## Leaf-carving criteria (HINT-009)

A capability becomes a leaf when ALL of the following hold:

1. It is **self-containable** — gated cleanly via `#[cfg(feature = "<leaf>")]`
   at the module or top-level item boundary (HINT-004).
2. Either (a) it has a **sole optional dependency** that no other leaf needs
   (HINT-005), OR (b) it is a pure-cfg-gate of an internal module worth
   exposing as a knob.
3. Disabling it does NOT break any always-on library/CLI surface.

A capability does NOT become a leaf when:

- It is foundational (parser, smush, layout) — disabling it would break
  every other capability.
- It is upstream-defined required behavior in the bare port (Strict mode,
  default justification, etc.).
- It would create more than ~6 leaves (per FR-007 + spec §HINT-003 — keep
  the matrix small enough that the README stays scannable at one glance).

## v0.2.0 Carved Leaves

Per the criteria above, rusty-figlet v0.2.0 carves the following leaves:

### `color`

- **Gates**: `src/color.rs` module + the `--color` flag dispatch arm in
  `main.rs` (`use_color` branch, `write_banner_with_color`, `render_normal_color`,
  `render_paragraph_color`).
- **Sole deps**: `anstyle`, `termcolor`. Both are already CLI-only.
- **Why**: Color output is the single biggest size lever — `termcolor` +
  `anstyle` together pull a non-trivial dep subgraph on Windows
  (Windows-Console fallback). Library consumers + headless-CI binary
  shippers want to strip it. Per the figlet-classic upstream-1:1 contract,
  `--color` does NOT exist in upstream `figlet(6)`, so `figlet-classic`
  excludes this leaf cleanly.

### `rainbow`

- **Gates**: the `--rainbow` flag dispatch + `rainbow_palette` /
  `write_rainbow_line` paths under `color.rs` and `output.rs`. Requires
  `color` (cannot rainbow without the underlying color writer).
- **Sole deps**: (none beyond what `color` already pulls)
- **Why**: Rainbow output is a toilet-style aesthetic — orthogonal to the
  upstream figlet contract. A user might want color (`color`) without
  rainbow gradient. Per FR-006 a leaf is the minimum self-containable
  capability; rainbow is one such capability.

### `terminal-width`

- **Gates**: `src/width.rs` module + the `-t` auto-apply branch in
  `width::resolve_width` + `is_stdout_tty()` callers that consult terminal
  width.
- **Sole dep**: `terminal_size`.
- **Why**: `-t` (terminal-width auto-detect) is an ergonomic convenience.
  A piped-stdout or scripted user gets nothing from `terminal_size`'s
  ioctl/Windows-Console probes. Strict mode does NOT auto-apply `-t`
  anyway (HINT-005), so `figlet-classic` excludes this leaf.

### `completions`

- **Gates**: the `completions <shell>` subcommand dispatch in `main.rs`
  (BinSubcommand::Completions arm) and the `BinSubcommand` enum itself.
- **Sole dep**: `clap_complete`.
- **Why**: Shell-completion script generation is a per-user-environment
  convenience. Users who pre-generate completions once (or ship binary
  packages with completions stuffed alongside) do not need the runtime
  generator. `clap_complete` is the only dep that exists ONLY to support
  this subcommand.

### `strict-compat`

- **Gates**: the `run_strict()` path in `main.rs` + the `strict` module's
  public-but-binary-consumer surface.
- **Sole dep**: none (hand-rolled parser).
- **Why**: Strict-mode is the byte-equal-with-upstream contract. It is the
  defining feature of `figlet-classic`. Some downstream users embed
  rusty-figlet as a library and never touch the upstream-compat path —
  they want to strip the 396 lines of hand-rolled getopt code at compile
  time. Per HINT-004 module-level cfg-gates make this clean.

### Leaves intentionally NOT carved

The following candidate leaves were considered + rejected:

- **`gradient-rainbow` / `gradient-metal` / `gradient-pastel`**: The v0.1.0
  rainbow implementation does NOT have multiple gradient styles; only
  rainbow exists. Per HINT-009, do not carve speculative leaves that have
  no source-tree backing. Future v0.x additions of `gradient-metal` etc.
  would each become a sibling leaf under the `gradient-` category prefix
  per the portfolio shared-leaf glossary (`docs/feature-vocabulary.md`).
- **`filters-mirror` / `filters-zoom`**: figlet has no toilet-style image
  filters in v0.1.0. The leaf names appear in the spec EXAMPLES (HINT-003)
  but are toilet-derived; rusty-figlet doesn't ship them.
- **`output-svg` / `output-html`**: rusty-figlet emits only text rows in
  v0.1.0. SVG/HTML output is not implemented.
- **`seq-utf8`**: UTF-8 input handling is ALWAYS-ON in Default mode + is
  the BREAKING-CHANGE vs upstream that lives in always-on library code.
  Latin-1 clamp for Strict mode is part of `strict-compat`. There is no
  separate "UTF-8 input" leaf to toggle.
- **`fonts-toilet`**: Only the 12 bundled-via-`include_bytes!` cmatsuoka
  fonts ship in v0.1.0. Future TLF / toilet TLF would be a new leaf.

## Preset bundles (FR-007 — 2-4 per port)

### `figlet-classic` (REQUIRED — bare port, 1:1 with upstream)

```toml
figlet-classic = ["cli", "strict-compat"]
```

- Includes `cli` (so the binary exists) and `strict-compat` (so
  Strict-mode byte-equal upstream contract is intact).
- Excludes `color`, `rainbow`, `terminal-width`, `completions` — none of
  those exist in upstream `figlet 2.2.5`.
- Use case: `cargo install rusty-figlet --no-default-features --features figlet-classic`
  for a drop-in upstream-figlet replacement.

### `figlet-minimal`

```toml
figlet-minimal = ["cli"]
```

- Includes `cli` only — bare-bones binary with no Strict mode, no color,
  no rainbow, no terminal-width auto-detect, no completions subcommand.
- Use case: smallest functional binary; library consumers who also want
  the `rusty-figlet` binary for ad-hoc rendering but never invoke any
  optional path.

### `figlet-color` (v0.2.x equivalent — retained)

```toml
figlet-color = ["cli", "color", "rainbow"]
```

- Includes `cli` + `color` + `rainbow` — covers a per-column rainbow
  gradient aesthetic reminiscent of toilet's `--gay` filter, without the
  heavier Strict-mode parser or `-t` auto-detect.
- Use case: a "modern figlet with color + gradient output" install that
  trims Strict-mode + `terminal_size` deps.
- **v0.2.x semantics retained per AD-010** so users who used the v0.2.x
  deprecated alias `figlet-toilet-compat = ["cli", "color", "rainbow"]`
  have a single-name migration target (`figlet-color`).

### `figlet-toilet-compat` (v0.3.0 BREAKING — restored to toilet parity)

```toml
figlet-toilet-compat = [
    "cli",
    "color",
    "rainbow",
    "tlf-parser",
    "filter-crop",
    "filter-gay",
    "filter-metal",
    "filter-flip",
    "filter-flop",
    "filter-rotate",
    "filter-border",
]
```

- **v0.2.0 / v0.2.1 (deprecated)**: alias for `figlet-color` (same set as
  `cli + color + rainbow`). v0.2.0 named this bundle aspirationally
  before the toilet capability surface existed.
- **v0.3.0 (this version)**: restored to mean ACTUAL toilet capability
  parity per E012 spec. Composes the v0.2.x `cli + color + rainbow`
  baseline + the new v0.3.0 toilet-parity leaves: TLF parser, all 10
  filters (`crop`, `gay`, `metal`, `flip`, `flop`, `rotate{180,left,right}`,
  `border`).
- Intentionally NOT in this bundle: `output-html`, `output-irc`,
  `output-svg`, `color-truecolor`, `color-256`, `toilet-strict-compat` —
  these are orthogonal capabilities (export formats and byte-equal
  upstream-compat mode) that users opt into individually.
- This is the v0.3.0 BREAKING for that specific feature bundle name
  (documented in `CHANGELOG.md` `[0.3.0] ### Changed (BREAKING)`). The
  overall public library API is additive-only — verified by
  `cargo public-api diff` in CI per SC-009.

### `figlet-full-cli` (= `full` umbrella alias for self-documentation)

Per FR-002, `full` IS the kitchen sink. We do NOT alias it to a separate
bundle — the four required umbrellas + the four preset bundles above stay
within the 2-4 preset-bundle envelope per FR-007 + spec §HINT-003.

## Cross-port glossary candidates

These leaves MAY be promoted to the portfolio-wide shared-leaf glossary
(`docs/feature-vocabulary.md`) when a SECOND port adopts the same name
with the same semantic per FR-053:

- `color` — accept and emit terminal SGR color escapes. Sibling-port
  candidates: rusty-pv (progress-bar color), rusty-pdfgrep (match
  highlighting).
- `rainbow` — apply a per-column hue-cycle gradient. Sibling-port
  candidates: any text-emitting port with toilet-style aesthetics.
- `terminal-width` — consult `terminal_size` / ioctl / Windows-Console for
  the current terminal width. Sibling-port candidates: rusty-pv (progress
  bar width), rusty-pdfgrep (column wrapping).
- `completions` — emit shell-completion scripts via `clap_complete`.
  Sibling-port candidates: every CLI-shipping port that has a binary.
- `strict-compat` — byte-equal-with-upstream parser path. Sibling-port
  candidates: any port whose upstream has documented stdout snapshots
  (rusty-pwgen, rusty-ts have minimal/no strict-mode-vs-modern split, so
  this may stay figlet-specific).

The `unicode-input` and `gradient-rainbow` glossary seed entries from
`docs/feature-vocabulary.md` v0.0 do NOT yet apply to rusty-figlet:

- `unicode-input` — rusty-figlet's UTF-8-in-Default-Latin1-in-Strict
  behavior is always-on library code, not a leaf. If a future port
  adopts `unicode-input` as a toggleable leaf, rusty-figlet's choice
  to NOT carve it stands per the glossary's "semantic consistency"
  rule (different semantic == different name, not a conflict).
- `gradient-rainbow` — we use the bare name `rainbow` instead of the
  glossary's category-prefixed `gradient-rainbow` because rusty-figlet
  has only one gradient style and the category prefix would be
  speculative. If a SECOND port adopts the category-prefix form, we
  align names at that point (rename to `gradient-rainbow` in v0.3.0
  per the second-usage rule).

## CI matrix shape (FR-010..FR-014)

Per plan §Per-Port v0.2.0 CI Matrix:

- **Tier 1 — `test-default`**: full DDR-003 cross-compile matrix
  (5 targets). Now equivalent to `--features full`.
- **Tier 2 — `test-no-default`**: Linux x86_64 only. `cargo test
  --no-default-features --lib` + dep-tree audit (SC-001 evidence).
- **Tier 3 — `test-<bundle>`**: one job per preset bundle. Linux only.
  - `test-figlet-classic`
  - `test-figlet-minimal`
  - `test-figlet-color`
  - `test-figlet-toilet-compat` (back-compat verification for deprecated alias)
- **Tier 4 — `check-leaf-<leaf>`**: one job per leaf. Linux only.
  - `check-leaf-color`
  - `check-leaf-rainbow`
  - `check-leaf-terminal-width`
  - `check-leaf-completions`
  - `check-leaf-strict-compat`
- **Tier 5 — `lint-convention`**: single Linux job invoking
  `umbrella/tools/feature-lint/run.sh`.

Per FR-014, leaf/bundle/lint jobs are constrained to Linux x86_64 (the
default-cross-compile matrix already covers the other targets via the
kitchen-sink test).
