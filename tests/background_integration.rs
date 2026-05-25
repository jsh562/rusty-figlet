//! E012 Phase 9 — `--background=<color>` integration tests (T064).
//!
//! Covers SC-007: typed background color parsing happens BEFORE export
//! emit so XSS-via-color-spec is impossible per spec Edge Cases. Tests
//! drive the binary via `assert_cmd` to exercise the full CLI surface.

#![cfg(all(feature = "cli", feature = "color"))]

use assert_cmd::Command;

fn cmd() -> Command {
    let mut c = Command::cargo_bin("rusty-figlet").expect("binary built");
    // Stable env: no terminal probing, no NO_COLOR override.
    c.env_clear();
    #[cfg(windows)]
    c.env("PATH", std::env::var("PATH").unwrap_or_default());
    c
}

// ---------------------------------------------------------------------------
// SC-007 — parse-time rejection of injection payloads (the security claim).
// ---------------------------------------------------------------------------

#[test]
fn background_rejects_ansi_escape_injection() {
    // ESC byte (0x1B) — the canonical SGR injection seed.
    let payload = "\x1b[Hclear";
    cmd()
        .args(["--background", payload, "X"])
        .assert()
        .failure();
}

#[test]
fn background_rejects_shell_metachars() {
    cmd()
        .args(["--background", "red; rm -rf /", "X"])
        .assert()
        .failure();
}

#[test]
fn background_rejects_partial_hex() {
    cmd().args(["--background", "#xx", "X"]).assert().failure();
    cmd().args(["--background", "#12", "X"]).assert().failure();
    cmd()
        .args(["--background", "#1234567", "X"])
        .assert()
        .failure();
}

#[test]
fn background_rejects_newline() {
    cmd()
        .args(["--background", "red\nINJECT", "X"])
        .assert()
        .failure();
}

#[test]
fn background_rejects_unknown_named_color() {
    cmd().args(["--background", "puce", "X"]).assert().failure();
}

// ---------------------------------------------------------------------------
// Accepts the documented set.
// ---------------------------------------------------------------------------

#[test]
fn background_accepts_named_color() {
    // Without `-E`, the CLI renders normally — we only assert exit success
    // (the spec is parsed, the color carries through type-safely).
    cmd().args(["--background", "red", "X"]).assert().success();
}

#[test]
fn background_accepts_bright_variant() {
    cmd()
        .args(["--background", "bright_blue", "X"])
        .assert()
        .success();
}

#[test]
fn background_accepts_rrggbb_hex() {
    cmd()
        .args(["--background", "#1A2B3C", "X"])
        .assert()
        .success();
}

#[test]
fn background_accepts_lowercase_hex() {
    cmd()
        .args(["--background", "#abcdef", "X"])
        .assert()
        .success();
}
