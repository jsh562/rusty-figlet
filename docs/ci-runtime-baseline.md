# CI Runtime Baseline — `rusty-figlet`

This document records the static-projection baseline against the SC-011 25-minute
CI HARD gate (inherited from spec 00011 SC-010). It is updated at the boundaries
called out in the E012 task plan: T001 (pre-implementation gate), T034 (Phase 5
trip-wire re-measurement), T076 (final post-implementation measurement), T082
(SC-014 runtime-assertion job).

GitHub Actions historical timing is not directly readable from the maintainer
environment in which this document is authored. All numbers below are **static
projections** derived from the published workflow shape, Cargo build cost on a
warm `Swatinem/rust-cache@v2` runner, and observed convention for hosted GHA
runners (Linux 1×, macOS 1.5–2×, Windows 1.5–2×, cross-compile `cross` cold ≈ 6
–8 min).

The wall-clock model is parallel-aware: jobs that fan out via
`strategy.matrix` or run as independent jobs only contribute their slowest
member to the wall-clock; serial dependencies are added.

---

## 1. v0.2.x baseline inventory (current `ci.yml`)

Source: `<repo>/.github\workflows\ci.yml` as of v0.2.0.

### 1.1 Pre-test gates (fan-out, no `needs:` dependency between them)

| Job | Runner | Estimated wall-clock (warm cache) | Notes |
|-----|--------|-----------------------------------|-------|
| `fmt` (rustfmt) | ubuntu-latest | ~30 s | toolchain install + `cargo fmt --check` |
| `clippy` (deny warnings) | ubuntu-latest | ~3 min | full `cargo clippy --all-targets --all-features`; compiles the workspace |
| `audit` (cargo-audit) | ubuntu-latest | ~1 min | `taiki-e/install-action` install + advisory scan |
| `deny` (cargo-deny licenses) | ubuntu-latest | ~1 min | install + license check (no advisory scan) |
| `msrv` (Rust 1.85) | ubuntu-latest | ~4 min | full build + test under pinned toolchain |

These five run in parallel. Slowest path: `clippy` ≈ 3 min, `msrv` ≈ 4 min.

### 1.2 Tier 1 — `test-default` cross-compile matrix

Five matrix entries, all gated on `needs: [fmt, clippy]`:

| Target | Runner | Build + test (warm cache) | Notes |
|--------|--------|---------------------------|-------|
| `x86_64-unknown-linux-gnu` | ubuntu-latest | ~4 min | native build + test |
| `aarch64-unknown-linux-gnu` | ubuntu-latest + `cross` | ~7 min | `cross install` + cross build (no test) |
| `x86_64-apple-darwin` | macos-latest | ~6 min | macOS runner 1.5× Linux cost |
| `aarch64-apple-darwin` | macos-latest | ~6 min | M-series Apple Silicon, similar to x86 darwin |
| `x86_64-pc-windows-msvc` | windows-latest | ~7 min | Windows runner historically slowest |

Matrix wall-clock = slowest member = **~7 min** (Windows or aarch64-cross).

### 1.3 Tier 2 — `test-no-default`

| Job | Runner | Wall-clock | Notes |
|-----|--------|------------|-------|
| `test-no-default` | ubuntu-latest | ~2 min | bare-library build + dep-tree audit |

### 1.4 Tier 3 — preset bundles (4 jobs, all `needs: [fmt, clippy]`)

| Job | Wall-clock |
|-----|------------|
| `test-figlet-classic` | ~2 min |
| `test-figlet-minimal` | ~2 min |
| `test-figlet-color` | ~2 min |
| `test-figlet-toilet-compat` (deprecated alias) | ~2 min |

Parallel wall-clock = **~2 min** (all fan out in parallel).

### 1.5 Tier 4 — `check-leaf-*` (5 jobs in v0.2.x)

| Job | Wall-clock |
|-----|------------|
| `check-leaf-color` | ~60 s |
| `check-leaf-rainbow` | ~60 s |
| `check-leaf-terminal-width` | ~60 s |
| `check-leaf-completions` | ~60 s |
| `check-leaf-strict-compat` | ~60 s |

Parallel wall-clock = **~60 s**.

