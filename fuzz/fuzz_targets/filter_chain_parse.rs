// E012 Phase 7 — Filter chain parser fuzz target (T049).
//
// Property: arbitrary UTF-8 input MUST NOT panic in `FilterChain::parse`.
// The parser is allowed to return Err for unknown filters or oversized
// names; absence of panics, UB, and unbounded allocation is required.

#![cfg_attr(feature = "fuzz-runtime", no_main)]

#[cfg(feature = "fuzz-runtime")]
use libfuzzer_sys::fuzz_target;

#[cfg(feature = "fuzz-runtime")]
fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = rusty_figlet::filter::FilterChain::parse(s);
    }
});

#[cfg(not(feature = "fuzz-runtime"))]
fn main() {}
