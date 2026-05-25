//! E012 Phase 7 T051 — HTML escape microbenchmark.
//!
//! Compares the hand-rolled 4-char escape (per AD-004) against two
//! third-party crates: `htmlescape` (single-author, 0.3.x line) and
//! `v_htmlescape` (vectorised SIMD path). The benchmark backs the
//! AD-004 maintainability rationale per SC-013 — owning ~30 lines of
//! escape code is justified only if the cost is not materially worse
//! than pulling in a dependency.
//!
//! ## Inputs
//!
//! Three corpora exercise common shapes:
//!   - `ascii_plain`     — 256 bytes, no metacharacters
//!   - `ascii_heavy`     — 256 bytes, half metacharacters
//!   - `xss_payloads`    — concatenated OWASP XSS cheat-sheet payloads
//!
//! Results are captured into `docs/perf-baseline.md` (manual step after
//! `cargo bench --bench html_escape`).
//!
//! ## Execution
//!
//! ```
//! cargo bench --bench html_escape -- --quick
//! ```
//!
//! On CI runners with limited resources, append `-- --sample-size 10`
//! to keep wall time bounded.

use criterion::{Criterion, black_box, criterion_group, criterion_main};

const ASCII_PLAIN: &str = "The quick brown fox jumps over the lazy dog. \
The quick brown fox jumps over the lazy dog. \
The quick brown fox jumps over the lazy dog. \
The quick brown fox jumps over the lazy dog.";

const ASCII_HEAVY: &str = "<<<>>>&&&\"\"\"<<<>>>&&&\"\"\"<<<>>>&&&\"\"\"<<<>>>&&&\"\"\" \
<<<>>>&&&\"\"\"<<<>>>&&&\"\"\"<<<>>>&&&\"\"\"<<<>>>&&&\"\"\"";

const XSS_PAYLOADS: &str = r#"<script>alert(1)</script>"><img src=x onerror=alert(1)>&amp;<svg onload=alert(1)><iframe src="javascript:alert(1)">"#;

/// Hand-rolled 4-char escape (the implementation under test — same as
/// the production version in `src/export/common.rs`). Inlined here so
/// the benchmark doesn't pay for module boundary overhead.
fn rusty_figlet_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 2);
    for c in s.chars() {
        match c {
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '&' => out.push_str("&amp;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(c),
        }
    }
    out
}

fn bench_escape(c: &mut Criterion) {
    for (name, input) in &[
        ("ascii_plain", ASCII_PLAIN),
        ("ascii_heavy", ASCII_HEAVY),
        ("xss_payloads", XSS_PAYLOADS),
    ] {
        let mut group = c.benchmark_group(*name);
        group.bench_function("rusty_figlet", |b| {
            b.iter(|| rusty_figlet_escape(black_box(input)));
        });
        group.bench_function("htmlescape", |b| {
            b.iter(|| htmlescape::encode_minimal(black_box(input)));
        });
        group.bench_function("v_htmlescape", |b| {
            b.iter(|| v_htmlescape::escape(black_box(input)).to_string());
        });
        group.finish();
    }
}

criterion_group!(benches, bench_escape);
criterion_main!(benches);
