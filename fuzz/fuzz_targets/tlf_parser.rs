// E012 Phase 7 — TLF parser fuzz target (T049).
//
// Property: arbitrary byte input MUST NOT panic in `parse_tlf`. The
// parser is allowed to return any `Result` value; we only require
// absence of panics, UB, and unbounded allocation.
//
// Run on Linux:
//   cargo +nightly fuzz run tlf_parser
//
// On Windows this file builds as a plain binary (without libfuzzer-sys)
// so `cargo check` succeeds; actual fuzzing is CI-only.

#![cfg_attr(feature = "fuzz-runtime", no_main)]

#[cfg(feature = "fuzz-runtime")]
use libfuzzer_sys::fuzz_target;

#[cfg(feature = "fuzz-runtime")]
fuzz_target!(|data: &[u8]| {
    let _ = rusty_figlet::tlf::parse_tlf(data);
});

#[cfg(not(feature = "fuzz-runtime"))]
fn main() {
    // No-op when not built under libfuzzer.
}
