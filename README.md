# rusty-figlet

<!-- BANNER:v0.2.0 -->
> **BREAKING (v0.2.0)**: Feature layout reorganized — see CHANGELOG for migration table.
<!-- /BANNER:v0.2.0 -->

[![crates.io](https://img.shields.io/crates/v/rusty-figlet.svg)](https://crates.io/crates/rusty-figlet)
[![docs.rs](https://docs.rs/rusty-figlet/badge.svg)](https://docs.rs/rusty-figlet)
[![CI](https://github.com/jsh562/rusty-figlet/actions/workflows/ci.yml/badge.svg)](https://github.com/jsh562/rusty-figlet/actions/workflows/ci.yml)
[![MSRV](https://img.shields.io/badge/MSRV-1.85-blue.svg)](#msrv)
[![license: MIT OR Apache-2.0](https://img.shields.io/crates/l/rusty-figlet.svg)](#license)

Render ASCII-art banners from text. A Rust port of [cmatsuoka's `figlet(6)`](http://www.figlet.org/) v2.2.5 with an in-house FIGfont 2.0 parser, all six horizontal smush rules + universal fallback, 12 bundled `.flf` fonts via `include_bytes!`, terminal-width-aware layout, color/rainbow output (toilet-style per-column gradient), byte-equal Strict-mode upstream compatibility, and a typed library API.

Part of the [Rusty portfolio](https://jsh562.github.io/rusty-portfolio).

## Install

```sh
cargo install rusty-figlet
# or, with prebuilt binaries:
cargo binstall rusty-figlet
# or, download directly from GitHub Releases:
# https://github.com/jsh562/rusty-figlet/releases
```

## Usage

```sh
# Default font (standard.flf)
rusty-figlet "Hello"

# Strict-compat (byte-equal upstream figlet v2.2.5 stdout)
rusty-figlet --strict "Hello"

# Font selection (-f <name> | <path>)
rusty-figlet -f slant "Title"
rusty-figlet -f ./my.flf "Hello"
rusty-figlet -d ./fonts -f mycustom "Hello"

# Output width
rusty-figlet -w 120 "wide banner"
rusty-figlet -t "use terminal width"  # auto-applied in Default if stdout is a tty

# Justification (last-wins)
rusty-figlet -c "centered"
rusty-figlet -l "left"
rusty-figlet -r "right"

# Layout overrides (last layout-class flag wins)
rusty-figlet -k "kerning"          # -k
rusty-figlet -W "full width"        # -W
rusty-figlet -S "force smush"       # -S
rusty-figlet -s "font smush"        # -s
rusty-figlet -o "overlap"           # -o
rusty-figlet -m 24 "explicit"       # -m N

# Color + rainbow
rusty-figlet --color=always "color"
rusty-figlet --rainbow "rainbow"   # per-column HSV gradient (toilet-style)

# Paragraph mode (preserve newlines vs collapse)
rusty-figlet -p "paragraph"
rusty-figlet -n "normal"

# Control files (Default: accepted-but-ignored with one-time warning)
rusty-figlet -C custom.flc "Hello"
rusty-figlet -N "Hello"

# Strict mode rejects color flags + control files + completions subcommand
rusty-figlet --strict --color=always "X"   # → exit 2, unrecognized option
```

## Library API

```rust,no_run
use rusty_figlet::{FigletBuilder, Font};

let figlet = FigletBuilder::new()
    .font(Font::Standard)
    .width(80)
    .build()
    .unwrap();

let banner = figlet.render("X").unwrap();
print!("{banner}");
```

For library-only consumers without CLI deps, see the [Cargo Features](#cargo-features) section below.

## Cargo Features

`default` enables `full` which composes every leaf; `figlet-classic` reproduces v0.1.x bare-port behavior matching upstream `figlet 2.2.5` 1:1. Library consumers strip the entire CLI surface via `default-features = false`.

### Feature matrix

| Feature | Description | Umbrella(s) |
|---|---|---|
| `color` | ANSI/SGR color writer (`--color=auto|always|never`). Pulls `anstyle` + `termcolor`. | `full`, `figlet-toilet-compat` |
| `rainbow` | Per-column HSV rainbow gradient (`--rainbow`, toilet-style). Implies `color`. | `full`, `figlet-toilet-compat` |
| `terminal-width` | `-t` auto-detect via `terminal_size`. Without this leaf only an explicit `-w` value (or the 80-col fallback) applies. | `full` |
| `completions` | `completions <shell>` subcommand emitting bash/zsh/fish/powershell scripts. Pulls `clap_complete`. | `full` |
| `strict-compat` | Hand-rolled upstream-byte-equal getopt parser + `--strict` dispatch path. | `full`, `figlet-classic` |

### Preset bundles

| Bundle | Composition | Use case |
|---|---|---|
| `figlet-classic` | `cli` + `strict-compat` | Drop-in upstream `figlet 2.2.5` replacement. No color, no rainbow, no terminal-width auto-detect, no completions subcommand. |
| `figlet-minimal` | `cli` | Bare-bones binary — no Strict mode, no extras. Smallest functional CLI. |
| `figlet-toilet-compat` | `cli` + `color` + `rainbow` | Modern figlet with the toilet `--gay` per-column gradient aesthetic; no Strict mode or `-t` auto-detect. |

### Keep-list workaround (Cargo features are union-only)

Cargo features cannot subtract from `default`. To get "everything except a specific leaf," disable defaults and enumerate the features you want:

```sh
cargo install rusty-figlet --no-default-features --features "cli color rainbow"
# → CLI + color + rainbow, but NO terminal-width auto-detect, NO completions,
#   NO strict-compat path.
```

For the common cases the named [preset bundles](#preset-bundles) above are usually sufficient.

### Library-only consumers

```toml
[dependencies]
rusty-figlet = { version = "0.2", default-features = false }
```

This strips `clap`, `clap_complete`, `anstyle`, `termcolor`, and `terminal_size`, leaving only `thiserror` and the in-house FIGfont parser. Verified by `tests/library_api.rs` and the CI `test-no-default` job's `cargo tree --no-default-features` audit (per SC-001 of spec 00011).

### Convention authority

This layout follows the **portfolio-wide Cargo Features Convention** codified in [ADR-0006](https://github.com/jsh562/rustylib/blob/main/specs/adrs/0006-cargo-features-convention-for-portfolio-ports.md) (the "why" — option analysis + rationale) and [`project-instructions.md` §Cargo Feature Surface](https://github.com/jsh562/rustylib/blob/main/project-instructions.md) (the "what" — canonical rules per per-port crate). Every Rusty port from v0.2 onward exposes the same umbrella set (`default`/`full`/`cli`/`<port>-classic`), per-port leaves named in kebab-case, and 2–4 preset bundles.

## Compatibility

`rusty-figlet` has two modes:

- **Default** — clap-styled flag parser; UTF-8 input; `--color`/`--rainbow` enabled; `-C`/`-N` accepted-but-ignored with a one-time stderr warning; `-t` auto-applied when stdout is a tty AND `-w` is not set; `completions` subcommand for shell-tab generation.
- **Strict** (`--strict` flag, `RUSTY_FIGLET_STRICT=1` env, or argv[0] = `figlet`/`figlet-alias`) — byte-equal stdout against upstream `figlet 2.2.5` for documented diagnostics; Latin-1 input clamp; last-wins flag resolution; rejects `-C`, `-N`, `--color`, `--rainbow`, `completions` with upstream-format getopt errors (short: `invalid option -- '<char>'`; long: `unrecognized option '--<name>'`); no `-t` auto-apply.

Precedence for Strict activation: `--strict` > `RUSTY_FIGLET_STRICT` env > argv[0]. `--no-strict` overrides every lower-precedence source; if `--strict` and `--no-strict` both appear on the command line, last-wins on the command line per upstream getopt convention.

### v0.1.0 excludes

- **Vertical smushing** (rarely exercised; deferred to v0.2.0)
- **Control files (`.flc`)** — `-C`/`-N` accepted-but-ignored in Default with one-time warning; rejected under Strict
- **Right-to-left rendering** (`-L`/`-R`) — niche; deferred
- **Font-info dump** (`-I <code>`) — debug-only; deferred
- **Non-Latin bundled fonts** (`ivrit`, `smtengwar`, `smscript`, `smshadow`, `smslant`, `mnemonic`, `term`) — low signal for the Latin/English 99% target. Users add via `-d <dir>`.
- **Custom non-FIGfont formats** (TLF / toilet TLF) — v0.1.0 targets `.flf` only.

### Excluded flags in Strict mode (upstream-format diagnostics)

| Flag | Type | Strict-mode diagnostic | Exit |
|------|------|------------------------|------|
| `-L`, `-R` | short | `rusty-figlet: invalid option -- '<L\|R>'` | 2 |
| `-I` | short | `rusty-figlet: invalid option -- 'I'` | 2 |
| `-N` | short | `rusty-figlet: invalid option -- 'N'` | 2 |
| `-C` | short | `rusty-figlet: invalid option -- 'C'` | 2 |
| `--color` | long | `rusty-figlet: unrecognized option '--color'` | 2 |
| `--rainbow` | long | `rusty-figlet: unrecognized option '--rainbow'` | 2 |
| `--info-dump` | long | `rusty-figlet: unrecognized option '--info-dump'` | 2 |
| `--no-controlfile` | long | `rusty-figlet: unrecognized option '--no-controlfile'` | 2 |
| `completions <shell>` | subcommand | rejected as unknown positional (exit 2) | 2 |

The program-name token is substituted `figlet:` → `rusty-figlet:` per `tests/common/mod.rs::strip_for_snapshot`; the **format** of the diagnostic mirrors upstream `figlet 2.2.5` getopt byte-for-byte.

### BREAKING-CHANGE vs upstream

- **stdin 1 MiB cap** — `rusty-figlet` buffers stdin to a 1 MiB hard ceiling; upstream buffers unbounded. One-time stderr warning per process invocation when triggered.
- **`-C`/`-N` Default behavior** — Default mode accepts the flags but emits a one-time `control files not yet implemented; ignoring -C/-N` stderr warning and proceeds rendering the input as-is (no transliteration). Strict mode rejects with upstream-format `invalid option -- 'C'` / `unrecognized option`.
- **UTF-8 input in Default** — Default mode accepts UTF-8 bytes (Latin-1 + multibyte codepoints) and falls back to the font's missing-character glyph + one-time stderr warning when a codepoint isn't in the font's `<hexcode>` table. Strict mode clamps input to Latin-1 (ISO-8859-1) bytes-as-codepoints so the upstream byte-equal contract is preserved.
- **`-t` Default auto-apply** — Default mode auto-applies `-t` when stdout is a tty AND `-w` is not set. Strict mode does NOT auto-apply `-t` (preserves byte-equal output at width 80).

See [`docs/COMPATIBILITY.md`](docs/COMPATIBILITY.md) for the full per-flag matrix.

## Lockstep SemVer

`rusty-figlet` follows the [Rusty portfolio SemVer policy](https://jsh562.github.io/rusty-portfolio/semver):

- **MAJOR**: change Strict-mode byte-exact output format; change `FigletError` variant payload signatures; change the SemVer surface of `FigletBuilder` / `Figlet` / `Banner`.
- **MINOR**: add a new `FigletError` variant via `#[non_exhaustive]`; add a new bundled font; add a new CLI flag; add a new `FigletBuilder` setter.
- **PATCH**: bug fixes; performance improvements; doc-only changes.

## MSRV

Rust **1.85** (edition 2024). Re-verified against the portfolio's stable-minus-two policy at each release.

## License

Dual-licensed under [MIT](LICENSE) or [Apache-2.0](LICENSE-APACHE) at your option. The 12 bundled `.flf` fonts under `assets/fonts/` are redistributed under the **Artistic License** as preserved in each font's `.flf` comment header; per-font attribution lives in [`THIRD_PARTY.md`](THIRD_PARTY.md) and the Artistic License is compatible with MIT/Apache-2.0 redistribution.
