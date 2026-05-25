# rusty-figlet v0.2.0 — E002 Quality Bar Walk-Through (T036)

**Subject**: `rusty-figlet` v0.2.0 (commit `b493d576191e2e65529536c6c94facc3b91d7db2`)
**Walked**: 2026-05-25
**Reference**: `c:\claudecode\rusty\docs\quality-bar.md` §Cargo Features Convention Compliance
**Spec context**: T036 of `specs/00011-e011-cargo-features-convention-backfill/tasks.md`
**Coverage**: completes FR-050 + SC-005 for the reference port

## Method

This document walks every checklist item in the umbrella's `docs/quality-bar.md`
§Cargo Features Convention Compliance section (authored in T009) and records
PASS / FAIL / N/A per item against the **published** rusty-figlet v0.2.0
crate. Section letters (a)..(g) mirror the umbrella checklist.

Automated items invoke `tools/feature-lint/lint.sh` (or `run.sh`); manual
items are evaluated by inspection of the published Cargo.toml, README,
CHANGELOG, and crates.io metadata.

## (a) Required umbrellas present

| # | Item | Result | Evidence |
|---|---|---|---|
| a.1 | `Cargo.toml` `[features]` declares `default = ["full"]` (FR-001) | PASS | `Cargo.toml:45` — `default = ["full"]` |
| a.2 | `Cargo.toml` `[features]` declares `full` umbrella composing every leaf (FR-002) | PASS | `Cargo.toml:48-55` — `full = ["cli", "color", "rainbow", "terminal-width", "completions", "strict-compat"]`; all five leaves enumerated |
| a.3 | `Cargo.toml` `[features]` declares `cli` umbrella with all CLI-only deps marked `optional = true` and referenced via `dep:<crate>` (FR-003) | PASS | `Cargo.toml:59` — `cli = ["dep:clap"]`; clap (line 98), clap_complete (99), anstyle (100), termcolor (101), terminal_size (102) all `optional = true` |
| a.4 | `Cargo.toml` `[features]` declares `<port>-classic` umbrella matching v0.1.x bare-port behavior (FR-004) | PASS | `Cargo.toml:65` — `figlet-classic = ["cli", "strict-compat"]`, drop-in upstream `figlet 2.2.5` replacement |

**Automated** lint output: `required-umbrellas PASS` (see §Lint evidence).

## (b) Leaf source-gate present (no phantom leaves)

| # | Item | Result | Evidence |
|---|---|---|---|
| b.1 | Every leaf in `[features]` has at least one `#[cfg(feature = "<leaf>")]` reference in `src/` (FR-008) | PASS | Leaves: `color`, `rainbow`, `terminal-width`, `completions`, `strict-compat` — all 5 gated in `src/lib.rs` + `src/main.rs` + `src/cli.rs` (verified by `feature-lint phantom-leaf PASS`). |

**Automated** lint output: `phantom-leaf PASS`.

## (c) Preset bundle count 2-4

| # | Item | Result | Evidence |
|---|---|---|---|
| c.1 | `Cargo.toml` `[features]` declares between 2 and 4 named preset bundles (FR-007); bundles SHOULD share the `<port>-` prefix | PASS | 3 bundles declared (within the 2-4 range): `figlet-classic` (also serves as the required `<port>-classic` umbrella), `figlet-minimal`, `figlet-toilet-compat`. All share the `figlet-` stem prefix. |

**Manual** review — bundle naming + composition is sensible for the port's
target audience (upstream-drop-in, bare-bones, toilet aesthetic).

## (d) README "Cargo Features" section format

| # | Item | Result | Evidence |
|---|---|---|---|
| d.1 | `README.md` contains an `## Cargo Features` H2 section (FR-020) | PASS | `README.md:91` |
| d.2 | Feature matrix table uses canonical column order `Feature | Description | Umbrella(s)` (FR-021 + FR-024) | PASS | `README.md:97` — `| Feature | Description | Umbrella(s) |` |
| d.3 | Preset bundles table uses canonical column order `Bundle | Composition | Use case` (FR-022 + FR-024) | PASS | `README.md:107` — `| Bundle | Composition | Use case |` |
| d.4 | Keep-list workaround subsection includes worked `cargo install <port> --no-default-features --features "<list>"` example (FR-023) | PASS | `README.md:117-120` — worked example with `cli color rainbow` |
| d.5 | Inline link to ADR-0006 present (FR-026) | PASS | `README.md:136` — `[ADR-0006](https://github.com/jsh562/rustylib/blob/main/specs/adrs/0006-cargo-features-convention-for-portfolio-ports.md)` |
| d.6 | Citation paragraph distinguishes ADR-0006 (why) from project-instructions §Cargo Feature Surface (what) (FR-028) | PASS | `README.md:136` — explicit "(the "why" — option analysis + rationale)" vs "(the "what" — canonical rules per per-port crate)" wording |