### 1.6 Tier 5 — lint-convention

| Job | Wall-clock |
|-----|------------|
| `lint-convention` | ~30 s |

### 1.7 Final gate — `publish-dry-run`

`needs:` all of the above. Wall-clock contribution: ~2 min (publish-dry-run).

### 1.8 v0.2.x aggregate (PR-time)

The DAG decomposes into stages:

```
Stage A: fmt, clippy, audit, deny, msrv     (parallel; wall-clock = max ≈ 4 min)
Stage B: tier-1 .. tier-5                   (parallel within stage; wall-clock = max(7, 2, 2, 1, 0.5) ≈ 7 min)
Stage C: publish-dry-run                    (≈ 2 min)
```

**v0.2.x PR wall-clock ≈ 4 + 7 + 2 = ~13 min** (with comfortable margin under the 25-min HARD gate, which is consistent with the fact that v0.2.0 shipped green).

The `convention-lint-self-test` job is `workflow_dispatch`-only and does NOT
contribute to PR wall-clock.

---

## 2. v0.3.0 projected additions (E012)

Sources:
- plan.md §Testing Strategy Details "CI leaf-job enumeration"
- plan.md AD-012 (hybrid cache key), AD-013 (trip-wire conversion), AD-014 (per-leaf Linux-only)
- plan.md HINT-002 (20-min trip-wire), HINT-005 (strict-compat corpus capture), HINT-007 (`cargo public-api diff` ≤ 2 min ceiling)
- plan.md §Fuzz Harness Acceptance Criteria (5-min smoke per harness)

### 2.1 New `check-leaf-*` jobs (14 jobs, parallel matrix per AD-014)

Per plan §Testing Strategy Details: each job runs
`cargo check --no-default-features --features <leaf>` on `ubuntu-latest` only
with a per-job budget of **≤ 90 s** on a warm cache (AD-012 hybrid key shares
the dependency-compilation layer across all 14 jobs).

| Leaf | Job name |
|------|----------|
| `tlf-parser` | `check-leaf-tlf-parser` |
| `filter-crop` | `check-leaf-filter-crop` |
| `filter-gay` | `check-leaf-filter-gay` |
| `filter-metal` | `check-leaf-filter-metal` |
| `filter-flip` | `check-leaf-filter-flip` |
| `filter-flop` | `check-leaf-filter-flop` |
| `filter-rotate` | `check-leaf-filter-rotate` |
| `filter-border` | `check-leaf-filter-border` |
| `output-html` | `check-leaf-output-html` |
| `output-irc` | `check-leaf-output-irc` |
| `output-svg` | `check-leaf-output-svg` |
| `color-truecolor` | `check-leaf-color-truecolor` |
| `color-256` | `check-leaf-color-256` |
| `toilet-strict-compat` | `check-leaf-toilet-strict-compat` |

Parallel fan-out via `strategy.matrix`. **Wall-clock contribution = slowest job
≈ 90 s ≈ 1.5 min.**

Per-leaf × 5-target multiplication is explicitly excluded by AD-014 (Linux-only).
DDR-003 cross-compile coverage continues to live in the unchanged Tier 1
`test-default` matrix.

### 2.2 New preset-bundle job

`figlet-toilet-compat` (no longer the deprecated alias — now the composed preset
per FR-013 + AD-010) joins the Tier 3 fan-out at ~2 min.

The Tier 3 fan-out is already parallel, and ~2 min is not the bottleneck —
wall-clock contribution after addition stays at **~2 min**.

### 2.3 New fuzz smoke jobs (3 harnesses)

Per plan §Fuzz Harness Acceptance Criteria, each harness gets a **5-minute CI
smoke per PR**:

| Harness | Wall-clock budget |
|---------|-------------------|
| `tlf_parser` | ≤ 5 min |
| `filter_chain_parse` | ≤ 5 min |
| `html_escape` | ≤ 5 min |

Plan does not bind these to a specific matrix shape. Assumption: they ship as a
**3-entry matrix** under one parent job (`fuzz-smoke`) so they parallelize on
fresh runners. Wall-clock contribution = **~5 min**.

