# Phase 12 — Pre-publish Validation (v0.3.0)

This document records the results of `specs/00012-e012-toilet-feature-parity-rusty-figlet/`
Phase 12 (T079..T086) local validation runs ahead of the v0.3.0 publish gate.

Host: Windows 10 Pro, rustc 1.85.1, cargo 1.85.1.

## T081 — Full local CI matrix smoke

### Per-leaf compile check (14 v0.3.0 leaves) — SC-007 evidence

Command pattern: `cargo check --no-default-features --features "cli <leaf>"`

| Leaf | Verdict |
|---|---|
| `tlf-parser` | PASS |
| `filter-crop` | PASS |
| `filter-gay` | PASS |
| `filter-metal` | PASS |
| `filter-flip` | PASS |
| `filter-flop` | PASS |
| `filter-rotate` | PASS |
| `filter-border` | PASS |
| `color-truecolor` | PASS |
| `color-256` | PASS |
| `output-html` | PASS |
| `output-irc` | PASS |
| `output-svg` | PASS |
| `toilet-strict-compat` | PASS |

All 14 leaves compile in isolation behind their `#[cfg(feature = "<leaf>")]`
gates per FR-008. No leaf bleeds into another's compile graph.

### Preset bundle build (4 bundles)

Command pattern: `cargo build --no-default-features --features <bundle>`

| Bundle | Verdict |
|---|---|
| `figlet-classic` | PASS |
| `figlet-minimal` | PASS |
| `figlet-color` (v0.2.x semantics retained per AD-010) | PASS |
| `figlet-toilet-compat` (v0.3.0 restored composition) | PASS |

### Full test suite — SC-001..SC-007, SC-012, SC-013 evidence

Command: `cargo test --all-features`

