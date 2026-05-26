# Changelog

All notable changes to `rusty-figlet` are documented here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.4] - 2026-05-26

### Fixed (tests only — no code or API changes)

- **`tests/filter_scaling.rs` flakes on Apple Silicon CI.** The `filter_chain_scales_linearly` test had a secondary sanity assertion (`t5 ≤ 6× t1`) that compared N=5 wall-clock against N=1. At N=1 (~5 µs) the per-call overhead dominates and CI runner noise produces unreliable ratios; observed in CI: N=1=4541 ns, N=5=29417 ns, N=10=17875 ns, N=20=34666 ns. N=10 measuring faster than N=5 confirmed the noise. Dropped the t5/t1 sanity check; kept the load-bearing SC-012 assertion (N=20 ≤ 2.5× N=10). A real quadratic regression would push N=20 to 4× N=10, well over the 2.5× limit, so the linearity contract is still enforced. Diagnostic timings are still printed to stderr. No production code touched.

## [0.3.3] - 2026-05-26

### Changed (docs only — no code changes)

- **Privacy: scrubbed hardcoded local filesystem paths from `docs/`**. Validation reports under `docs/` (`ci-runtime-baseline.md`, `phase12-validation.md`, `phase2-validation.md`, `public-api-diff-baseline.md`, `publish-verification.md`, `quality-bar-v0.2.0.md`, `tlf-derivation.md`) and `tools/feature-lint/README.md` shipped in v0.3.0, v0.3.1, and v0.3.2 with `c:\claudecode\rusty\` and `c:\claudecode\rusty-figlet\` strings from the maintainer's development workstation. Paths are replaced with relative or placeholder forms (`<umbrella>`, `<repo>`, `<repo-root>`). No source code, no public API, no behavior changes. v0.3.0, v0.3.1, & v0.3.2 are yanked; users should pin v0.3.3 or later.

## [0.3.2] - 2026-05-25 [YANKED]

### Added (additive only — no v0.2.x behavior changed)

- **`figlet-v01-compat` preset bundle** restoring v0.1.0's `cli` umbrella composition. v0.2.0 narrowed `cli` from `["dep:clap", "dep:clap_complete", "dep:anstyle", "dep:termcolor", "dep:terminal_size"]` to just `["dep:clap"]`, with color/completions/terminal-width carved out as separate leaves. Users who had `default-features = false, features = ["cli"]` in v0.1.0 got a narrower build starting at v0.2.0. The new `figlet-v01-compat` alias composes `["cli", "color", "rainbow", "terminal-width", "completions"]` to restore the v0.1.0 semantic. Migration:
  ```toml
  rusty-figlet = { version = "0.3", default-features = false, features = ["figlet-v01-compat"] }
  ```
  No deprecation: `cli` retains its v0.2.x-narrow semantic; `figlet-v01-compat` is the explicit-opt-in v0.1.0 surface restoration.

## [0.3.1] - 2026-05-25 [YANKED]

### Changed

- **Docs only**. README rewritten for readability per the `no-ai-slop` & `rossmann-voice` skills: removed em-dashes, tightened the description paragraph, restructured the Safe-to-embed HTML/SVG section, updated the "What's not shipped" list to reflect v0.3.0's TLF support, expanded the Usage section with example invocations for every new flag (`-F`, `-E`, `--truecolor`, `--ansi256`, `--background`, `--no-downgrade-warning`), and added Library-API sub-examples for `Figlet::from_tlf`, `FilterChain`, & `write_export`. No code changes; no public API change; SemVer-patch release per [Keep a Changelog].

### Planned for v0.1.1 (v0.1.x maintenance line)

- Replace the 12 placeholder `.flf` fonts under `assets/fonts/` with the verbatim upstream cmatsuoka `figlet 2.2.5` fonts once a Linux-host capture pass is available. The v0.1.0 release ships syntactically-valid placeholder glyphs (height=1, 95 ASCII + 7 German codepoints via `<hexcode>` codetag blocks) — every code path (parser, smush, layout, rendering) is real and verified by 214 tests. Only the bundled glyph **art** is placeholder. See `THIRD_PARTY.md` §Pragmatic-Path Note for details.
- Capture upstream `figlet 2.2.5` snapshot fixtures on a Linux host and engage the deferred byte-equal Strict-mode tests (T085, T086, T087, T088, T089 in `specs/00009-figlet-port/tasks.md`).

## [0.3.0] - 2026-05-25 [YANKED]

<!-- BANNER:v0.3.0 -->
> **BREAKING (v0.3.0)**: Toilet feature parity added — TLF parser, 10 filters, HTML/IRC/SVG export. See migration table below.
<!-- /BANNER:v0.3.0 -->

### Added

- **TLF (`tlf2a`) font parser** (`src/tlf.rs`) — TheLetter font format used by upstream `toilet(1)`. UTF-8 multi-column glyphs, per-cell color attributes, 1-indexed parse errors with byte offsets for O(1) error cost (FR-028). New API: `Figlet::from_tlf`, `Figlet::from_tlf_bytes`. Three bundled `.tlf` placeholder fonts at `assets/fonts/{mono9,future,pagga}.tlf` (per AD-006). Clean-room derivation log at `docs/tlf-derivation.md`.
- **`RenderGrid` + `FilterChain` framework** (`src/filter.rs`) — typed `RenderGrid { cells, width, height }` with bounded ~16-byte `Cell` footprint (AD-011); immutable filter pipeline returning owned grids per AD-002. Cost bound: `O(n · w · h)` for an `n`-filter chain on a `w × h` grid, surfaced in public rustdoc per FR-030 + HINT-006.
- **10 toilet-compatible filters** — `Filter::Crop`, `Gay`, `Metal`, `Flip`, `Flop`, `Rotate180`, `RotateLeft`, `RotateRight`, `Border`, `Nothing` (identity, always available). Each non-Nothing variant is gated by its leaf feature.
- **`-F <chain>` CLI flag** — toilet-style colon-separated filter chain syntax (FR-002). Multiple `-F` flags concatenate. Unknown filter names exit non-zero with the enumerated valid set (FR-016).
- **HTML5, IRC, SVG 1.1 export backends** (`src/export/{html,irc,svg}.rs`) — gated by `output-html`, `output-irc`, `output-svg` leaves respectively. Pre-sized writers per FR-027; hand-rolled 4-char XSS escape per AD-004 + HINT-004 (`<` → `&lt;`, `>` → `&gt;`, `&` → `&amp;`, `"` → `&quot;`) applied to text content AND double-quoted attribute positions. mIRC export strips C0/C1 non-printable bytes per FR-015 (UTF-8 multi-byte continuations preserved).
- **`-E <format>` CLI flag** — emits the rendered banner as `html`, `irc`, or `svg`. Unknown/leaf-disabled formats exit non-zero with the enumerated available list (FR-016 — `FigletError::UnsupportedExportFormat`).
- **24-bit truecolor + 256-color SGR emitters** (`src/color_depth.rs`) — `ColorDepth::{Truecolor, Color256, Color16}` with COLORTERM-based detection (`COLORTERM=truecolor`/`24bit` → `Truecolor`; otherwise `Color16`); non-TTY stdout always returns `Color16` to avoid polluting redirected output. Graceful downgrade via `resolve_depth(requested, detected, suppress_warning)`; FIXED stderr warning string (no env bytes interpolated) per FR-018 + spec Security Posture. FR-029 zero-cost: `--no-downgrade-warning` short-circuits BEFORE format-args evaluation. New API: `ColorDepth`, `Figlet::color_depth`, `Figlet::set_color_depth`, `FigletBuilder::color_depth`.
- **`--truecolor` / `--ansi256` / `--no-downgrade-warning` CLI flags** — request 24-bit / 256-color emission and suppress the downgrade warning.
- **`toilet-strict-compat` mode** (`src/strict_toilet.rs`) — byte-equal-to-toilet-0.3-1 renderer. Distinct from the existing `strict-compat` leaf (which targets figlet 2.2.5) per AD-005. Enforces toilet's 16-color floor; same XSS escape + IRC-strip defenses as the default path. New API: `strict_toilet::strict_render(input, chain, target)`, `StrictTarget::Toilet031`, `FigletError::StrictCompatViolation`.
- **`--background=<color>` CLI flag** (E012 US7 — SC-007) — typed background color spec. Accepts the 16 named ANSI colors (case-insensitive) and `#RRGGBB` hex. Anything else (newlines, ANSI escape bytes, shell metachars, partial hex) is rejected at parse time BEFORE export emit; no path exists from the spec string into an SGR or HTML attribute.
- **14 new Cargo leaves** per ADR-0006 — `tlf-parser`, `filter-crop`, `filter-gay`, `filter-metal`, `filter-flip`, `filter-flop`, `filter-rotate`, `filter-border`, `output-html`, `output-irc`, `output-svg`, `color-truecolor`, `color-256`, `toilet-strict-compat`. All kebab-case and category-prefixed; auto-discovered by `tools/feature-lint/run.sh` without manual exemptions (FR-020 + SC-008).
- **`figlet-toilet-compat` v0.3.0 BREAKING composition** — see `### Changed (BREAKING)` below.
- **Performance instrumentation** — `tests/filter_scaling.rs` SC-012 linear-scaling guarantee (N=20 wall-clock ≤ 2.5× N=10 — 10% tolerance over linear); `benches/html_escape.rs` criterion microbench comparing hand-rolled escape vs `htmlescape` / `v_htmlescape`; results in `docs/perf-baseline.md`.

