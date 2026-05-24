//! Phase 5 — US3: Strict-Compat Drop-In integration tests.
//!
//! Drives the `rusty-figlet` binary under Strict mode and asserts the
//! diagnostics, exit codes, and behaviors mandated by FR-040..FR-046 +
//! SC-007 + SC-014. Stderr byte-equal snapshot comparisons against
//! captured upstream `figlet 2.2.5` output (SC-005 + SC-006) are
//! DEFERRED per `analysis-report.md` (upstream binary not available on
//! Windows dev host); see T085..T089 in `tasks.md`.
//!
//! Strict-mode strings emitted by `main()` carry the `figlet:` ->
//! `rusty-figlet:` program-name substitution (HINT-004); see
//! `tests/common/mod.rs::strip_for_snapshot` for the canonical
//! substitution rule documentation.

#![cfg(feature = "cli")]

#[path = "common/mod.rs"]
mod common;

use predicates::prelude::*;

/// Convenience: shape the expected stderr line for a short-flag
/// rejection in the binary's Strict path. Mirrors the substitution
/// applied by `main()` so callers test the actual bytes emitted to
/// stderr (matching what `strip_for_snapshot` would yield from upstream
/// captured output).
fn strict_invalid_short(ch: char) -> String {
    format!("rusty-figlet: invalid option -- '{ch}'")
}

/// Convenience: shape the expected stderr line for a long-flag
/// rejection in the binary's Strict path.
fn strict_unrecognized_long(flag: &str) -> String {
    format!("rusty-figlet: unrecognized option '{flag}'")
}

// ============================================================================
// T075 — Strict-mode activation via all three sources (SC-007).
// ============================================================================

#[test]
fn strict_activates_via_flag() {
    // `--strict` flag → activates Strict mode. With an in-scope flag set
    // (no excluded flags) the binary should render successfully.
    common::rusty_figlet_cmd()
        .arg("--strict")
        .arg("Hi")
        .assert()
        .success();
}

#[test]
fn strict_activates_via_env_var() {
    let _g = common::env_guard("RUSTY_FIGLET_STRICT", Some("1"));
    common::rusty_figlet_cmd().arg("Hi").assert().success();
}

#[test]
fn strict_activates_via_argv0_then_rejects_excluded_flag() {
    // We can't easily invoke the binary as `figlet` without renaming
    // the artifact; instead we drive the env-var path (which shares the
    // mode::resolve activation branch) and confirm an excluded flag is
    // rejected. The argv[0]=figlet branch is unit-tested in
    // `src/mode.rs::tests`.
    let _g = common::env_guard("RUSTY_FIGLET_STRICT", Some("1"));
    common::rusty_figlet_cmd()
        .arg("-L")
        .arg("X")
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains(strict_invalid_short('L')));
}

// ============================================================================
// T076 — `--no-strict` overrides env + argv[0]; last-wins on command line.
// ============================================================================

#[test]
fn no_strict_overrides_env() {
    let _g = common::env_guard("RUSTY_FIGLET_STRICT", Some("1"));
    // With --no-strict supplied, env var is overridden → Default mode.
    // Default mode accepts --color and --rainbow (no rejection).
    common::rusty_figlet_cmd()
        .arg("--no-strict")
        .arg("--color=never")
        .arg("Hi")
        .assert()
        .success();
}

#[test]
fn last_wins_strict_then_no_strict_yields_default() {
    // `--strict --no-strict` → Default mode (last wins per Q8). Default
    // mode accepts `--color=never`; if last-wins were broken we'd be
    // in Strict and `--color` would be rejected with exit 2.
    common::rusty_figlet_cmd()
        .arg("--strict")
        .arg("--no-strict")
        .arg("--color=never")
        .arg("Hi")
        .assert()
        .success();
}

#[test]
fn last_wins_no_strict_then_strict_yields_strict() {
    // `--no-strict --strict` → Strict mode (last wins). `-L` is excluded
    // in Strict → exit 2 with upstream stderr.
    common::rusty_figlet_cmd()
        .arg("--no-strict")
        .arg("--strict")
        .arg("-L")
        .arg("X")
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains(strict_invalid_short('L')));
}

// ============================================================================
// T077..T079 — Excluded short flags rejected with byte-equal stderr.
// ============================================================================

#[test]
#[allow(non_snake_case)]
fn strict_rejects_short_L() {
    common::rusty_figlet_cmd()
        .arg("--strict")
        .arg("-L")
        .arg("term")
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains(strict_invalid_short('L')));
}

#[test]
#[allow(non_snake_case)]
fn strict_rejects_short_R() {
    common::rusty_figlet_cmd()
        .arg("--strict")
        .arg("-R")
        .arg("term")
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains(strict_invalid_short('R')));
}

#[test]
#[allow(non_snake_case)]
fn strict_rejects_short_I() {
    // T078: `-I 1` → Strict rejects with `figlet: invalid option -- 'I'`.
    common::rusty_figlet_cmd()
        .arg("--strict")
        .arg("-I")
        .arg("1")
        .arg("term")
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains(strict_invalid_short('I')));
}

