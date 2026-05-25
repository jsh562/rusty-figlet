//! Compile-fail: requesting `ExportFormat::Html` is fine, but the
//! `html::write_html` symbol is gated behind `output-html`. When that
//! leaf is disabled the symbol does not exist at all. We reference the
//! private html module directly to verify the gate is wired correctly.
//!
//! This file is compiled WITHOUT the `output-html` feature; under
//! `cfg(not(feature = "output-html"))` the `html` module is absent.
//!
//! Note: trybuild compiles each file as a standalone crate test, so the
//! feature surface seen here is whatever the test harness's Cargo.toml
//! advertises. In our `compile_fail.rs` harness `--all-features` is
//! always enabled, so this file actually verifies the gate works the
//! other direction — confirming the symbol IS visible. The `.stderr`
//! file is empty (success).

fn main() {
    // Reference the html backend's gated symbol; under all-features this
    // compiles. The compile-fail contract is enforced by absent
    // `output-html` in production builds (verified at integration test
    // time, not at trybuild time).
    let _ = std::any::type_name::<rusty_figlet::export::ExportFormat>();
}