### Changed (BREAKING)

- **`figlet-toilet-compat` preset bundle restored to toilet-feature-parity composition.** In v0.2.0/v0.2.1 this name was a deprecated alias for `figlet-color = ["cli", "color", "rainbow"]` because v0.2.0 named the bundle aspirationally before the toilet capability surface existed. In v0.3.0 the name is restored to its honest meaning: it now composes `cli + color + rainbow + tlf-parser + filter-crop + filter-gay + filter-metal + filter-flip + filter-flop + filter-rotate + filter-border` — i.e., the actual toilet capability parity surface. This IS a SemVer-breaking change for that specific feature bundle name. The overall public library API is unchanged (additive-only — verified by `cargo public-api diff` in CI per SC-009). The `figlet-color` preset retains v0.2.x semantics for users who relied on `figlet-toilet-compat` as an alias for color+rainbow (see migration table below).
- Both `figlet-classic` and `figlet-minimal` preset bundles are **unchanged** from v0.2.x.

### Migration table (v0.3.0 BREAKING-communication surface — FR-031 + AD-006)

| Old name (v0.2.x) | New name (v0.3.0) | Notes |
|---|---|---|
| `figlet-toilet-compat` (`cli + color + rainbow`) | `figlet-color` (`cli + color + rainbow`) | **Migrate the bundle name.** v0.2.x users who used `figlet-toilet-compat` because they wanted only color + rainbow should switch to `figlet-color`, which has identical semantics. The v0.3.0 `figlet-toilet-compat` adds the toilet-parity leaves and is no longer equivalent. |
| `figlet-color` | `figlet-color` | **No change.** v0.2.x semantics preserved per AD-010. |
| `figlet-classic` | `figlet-classic` | **No change.** Bare port, drop-in upstream-figlet replacement. |
| `figlet-minimal` | `figlet-minimal` | **No change.** Bare-bones binary, no extras. |
| `cli`, `color`, `rainbow`, `terminal-width`, `completions`, `strict-compat` | (unchanged) | All v0.2.x leaves preserved verbatim. |
| `default = ["full"]` | `default = ["full"]` | **No change** — but the `full` umbrella now includes the 14 new v0.3.0 leaves, so casual `cargo install` users automatically pick up TLF, all 10 filters, HTML/IRC/SVG export, truecolor, and toilet-strict-compat. |
| (new) | `tlf-parser` | TLF font format parser. |
| (new) | `filter-crop`, `filter-gay`, `filter-metal`, `filter-flip`, `filter-flop`, `filter-rotate`, `filter-border` | 7 filter leaves (10 filters; `rotate*` share one; `Nothing` always available). |
| (new) | `color-truecolor`, `color-256` | 24-bit + 256-color SGR emission. Both imply `color`. |
| (new) | `output-html`, `output-irc`, `output-svg` | Multi-format export backends. |
| (new) | `toilet-strict-compat` | Byte-equal-to-toilet-0.3-1 renderer (distinct from existing `strict-compat` which targets figlet 2.2.5). |

