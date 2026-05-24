# Compatibility — rusty-figlet vs upstream `figlet(6)` v2.2.5

This document is the per-port compatibility matrix for v0.1.0. Every flag listed in upstream `figlet 2.2.5` plus every rusty-figlet-added flag is enumerated below with Default/Strict columns; the five BREAKING-CHANGE divergences are documented in the Behavioral Divergences section. Updated at Polish phase per `specs/00009-figlet-port/tasks.md::T145`.

## Flag Surface

| Flag | Upstream `figlet 2.2.5` | rusty-figlet **Default** | rusty-figlet **Strict** |
|------|-------------------------|--------------------------|--------------------------|
| `-f <name|path>` | ✓ | ✓ | ✓ |
| `-d <dir>` (repeatable) | ✓ | ✓ | ✓ |
| `-w <int>` | ✓ | ✓ | ✓ |
| `-t` (terminal width) | ✓ | ✓ (auto-applied when stdout is a tty AND `-w` not set) | ✓ (NOT auto-applied; explicit only) |
| `-c` / `-l` / `-r` / `-x` | ✓ (last-wins) | ✓ (last-wins per FR-022) | ✓ (last-wins per FR-022) |
| `-k` / `-W` / `-S` / `-s` / `-o` / `-m <N>` | ✓ (last layout-class wins) | ✓ (last layout-class wins per FR-023) | ✓ (last layout-class wins per FR-023) |
| `-p` / `-n` (paragraph / normal) | ✓ | ✓ | ✓ |
| `-C <file>` (control file) | ✓ | ✓ (accepted-but-ignored; one-time warning per FR-046) | ✗ (rejected with `figlet: invalid option -- 'C'` per FR-042 + FR-046) |
| `-N` (no control file) | ✓ | ✓ (accepted-but-ignored; one-time warning per FR-046) | ✗ (rejected with `figlet: invalid option -- 'N'` per FR-042) |
| `-L` / `-R` (LTR / RTL) | ✓ | ✗ (deferred to v0.2.0; Strict rejects per FR-042) | ✗ (rejected with `figlet: invalid option -- 'L'`/`'R'` per FR-042) |
| `-I <code>` (info dump) | ✓ | ✗ (deferred to v0.2.0; Strict rejects per FR-042) | ✗ (rejected with `figlet: invalid option -- 'I'` per FR-042) |
| `--color=auto|always|never` | ✗ | ✓ (per FR-030 + AD-011) | ✗ (rejected with `figlet: unrecognized option '--color'` per FR-045) |
| `--rainbow` | ✗ | ✓ (per FR-031 + AD-012) | ✗ (rejected with `figlet: unrecognized option '--rainbow'` per FR-045) |
| `--strict` / `--no-strict` | ✗ | ✓ (mode-resolver controls; `--no-strict` overrides env + argv[0]; last-wins on command line per FR-040) | ✓ (same precedence ladder) |
| `completions <shell>` subcommand | ✗ | ✓ (Default-mode only; bash/zsh/fish/powershell per FR-060) | ✗ (rejected per US7 AS3) |

## Excluded Flags (rusty-figlet v0.1.0)

The following upstream flags are out-of-scope for v0.1.0 and rejected in both modes:

- `-L` / `-R` — right-to-left rendering (deferred)
- `-I <code>` — info dump (deferred; debug-only)
- `-N` — no control file (paired with `-C`; rejected in Strict, accepted-but-ignored in Default)

Strict-mode rejection format follows upstream `figlet` getopt:
- **Short flags** (`-L`, `-R`, `-I`, `-N`, `-C`): `figlet: invalid option -- '<char>'` (program-name token substituted `figlet:` → `rusty-figlet:` per HINT-004; format preserved).
- **Long flags** (`--color`, `--rainbow`, `--info-dump`, `--no-controlfile`): `figlet: unrecognized option '<flag>'` (program-name substitution as above).

## Behavioral Divergences

### BREAKING-CHANGE vs upstream

These four divergences are documented in `CHANGELOG.md` v0.1.0 BREAKING-CHANGE block and enumerated below for the per-port compatibility matrix:

#### (a) stdin 1 MiB cap

- `rusty-figlet` buffers stdin to a 1 MiB hard ceiling per AD-014.
- Upstream `figlet` buffers stdin unbounded.
- One-time stderr warning per process invocation when triggered: `rusty-figlet: stdin input capped at 1 MiB; remaining input discarded`.
- Truncated output still renders the first 1 MiB worth.

#### (b) `-C`/`-N` Default warn-and-ignore + render input as-is (no transliteration)

