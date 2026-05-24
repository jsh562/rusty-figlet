//! Phase 10 — Polish (T148): SC → test traceability matrix lint.
//!
//! Asserts that every SC-### success criterion (SC-001..SC-019) defined
//! in `specs/00009-figlet-port/spec.md` appears at least once in the
//! per-port `tests/SC_COVERAGE.md` traceability matrix. Per portfolio
//! convention QC fails if any SC is unmapped.
//!
//! SC-005, SC-006, SC-019 are documented in `SC_COVERAGE.md` with the
//! `(DEFERRED)` qualifier — they still count as "mapped" because the
//! tracking row + deferred reason is explicitly recorded.

use std::fs;

const EXPECTED_SC_RANGE: std::ops::RangeInclusive<u32> = 1..=19;

fn read_coverage_matrix() -> String {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/SC_COVERAGE.md");
    fs::read_to_string(path).expect("SC_COVERAGE.md must exist at tests/SC_COVERAGE.md")
}

#[test]
fn every_sc_is_mapped_in_coverage_matrix() {
    let matrix = read_coverage_matrix();
    let mut missing = Vec::new();

    for n in EXPECTED_SC_RANGE {
        let needle = format!("SC-{n:03}");
        if !matrix.contains(&needle) {
            missing.push(needle);
        }
    }

    assert!(
        missing.is_empty(),
        "SC_COVERAGE.md is missing rows for: {missing:?}. Every SC-### in spec.md MUST have at least one row in the traceability matrix (a `(DEFERRED)` row is acceptable when the underlying test is gated by an upstream artifact that has not yet been captured)."
    );
}

#[test]
fn coverage_matrix_documents_test_isolation_policy() {
    let matrix = read_coverage_matrix();
    assert!(
        matrix.contains("Test-Isolation Policy") || matrix.contains("Test Isolation"),
        "SC_COVERAGE.md must document the test-isolation policy header (T148)"
    );
    // Specific isolation invariants must be enumerated.
    for needle in [
        "tempfile::TempDir",
        "sandbox()",
        "env_guard",
        "MUST NOT write to relative paths",
        "MUST NOT write under `$HOME`",
    ] {
        assert!(
            matrix.contains(needle),
            "SC_COVERAGE.md must document the `{needle}` invariant in the test-isolation policy section"
        );
    }
}
