# Strict-Compat Corpus Capture — `rusty-figlet` `toilet-strict-compat`

This document records the decision for HOW the `toilet 0.3-1` strict-compat
byte-equal corpus (per spec 00012 SC-006 + HINT-005 + Plan Watch Item #2) is
captured and committed to `tests/fixtures/toilet-corpus/`.

The decision must be recorded BEFORE any strict-compat implementation task
(Phase 8) begins. T002 of the E012 task list owns this artifact.

---

## 1. Decision

**Chosen approach: (b) CI-driven `workflow_dispatch` capture job on a Linux GHA
runner.**

A GitHub Actions workflow named `capture-toilet-corpus` (separate file from
`ci.yml` to keep PR-time CI clean) is triggered manually by the maintainer via
`workflow_dispatch`. The job:

1. Runs on `ubuntu-latest` (Ubuntu 22.04 LTS or 24.04 LTS, whichever is the
   current default at run time).
2. `apt-get update && apt-get install -y toilet` to obtain the upstream package
   (toilet 0.3-1 on Ubuntu LTS at the time of this writing).
3. Iterates over a manifest of `(input, filter-chain)` pairs and runs
   `toilet -F <chain> <input>` for each, capturing stdout bytes verbatim into
   `tests/fixtures/toilet-corpus/<category>/<test-id>.bytes`.
4. Emits a sibling `manifest.yaml` recording the source command, capture
   timestamp, `toilet --version` output, `uname -a` host kernel string, and the
   `apt-cache policy toilet` installed-version line per fixture.
5. Commits the captured fixtures and `manifest.yaml` to a new branch
   (`corpus-refresh-<YYYY-MM-DD>`) and opens a PR for maintainer review.

The PR is reviewed for embedded attacker payloads, runs the standard PR-time
CI (which now includes the strict-compat byte-equality tests against the new
corpus), and on green merges to `main`.

## 2. Rationale (why CI-driven over one-time maintainer-machine capture)

The two options under consideration per Plan Watch Item #2:

| Option | Pros | Cons |
|--------|------|------|
| (a) One-time maintainer-machine capture with documented procedure | Lowest setup cost; maintainer fully controls capture environment | Not reproducible by anyone else; hard to audit; corpus refresh requires the same maintainer + same machine; bus factor = 1 |
| (b) CI-driven `workflow_dispatch` Linux capture | Reproducible by anyone with repo write access; capture environment is the well-known `ubuntu-latest` image; corpus refresh is a single button click; audit trail in GHA logs; bus factor = N (number of maintainers) | Slight overhead to write the workflow once; depends on Ubuntu APT continuing to ship `toilet` (low risk — `toilet` has been in Debian/Ubuntu since the mid-2000s) |

Option (b) wins on three Rusty-portfolio principles:

- **Maintainability before feature breadth (project-instructions §I)** — the
  single-maintainer-at-a-time bus factor of option (a) compounds as a
  maintenance liability when the corpus needs refresh (e.g., on Ubuntu shipping
  a newer toilet version).
- **Honest gap accounting (§VI)** — reproducibility is the point of
  byte-equality testing; an opaque "trust me, I captured these on my Linux box"
  approach undermines the audit trail.
- **Cross-platform by default (§IV)** — although the corpus is captured on
  Linux, the resulting byte fixtures are platform-agnostic; CI-driven capture
  removes the host-specific bottleneck.

Option (b) also aligns with HINT-005's amended guidance to keep the capture job
out of the PR-time matrix (`workflow_dispatch` only), so it contributes 0 min
to the SC-011 25-min HARD-gate budget recorded in `docs/ci-runtime-baseline.md`.

## 3. Capture environment specification

| Field | Value |
|-------|-------|
| Workflow file | `.github/workflows/capture-strict-compat-corpus.yml` (created at T052) |
| Trigger | `workflow_dispatch` only — never `push` or `pull_request` |
| Runner | `ubuntu-latest` (binding; do not switch to a self-hosted runner — that defeats the audit-friendliness) |
| Package source | `apt-get install -y toilet` from the default Ubuntu archive |
| Capture host kernel | Recorded per-run via `uname -a` into `manifest.yaml` |
| Toilet version | Recorded per-run via `toilet --version` into `manifest.yaml` |
| Locale | `LC_ALL=C.UTF-8` (matches `ci.yml` env binding for byte-equality determinism) |
| Output encoding | Raw bytes (`bash` redirect `> file.bytes`, NEVER `cat`-piped through any UTF-8 normaliser) |

## 4. Manifest schema (per-fixture entry)

`tests/fixtures/toilet-corpus/manifest.yaml` is a YAML list. Each entry:

```yaml
- id: ascii/hello_world_crop
  category: ascii
  input: "hello world"
  filter_chain: "crop"
  command: "toilet -F crop 'hello world'"
  captured_at: "2026-05-25T12:34:56Z"
  toilet_version: "toilet 0.3-1"
  host_kernel: "Linux runner 5.15.0-azure ..."
  apt_policy: "Installed: 0.3-1ubuntu1"
  bytes_path: "ascii/hello_world_crop.bytes"
  bytes_sha256: "<hex digest>"
  review_status: "reviewed-no-attacker-payload"
```

The `bytes_sha256` field gives a tamper-evident anchor; the `review_status`
field is set to `"reviewed-no-attacker-payload"` by the maintainer reviewing
the corpus-refresh PR per §5 below, NOT by the capture job itself.

## 5. Corpus review-for-XSS / attacker-payload procedure

Every fixture is reviewed by the merging maintainer BEFORE the corpus-refresh
PR merges. The review covers:

1. **HTML/SVG escape probes** — verify no fixture under `ascii/`,
   `utf8-multicolumn/`, or `edge-width/` contains a captured byte sequence that
   would, when re-emitted by `rusty-figlet --strict`, render unescaped `<script>`
   or `javascript:` content. Strict-compat does NOT bypass the FR-014 XSS
   defense per spec Security Posture — but the corpus itself should not seed
   XSS payloads into the test surface.
2. **IRC color-span escape probes** — verify no captured fixture contains a
   sequence that could prematurely terminate or re-open an IRC `^C` color span
   (test the FR-015 non-printable strip).
3. **Pathological size** — any fixture exceeding 64 KiB must be flagged; the
   strict-compat tier is not the right place to assert mass-rendering behavior
   (use `benches/` instead).
4. **Non-deterministic captures** — if two consecutive runs of the same
   `(input, filter-chain)` produce different bytes, the fixture is dropped
   (toilet itself should be deterministic; non-determinism is a capture-env
   bug, not corpus-worthy).

The review-status field is set per-fixture; the merging maintainer signs the PR
review with a `review_status: reviewed-no-attacker-payload` comment per fixture
or a single PR-level "all N fixtures reviewed" comment.

## 6. Corpus partitioning (per plan §Strict-compat corpus categorization)

| Category | Min fixtures | Contents |
|----------|--------------|----------|
| `ascii/` | ≥ 10 | manpage examples + ASCII-only inputs |
| `utf8-multicolumn/` | ≥ 6 | CJK, emoji, combining-mark inputs |
| `edge-width/` | ≥ 6 | empty input, single-char, max-width banners, long chains n>10 |

Total minimum corpus size: **22 fixtures**. Plan §8 (T052) sets the target at
"manpage examples + ≥20 additional inputs", which matches 22+ as the lower bound.

## 7. Refresh cadence

The corpus refresh workflow is triggered:

- When Ubuntu LTS upgrades to a major release that ships a newer `toilet`
  package version (verify via `apt-cache policy toilet` in the new image).
- When a strict-compat test failure is triaged as "corpus stale" per plan
  §Strict-compat assertion criterion (rather than "implementation drift").
- Opportunistically on the maintainer's discretion — there is no scheduled
  cadence and no expiration date on captured fixtures.

The capture timestamp + toilet version + host kernel in `manifest.yaml` are the
audit anchors for "when was this last refreshed?"; no separate CHANGELOG entry
is required for corpus refresh PRs unless the byte fixtures shift in a way that
forces an implementation change.

---

## 8. Trigger procedure

The capture job is invoked manually by a maintainer:

1. Navigate to the repo's **Actions** tab on github.com.
2. Select **"Capture strict-compat corpus"** from the workflow list.
3. Click **"Run workflow"** → pick `main` (or a feature branch) → **Run workflow**.
4. The job runs on `ubuntu-latest` in ~3-5 min and:
   - Installs `toilet 0.3-1` via `apt-get install -y toilet`,
   - Asserts the version line starts with `toilet 0.3` (else fails fast),
   - Records `uname -a`, `apt-cache policy toilet`, and the capture timestamp,
   - Captures byte fixtures for the 24-entry input manifest enumerated in
     `scripts/capture-toilet-corpus.sh`,
   - Verifies determinism by running each capture twice and `cmp`-ing the
     outputs (non-deterministic captures fail the job),
   - Opens a PR titled `chore: refresh strict-compat corpus` from a new
     branch `corpus-refresh-<run-id>`.
5. Maintainer reviews the PR per §5 above (XSS / IRC / pathological-size /
   `review_status` update) and merges on green.

The first capture run (E012 T052) replaces the **synthetic seed corpus**
shipped at v0.3.0 (3 hand-crafted fixtures derived from toilet manpage
documentation; see `tests/fixtures/toilet-corpus/MANIFEST.md`) with
authoritative bytes from the upstream package. Mismatches between the
synthetic fixtures and the captured bytes surface as PR diffs — the
review process exists precisely to catch divergence.

## 9. Decision history

| Date | Task | Author | Decision | Rationale |
|------|------|--------|----------|-----------|
| 2026-05-25 | T002 | E012 pre-impl gate | **CI-driven `workflow_dispatch` Linux capture (option b)** | Reproducibility, audit-friendliness, bus factor; aligns with project-instructions §I §IV §VI; PR-CI cost = 0 min (workflow_dispatch only, not in `ci.yml` PR matrix) |
| 2026-05-25 | T052 | E012 Phase 8 iteration 5 | Workflow + capture script + synthetic seed corpus landed | Synthetic seed unblocks the strict-compat integration tests; real CI capture replaces them via the PR loop above |
