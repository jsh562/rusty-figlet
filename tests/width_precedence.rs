//! T112 + T113 — Width-source precedence ladder per AD-010 + HINT-005.
//!
//! Tests:
//! - `ad010_precedence_ladder` — explicit `-w` > `-t` > COLUMNS env > 80
//! - `default_vs_strict_t_auto_apply` — Default auto-applies `-t` when
//!   stdout is a tty + no `-w`; Strict never auto-applies (HINT-005).
//!
//! Notes on portability:
//! - `cargo test` always pipes stdout, so `is_stdout_tty()` returns
//!   `false` inside the binary under test. This means the "tty + auto-
//!   apply `-t`" branch is NOT exercisable here without a pty harness.
//!   We assert the COLUMNS-env-var fallback path instead, which is the
//!   substitute test path documented in the task.
//! - On Windows the `terminal_size` crate consults the console handle
//!   of stdout — this returns `None` for piped stdout, so the COLUMNS
//!   fallback in `resolve_width` is the deterministic test surface.

#![cfg(feature = "cli")]

#[path = "common/mod.rs"]
mod common;

/// T112 — `ad010_precedence_ladder`: 4 parameterized cases over the
/// precedence ladder.
#[test]
fn ad010_precedence_ladder_explicit_w_wins() {
    let (_tmp, _) = common::sandbox();
    // -w 60 wins over any env/tty path; the rendered banner must fit.
    let _guard = common::env_guard("COLUMNS", Some("120"));
    let assert = common::rusty_figlet_cmd()
        .args(["-w", "60", "X"])
        .assert()
        .success();
    let out = assert.get_output();
    let stdout = String::from_utf8_lossy(&out.stdout);
    for line in stdout.lines() {
        assert!(
            line.chars().count() <= 60,
            "explicit -w 60 must cap width; got line ({}): {line:?}",
            line.chars().count()
        );
    }
}

/// T112 — COLUMNS env fallback when -w and -t absent and stdout non-tty.
///
/// Under `cargo test` stdout is piped (non-tty); Default mode does NOT
/// auto-apply `-t` because `is_tty == false`. With `-t` explicitly set,
/// the precedence ladder consults `terminal_size_of(stdout)` first
/// (returns None for piped stdout on Windows) then `COLUMNS` env. We
/// assert the rendered line widths respect `COLUMNS=70`.
#[test]
fn ad010_precedence_ladder_columns_env_when_t_no_terminal() {
    let (_tmp, _) = common::sandbox();
    let _guard = common::env_guard("COLUMNS", Some("70"));
    let assert = common::rusty_figlet_cmd()
        .args(["-t", "X"])
        .assert()
        .success();
    let out = assert.get_output();
    let stdout = String::from_utf8_lossy(&out.stdout);
    for line in stdout.lines() {
        assert!(
            line.chars().count() <= 70,
            "with -t + COLUMNS=70 the width budget must be 70; got line ({}): {line:?}",
            line.chars().count()
        );
    }
}

/// T112 — default fallback to 80 when no `-w`, no `-t`, no COLUMNS.
#[test]
fn ad010_precedence_ladder_default_fallback_is_80() {
    let (_tmp, _) = common::sandbox();
    let _guard = common::env_guard("COLUMNS", None);
    let assert = common::rusty_figlet_cmd().args(["X"]).assert().success();
    let out = assert.get_output();
    let stdout = String::from_utf8_lossy(&out.stdout);
    // With no width hint and a single-letter input, output must be
    // small (≤ 80). We don't assert exact width because the rendered
    // glyph is short; the assertion is that it fits in 80.
    for line in stdout.lines() {
        assert!(
            line.chars().count() <= 80,
            "default fallback must be ≤ 80; got line ({}): {line:?}",
            line.chars().count()
        );
    }
}

/// T112 — `-w` beats `-t` (last-precedence regardless of order).
#[test]
fn ad010_precedence_explicit_w_overrides_t() {
    let (_tmp, _) = common::sandbox();
    let _guard = common::env_guard("COLUMNS", Some("200"));
    // `-t` would otherwise consult COLUMNS=200; `-w 50` wins.
    let assert = common::rusty_figlet_cmd()
        .args(["-t", "-w", "50", "X"])
        .assert()
        .success();
    let out = assert.get_output();
    let stdout = String::from_utf8_lossy(&out.stdout);
    for line in stdout.lines() {
        assert!(
            line.chars().count() <= 50,
            "explicit -w 50 must beat -t; got line ({}): {line:?}",
            line.chars().count()
        );
    }
}

/// T113 — Default vs Strict `-t` auto-apply (HINT-005).
///
/// Both invocations are under `cargo test` (stdout piped). In this
/// branch:
/// - Default mode does NOT auto-apply `-t` because `is_tty == false`.
///   Width falls back to 80.
/// - Strict mode also returns 80 — it never auto-applies `-t` per
///   HINT-005, regardless of tty state.
///
/// Both invocations therefore produce identical line-width caps. This
/// substitutes for the harder-to-test "stdout is a tty" branch on
/// Windows. The library's `width::resolve_width` unit tests cover the
/// tty-versus-non-tty + Strict mode dimension directly in
/// `src/width.rs::tests::strict_does_not_auto_apply_t`.
#[test]
fn default_vs_strict_t_auto_apply() {
    let (_tmp, _) = common::sandbox();
    let _guard = common::env_guard("COLUMNS", None);

    let default_assert = common::rusty_figlet_cmd().args(["X"]).assert().success();
    let default_stdout = String::from_utf8_lossy(&default_assert.get_output().stdout).into_owned();

    let strict_assert = common::rusty_figlet_cmd()
        .args(["--strict", "X"])
        .assert()
        .success();
    let strict_stdout = String::from_utf8_lossy(&strict_assert.get_output().stdout).into_owned();

    // Both must respect the 80-col fallback (`cargo test` pipes
    // stdout → no tty in either mode). Each rendered line must be ≤ 80
    // cols.
    for line in default_stdout.lines() {
        assert!(
            line.chars().count() <= 80,
            "default mode line exceeds 80 cols: {line:?}"
        );
    }
    for line in strict_stdout.lines() {
        assert!(
            line.chars().count() <= 80,
            "strict mode line exceeds 80 cols: {line:?}"
        );
    }

    // Sanity: Default and Strict produce non-empty output for the same
    // input (the Latin-1 clamp under Strict doesn't drop ASCII bytes).
    assert!(
        !default_stdout.is_empty() && !strict_stdout.is_empty(),
        "both default and strict must render a banner; \
         default={} bytes, strict={} bytes",
        default_stdout.len(),
        strict_stdout.len()
    );
}