**Automated** lint output: `readme-matrix PASS` (header column order + leaf
coverage).

## (e) CHANGELOG migration table exhaustive

| # | Item | Result | Evidence |
|---|---|---|---|
| e.1 | `CHANGELOG.md` `## [0.2.0]` section contains a `### BREAKING-CHANGE` subsection with a migration table (FR-030 + FR-031) | PASS | `CHANGELOG.md:12,30` — `## [0.2.0] - 2026-05-25` heading + `### BREAKING-CHANGE` subheading. |
| e.2 | Migration table uses canonical column order `Old name (v0.1.x) | New name (v0.2.0) | Notes` (FR-031) | PASS | `CHANGELOG.md:36` — `| Old name (v0.1.x) | New name (v0.2.0) | Notes |` |
| e.3 | Every v0.1.x feature name appears as a row | PASS | v0.1.x figlet had only 2 features: `default` and `cli`. Both appear as rows: `default → full`, `cli → cli (preserved)` (`CHANGELOG.md:38-39`). |

**Automated** lint output: `changelog-migration PASS`.

## (f) Multi-surface BREAKING communication

| # | Item | Result | Evidence |
|---|---|---|---|
| f.1 | CHANGELOG entry per (e) above | PASS | See (e). |
| f.2 | README banner at top, wrapped in `<!-- BANNER:v0.2.0 -->` / `<!-- /BANNER:v0.2.0 -->` delimiters per HINT-008 | PASS | `README.md:3-5` — verbatim FROZEN canonical wording with em-dash (U+2014) and matched delimiter pair. |
| f.3 | `Cargo.toml` `[package].description` prefixed/appended with "v0.2: feature layout reorganized — see CHANGELOG" (FR-033(c) + AD-010) | PASS | `Cargo.toml:8` — description ends with `... v0.2: feature layout reorganized — see CHANGELOG.` Confirmed live on crates.io via `cargo info rusty-figlet`. |
| f.4 | GitHub Release notes reproduce the migration table | USER-VERIFY | Not locally verifiable in this environment (no web/`gh` tool). Local-evidence support: tag `v0.2.0` points at `b493d57`; CHANGELOG migration table is canonical; release-drafter convention pulls from CHANGELOG `## [0.2.0]`. Recorded as USER-VERIFY in `c:\claudecode\rusty-figlet\docs\publish-verification.md` (T034). |

**Manual** review — 3 of 4 surfaces locally verified PASS; surface f.4 is
maintainer-confirmable via browser visit to
`https://github.com/jsh562/rusty-figlet/releases/tag/v0.2.0`.

## (g) Shared-leaf glossary consistency

| # | Item | Result | Evidence |
|---|---|---|---|
| g.1 | Every leaf in `Cargo.toml` is either (i) local-only OR (ii) present in `docs/feature-vocabulary.md` with consistent semantic (FR-053) | PASS | rusty-figlet leaves: `color`, `rainbow`, `terminal-width`, `completions`, `strict-compat`. None of these appear in the umbrella `docs/feature-vocabulary.md` glossary yet (the only seeded entries are `unicode-input` and `gradient-rainbow`, both with empty cross-port columns and `(not yet backfilled)` first-used port). Per the second-usage-triggers-glossary-entry rule (FR-053 §Maintenance Rule), local-only leaves do NOT require glossary entries. Compliant by definition. |

**Manual** review — semantic consistency is human judgment; the automated
lint does NOT validate glossary semantics. Note: rusty-figlet's `rainbow`
leaf is semantically related to but distinct from the seeded
`gradient-rainbow` entry (figlet uses per-column HSV, while
`gradient-rainbow` is the broader cross-port leaf). If a second port adopts
the same leaf name, an entry must be added per FR-053.

## Lint evidence

Full `tools/feature-lint/run.sh` invocation against the published
rusty-figlet repo at commit `b493d57`:

```
$ UMBRELLA_PATH=/c/claudecode/rusty PORT_PATH=/c/claudecode/rusty-figlet \
    bash /c/claudecode/rusty/tools/feature-lint/run.sh
---
feature-lint sub-check summary:
  required-umbrellas      PASS
  leaf-ci-matrix          PASS
  phantom-leaf            PASS
  readme-matrix           PASS
  changelog-migration     PASS
feature-lint: PASS
```

## Final verdict

**PASS** — rusty-figlet v0.2.0 satisfies every item of the §Cargo Features
Convention Compliance section of the umbrella quality bar, with one
USER-VERIFY pending on surface (f.4) (GitHub Release notes — see
`publish-verification.md` for the same hold).

The reference port satisfies the convention it anchors; no critical defect.
SC-005 + FR-050 are satisfied for rusty-figlet.

T036 marked `[X]`.
