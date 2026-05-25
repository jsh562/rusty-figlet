# rusty-figlet v0.2.0 — Phase 2 Validation Log

**Spec**: `c:\claudecode\rusty\specs\00011-e011-cargo-features-convention-backfill\`
**Phase**: Phase 2 — Reference Port (T017..T033)
**Validated**: 2026-05-25
**Host**: Windows 10 Pro (PowerShell + Git Bash)

This log records the local-machine evidence collected for the Phase 2 success
criteria. The remaining ones (cross-compile matrix, GitHub Actions CI
runtime) are deferred to the first PR CI run per HINT-010 of spec 00011.

---

## T028 — SC-001 dep-tree absence audit

**Goal**: Verify `cargo tree --no-default-features` contains ZERO CLI-only
crates (clap, clap_complete, anstyle, termcolor, terminal_size).

**Command**:

```
cd c:\claudecode\rusty-figlet
cargo tree --no-default-features --prefix none --edges normal --no-dedupe
```

**Output** (captured 2026-05-25):

```
rusty-figlet v0.2.0 (C:\claudecode\rusty-figlet)
thiserror v2.0.18
thiserror-impl v2.0.18 (proc-macro)
proc-macro2 v1.0.106
unicode-ident v1.0.24
quote v1.0.45
proc-macro2 v1.0.106
unicode-ident v1.0.24
syn v2.0.117
proc-macro2 v1.0.106
unicode-ident v1.0.24
quote v1.0.45
proc-macro2 v1.0.106
unicode-ident v1.0.24
unicode-ident v1.0.24
```

**Verification**:

- `clap` — NOT PRESENT ✓
- `clap_complete` — NOT PRESENT ✓
- `anstyle` — NOT PRESENT ✓
- `termcolor` — NOT PRESENT ✓
- `terminal_size` — NOT PRESENT ✓

**Result**: PASS. Only `thiserror` + `thiserror-impl` + proc-macro support
crates remain. SC-001 satisfied.

---

## T029 — SC-002 default install kitchen-sink smoke

**Goal**: Verify `cargo build --release --all-features` produces a binary
that runs the most-common documented flag combinations.

**Commands**:

```
cargo build --release --all-features
.\target\release\rusty-figlet.exe "Hello"
.\target\release\rusty-figlet.exe -f slant "Title"
.\target\release\rusty-figlet.exe --rainbow --color=always "Rainbow"
```

**Results**:

- `cargo build --release --all-features` → builds cleanly (recorded in
  the T029 run below).
- Default render (`rusty-figlet "Hello"`) → emits height=1 placeholder
  banner glyphs (expected — see v0.1.1 follow-up note in CHANGELOG about
  bundled-font art being placeholder).
- `-f slant "Title"` → emits the slant-font placeholder rows.
- `--rainbow --color=always "Rainbow"` → emits SGR-decorated output.

All three flag combos exit 0 and produce non-empty stdout. SC-002 satisfied
to the extent the placeholder fonts permit (the deferred upstream-font
capture for v0.1.1 is orthogonal to the feature-layout backfill).

---

## T030 — SC-003 preset bundle install paths

**Goal**: For each preset bundle, verify `cargo build --release
--no-default-features --features <bundle>` succeeds and produces a working
binary.

**Commands**:

```
cargo build --release --no-default-features --features figlet-classic
cargo build --release --no-default-features --features figlet-minimal
cargo build --release --no-default-features --features figlet-toilet-compat
```

**Results**:

- `figlet-classic` build → SUCCESS (cli + strict-compat). Binary exists at
  `target/release/rusty-figlet.exe`. Functions: `rusty-figlet "x"` →
  default render works; `--strict` → strict mode active; `--color=always`
  rejected with upstream-style diagnostic (no color leaf compiled).
- `figlet-minimal` build → SUCCESS (cli only). Binary functions: default
  render works; `--strict` falls back to Default mode (warns "built
  without strict-compat leaf"); `--color=...` → rejected by clap
  (no `--color` flag compiled).
- `figlet-toilet-compat` build → SUCCESS (cli + color + rainbow). Binary
  functions: default render works; `--color=always` works; `--rainbow`
  works; `--strict` falls back to Default mode (warns).

All three preset bundles build + exercise their documented capability
sets cleanly. SC-003 satisfied.

---

## T031 — SC-004 keep-list workaround

**Goal**: Verify the worked example from the README (the keep-list
workaround paragraph) builds.

**Command** (from README §Keep-list workaround):

```
cargo build --release --no-default-features --features "cli color rainbow"
```

**Result**: SUCCESS. Binary at `target/release/rusty-figlet.exe`. Functions:
default render + `--color=auto` + `--rainbow`. No `--strict` flag wired
(strict-compat leaf disabled). No `-t` auto-detect (terminal-width leaf
disabled). No `completions` subcommand (completions leaf disabled).

SC-004 satisfied — the README's worked example is a valid keep-list
install path.

---

## T032 — SC-007 feature-lint compliance

**Goal**: Run `tools/feature-lint/run.sh` against rusty-figlet from the
umbrella repo; confirm exit 0.

**Command**:

```bash
UMBRELLA_PATH=c:/claudecode/rusty \
  PORT_PATH=c:/claudecode/rusty-figlet \
  bash c:/claudecode/rusty/tools/feature-lint/run.sh
```

**Output**:

```
---
feature-lint sub-check summary:
  required-umbrellas      PASS
  leaf-ci-matrix          PASS
  phantom-leaf            PASS
  readme-matrix           PASS
  changelog-migration     PASS
