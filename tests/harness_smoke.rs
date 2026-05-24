//! Smoke test ensuring the shared test harness in `tests/common/mod.rs`
//! compiles and exposes the documented helpers. Real integration tests
//! land in later phases (US1..US7).

#![cfg(feature = "cli")]

#[path = "common/mod.rs"]
mod common;

#[test]
fn snapshot_strip_substitutes_program_name() {
    let stripped = common::strip_for_snapshot(b"figlet: invalid option");
    assert_eq!(stripped, b"rusty-figlet: invalid option".to_vec());
}

#[test]
fn sandbox_returns_existing_tempdir() {
    let (_guard, path) = common::sandbox();
    assert!(path.exists(), "sandbox path must exist: {path:?}");
}

#[test]
fn minimal_flf_is_non_empty() {
    let bytes = common::make_minimal_flf(1, '$');
    assert!(bytes.len() > 100);
}