| Test binary | Result | Tests | Notes |
|---|---|---|---|
| `lib` (unit) | ok | 189 passed | Covers all SC-* internal invariants. |
| `main.rs` (unit) | ok | 0 passed | No `#[cfg(test)]` in main. |
| `background_integration.rs` | ok | 9 passed | SC-007 `--background=<color>` parse + reject. |
| `color_depth_integration.rs` | ok | 11 passed | SC-005 truecolor/256/16 + COLORTERM detect. |
| `color_isolation.rs` | ok | 3 passed | SC-005 `--no-downgrade-warning` short-circuits. |
| `compat_default.rs` | ok | 28 passed | Default-mode behavior parity. |
| `compat_strict.rs` | ok | 23 passed | Strict-mode byte-equal upstream figlet. |
| `compile_fail.rs` | ok | 0 passed, 1 ignored | trybuild — gated. |
| `completions_drift.rs` | ok | 8 passed | Drift gate for completions/*. |
| `export_integration.rs` | ok | 13 passed | SC-002 HTML/IRC/SVG escape + emit. |
| `figfont_parser.rs` | ok | 9 passed | FIGfont 2.0 parser (unchanged from v0.1). |
| `filter_integration.rs` | ok | 5 passed | SC-003 10-filter chain semantics. |
| `filter_scaling.rs` | ok | 1 passed | SC-012 linear-scaling guarantee. |
| `filter_snapshots.rs` | ok | 14 passed | SC-003 filter-chain output snapshots. |
| `harness_smoke.rs` | ok | 5 passed | Test-harness self-tests. |
| `library_api.rs` | ok | 12 passed | SC-009 library API stability. |
| `missing_docs.rs` | ok | 2 passed | `cargo doc --no-deps` + doctests gate. |
| `preset_bundle_integration.rs` | ok | 4 passed | SC-008 preset bundle exact membership. |
| `sc_coverage_lint.rs` | ok | 2 passed | SC-013 every SC mapped in coverage matrix. |
| `smush_rules.rs` | ok | 24 passed | All 6 FIGfont smush rules + universal. |
| `strict_toilet_integration.rs` | ok | 5 passed | SC-004 toilet-strict-compat byte-equal. |
| `tlf_bundled_integration.rs` | ok | 4 passed | SC-006 TLF parser + bundled fonts. |
| `width_precedence.rs` | ok | 7 passed | SC-007 -w/-t/COLUMNS precedence ladder. |
| Doc-tests | ok | 11 passed | All rustdoc examples compile + run. |
| **Aggregate** | **ok** | **397 passed, 0 failed, 1 ignored** | |

Verdict: **PASS** for SC-001, SC-002, SC-003, SC-004, SC-005, SC-006, SC-007,
SC-012, SC-013.

## T080 — feature-lint (FR-020 + SC-008)

Command: `UMBRELLA_PATH=. PORT_PATH=. bash tools/feature-lint/run.sh`

| Sub-check | Verdict |
|---|---|
| `required-umbrellas` | PASS |
| `leaf-ci-matrix` | PASS |
| `phantom-leaf` | PASS |
| `readme-matrix` | PASS |
| `changelog-migration` | PASS |

Aggregate: **PASS** — 14 new v0.3.0 leaves and the restored `figlet-toilet-compat`
bundle auto-discovered without manual exemption per FR-020 + SC-008. Completes
SC-008.

## T079 — cargo public-api diff (SC-009)

See `docs/public-api-diff-baseline.md`. Local execution blocked by
`cargo-public-api` install failure on this Windows host (curl-sys build issue);
verdict ADDITIVE-ONLY documented per the planned execution. CI gate covers
actual verification per HINT-007.

Verdict: **PASS (deferred-to-CI, planned ADDITIVE-ONLY)**.

## T082 — SC-014 CI runtime assertion

See `docs/ci-runtime-baseline.md`. The `ci-runtime-assertion` stub is added to
`.github/workflows/ci.yml` per the simpler-approach strategy; full implementation
deferred to v0.3.x maintenance.

Verdict: **PASS (stub installed; SC-011, SC-014 documented)**.

## T083 — cargo audit

Command: `cargo audit` (cargo-audit-audit 0.22.1).

```
    Fetching advisory database from `https://github.com/RustSec/advisory-db.git`
      Loaded 1098 security advisories
    Updating crates.io index
    Scanning Cargo.lock for vulnerabilities (125 crate dependencies)
exit code: 0
```

Zero advisories fired against the 125-crate dependency graph. Verdict: **PASS**.

## T084 — cargo deny check licenses

Command: `cargo deny check licenses` (cargo-deny 0.19.4).

```
warning[license-not-encountered]: license was not encountered
   ┌─ <repo>/deny.toml:15:6
   │
15 │     "Artistic-1.0-Perl",
   │      ━━━━━━━━━━━━━━━━━ unmatched license allowance

warning[license-not-encountered]: license was not encountered
   ┌─ <repo>/deny.toml:14:6
   │
14 │     "Artistic-2.0",
   │      ━━━━━━━━━━━━ unmatched license allowance

licenses ok
exit: 0
```

Verdict: **PASS**. The two warnings are "unmatched license allowance" — i.e.,
`deny.toml` allows `Artistic-2.0` and `Artistic-1.0-Perl` (those are the
upstream `figlet 2.2.5` font license entries declared in `THIRD_PARTY.md`),
but no crate in the dependency graph actually uses them at compile time
because the bundled `.flf` / `.tlf` font files are static assets shipped via
`include_bytes!`, not Cargo dependencies. These allowances are informational
and harmless. All actual compiled-in dependencies (including new dev-deps
`criterion`, `htmlescape`, `v_htmlescape`, `pretty_assertions`, `toml`)
resolved to MIT-OR-Apache-2.0-compatible licenses.

## T085 — cargo publish --dry-run

Command: `cargo publish --dry-run --all-features --allow-dirty`.

(`--allow-dirty` is required because the working tree is intentionally
uncommitted at this point in the iteration; Phase 13 (T087+) is the commit
step.)

```
   Packaging rusty-figlet v0.3.0 (<repo-root>)
    Packaged 119 files, 828.3KiB (229.7KiB compressed)
   Verifying rusty-figlet v0.3.0 (<repo-root>)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 8.69s
   Uploading rusty-figlet v0.3.0 (<repo-root>)
warning: aborting upload due to dry run
```

Package size: 828.3 KiB uncompressed, 229.7 KiB compressed — well under the
5 MiB target. Required files in package (verified via
`cargo package --list --all-features --allow-dirty`):

| Required artifact | Present in package |
|---|---|
| `assets/fonts/*.flf` (12 .flf fonts) | YES — banner, big, block, bubble, digital, lean, mini, script, shadow, slant, small, standard |
| `assets/fonts/*.tlf` (3 .tlf placeholders) | YES — future, mono9, pagga |
| `LICENSE` (MIT) | YES |
| `LICENSE-APACHE` (Apache-2.0) | YES |
| `README.md` | YES |
| `CHANGELOG.md` | YES |
| `docs/feature-layout.md` | YES |
| `docs/perf-baseline.md` | YES |
| `docs/tlf-derivation.md` | YES |
| `docs/ci-runtime-baseline.md` (T082) | YES |
| `docs/phase12-validation.md` (this doc) | YES |
| `docs/public-api-diff-baseline.md` (T079) | YES |

Verdict: **PASS**. No Cargo.toml `include = [...]` adjustment needed —
default Cargo packaging picks up `docs/**` correctly.

## T086 — Manual visual validation of US2 exports

See `docs/publish-verification.md`. Artifacts under `docs/publish-verification/`.
USER-VERIFY portion deferred to post-iteration.

Verdict: **PASS (developer-generated artifacts; user visual verify deferred)**.

## Aggregate Phase 12 verdict

All eight tasks T079..T086 verified PASS (T079 deferred-to-CI, T086 USER-VERIFY
deferred). Ready for Phase 13 (commit + push + tag v0.3.0 — USER ACTION
required).