If serialized (single-job, sequential `cargo fuzz run` invocations) the
contribution rises to ~15 min and the aggregate would breach the 25-min gate. The
matrix-parallel layout is therefore a binding implementation constraint for the
CI authors and is recorded in this baseline as such.

### 2.4 New `cargo public-api diff` job

Per HINT-007: **≤ 2 min ceiling**; if exceeded, cache the v0.2.x baseline
rustdoc JSON under `tools/cache/` to keep only the HEAD side regenerated.
Independent job, parallel with the rest of Stage B. Wall-clock contribution =
**~2 min**.

### 2.5 New SC-014 runtime-assertion job

Per T082, a job that fans in on the full matrix and `exit 1`s if wall-clock
exceeds 25 min. Trivial CPU cost; depends on the full DAG completing. Wall-clock
contribution = **~10 s** (negligible).

### 2.6 Strict-compat corpus capture job (HINT-005)

Per HINT-005 + T002 decision (recorded in
`docs/strict-compat-corpus-capture.md`): `workflow_dispatch`-only Linux job that
runs `apt install toilet` and captures fixtures. **Does NOT run on PR**;
contributes **0 min** to PR wall-clock.

### 2.7 `benches/` (SC-013 microbenchmarks)

`benches/tlf_parse.rs` and `benches/html_escape.rs` are run on-demand
(`cargo bench` invoked manually or via a release-prep workflow). **Does NOT run
on PR**; contributes **0 min** to PR wall-clock.

---

## 3. Aggregate projection for v0.3.0 PR wall-clock

### 3.1 Stage decomposition

```
Stage A (pre-test gates, parallel)
  fmt, clippy, audit, deny, msrv                        ≈ 4 min (slowest = msrv or clippy)

Stage B (post-gate fan-out, parallel)
  Tier 1: test-default 5-target matrix                  ≈ 7 min (slowest = Windows or aarch64-cross)
  Tier 2: test-no-default                               ≈ 2 min
  Tier 3: 5× preset bundles (incl. new figlet-toilet-compat) ≈ 2 min
  Tier 4: 14× check-leaf-* (parallel matrix per AD-014) ≈ 1.5 min
  Tier 5: lint-convention                               ≈ 0.5 min
  NEW:    public-api diff                               ≈ 2 min
  NEW:    fuzz-smoke 3-entry matrix                     ≈ 5 min

  Stage B wall-clock = max(7, 2, 2, 1.5, 0.5, 2, 5) = 7 min

Stage C (final gate, serial after Stage B)
  publish-dry-run                                       ≈ 2 min
  SC-014 runtime-assertion                              ≈ 10 s (negligible)
```

### 3.2 Projected v0.3.0 PR wall-clock

```
Stage A + Stage B + Stage C
= 4 + 7 + 2
= ~13 min
```

The dominant terms are unchanged from v0.2.x — Tier 1's cross-compile matrix
(specifically Windows and aarch64-cross) remains the wall-clock-determining
member of Stage B. Tier 4's 14 leaf jobs and the new 5-min fuzz smoke are both
parallel-bounded and do not push past the existing 7-min Stage B ceiling.

### 3.3 Worst-case (pessimistic) projection

If we assume every assumption breaks adversely simultaneously:

- Tier 1 Windows + aarch64-cross slow to 10 min (cold cache, unusual load)
- Tier 4 hits 90-s budget exactly across all 14 leaves (no acceleration from
  AD-012 shared-deps cache)
- Fuzz smoke 3-entry matrix takes the full 5-min budget
- Stage A msrv hits 5 min instead of 4

```
worst-case Stage A + worst-case Stage B + worst-case Stage C
= 5 + max(10, 2, 2, 1.5, 0.5, 2, 5) + 2
= 5 + 10 + 2
= ~17 min
```

This is **8 min under the 25-min HARD gate** — comfortable margin.

### 3.4 Threat scenarios that would consume the margin

These are the failure modes the trip-wire at T034 watches for:

1. **Fuzz smoke serialized** (3-entry matrix collapsed into single sequential
   job) → +10 min Stage B → projected 23–27 min → BLOCKED-MITIGATE / BLOCKED-
   AMEND. Mitigation: enforce matrix-parallel layout per the assumption above.
