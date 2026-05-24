# Strict-Mode Snapshot Suite — upstream `figlet 2.2.5`

This directory hosts the captured upstream `figlet 2.2.5` stdout fixtures that drive `tests/compat_strict.rs` byte-equality assertions per plan.md §Strict-Mode Snapshot Capture & Coverage + spec SC-005 + SC-006 + FR-041. Provenance lives in `PROVENANCE.txt` (per HINT-004 + T020).

## Refresh Policy

Snapshot refresh is a **deliberate maintenance step on upstream version bump**, NOT a silent CI refresh. CI consumes the committed snapshots as-is; any change to snapshot bytes MUST land in a commit whose message carries the `[snapshot-refresh]` tag so the diff is explicit and reviewable. CI does not regenerate snapshots automatically.

When upstream `figlet` bumps to a new release (e.g., 2.2.6 or 3.0.0), the maintainer:

1. Updates `PROVENANCE.txt` to record the new upstream version, capture host, capture date, and the upstream source location.
2. Renames this directory from `upstream_v2_2_5/` to `upstream_vX_Y_Z/` to match the new version. Also update `tests/common/mod.rs::strip_for_snapshot` if the program-name token in any captured stderr changes.
3. Re-runs the capture procedure from `PROVENANCE.txt` against the new upstream.
4. Commits the new snapshots + manifests + provenance under a single `[snapshot-refresh] upstream → vX.Y.Z` commit.

## Minimum Matrix (CHK001 + CHK002 + CHK003)

The committed Strict-mode snapshot scenarios MUST cover at minimum:

### Base matrix: 12 fonts × 5 inputs = 60 fixtures (CHK001)

For each of the 12 bundled fonts (`standard`, `slant`, `small`, `big`, `mini`, `banner`, `block`, `bubble`, `digital`, `lean`, `script`, `shadow`), capture 5 input categories:

| Category | Input | Notes |
|----------|-------|-------|
| single-word | `Hello` | Baseline glyph-by-glyph render |
| multi-word-with-space | `Hello World` | Tests inter-word spacing + smush across word boundary |
| empty | `` (empty string) | Tests empty-banner behavior |
| mixed-case | `MiXeD` | Tests upper/lower glyph coverage |
| max-length-near-80-cols | `Lorem ipsum dolor sit amet consectetur` (≈40 chars; renders ≈70-cols-wide in `standard`) | Tests near-default-width boundary |

### Layout-flag permutations: ≥20 on `standard.flf` (CHK003)

Each of `-k`/`-W`/`-S`/`-s`/`-o` solo + paired with `-c`/`-l`/`-r` + `-m N` for N ∈ {0, 24, 63}. Example permutations:

- `-k Hello`
- `-W Hello`
- `-S Hello`
- `-s Hello`
- `-o Hello`
- `-c Hello`
- `-l Hello`
- `-r Hello`
- `-k -c Hello`
- `-W -l Hello`
- `-S -r Hello`
- `-m 0 Hello`
- `-m 24 Hello`
- `-m 63 Hello`
- `-w 60 Hello`
- `-w 120 Hello`
- `-c -l Hello` (last-wins: `-l`)
- `-r -l -c Hello` (last-wins: `-c`)
- `-W -k Hello` (last layout-class wins: `-k`)
- `-k -W Hello` (last layout-class wins: `-W`)

## Snapshot File Layout (CHK004 + CHK005)

Each scenario has:

```
outputs/<scenario_id>.stdout      ← captured upstream stdout (LF eol; `.gitattributes` enforces)
outputs/<scenario_id>.stderr      ← captured upstream stderr (only when non-empty)
manifests/<scenario_id>.toml      ← scenario metadata (CHK005)
```

The manifest format:

```toml
input = "Hello World"
font = "standard"
args = ["-f", "standard", "-w", "80", "Hello World"]
stdout_path = "outputs/standard_multi_word.stdout"
exit_code = 0
upstream_version = "2.2.5"
capture_host = "Debian 12 (Bookworm) x86_64"
capture_date = "2026-MM-DD"
```

## Snapshot-Strip Helper

`tests/common/mod.rs::strip_for_snapshot(raw: &[u8]) -> Vec<u8>` is the SOLE canonical snapshot-strip helper. It performs the single literal substitution `figlet:` → `rusty-figlet:` in any captured stderr bytes (per HINT-004). Per-test ad-hoc regexes are FORBIDDEN — any new normalization MUST be added to the central helper with an accompanying rustdoc note explaining why.

## Status (Phase 1 Setup)

T085 (snapshot capture) is **DEFERRED** because the development environment cannot run upstream `figlet 2.2.5`. Dependent tests T086 (`strict_byte_equal_60_base_snapshots`) and T087 (`strict_byte_equal_20_layout_perm_snapshots`) are likewise deferred per `tasks.md` Upstream Dependency Caveat. Polish-phase work re-engages the capture pass on a Linux host before the v0.1.0 release.
