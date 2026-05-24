# rusty-figlet — Design Notes

Authoritative spec/plan: [`specs/00009-figlet-port/`](../../rusty/specs/00009-figlet-port/) in the umbrella repo (`plan.md`, `spec.md`, `research.md`).

## Single-Binary Policy (Constitution III + AD-001)

`rusty-figlet` ships exactly ONE binary at v0.1.0 named `rusty-figlet`. Per portfolio Constitution III (One Job Per Crate) and AD-001, there is no companion binary at v0.1.0. The crate is split logically into a CLI binary + library API gated by the `cli` Cargo feature (`default = ["cli"]`); library consumers can depend on `rusty-figlet = { version = "0.1", default-features = false }` to strip every CLI dep (`clap` + `clap_complete` + `anstyle` + `termcolor` + `terminal_size`), keeping only `thiserror` plus the in-house parser/smush/layout/`Banner` foundation.

## Upstream Dependency Status

E003 (reusable `port-ci.yml` workflow) is NOT yet shipped in the umbrella repo at v0.1.0 time. Inline workflows (`ci.yml`, `release.yml`) are duplicated from `rusty-pdfgrep` (post-fix versions including the `taiki-e/install-action@v2` audit replacement, `rustup target add --toolchain 1.85` cross-target fix, and correct `rusty-figlet` bin name everywhere — no leftover `rusty-pdfgrep` / `rusty-pv` references) as a pragmatic-path solution. When E003 v1.0.0 lands, these files are replaced by thin callers pinned to that tag. Tracked as tech debt for the back-port.

Strict-mode byte-equal stdout snapshot capture (T085 in `tasks.md`) is **DEFERRED** because the Windows dev environment cannot run upstream `figlet 2.2.5`. The dependent strict-byte-equal integration tests (T086, T087) are likewise deferred pending a Linux host capture pass. Polish phase re-engages the snapshot suite via a CI workflow step or a manual capture script.

## Library Foundation Choices

- **In-house FIGfont 2.0 parser (`src/figfont.rs`)** — AD-004. `figlet-rs` (33k/mo, MIT) renders correctly for default cases but does NOT expose CLI knobs for `-W`/`-S`/`-k`/`-m`, width wrapping, or justification — the CLI semantics layer is the bulk of the work, and tightening the kernel under our control simplifies byte-equal Strict-mode parity. Parser is ~300 LoC (FIGfont 2.0 spec is small). Forward-review at v0.2.0 whether to delegate to a hardened library if one emerges.
- **Inline `match` smush rules (`src/smush.rs`)** — AD-005. Six rules + universal: each is a constant predicate on two chars. Inline `match` arms in `smush_pair(left, right, rules, hardblank) -> Option<char>` is the simplest, fastest, and easiest to audit against the FIGfont spec. Rule precedence 1→2→3→4→5→6→universal; first applicable rule wins.
- **Hand-rolled Strict-mode argv parser (`src/strict.rs`)** — AD-007. clap's diagnostics cannot byte-equal upstream `figlet(6)`. Hand-rolled implements last-wins (FR-022/023), short-vs-long excluded-flag diagnostics per portfolio precedent, and scope-bounded "invalid option" / "unrecognized option" for excluded flags (`-L`, `-R`, `-I`, `-N`, `--color`, `--rainbow`, plus `-C` per FR-046). Same approach as the four prior bigger ports (`rusty-pdfgrep`, `rusty-pv`, `rusty-detox`, `rusty-pwgen`).
- **`anstyle` + `termcolor` color write path** — AD-011. `termcolor` handles legacy Windows cmd.exe and Win10 pre-build-10586 consoles via Win32 API fallback automatically; `anstyle` provides the style values. CLI-feature-gated; library API doesn't expose styles directly. `NO_COLOR` env var honored at the CLI boundary regardless of `--color` (FR-032).
- **`include_bytes!` 12 bundled fonts (`assets/fonts/*.flf`)** — AD-008 + AD-016. Zero runtime IO for the default case (no font resolution path triggered for `rusty-figlet "X"`); single static binary works on Windows without a system `figlet` install. 12 fonts at ~10 KiB each = ~120 KiB binary cost. Per-font Artistic-License attribution preserved in `THIRD_PARTY.md`. External `.flf` loading via `-f <path>` / `-d <dir>` remains supported for user-supplied fonts.
- **Lazy per-font `OnceLock` parse (HINT-003)** — parsing all 12 fonts on every invocation would be wasteful; each font is parsed on first `-f` resolution and cached in a `std::sync::OnceLock<FIGfont>` keyed by name. No extra crate dependency.

