//! US7 (Shell completions) drift gate (T134 + T135 + T136).
//!
//! Asserts the committed `completions/` artifacts match what
//! `clap_complete` generates today. Regenerate via:
//!
//! ```sh
//! cargo run -- completions bash       > completions/rusty-figlet.bash
//! cargo run -- completions zsh        > completions/_rusty-figlet
//! cargo run -- completions fish       > completions/rusty-figlet.fish
//! cargo run -- completions powershell > completions/rusty-figlet.ps1
//! ```
//!
//! On intentional flag additions, the developer regenerates the four
//! files locally and commits the refresh in the same PR (Plan
//! §Shell Completions Drift Gate).

mod common;

use std::fs;
use std::path::PathBuf;

/// Read the committed completion file for the given shell from
/// `completions/` and normalize CRLF→LF so the comparison is platform-
/// neutral (Windows checkouts may rewrite EOLs despite the `.gitattributes`
/// `eol=lf` directive when committed via a non-Git-aware editor).
fn committed(name: &str) -> Vec<u8> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("completions")
        .join(name);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("missing committed file {path:?}: {e}"));
    normalize(&bytes)
}

/// Invoke the rusty-figlet binary's `completions <shell>` subcommand,
/// capture stdout, and normalize CRLF→LF.
fn generate(shell: &str) -> Vec<u8> {
    let output = common::rusty_figlet_cmd()
        .arg("completions")
        .arg(shell)
        .output()
        .expect("completions subcommand runs");
    assert!(
        output.status.success(),
        "completions {shell} exited non-zero: {:?}\nstderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    normalize(&output.stdout)
}

/// Strip `\r` bytes so a Windows-host run that emits CRLF line endings
/// compares cleanly against a `.gitattributes`-enforced LF-only file.
fn normalize(bytes: &[u8]) -> Vec<u8> {
    bytes.iter().copied().filter(|b| *b != b'\r').collect()
}

// ===========================================================================
// T134 — byte-equal drift gate (4 shells)
// ===========================================================================

#[test]
fn drift_bash() {
    assert_eq!(
        committed("rusty-figlet.bash"),
        generate("bash"),
        "bash completion drift — regenerate via `cargo run -- completions bash > completions/rusty-figlet.bash`"
    );
}

#[test]
fn drift_zsh() {
    assert_eq!(
        committed("_rusty-figlet"),
        generate("zsh"),
        "zsh completion drift — regenerate via `cargo run -- completions zsh > completions/_rusty-figlet`"
    );
}

#[test]
fn drift_fish() {
    assert_eq!(
        committed("rusty-figlet.fish"),
        generate("fish"),
        "fish completion drift — regenerate via `cargo run -- completions fish > completions/rusty-figlet.fish`"
    );
}

#[test]
fn drift_powershell() {
    assert_eq!(
        committed("rusty-figlet.ps1"),
        generate("powershell"),
        "powershell completion drift — regenerate via `cargo run -- completions powershell > completions/rusty-figlet.ps1`"
    );
}

// ===========================================================================
// T135 — bash completion structural sanity per US7 AS2
// ===========================================================================
//
// `--font` (`-f`) is an `Option<String>` (not a clap `ValueEnum`), so
// clap_complete emits a file-completion `compgen -f` candidate for the
// argument rather than the 12 bundled font names verbatim. Per US7 AS2
// the bash completion script MUST still be functionally complete: the
// `complete -F _rusty-figlet` registration MUST be present, every long
// flag MUST be listed under `opts`, and the `completions` subcommand
// MUST list all four supported shells. The assertion below codifies
// that structural contract so an accidental clap-derive refactor that
// silently drops a flag fails CI alongside the byte-equal drift gate.

#[test]
fn bash_completion_is_structurally_complete() {
    let bash =
        String::from_utf8(committed("rusty-figlet.bash")).expect("bash completion is valid UTF-8");

    // The `complete -F` registration that hooks the script onto the
    // `rusty-figlet` command name MUST be present (otherwise the script
    // is inert when sourced).
    assert!(
        bash.contains("complete -F _rusty-figlet") && bash.contains(" rusty-figlet"),
        "bash completion missing `complete -F _rusty-figlet … rusty-figlet` registration"
    );

    // Top-level `--font` / `-f` MUST appear in the opts list so users
    // get tab-completion of the flag itself (the value is then taken by
    // the per-arg `compgen -f` branch).
    assert!(
        bash.contains("--font") && bash.contains("-f"),
        "bash completion missing --font / -f"
    );
    assert!(
        bash.contains("--fontdir") && bash.contains("-d"),
        "bash completion missing --fontdir / -d"
    );

    // The completions subcommand itself MUST be listed and MUST expose
    // all four shells we support (clap_complete also emits `elvish`;
    // we accept-but-do-not-require it).
    assert!(
        bash.contains("completions"),
        "bash completion missing `completions` subcommand"
    );
    for shell in ["bash", "zsh", "fish", "powershell"] {
        assert!(
            bash.contains(shell),
            "bash completion missing shell name `{shell}` under `completions` subcommand"
        );
    }
}

// ===========================================================================
// T136 — Strict mode rejects `completions` (cross-ref T084) [COMPLETES SC-014]
// ===========================================================================

#[test]
fn strict_mode_rejects_completions_subcommand() {
    // SC-014 + US7 AS3: Strict mode is byte-for-byte upstream
    // `figlet 2.2.5` and therefore does NOT recognize the rusty-figlet
    // `completions <shell>` subcommand. The Strict-mode dispatcher in
    // `main::run_strict` translates the unknown positional into the
    // upstream-format unrecognized-option diagnostic (treated as a flag
    // for upstream-byte-equality with the existing FR-043 path). The
    // exit code MUST be 2 (FR-042 + FR-043) and the binary MUST NOT
    // print a completion script.
    let assert = common::rusty_figlet_cmd()
        .env_remove("RUSTY_FIGLET_STRICT")
        .arg("--strict")
        .arg("completions")
        .arg("bash")
        .assert()
        .failure()
        .code(2);

    let output = assert.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // No completion script bytes leaked to stdout.
    assert!(
        !stdout.contains("complete -F")
            && !stdout.contains("_rusty-figlet()")
            && !stdout.contains("#compdef"),
        "Strict mode must NOT emit a completion script; got stdout: {stdout:?}"
    );

    // Strict-mode rejection lands on stderr per FR-043. Accept either
    // the `unrecognized option` (long-style) or `invalid option`
    // (short-style) wording — both are upstream-byte-equal diagnostics
    // and the binary picks the right one based on how the parser
    // classifies the token.
    assert!(
        stderr.contains("unrecognized option") || stderr.contains("invalid option"),
        "Strict mode must emit upstream-format rejection; got stderr: {stderr:?}"
    );
    assert!(
        stderr.contains("rusty-figlet:"),
        "Strict mode stderr must carry `rusty-figlet:` program-name prefix; got: {stderr:?}"
    );
}