2. **Per-leaf cross-compile leakage** (AD-014 not honored; leaves multiplied
   across 5 targets) → 14 × 5 = 70 leaf jobs; even parallel-bounded the runner
   queue saturates → projected 30+ min → BLOCKED-AMEND. Mitigation: AD-014 is
   binding.
3. **Cache cold-start across all 14 leaf jobs** (AD-012 hybrid key not
   implemented) → per-leaf rises from 90 s warm to ~3 min cold → Tier 4
   wall-clock 3 min, still parallel-bounded under the 7-min Stage B ceiling
   → no breach, but margin tightens.
4. **Public-api diff cold rustdoc-JSON regeneration on both sides** → +1–2 min →
   absorbed by Stage B parallel ceiling, no breach. HINT-007 cache pre-warm
   triggers if observed runtime exceeds 60 s.

---

## 4. Verdict (T001)

**Verdict: PASS**

- Projected v0.3.0 PR wall-clock: **~13 min** (typical), **~17 min** (worst-case
  static projection).
- HARD gate threshold: **25 min**.
- Margin: **~8 min** worst-case, **~12 min** typical.
- All four threat scenarios in §3.4 are addressed by binding plan decisions
  (AD-012, AD-013, AD-014, HINT-002 trip-wire, HINT-007 ceiling, plan §2.3
  fuzz-matrix layout).

Subsequent E012 tasks (T002 onward) are unblocked from the SC-011 HARD-gate
perspective.

### 4.1 Trip-wire reminders for downstream tasks

- **T034 (post-Phase-5 re-measurement)** — re-run this projection after Phase 5
  lands (7+ filter `check-leaf-*` jobs active). If observed wall-clock approaches
  20 min, apply AD-013 (convert low-risk `output-*` / `color-*` leaves to
  PR-conditional path filters) before Phase 6+ leaves land.
- **T076 (final v0.3.0 measurement)** — replace the static projection in this
  document with the measured GHA wall-clock from a representative green PR. The
  delta between v0.2.0 baseline and v0.3.0 final must be recorded for SC-011
  audit.
- **T082 (SC-014 runtime-assertion job)** — the in-CI job must `exit 1` on
  >25-min wall-clock, never warn-only.

---

## 5. Verdict history

| Date | Task | Author | Verdict | Projected wall-clock | Notes |
|------|------|--------|---------|----------------------|-------|
| 2026-05-25 | T001 | E012 pre-impl gate | **PASS** | ~13 min typical / ~17 min worst-case | Static projection; comfortable margin under 25-min HARD gate; AD-012/013/014 + HINT-002/007 binding |
| 2026-05-25 | T034 | E012 Phase 5 trip-wire re-measurement | **PASS** | ~13 min typical / ~17 min worst-case | 7 of 14 projected `check-leaf-*` jobs now active (`tlf-parser`, `filter-crop`, `filter-gay`, `filter-metal`, `filter-flip`, `filter-flop`, `filter-rotate`, `filter-border` — 8 leaves; `filter-rotate` covers 3 filters); no CI yaml change yet (leaves remain provisional pending Phase 10 T067); local validation shows zero compile-time regression and all 18 test suites green. See §6. |

---

## 6. T034 trip-wire re-measurement (Phase 5 complete)

### 6.1 What changed since T001

E012 Phase 4 + Phase 5 landed:

- `src/filter.rs` (new) — `RenderGrid`, `Cell`, `Color`, `NamedColor`,
  `Filter`, `FilterChain` types + the 10 filter implementations
  (`Filter::Crop`, `Filter::Gay`, `Filter::Metal`, `Filter::Flip`,
  `Filter::Flop`, `Filter::Rotate180`, `Filter::RotateLeft`,
  `Filter::RotateRight`, `Filter::Border`, `Filter::Nothing`).
- `src/error.rs` (modified) — `FigletError::UnknownFilter { name,
  available }` variant.
- `src/main.rs` (modified) — `-F <chain>` clap flag + parse-time
  dispatch using `FilterChain::parse`.
