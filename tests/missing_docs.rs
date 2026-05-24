//! Phase 10 — Polish: missing-docs gate (T138).
//!
//! Asserts the public `rusty-figlet` library surface satisfies
//! `#![deny(missing_docs)]` (declared at the crate root in
//! `src/lib.rs::T024`) by driving `cargo doc --no-deps` in both feature
//! configurations and `cargo test --doc`. Closes **FR-055** and verifies
//! **SC-010** (at least one doctest per public type).
//!
//! Cross-platform-neutral per plan §Library Default-Features Dep-Tree
//! Test Environment: discovers `cargo` via the `CARGO` env var (set by
//! Cargo when running `cargo test`) with `"cargo"` as PATH fallback;
//! runs from `CARGO_MANIFEST_DIR`; omits `--target` so `cargo doc`
//! resolves the host target identically on all DDR-003 runners.

use std::env;
use std::process::Command;

/// T138 — `cargo doc --no-deps` succeeds in both feature configurations.
///
/// `#![deny(missing_docs)]` is declared at the crate root (T024) — any
/// undocumented public item fails `cargo doc` (and `cargo build --lib`)
/// at compile time. Running it under both feature configurations
/// (`--no-default-features` and `--all-features`) verifies the gate
/// holds for the library-only surface AND the full CLI + library
/// surface.
#[test]
fn cargo_doc_no_deps_succeeds_with_deny_missing_docs() {
    let cargo = env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let manifest_dir = env!("CARGO_MANIFEST_DIR");

    // Library-only (default-features = false) — proves the library
    // surface is fully documented when CLI deps are stripped.
    let out_lib = Command::new(&cargo)
        .args(["doc", "--no-deps", "--no-default-features"])
        .current_dir(manifest_dir)
        .output()
        .expect("invoke cargo doc --no-default-features");

    assert!(
        out_lib.status.success(),
        "cargo doc --no-deps --no-default-features failed (status={:?}):\nstdout:\n{}\nstderr:\n{}",
        out_lib.status,
        String::from_utf8_lossy(&out_lib.stdout),
        String::from_utf8_lossy(&out_lib.stderr),
    );

    // Full surface (--all-features) — proves the CLI-gated modules
    // (cli, color, output, width) are also fully documented.
    let out_full = Command::new(&cargo)
        .args(["doc", "--no-deps", "--all-features"])
        .current_dir(manifest_dir)
        .output()
        .expect("invoke cargo doc --all-features");

    assert!(
        out_full.status.success(),
        "cargo doc --no-deps --all-features failed (status={:?}):\nstdout:\n{}\nstderr:\n{}",
        out_full.status,
        String::from_utf8_lossy(&out_full.stdout),
        String::from_utf8_lossy(&out_full.stderr),
    );
}

/// T138 — `cargo test --doc` runs every doctest in the public API
/// surface and asserts they all pass (SC-010 baseline: at least one
/// doctest per public type — `FigletBuilder`, `Figlet`, `Banner`,
/// `Font`, `FigletError`, `CompatibilityMode`, `Justify`).
#[test]
fn cargo_test_doc_all_doctests_pass() {
    let cargo = env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let manifest_dir = env!("CARGO_MANIFEST_DIR");

    let output = Command::new(&cargo)
        .args(["test", "--doc", "--all-features"])
        .current_dir(manifest_dir)
        .output()
        .expect("invoke cargo test --doc");

    assert!(
        output.status.success(),
        "cargo test --doc --all-features failed (status={:?}):\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}