- Default mode accepts the `-C <file>` and `-N` flag values but emits a one-time stderr warning (`rusty-figlet: control files not yet implemented; ignoring -C/-N`) and proceeds to render the input as-is (no transliteration).
- Non-ASCII codepoints in the input follow the FR-005 UTF-8 missing-glyph fallback (font's missing-character glyph + one-time warning).
- Strict mode rejects `-C` with `figlet: invalid option -- 'C'`; rejects `-N` with `figlet: invalid option -- 'N'`; rejects `--no-controlfile`-style long forms with `figlet: unrecognized option '--<name>'`.
- See spec Clarifications Q7 + FR-046.

#### (c) UTF-8 vs Latin-1 input encoding

- Default mode decodes input as UTF-8; codepoints absent from the active font's `<hexcode>` table fall back to the font's missing-character glyph plus a one-time stderr warning per process invocation per FR-005.
- Strict mode clamps input to Latin-1 (ISO-8859-1) bytes-as-codepoints **before** glyph lookup so the upstream byte-equal contract is preserved per FR-044. Bytes >127 pass through as Latin-1 (NOT decoded as UTF-8).

#### (d) `--color` / `--rainbow` Default-only

- Default mode supports `--color=auto|always|never` per FR-030 (defaults to `auto`); `--rainbow` per FR-031 emits a per-column HSV gradient.
- Strict mode rejects both flags with `figlet: unrecognized option '<flag>'` per FR-045 + SC-014. The byte-equal upstream contract cannot accommodate ANSI escape sequences in stdout.

#### (e) `-t` auto-apply Default-only

- Default mode auto-applies `-t` (terminal-width detect) when stdout is a tty AND `-w` is not set per HINT-005, matching common Unix CLI etiquette.
- Strict mode does NOT auto-apply `-t` (returns 80 fallback when `-w` and `-t` are both absent) so the upstream byte-equal contract holds at width 80.

### Stream Policy

| Stream | rusty-figlet (Default + Strict) | Upstream `figlet` |
|--------|---------------------------------|---------------------|
| Banner lines | stdout | stdout |
| Warnings | stderr | stderr |
| Errors (font-load, IO, parse) | stderr + nonzero exit | stderr + nonzero exit |

### Library API

`rusty-figlet` exposes a Rust library API in addition to the CLI binary. Upstream `figlet(6)` ships only a CLI. The library API surface (`Figlet`, `FigletBuilder`, `Banner`, `Font`, `FigletError`) is pinned for v0.1.0 SemVer; only `FigletError` is `#[non_exhaustive]` per AD-013. `default-features = false` strips every CLI dep (`clap` + `clap_complete` + `anstyle` + `termcolor` + `terminal_size`), leaving only `thiserror` per FR-051.

## v0.1.0 Status

- **Flag-surface table** — finalized; every upstream flag + rusty-figlet flag enumerated above with Default/Strict columns per FR-040..FR-046.
- **BREAKING-CHANGE block** — five divergences documented (stdin 1 MiB cap; `-C`/`-N` Default warn-and-ignore + render-as-is; UTF-8 vs Latin-1; `--color`/`--rainbow` Default-only; `-t` auto-apply Default-only per HINT-005).
- **Test cross-references** — each Default-mode row exercised by `tests/compat_default.rs`; each Strict-mode row exercised by `tests/compat_strict.rs`. See `tests/SC_COVERAGE.md` for the SC/FR → test matrix.
- **Byte-equal upstream stderr snapshot capture** — DEFERRED (tracked as T085, T086, T087, T088, T089 in `specs/00009-figlet-port/tasks.md`); requires upstream `figlet 2.2.5` binary on a Linux host. Format equivalence is verified via the in-binary `figlet:` → `rusty-figlet:` substitution helper (`tests/common/mod.rs::strip_for_snapshot`) so the v0.1.0 Strict-mode contract is asserted behaviorally; byte-equal snapshot capture will land in a follow-up patch release.

## Polish-Phase Maintenance Policy

Polish-phase tasks (T143+) MUST:

1. Complete every "TBD" cell in the Flag Surface table from observation (not speculation).
2. Append any newly-discovered Strict-mode divergence to the BREAKING-CHANGE block.
3. Cross-reference each row to the integration test that exercises it (`tests/compat_default.rs` or `tests/compat_strict.rs`).
4. Verify the `figlet: invalid option -- '<char>'` and `figlet: unrecognized option '<flag>'` formats match upstream byte-for-byte via captured snapshots in `tests/snapshots/upstream_v2_2_5/` (DEFERRED in T085 pending Linux-host capture).