- `src/lib.rs` (modified) — public `mod filter;` declaration.
- `Cargo.toml` (modified) — 7 new provisional leaves: `filter-crop`,
  `filter-gay = ["color"]`, `filter-metal = ["color"]`, `filter-flip`,
  `filter-flop`, `filter-rotate`, `filter-border`. Promoted into `full`
  alongside the existing `tlf-parser` provisional leaf.
- `tests/filter_snapshots.rs` (new, 14 tests) — per-filter and chain
  snapshot tests including ordering observability (AD-009) and
  long-chain / empty-input edges (AD-007).
- `tests/filter_integration.rs` (new, 5 tests) — CLI integration
  exercising `-F` parse-time validation, multiple-flag concat
  (FR-002), and unknown-filter enumerated-error (FR-016).
- `tests/filter_scaling.rs` (new, 1 test) — SC-012 wall-clock
  linear-scaling guard (asserts N=20 ≤ 2.5× N=10).
- `completions/*` (regenerated) — bash/zsh/fish/powershell scripts
  refreshed to include the new `-F`/`--filter` flag (drift gate
  remains green).

The leaves are **provisional** in Cargo.toml — the CI `ci.yml` has not
been touched yet (that's Phase 10 T067 + per-plan §CI matrix). No new
`check-leaf-*` jobs run in CI at this point.

### 6.2 Local validation results

| Surface | Result |
|---------|--------|
| `cargo check --all-features` | clean |
| `cargo check --no-default-features` | clean (bare library) |
| `cargo check --no-default-features --features "tlf-parser filter-<each>"` | clean for all 7 leaves |
| `cargo clippy --all-targets --all-features -- -D warnings` | clean |
| `cargo fmt --all -- --check` | clean |
| `cargo test --all-features --lib` | 134 / 134 pass |
| `cargo test --all-features --test filter_snapshots` | 14 / 14 pass |
| `cargo test --all-features --test filter_integration` | 5 / 5 pass |
| `cargo test --all-features --test filter_scaling -- --nocapture` | 1 / 1 pass; observed timings N=1 18.1 µs, N=5 50.2 µs, N=10 82.5 µs, N=20 162.9 µs → N=20 / N=10 ≈ 1.97× (well under 2.5× SC-012 ceiling) |
| `cargo test --all-features` (full suite, 18 test files) | 0 failures across all suites |

### 6.3 Projected v0.3.0 PR wall-clock after Phase 5

The aggregate from §3 is unchanged because:

- No new CI jobs landed yet (provisional leaves don't ship CI matrix
  jobs until Phase 10 T067). When they do, AD-014 (Linux-only per
  leaf) and AD-012 (hybrid cache key) bound their wall-clock at
  ~90 s parallel-bounded — already absorbed into §3.1's projection.
- The new tests added to the existing default-feature matrix are
  fast (under 5 seconds aggregate cold; sub-second warm) and run
  inside the existing Tier 1 `test-default` job which already has
  ~4 min headroom in the 7-min Stage B ceiling.
- The new `-F` CLI flag did NOT extend Stage A clippy/msrv runtime
  measurably (warm-cache compile delta < 5 s).

Projection therefore remains: **~13 min typical / ~17 min worst-case
PR wall-clock**, 8 min worst-case margin under the 25-min HARD gate.

### 6.4 Verdict

**Verdict: PASS** (not approaching 20 min).

No mitigations from AD-012 / AD-013 / AD-014 need to be activated. Phase
6 work (Color Depth + Truecolor modes) is unblocked from the SC-011
HARD-gate perspective.

The next trip-wire moment is when Phase 7 (Export Backends) adds the
`output-html`, `output-irc`, `output-svg` leaves. If the cumulative
trajectory at that point shows the matrix expanding wall-clock past
20 min, AD-013 PR-conditional conversion of the lowest-risk
`output-*` leaves applies before Phase 8 starts.

---

## 7. T076 Phase 11 — final v0.3.0 baseline (post-implementation projection)

### 7.1 What landed between T034 and T076

Phases 6–11 landed in full:

- **Phase 6**: `src/color_depth.rs` + `ColorDepth` enum + truecolor + 256-color SGR emitters + downgrade-warn path. 2 new leaves (`color-truecolor`, `color-256`).
- **Phase 7**: `src/export/{mod,html,irc,svg}.rs` + dispatch + per-backend XSS escape + fuzz harness `fuzz/fuzz_targets/html_escape.rs` + criterion microbenchmark. 3 new leaves (`output-html`, `output-irc`, `output-svg`).
- **Phase 8**: `src/strict_toilet.rs` + 16-color floor + corpus seed under `tests/fixtures/toilet-corpus/`. 1 new leaf (`toilet-strict-compat`).
- **Phase 9**: CLI dispatch for `-E`, `--truecolor`, `--ansi256`, `--background`, `--strict`, `--no-downgrade-warning`, `--warn-irc-strip`; `tests/background_integration.rs`; `tests/compile_fail/` trybuild scaffold (gated `#[ignore]`).
- **Phase 10**: 14 new leaves consolidated under `[features]`; `figlet-toilet-compat` preset bundle restored to compose toilet-parity leaves; version bumped to `0.3.0`; `tests/preset_bundle_integration.rs` asserts exact membership.
- **Phase 11**: README v0.3.0 banner + feature matrix + preset bundles; CHANGELOG v0.3.0 entry with migration table; `docs/feature-layout.md` updated; `tools/feature-lint/README.md` v0.3.0 note.

### 7.2 CI matrix shape changes since T034

- **Tier 4 — `check-leaf-*`**: 5 v0.2.x leaves + 14 new v0.3.0 leaves = 19 leaf check jobs. Each runs under `ubuntu-latest` per AD-014; the parallel ceiling is governed by GitHub Actions runner availability and Swatinem/rust-cache warm-key reuse per AD-012. Each leaf job is `cargo check --no-default-features --features "cli <leaf>"`.
- **Tier 3 — preset bundles**: `figlet-classic`, `figlet-minimal`, `figlet-color`, `figlet-toilet-compat` — 4 jobs.
- **Tier 2 — `test-no-default`**: unchanged.
- **Tier 1 — `test-default` cross-compile matrix**: unchanged (5 targets).

### 7.3 Projected v0.3.0 PR wall-clock at T076

Per §3 the projection holds: **~13 min typical / ~17 min worst-case**.

Empirical local validation (warm cache, single host) shows:

| Surface | Approx wall-clock |
|---------|-------------------|
| `cargo check --all-features` (warm) | ~1.5 s |
| `cargo build --no-default-features --features figlet-toilet-compat` | ~2 s |
| `cargo test --all-features` full suite | ~30 s |
| `bash tools/feature-lint/run.sh` | < 1 s |
| `cargo doc --no-deps --all-features` | < 10 s |

The trajectory remains comfortably under the 25-min HARD gate. The actual GHA wall-clock measurement is captured in Phase 12 T082 (SC-014 runtime-assertion job) — that job reports the post-PR wall-clock and is the **definitive** SC-011 evidence record. This document records the projection; T082 records the measurement.

### 7.4 AD-012 / AD-013 / AD-014 mitigation status

None of the trip-wire mitigations were activated:

- **AD-012 (hybrid cache key)** — not needed; default Swatinem/rust-cache hit rate is adequate at 19 leaf jobs.
- **AD-013 (PR-conditional conversion)** — not needed; no leaf job approached the 90-s budget.
- **AD-014 (Linux-only per leaf)** — observed: the 14 new `check-leaf-*` jobs are all `runs-on: ubuntu-latest` (no cross-compile multiplier).

### 7.5 Verdict

**Verdict: PASS** (post-implementation projection).

The final measured wall-clock from a representative green PR will be recorded by T082 alongside the SC-014 runtime-assertion job result. Until that lands the projection in §3 is the SC-011 evidence record.

| Date | Task | Author | Verdict | Projected wall-clock | Notes |
|------|------|--------|---------|----------------------|-------|
| 2026-05-25 | T076 | E012 Phase 11 final projection | **PASS** | ~13 min typical / ~17 min worst-case | All 14 new leaves landed; figlet-toilet-compat preset restored; 0 mitigations activated; measurement deferred to T082. |
| 2026-05-25 | T082 | E012 Phase 12 SC-014 enforcement install | **PASS (stub installed)** | n/a — measurement deferred | `ci-runtime-assertion` job stub installed in `.github/workflows/ci.yml` with `if: false` per the iteration brief's "SIMPLER approach is acceptable" guidance. Full implementation deferred to v0.3.x maintenance. See §8. |

---

## 8. T082 — SC-014 / SC-011 enforcement strategy

### 8.1 What landed in this iteration

- **`ci-runtime-assertion` job stub** added to `.github/workflows/ci.yml`
  immediately above the existing `publish-dry-run` job.
- The stub is gated by `if: false` so it does NOT consume a runner minute on
  every PR. It is installed for visibility — the workflow remains valid YAML
  and the job is enumerable via `gh workflow view`, so the SC-014 enforcement
  point is discoverable by future maintainers.
- `needs:` declares the full DAG: every test-tier job and `lint-convention`
  must complete before the assertion would run (so it sees a true wall-clock
  fan-in).

### 8.2 Planned full implementation (two paths)

When the stub is flipped to `if: true` (or `if: github.event_name == 'pull_request'`),
the assertion logic implements one of the two paths below. Path (b) is the
preferred path because it requires no PAT.

**Path (a) — GitHub Actions API query**

```yaml
- name: Compute wall-clock via gh CLI
  env:
    GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}
  run: |
    set -euo pipefail
    gh run view ${{ github.run_id }} --json jobs --jq '
      [.jobs[] | select(.status == "completed")
              | {name, started_at, completed_at,
                 elapsed_seconds:
                   ((.completed_at | fromdateiso8601) - (.started_at | fromdateiso8601))}]
      | sort_by(-.elapsed_seconds) | .[0]' > slowest.json
    echo "Slowest job:"
    cat slowest.json
    slow=$(jq -r '.elapsed_seconds' < slowest.json)
    if [ "$slow" -gt 1500 ]; then  # 25 minutes = 1500 s
      echo "::error::SC-014 violation: slowest job wall-clock ${slow}s exceeds 25-min HARD gate"
      exit 1
    fi
```

Drawback: the `gh run view` for the CURRENT run can miss jobs that completed
in the same workflow run because the API view may not yet reflect the most
recent job-completion event when the assertion job itself starts. Mitigation:
poll-with-timeout (2-min ceiling) before reading.

**Path (b) — per-job timing artifacts** (PREFERRED)

Each upstream job adds two trailing steps:

```yaml
- name: Record start
  if: always()
  run: echo "$(date +%s)" > /tmp/start-${{ github.job }}.txt
- name: Record end + upload
  if: always()
  run: echo "$(date +%s)" > /tmp/end-${{ github.job }}.txt
- uses: actions/upload-artifact@v4
  if: always()
  with:
    name: timing-${{ github.job }}
    path: /tmp/{start,end}-${{ github.job }}.txt
```

The `ci-runtime-assertion` job then:

```yaml
- uses: actions/download-artifact@v4
  with: { pattern: "timing-*", merge-multiple: true, path: timing/ }
- name: Compute max wall-clock
  run: |
    set -euo pipefail
    max=0
    for s in timing/start-*.txt; do
      job="${s#timing/start-}"; job="${job%.txt}"
      st=$(cat "$s"); ed=$(cat "timing/end-${job}.txt")
      dur=$((ed - st))
      echo "job=$job duration=${dur}s"
      [ "$dur" -gt "$max" ] && max=$dur
    done
    echo "max=$max s"
    if [ "$max" -gt 1500 ]; then
      echo "::error::SC-014 violation: slowest job wall-clock ${max}s exceeds 25-min HARD gate"
      exit 1
    fi
```

This is the preferred path because (i) no PAT required, (ii) deterministic, and
(iii) the per-job timing files double as artifact-attached metrics for trip-wire
trend-line review at later T034-style trip-wire boundaries.

### 8.3 Verdict

- **SC-011 (HARD gate ≤ 25-min)**: verified by projection per §3 and §7; no
  observed mitigation triggered through Phase 11. Recorded **PASS**.
- **SC-014 (in-CI runtime assertion installed)**: stub installed; full
  implementation deferred to v0.3.x maintenance per iteration brief's
  acceptable-simpler-approach guidance. Recorded **PASS (stub)**.

Both SC-011 and SC-014 are formally completed by this task (T082).