## SemVer Bump Policy

- **MAJOR**: change Strict-mode byte-exact output format; change `FigletError` variant payload signatures; change `Banner` line-iterator semantics; change the SemVer surface of `FigletBuilder` / `Figlet`; change default match-output formatting in Default mode.
- **MINOR**: add a new `FigletError` variant (additive via `#[non_exhaustive]`); add a new bundled font; add a new CLI flag; add a new `FigletBuilder` setter; raise an optional CLI-feature-gated dep's minor pin.
- **PATCH**: bug fixes; performance improvements; doc-only changes.

`FigletError` is the ONLY public type marked `#[non_exhaustive]`. The other four types (`FigletBuilder`, `Figlet`, `Banner`, `Font`) are exhaustive — their field/variant sets are pinned for v0.1.0 SemVer; adding a variant or field to those is a MAJOR bump.

## Build / Feature Matrix

| Feature combination | Binary | Library deps |
|---|---|---|
| `default = ["cli"]` | `rusty-figlet` | `thiserror` + `clap` + `clap_complete` + `anstyle` + `termcolor` + `terminal_size` |
| `default-features = false` | none | `thiserror` only |

Verified by `tests/library_api.rs::default_features_off_dep_tree` (shells `cargo tree --no-default-features --prefix none` and asserts allowlist).

## Thread-Safety Posture

`FigletBuilder`, `Figlet`, `Banner` are `Send + Sync` (verified by `static_assertions::assert_impl_all!` per SC-009). `FigletError` is `Send + Sync + 'static` — intentional asymmetry vs the other types' lack of `'static` so the error works across async await points and thread boundaries. The other types are not required to be `'static` because they may borrow from caller-supplied input (`font_bytes(&[u8])`). See spec Clarifications 2026-05-23 Q2.

## Test Isolation

Every integration test in `tests/` owns a freshly-constructed `tempfile::TempDir` via the `sandbox()` helper in `tests/common/mod.rs`. The portfolio convention enforces these invariants:

- Tests MUST NOT write to relative paths (which would collide between parallel `cargo test` worker threads).
- Tests MUST NOT write under `$HOME` (would pollute the developer's user profile across runs).
- Tests MUST NOT share a global mutable temp directory (would defeat per-test isolation).
- All filesystem paths used by a test MUST flow from the `sandbox()` helper's returned `(TempDir, PathBuf)`. The `TempDir` guard is dropped at end-of-test for cleanup.
- Per-test environment-variable mutations (`NO_COLOR`, `COLUMNS`, `RUSTY_FIGLET_STRICT`, argv0) MUST be scoped via the RAII `env_guard()` helper. The guard's `Drop` impl restores the prior value so changes do NOT leak across tests. This is exercised explicitly by `tests/color_isolation.rs::no_color_test_isolation_raii_contract` (T129).
- Snapshot comparisons MUST route bytes through the canonical `strip_for_snapshot()` helper. Per-test ad-hoc regex substitution is forbidden.

Concurrent `cargo test -- --test-threads=N` for any N is supported and exercised by the CI matrix on all five DDR-003 targets at the default thread count.

## Rustdoc Coverage Policy

The crate root declares `#![deny(missing_docs)]` (compile-fail gate for any undocumented public item) and ships at least one doctest per public type exercising the happy path (per SC-010). The `#[deny(missing_docs)]` lint is the enforcement mechanism — a public item lacking rustdoc fails `cargo build --lib` (and `cargo test --doc`) at compile time.

## Excluded From v0.1.0

Per spec §Excluded, the following are explicitly out-of-scope for v0.1.0 and tracked as forward-review for v0.2.0+:

- Vertical smushing (multi-line stacking only via `-p`; seam smushing deferred).
- `.flc` control file parsing (Hebrew/Katakana/JIS-Roman transliteration); flags `-C <file>` / `-N` accepted-but-ignored in Default with one-time warning; rejected in Strict.
- Right-to-left rendering (`-L`/`-R`).
- Font-info dump (`-I <code>`).
- Non-Latin bundled fonts (`ivrit`, `smtengwar`, `smscript`, `smshadow`, `smslant`, `mnemonic`, `term`).
- Animated/streaming output, GUI mode, web service.
- Custom non-FIGfont formats (TLF / toilet TLF).