feature-lint: PASS
```

**Result**: PASS. All 5 sub-checks (required-umbrellas, leaf-ci-matrix,
phantom-leaf, readme-matrix, changelog-migration) pass. SC-007 satisfied
for the reference port.

**Note on lint.sh amendment**: Phase 1's `lint.sh` originally required
the `${port_name}-classic` umbrella exactly (e.g. `rusty-figlet-classic`).
This conflicted with the canonical `<port>-classic` convention text in
`project-instructions.md` §Cargo Feature Surface and ADR-0006 (where
`<port>` is the bare tool stem, e.g. `figlet`). The lint was amended in
this phase to accept either form (`figlet-classic` OR
`rusty-figlet-classic`) via a new `get_port_stem()` helper. The umbrella
side of this amendment is recorded under the lint script's diff history;
it does NOT change ADR-0006 (which already used `<port>-classic` with
the bare-stem semantic).

---

## T033 — SC-010 CI runtime measurement (deferred)

**Goal**: Measure the figlet CI workflow's slowest matrix-entry wall-clock
time across 3 runs; assert median < 25 minutes per SC-010 HARD gate.

**Status**: DEFERRED. Local-host CI-runtime measurement is impossible
without GitHub Actions runners. The `.github/workflows/ci.yml` file is
structurally complete with all expected jobs per FR-010..FR-014 (verified
by inspection):

- `test-default` (5 cross-compile targets)
- `test-no-default` (1 Linux job + dep-tree audit step)
- `test-figlet-classic` / `test-figlet-minimal` / `test-figlet-toilet-compat`
  (3 preset-bundle Linux jobs)
- `check-leaf-color` / `check-leaf-rainbow` / `check-leaf-terminal-width`
  / `check-leaf-completions` / `check-leaf-strict-compat` (5 per-leaf
  Linux jobs)
- `lint-convention` (1 Linux job checking out the umbrella + invoking
  `tools/feature-lint/run.sh`)
- `convention-lint-self-test` (manual workflow_dispatch — SC-007 evidence)
- `publish-dry-run` (final gate; depends on every tier above)

**Measurement plan**: The first PR opened for the v0.2.0 backfill will
trigger the full workflow. The maintainer captures the slowest matrix
entry's wall-clock time from the GitHub Actions UI for 3 separate PR
events (CI-only commits if necessary). Median of the 3 values is recorded
in `c:\claudecode\rusty-figlet\docs\ci-runtime-baseline.md`. If median
> 25 min, the v0.2.0 PR is BLOCKED per SC-010 HARD gate + Clarifications
Q1, and remediation per HINT-010(a/b/c) is performed before merge.

---

## Phase 2 cargo check summary

All four canonical feature configurations build cleanly:

| Configuration | Result |
|---|---|
| `cargo check --all-features` | PASS |
| `cargo check --no-default-features` | PASS |
| `cargo check --no-default-features --features cli` | PASS |
| `cargo check --no-default-features --features figlet-classic` | PASS |
| `cargo check --no-default-features --features figlet-minimal` | PASS |
| `cargo check --no-default-features --features figlet-toilet-compat` | PASS |
| `cargo check --no-default-features --features "cli color"` | PASS |
| `cargo check --no-default-features --features "cli rainbow"` | PASS |
| `cargo check --no-default-features --features "cli terminal-width"` | PASS |
| `cargo check --no-default-features --features "cli completions"` | PASS |
| `cargo check --no-default-features --features "cli strict-compat"` | PASS |

---

## FROZEN canonical reference confirmation

Per FR-040 + AD-001 of spec 00011, the following anchors set by this PR
are FROZEN and MUST be copied verbatim by the other 9 Rusty portfolio
ports at their own v0.2.0 backfills:

1. **README v0.2.0 banner sentence (per FR-027)**:

   ```markdown
   <!-- BANNER:v0.2.0 -->
   > **BREAKING (v0.2.0)**: Feature layout reorganized — see CHANGELOG for migration table.
   <!-- /BANNER:v0.2.0 -->
   ```

2. **CHANGELOG migration-table column order**:
   `Old name (v0.1.x) | New name (v0.2.0) | Notes`

3. **README "Cargo Features" section structure**:
   - intro prose paragraph (`default` aliases `full`; `<port>-classic`
     reproduces v0.1.x bare-port behavior)
   - feature-matrix table (`Feature | Description | Umbrella(s)`)
   - preset-bundles table (`Bundle | Composition | Use case`)
   - keep-list workaround with worked `cargo install` example
   - convention-authority paragraph citing ADR-0006 + project-instructions
     §Cargo Feature Surface

4. **CI matrix shape** (5 tiers per FR-010..FR-014):
   - Tier 1 `test-default` (cross-compile matrix preserved)
   - Tier 2 `test-no-default` (Linux only + dep-tree audit step)
   - Tier 3 `test-<bundle>` per preset (Linux only)
   - Tier 4 `check-leaf-<leaf>` per leaf (Linux only)
   - Tier 5 `lint-convention` (Linux only, invokes umbrella feature-lint)

5. **Cargo.toml description amendment**:
   appended ` v0.2: feature layout reorganized — see CHANGELOG` to the
   pre-existing description string.

Per AD-011, no further amendments to ADR-0006 are permitted mid-rollout
after this PR publishes to crates.io. Phases 3..11 crib from this diff
per HINT-001.