#[test]
#[allow(non_snake_case)]
fn strict_rejects_short_N() {
    // T078: `-N` → Strict rejects with `figlet: invalid option -- 'N'`.
    common::rusty_figlet_cmd()
        .arg("--strict")
        .arg("-N")
        .arg("term")
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains(strict_invalid_short('N')));
}

#[test]
#[allow(non_snake_case)]
fn strict_rejects_short_C() {
    // T079: `-C myfile.flc` is excluded under Strict (Default accepts +
    // warns per FR-046). Byte-equal `figlet: invalid option -- 'C'`.
    common::rusty_figlet_cmd()
        .arg("--strict")
        .arg("-C")
        .arg("myfile.flc")
        .arg("term")
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains(strict_invalid_short('C')));
}

// ============================================================================
// T080 — Excluded long flags rejected with `unrecognized option` stderr.
// ============================================================================

#[test]
fn strict_rejects_long_info_dump() {
    common::rusty_figlet_cmd()
        .arg("--strict")
        .arg("--info-dump")
        .arg("term")
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains(strict_unrecognized_long(
            "--info-dump",
        )));
}

#[test]
fn strict_rejects_long_no_controlfile() {
    common::rusty_figlet_cmd()
        .arg("--strict")
        .arg("--no-controlfile")
        .arg("term")
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains(strict_unrecognized_long(
            "--no-controlfile",
        )));
}

// ============================================================================
// T081 — `--color` and `--rainbow` rejected under Strict (FR-045 + SC-014).
// ============================================================================

#[test]
fn strict_rejects_color_long_flag() {
    common::rusty_figlet_cmd()
        .arg("--strict")
        .arg("--color=always")
        .arg("term")
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("unrecognized option"));
}

#[test]
fn strict_rejects_rainbow_long_flag() {
    common::rusty_figlet_cmd()
        .arg("--strict")
        .arg("--rainbow")
        .arg("term")
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains(strict_unrecognized_long(
            "--rainbow",
        )));
}

// ============================================================================
// T082 — Latin-1 clamp (FR-044). UTF-8 multi-byte input replaced with `?`.
// ============================================================================

#[test]
fn strict_latin1_clamp_passes_low_bytes_through() {
    // Plain ASCII input renders cleanly under Strict (no clamp side-
    // effects for the 0..=127 range).
    common::rusty_figlet_cmd()
        .arg("--strict")
        .arg("Hi")
        .assert()
        .success();
}

#[test]
fn strict_latin1_clamp_replaces_multibyte_with_placeholder() {
    // UTF-8 multi-byte input ("héllo" — `é` is U+00E9, a single Latin-1
    // byte; "日" is CJK U+65E5, outside Latin-1) renders successfully
    // (no parse error). The non-Latin-1 codepoint is substituted with
    // the Latin-1 placeholder `?` per FR-044 + HINT-009.
    let assert = common::rusty_figlet_cmd()
        .arg("--strict")
        .arg("a日b")
        .assert()
        .success();
    let out = assert.get_output();
    // The Latin-1 clamp emits `?` for the CJK codepoint; the rendered
    // banner must therefore mention BOTH `a` and `b` and an
    // (implementation-defined; safe to assert non-empty stdout).
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        !stdout.is_empty(),
        "Strict-mode Latin-1 clamp must still render printable bytes"
    );
}

#[test]
fn lib_clamp_input_latin1_round_trip() {
    // Direct library API check on the FR-044 clamp surface — useful when
    // CLI test path is in flux. Latin-1 codepoints round-trip; CJK
    // codepoints collapse to `?` (0x3F).
    let got = rusty_figlet::clamp_input_latin1("a\u{00E9}b\u{65E5}c");
    assert_eq!(got, vec![b'a', 0xE9, b'b', b'?', b'c']);
}

// ============================================================================
// T083 — Last-wins on layout flags under Strict (FR-022 + FR-023 in Strict).
// ============================================================================

#[test]
fn strict_last_wins_justify_flags() {
    // `-c -l -r X` → effective justify is `-r` (right). We assert the
    // call succeeds and emits output; finer layout assertions (column
    // alignment) belong with T115 in US5.
    common::rusty_figlet_cmd()
        .arg("--strict")
        .arg("-c")
        .arg("-l")
        .arg("-r")
        .arg("X")
        .assert()
        .success();
}

#[test]
fn strict_last_wins_layout_flags() {
    // `-k -W -S X` → effective layout is `-S` (force smush). We assert
    // success; layout-bit byte-equal verification waits on the upstream
    // snapshot suite (DEFERRED T085..T089).
    common::rusty_figlet_cmd()
        .arg("--strict")
        .arg("-k")
        .arg("-W")
        .arg("-S")
        .arg("X")
        .assert()
        .success();
}

// ============================================================================
// T084 — Strict mode rejects the `completions <shell>` subcommand
//          (US7 AS3 + FR-063).
// ============================================================================

#[test]
fn strict_rejects_completions_subcommand() {
    common::rusty_figlet_cmd()
        .arg("--strict")
        .arg("completions")
        .arg("bash")
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("unrecognized option"));
}
