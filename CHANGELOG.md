# Changelog

All notable changes to `rusty-figlet` are documented here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Planned for v0.1.1 (v0.1.x maintenance line)

- Replace the 12 placeholder `.flf` fonts under `assets/fonts/` with the verbatim upstream cmatsuoka `figlet 2.2.5` fonts once a Linux-host capture pass is available. The v0.1.0 release ships syntactically-valid placeholder glyphs (height=1, 95 ASCII + 7 German codepoints via `<hexcode>` codetag blocks) — every code path (parser, smush, layout, rendering) is real and verified by 214 tests. Only the bundled glyph **art** is placeholder. See `THIRD_PARTY.md` §Pragmatic-Path Note for details.
- Capture upstream `figlet 2.2.5` snapshot fixtures on a Linux host and engage the deferred byte-equal Strict-mode tests (T085, T086, T087, T088, T089 in `specs/00009-figlet-port/tasks.md`).

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

[Unreleased]: https://github.com/jsh562/rusty-figlet/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/jsh562/rusty-figlet/releases/tag/v0.2.0
[0.1.0]: https://github.com/jsh562/rusty-figlet/releases/tag/v0.1.0
