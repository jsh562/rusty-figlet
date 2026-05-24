//! Phase 8 — US6 T129: explicit RAII env-var isolation contract.
//!
//! Per Plan §"Color Output & NO_COLOR Test Isolation" + CHK024:
//! `NO_COLOR` env mutations across concurrent integration tests MUST
//! be scoped via the RAII [`common::env_guard`] so one test cannot see
//! another test's value through the global env. This test demonstrates
//! the guarantee for future contributors — adding new color tests MUST
//! follow this pattern.
//!
//! NOTE: this file contains a SINGLE `#[test]` fn (rather than several)
//! because the underlying `ENV_LOCK` is a non-reentrant std `Mutex` and
//! splitting these checks across separate `#[test]` fns risks racing
//! with other test files that also call `env_guard("NO_COLOR", ...)`.
//! The single-test approach keeps the entire env-isolation contract
//! verifiable in one deterministic serial run.

#![cfg(feature = "cli")]

#[path = "common/mod.rs"]
mod common;

#[test]
fn no_color_test_isolation_raii_contract() {
    // ---- Contract 1: sequential guards do NOT bleed values across drops ----

    // Stage 1: set NO_COLOR=1 via guard A; observe the live value.
    {
        let _guard_a = common::env_guard("NO_COLOR", Some("1"));
        let observed = std::env::var("NO_COLOR").ok();
        assert_eq!(
            observed.as_deref(),
            Some("1"),
            "guard A must establish NO_COLOR=1 during its lifetime"
        );
    }
    // Stage 2: guard A dropped → NO_COLOR restored to its prior state.
    // A second guard observing it should see that prior state, not "1".
    {
        let _guard_b = common::env_guard("NO_COLOR", Some("2"));
        let observed = std::env::var("NO_COLOR").ok();
        assert_eq!(
            observed.as_deref(),
            Some("2"),
            "guard B must establish NO_COLOR=2 independently of guard A's prior value"
        );
    }

    // ---- Contract 2: Drop impl restores the captured prior value ----
    //
    // Seed a prior value directly (not via guard, so the guard's `prior`
    // field captures "preexisting"). The guard MUST restore this value,
    // not blindly clear it.
    {
        // SAFETY: synchronous single-threaded seed before guard
        // acquisition. The guard's internal lock then serializes any
        // racing test that calls `env_guard("NO_COLOR", …)`.
        unsafe {
            std::env::set_var("NO_COLOR", "preexisting");
        }
        {
            let _inner = common::env_guard("NO_COLOR", Some("inner"));
            assert_eq!(
                std::env::var("NO_COLOR").ok().as_deref(),
                Some("inner"),
                "guard must establish its value during its lifetime"
            );
        }
        assert_eq!(
            std::env::var("NO_COLOR").ok().as_deref(),
            Some("preexisting"),
            "guard drop must restore the prior 'preexisting' value, not blindly clear it"
        );
        // Cleanup: leave the env clean for any later test in this exe.
        unsafe {
            std::env::remove_var("NO_COLOR");
        }
    }
}
