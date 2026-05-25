# feature-lint

Portfolio-wide convention compliance lint for per-port Cargo features.

This tool enforces the Cargo features convention codified in:

- `specs/adrs/0006-cargo-features-convention-for-portfolio-ports.md`
- `project-instructions.md` §Cargo Feature Surface (v1.1.0)

The lint script is implemented once in the umbrella governance repo
(`c:\claudecode\rusty\`) per ADR-0003 (Shared Automation Strategy) and AD-008
of spec 00011. Per-port CI workflows check out the umbrella and invoke this
script — there is NO copy of the lint script in any per-port repo.

## Files

- `lint.sh` — POSIX bash script implementing all 5 lint sub-rules
  (T003..T007 of spec 00011).
- `run.sh` — top-level runner that invokes every sub-check in sequence,
  accumulates violations, and prints a final summary.
- `.shellcheck` — shellcheck configuration for both scripts.
- `README.md` — this file.

## Invocation Contract

The scripts are invoked from a per-port CI workflow after a checkout of both
the port repo and the umbrella governance repo. The environment-variable
interface is:

| Variable | Required? | Meaning |
|---|---|---|
| `PORT_PATH` | Yes | Absolute path to the per-port repo root (the directory containing the port's `Cargo.toml`). |
| `UMBRELLA_PATH` | Yes | Absolute path to the umbrella governance repo root (the directory containing `tools/feature-lint/`). |
| `STRICT_MODE` | No (default: `1`) | When `1`, every violation is fatal. When `0`, violations are reported but the script exits 0. Reserved for opt-in scaffolding usage. |

Example invocation from a per-port CI workflow:

```yaml
- name: Checkout umbrella
  uses: actions/checkout@v4
  with:
    repository: jsh562/rusty
    path: umbrella

- name: Run feature-lint
  run: |
    UMBRELLA_PATH="${GITHUB_WORKSPACE}/umbrella" \
    PORT_PATH="${GITHUB_WORKSPACE}" \
    bash umbrella/tools/feature-lint/run.sh
```

## Exit Codes

| Exit code | Meaning |
|---|---|
| 0 | Compliance — all sub-checks passed. |
| 2 | At least one violation — the violated rule and offending file are named on stderr. |

`run.sh` aggregates results across sub-checks; its final exit code is the
maximum across all sub-check exit codes (i.e., 0 if every sub-check passed,
2 if any sub-check failed).

## Sub-Checks (per FR-052 of spec 00011)

1. **Required umbrellas present** (T003) — `Cargo.toml` `[features]` MUST
   declare `default`, `full`, `cli`, and `<port>-classic`.
2. **Leaf has CI matrix entry** (T004) — every declared leaf MUST have a
   `check-leaf-<leaf>` job in `.github/workflows/ci.yml`.
3. **No phantom leaves** (T005) — every declared leaf MUST be referenced by
   at least one `#[cfg(feature = "<leaf>")]` in the port's `src/` tree.
4. **README feature-matrix sync** (T006) — the README's `## Cargo Features`
   matrix MUST list every leaf with the canonical column order.
5. **CHANGELOG migration-table exhaustiveness** (T007) — the CHANGELOG's
   `## [0.2.0]` `### BREAKING-CHANGE` migration table MUST list every
   v0.1.x feature name with the canonical column order.

## Local Development

To run the lint against a local port checkout (e.g., from `c:\claudecode\rusty-figlet`):

```bash
UMBRELLA_PATH=/c/claudecode/rusty PORT_PATH=/c/claudecode/rusty-figlet \
  bash /c/claudecode/rusty/tools/feature-lint/run.sh
```

To run a single sub-check directly:

```bash
UMBRELLA_PATH=/c/claudecode/rusty PORT_PATH=/c/claudecode/rusty-figlet \
  bash /c/claudecode/rusty/tools/feature-lint/lint.sh --check required-umbrellas
```

Valid `--check` values: `required-umbrellas`, `leaf-ci-matrix`, `phantom-leaf`,
`readme-matrix`, `changelog-migration`. With no `--check` flag, `lint.sh` runs
all sub-checks (equivalent to `run.sh`).

## ShellCheck

Both scripts are written for POSIX bash 4+ and pass `shellcheck` with the
configuration in `.shellcheck`. Run linting locally with:

```bash
shellcheck tools/feature-lint/lint.sh tools/feature-lint/run.sh
```