### References

- [ADR-0006](https://github.com/jsh562/rustylib/blob/main/specs/adrs/0006-cargo-features-convention-for-portfolio-ports.md) — leaf-feature convention.
- [`project-instructions.md` §Cargo Feature Surface](https://github.com/jsh562/rustylib/blob/main/project-instructions.md) — canonical portfolio-wide rules.
- [E012 spec](https://github.com/jsh562/rustylib/blob/main/specs/00012-e012-toilet-feature-parity-rusty-figlet/spec.md) — feature requirements, user stories, success criteria.
- [E012 ADRs](https://github.com/jsh562/rustylib/blob/main/specs/00012-e012-toilet-feature-parity-rusty-figlet/plan.md) — AD-001..AD-014 (in-spec architecture decisions for this feature).

### v0.2.x maintenance window

The v0.2.x line is maintained for **6 months** from v0.3.0's publish date:

- **v0.3.0 publish date**: 2026-05-25
- **v0.2.x EOL date**: 2026-11-25

During the maintenance window, security-only patches (CVE / RUSTSEC advisories) land on v0.2.x as v0.2.N patch releases. No feature work, no behavioral changes, no font-art refreshes. After 2026-11-25, the v0.2.x line is end-of-lifed and no further patches will ship; users MUST upgrade to v0.3.x.

**Yank is NOT used** for any reason during the maintenance window per spec 00011 FR-042 + AD-005. Bad releases are addressed via v0.2.(N+1) patch fixes, not via crates.io `cargo yank`.

## [0.2.0] - 2026-05-25

### Added

- Portfolio-wide [Cargo Features Convention](https://github.com/jsh562/rustylib/blob/main/specs/adrs/0006-cargo-features-convention-for-portfolio-ports.md) layout per ADR-0006 + `project-instructions.md` §Cargo Feature Surface. `rusty-figlet` is the canonical reference port — its Cargo.toml `[features]` block, README "Cargo Features" section, CI matrix, and this migration table are the FROZEN format anchor that the other 9 Rusty portfolio ports crib from at their own v0.2.0 backfills.
- **Required umbrellas**: `default = ["full"]`, `full = [<every leaf>]`, `cli` (CLI-only deps + modules), `figlet-classic` (bare port — 1:1 with upstream `figlet 2.2.5`).
- **Leaves** (one per self-containable capability per FR-006 of spec 00011):
  - `color` — ANSI/SGR color writer (anstyle + termcolor)
  - `rainbow` — per-column HSV gradient (implies `color`)
  - `terminal-width` — `-t` auto-detect via `terminal_size`
  - `completions` — `completions <shell>` subcommand (clap_complete)
  - `strict-compat` — hand-rolled upstream-byte-equal getopt parser
- **Preset bundles** (FR-007):
  - `figlet-classic = ["cli", "strict-compat"]` — drop-in upstream replacement.
  - `figlet-minimal = ["cli"]` — bare-bones binary.
  - `figlet-toilet-compat = ["cli", "color", "rainbow"]` — toilet `--gay` aesthetic.
- See `docs/feature-layout.md` for the per-leaf carving rationale and the source-tree walk.

### BREAKING-CHANGE

The `[features]` block has been completely reorganized. The single `cli` feature from v0.1.x is replaced by a layered umbrella + leaves layout. All v0.1.x feature names that previously existed are mapped below; the column order is the portfolio-wide canonical form per spec 00011 FR-031.

#### Feature-name migration table

| Old name (v0.1.x) | New name (v0.2.0) | Notes |
|---|---|---|
| `default` | `full` | `default` is renamed semantically: it now aliases `full` (kitchen sink) rather than `cli`. Users who relied on `default = ["cli"]` for v0.1.x behavior should pin `--features figlet-classic` (matches upstream 1:1) or `--features cli` (bare CLI, no Strict mode, no color, no rainbow, no completions, no terminal-width). |
| `cli` | `cli` | no rename — preserved. v0.2 `cli` umbrella scope is narrower: it now pulls **only** the `clap` dep. Color (`anstyle`/`termcolor`), terminal_size, and `clap_complete` have moved to their own leaves (`color`, `terminal-width`, `completions`). |

#### Capability-to-leaf migration

Capabilities that were always-on under v0.1.x `cli` are now toggleable leaves under v0.2.0. The default install behavior is preserved (`default = ["full"]` re-enables every leaf), but selective consumers can now strip individual capabilities:

| v0.1.x behavior | v0.2.0 enabling feature(s) | Notes |
|---|---|---|
| `--color=auto|always|never` flag | `color` leaf (in `full`, `figlet-toilet-compat`) | Disabling `color` removes the `--color` flag from the CLI surface entirely. |
| `--rainbow` flag | `rainbow` leaf (in `full`, `figlet-toilet-compat`; implies `color`) | Disabling `rainbow` removes the `--rainbow` flag from the CLI surface. |
| `-t` terminal-width auto-detect | `terminal-width` leaf (in `full`) | Disabling this leaf means `-t` falls back to 80 (and the binary doesn't pull `terminal_size`). |
| `completions <shell>` subcommand | `completions` leaf (in `full`) | Disabling this leaf removes the subcommand entirely (`clap_complete` is no longer pulled). |
| `--strict` mode + byte-equal upstream parser | `strict-compat` leaf (in `full`, `figlet-classic`) | Disabling this leaf falls back to Default mode dispatch when `--strict` is requested, emitting a one-time stderr warning. |

### Migration path

To preserve v0.1.x behavior verbatim (drop-in upstream `figlet 2.2.5` replacement):

```sh
cargo install rusty-figlet --no-default-features --features figlet-classic
```

To get the v0.1.x default install (`cli` + all color + rainbow + completions, no Strict mode):

```sh
cargo install rusty-figlet --no-default-features --features "cli color rainbow completions terminal-width"
```

For library-only consumers, no migration is required — `default-features = false` continues to strip every CLI-only dep:

```toml
[dependencies]
rusty-figlet = { version = "0.2", default-features = false }
```

### v0.1.x maintenance window

The v0.1.x line is maintained for **6 months** from v0.2.0's publish date:

- **v0.2.0 publish date**: 2026-05-25
- **v0.1.x EOL date**: 2026-11-25

During the maintenance window, security-only patches (CVE / RUSTSEC advisories) land on v0.1.x as v0.1.N patch releases. No feature work, no behavioral changes, no font-art refreshes. After 2026-11-25, the v0.1.x line is end-of-lifed and no further patches will ship; users MUST upgrade to v0.2.x.

**Yank is NOT used** for any reason during the maintenance window per spec 00011 FR-042 + AD-005. Bad releases are addressed via v0.1.(N+1) patch fixes, not via crates.io `cargo yank`.

### Notes

- See the new `## Cargo Features` section in `README.md` for the feature matrix, preset bundles, keep-list workaround, and convention authority citations.
- Reference: [ADR-0006](https://github.com/jsh562/rustylib/blob/main/specs/adrs/0006-cargo-features-convention-for-portfolio-ports.md) (why this layout) + [`project-instructions.md` §Cargo Feature Surface](https://github.com/jsh562/rustylib/blob/main/project-instructions.md) (what the rules are).
- CI matrix expanded per spec 00011 FR-010..FR-014: now includes `test-default` (kitchen sink + cross-compile), `test-no-default` (bare library + dep-tree audit), `test-figlet-{classic,minimal,toilet-compat}` (preset bundles), `check-leaf-{color,rainbow,terminal-width,completions,strict-compat}` (per-leaf compile checks), and `lint-convention` (umbrella `tools/feature-lint/run.sh` invocation).

## [0.1.0] - 2026-05-24

### Added

- Initial release. Faithful Rust port of [cmatsuoka's `figlet(6)`](http://www.figlet.org/) v2.2.5.
- In-house FIGfont 2.0 parser (`src/figfont.rs`) covering header decode (`flf2a<hardblank>` + height + baseline + max_length + old_layout + comment_lines + optional print_direction + full_layout + codetag_count), required ASCII 32..=126 + seven German chars (196,214,220,228,246,252,223), `<hexcode>` codetag blocks parsed as hexadecimal; enumerated rejection cases for malformed `.flf` (bad signature, truncated header, comment_lines mismatch, short glyph block, missing endmark, codetag_count divergence, `old_layout` out of -1..=63 range).
- All six horizontal smush rules (equal, underscore, hierarchy, opposite-pair, big-X, hardblank) plus universal-smush fallback per the FIGfont 2.0 spec (`src/smush.rs`); precedence 1→2→3→4→5→6→universal, first applicable rule wins.
- 12 bundled fonts ingested via `include_bytes!` (`assets/fonts/*.flf`): `standard`, `slant`, `small`, `big`, `mini`, `banner`, `block`, `bubble`, `digital`, `lean`, `script`, `shadow`. Default = `standard`. Per-font Artistic-License attribution preserved in `THIRD_PARTY.md`.
- Font selection via `-f <name|path>` (with or without `.flf` suffix); repeated `-d <dir>` font directories; resolution precedence: exact path → bundled → `-d` dirs → `~/.local/share/figlet/` (Unix) / `%APPDATA%\figlet\fonts\` (Windows) → `/usr/share/figlet/` (Unix only).
- Horizontal layout control: `-c`/`-l`/`-r`/`-x` justification (last-wins), `-w <int>` output width, `-t` terminal-width auto-detect (`terminal_size` → `COLUMNS` → 80 fallback), `-k` kerning, `-W` full-width, `-S` force smush, `-s` font-default smush, `-o` overlap, `-m <0..=63>` explicit layout. Layout-class flags resolve last-wins.
- Paragraph-mode input: `-p` (paragraph) and `-n` (normal) newline behavior.
- Color output in Default mode: `--color=auto|always|never` (via `anstyle` + `termcolor` for Windows console fallback) and `--rainbow` (per-column HSV gradient, toilet-style). Honors the `NO_COLOR` env var regardless of `--color`.
- Strict-compat mode (`--strict` flag, `RUSTY_FIGLET_STRICT=1` env var, or argv[0] = `figlet`/`figlet-alias`) with byte-equal stdout against upstream v2.2.5 for documented snapshots (60 base + 20 layout permutations); short-flag rejection format `figlet: invalid option -- '<char>'`; long-flag format `figlet: unrecognized option '--<name>'`; precedence ladder `--strict > RUSTY_FIGLET_STRICT > argv[0] > Default`; `--no-strict` overrides env + argv[0]; `--strict`/`--no-strict` last-wins on the command line.
- Library API: `Figlet`, `FigletBuilder` (sole construction entry via `::new()`), `Banner` (lazy line iterator + `Display`), `Font` (12 bundled variants + `External(PathBuf)`), `FigletError` (`#[non_exhaustive]`). `default-features = false` strips clap / clap_complete / anstyle / termcolor / terminal_size; library consumers retain `thiserror` only.
- Pre-generated shell completions for bash / zsh / fish / powershell under `completions/`, with a drift gate that fails CI if regeneration diverges from the committed reference scripts.

### BREAKING-CHANGE vs upstream

- **stdin 1 MiB cap** — `rusty-figlet` buffers stdin to a 1 MiB hard ceiling. Upstream `figlet` buffers stdin unbounded (risks OOM on huge inputs). One-time stderr warning per process invocation when the cap is reached: `rusty-figlet: stdin input capped at 1 MiB; remaining input discarded`. Truncated output still renders the first 1 MiB worth.
- **`-C`/`-N` Default warn-and-ignore + render input as-is (no transliteration)** — Default mode accepts the `-C <file>` and `-N` flag values but emits a one-time stderr warning (`rusty-figlet: control files not yet implemented; ignoring -C/-N`) and proceeds to render the input as-is (no transliteration). Non-ASCII codepoints in the input follow the FR-005 UTF-8 missing-glyph fallback (font's missing-character glyph + one-time warning). Strict mode rejects `-C` (short) with `figlet: invalid option -- 'C'` and `--no-controlfile`-style long forms with `figlet: unrecognized option '--<name>'`. See spec Clarifications Q7.
- **Default mode accepts UTF-8 input vs Strict mode Latin-1 clamp** — Default mode decodes input as UTF-8; codepoints absent from the active font's `<hexcode>` table fall back to the font's missing-character glyph plus a one-time stderr warning per process invocation. Strict mode clamps input to Latin-1 (ISO-8859-1) bytes-as-codepoints before glyph lookup so the upstream byte-equal contract is preserved; bytes >127 pass through as Latin-1 (NOT decoded as UTF-8).
- **`-t` auto-apply Default-only** — Default mode auto-applies `-t` (terminal-width detect) when stdout is a tty AND `-w` is not set, matching common Unix CLI etiquette. Strict mode does NOT auto-apply `-t` (returns 80 fallback when `-w` and `-t` are both absent) so the upstream byte-equal contract holds. Document the auto-apply divergence in `docs/COMPATIBILITY.md`.

### Notes

- MSRV: Rust **1.85** (edition 2024). Pinned via `rust-toolchain.toml`; declared via `rust-version = "1.85"` in `Cargo.toml`. CI MSRV gate job builds + tests against `dtolnay/rust-toolchain@1.85` on every PR.
- Excluded from v0.1.0: vertical smushing, `.flc` control file parsing (flags accepted-but-ignored in Default; rejected in Strict), right-to-left rendering (`-L`/`-R`), font-info dump (`-I <code>`), non-Latin bundled fonts (`ivrit`/`smtengwar`/`smscript`/`smshadow`/`smslant`/`mnemonic`/`term`), animated/streaming output, custom TLF / toilet TLF formats.
- Upstream `figlet 2.2.5` is the reference baseline. Strict-mode snapshot capture procedure documented in `tests/snapshots/upstream_v2_2_5/README.md`; refresh is a deliberate maintenance step on upstream version bump (NOT a silent CI refresh).

[Unreleased]: https://github.com/jsh562/rusty-figlet/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/jsh562/rusty-figlet/releases/tag/v0.3.0
[0.2.0]: https://github.com/jsh562/rusty-figlet/releases/tag/v0.2.0
[0.1.0]: https://github.com/jsh562/rusty-figlet/releases/tag/v0.1.0
