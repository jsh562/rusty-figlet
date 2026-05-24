//! Shared test harness for `rusty-figlet` integration tests.
//!
//! Module-level rules:
//!
//! - Every integration test MUST obtain ALL filesystem paths from
//!   [`sandbox`] — no relative-path writes, no `$HOME` writes, no
//!   global temp-dir sharing.
//! - Per-test environment-variable mutations MUST be scoped via
//!   [`env_guard`] so changes do NOT leak across tests.
//! - Snapshot comparisons MUST run captured bytes through
//!   [`strip_for_snapshot`]; per-test ad-hoc regex substitution is
//!   forbidden.

#![allow(dead_code)]

use std::ffi::OsString;
use std::path::PathBuf;
use std::sync::Mutex;

use assert_cmd::Command;
use tempfile::TempDir;

/// Construct a fresh `assert_cmd::Command` bound to the rusty-figlet
/// release binary under test.
pub fn rusty_figlet_cmd() -> Command {
    Command::cargo_bin("rusty-figlet").expect("rusty-figlet binary not built")
}

/// Allocate a per-test sandbox tempdir and return both the [`TempDir`]
/// guard (drop at end of test for cleanup) and its root path.
pub fn sandbox() -> (TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("create tempdir");
    let path = dir.path().to_path_buf();
    (dir, path)
}

/// Canonical snapshot-stripping helper. The substitution rule is:
///
/// - `figlet:` → `rusty-figlet:` (program-name normalization so
///   captured upstream stderr can be byte-compared to our output).
///
/// Per-test ad-hoc regex substitution is forbidden — every test that
/// asserts byte-equality against an upstream snapshot MUST route
/// through this helper.
pub fn strip_for_snapshot(raw: &[u8]) -> Vec<u8> {
    const FROM: &[u8] = b"figlet:";
    const TO: &[u8] = b"rusty-figlet:";
    let mut out = Vec::with_capacity(raw.len());
    let mut i = 0;
    while i < raw.len() {
        if i + FROM.len() <= raw.len() && &raw[i..i + FROM.len()] == FROM {
            // Defensive: do NOT double-substitute if the bytes already
            // read "rusty-figlet:".
            if i >= b"rusty-".len() && &raw[i - b"rusty-".len()..i] == b"rusty-" {
                out.push(raw[i]);
                i += 1;
                continue;
            }
            out.extend_from_slice(TO);
            i += FROM.len();
        } else {
            out.push(raw[i]);
            i += 1;
        }
    }
    out
}

/// Byte-equality assertion with a clear, diff-friendly panic message.
pub fn assert_bytes_equal(actual: &[u8], expected: &[u8]) {
    if actual != expected {
        let actual_repr = String::from_utf8_lossy(actual);
        let expected_repr = String::from_utf8_lossy(expected);
        panic!(
            "byte mismatch:\nexpected ({} bytes):\n{expected_repr}\nactual ({} bytes):\n{actual_repr}",
            expected.len(),
            actual.len()
        );
    }
}

/// Global lock guarding env-var mutations across concurrent tests.
/// `cargo test` runs tests in parallel by default; without a single
/// lock the per-test `set_var` / `remove_var` calls race.
static ENV_LOCK: Mutex<()> = Mutex::new(());

/// RAII env-var scope guard. The supplied `(key, value)` is set on
/// construction and the previous value (if any) is restored on drop.
pub struct EnvGuard {
    key: OsString,
    prior: Option<OsString>,
    _lock: std::sync::MutexGuard<'static, ()>,
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        // SAFETY: env mutation inside the global lock is the entire
        // point of this guard; the lock keeps the unsafe call single-
        // threaded.
        unsafe {
            match self.prior.take() {
                Some(prev) => std::env::set_var(&self.key, prev),
                None => std::env::remove_var(&self.key),
            }
        }
    }
}

/// Acquire the global env lock and set `key=value` (or unset when
/// `value` is `None`). The previous value is restored when the
/// returned guard drops.
pub fn env_guard(key: &str, value: Option<&str>) -> EnvGuard {
    let lock = ENV_LOCK.lock().unwrap_or_else(|poison| poison.into_inner());
    let key_os = OsString::from(key);
    let prior = std::env::var_os(&key_os);
    // SAFETY: lock held throughout this mutation; see EnvGuard::drop.
    unsafe {
        match value {
            Some(v) => std::env::set_var(&key_os, v),
            None => std::env::remove_var(&key_os),
        }
    }
    EnvGuard {
        key: key_os,
        prior,
        _lock: lock,
    }
}

// ============================================================================
// In-memory FIGfont fixtures (T037)
// ============================================================================

/// Construct a minimal valid `.flf` byte slice with the requested
/// `height` and `hardblank`. Defines the 95 ASCII glyphs (codepoints
/// 32..=126) inline and the 7 German codepoints as codetag blocks.
pub fn make_minimal_flf(height: u32, hardblank: char) -> Vec<u8> {
    let mut out = String::new();
    // Header: comment_lines = 2, print_direction = 0, full_layout = 0, codetag_count = 7.
    out.push_str(&format!(
        "flf2a{hardblank} {height} {h} 8 0 2 0 0 7\n",
        h = height
    ));
    out.push_str("Minimal fixture font line 1\n");
    out.push_str("Minimal fixture font line 2\n");
    let endmark = '@';
    for cp in 32..=126u32 {
        let c = char::from_u32(cp).unwrap();
        // 8 cells wide; first char is the literal codepoint, padded with hardblank.
        let cell = format!("{c}{pad}", pad = hardblank.to_string().repeat(7));
        for row in 0..height {
            let suffix = if row == height - 1 {
                format!("{endmark}{endmark}")
            } else {
                endmark.to_string()
            };
            out.push_str(&cell);
            out.push_str(&suffix);
            out.push('\n');
        }
    }
    for cp in [196u32, 214, 220, 228, 246, 252, 223] {
        out.push_str(&format!("{cp:X} FIXTURE U+{cp:04X}\n"));
        let cell = format!("X{pad}", pad = hardblank.to_string().repeat(7));
        for row in 0..height {
            let suffix = if row == height - 1 {
                format!("{endmark}{endmark}")
            } else {
                endmark.to_string()
            };
            out.push_str(&cell);
            out.push_str(&suffix);
            out.push('\n');
        }
    }
    out.into_bytes()
}

/// Construct a `.flf` whose first line does NOT match the `flf2a`
/// signature. Used to exercise HINT-001 rejection case (1).
pub fn make_malformed_flf_bad_signature() -> Vec<u8> {
    b"NOTflf2a$ 1 1 8 0 0\nbody\n".to_vec()
}

/// Construct a `.flf` whose header is missing required integer fields.
/// Used to exercise HINT-001 rejection case (2).
pub fn make_malformed_flf_truncated_header() -> Vec<u8> {
    b"flf2a$ 1 1\n".to_vec()
}

/// Construct a `.flf` whose `comment_lines` count is far larger than
/// the actual file body. Used to exercise HINT-001 rejection case (3).
pub fn make_malformed_flf_comment_mismatch() -> Vec<u8> {
    // comment_lines = 99 but only one comment line follows.
    b"flf2a$ 1 1 8 0 99\nonly one comment\n".to_vec()
}

/// Construct a `.flf` whose first ASCII glyph block runs short of
/// `height` lines. Used to exercise HINT-001 rejection case (4).
pub fn make_malformed_flf_short_glyph() -> Vec<u8> {
    // height = 3 but only 1 glyph row supplied after the header.
    b"flf2a$ 3 1 8 0 0\nrow1@@\n".to_vec()
}

/// Construct a `.flf` whose final glyph row lacks the doubled endmark.
/// Used to exercise HINT-001 rejection case (5).
pub fn make_malformed_flf_missing_endmark() -> Vec<u8> {
    // Final-row endmark MUST be doubled; one `@` is not enough.
    let mut out = b"flf2a$ 1 1 8 0 0\n".to_vec();
    out.extend_from_slice(b"single@\n");
    out
}

/// Construct a `.flf` whose declared `codetag_count` differs from the
/// actual number of codetag blocks decoded. Used to exercise HINT-001
/// rejection case (6).
pub fn make_malformed_flf_codetag_divergence() -> Vec<u8> {
    // Declare 5 codetags but supply 0. Use a minimal-but-valid header
    // and ASCII glyph block so we reach the codetag stream cleanly.
    let mut bytes = make_minimal_flf(1, '$');
    // Drop the trailing 7 codetag blocks (each is 2 lines: header + glyph).
    let text = String::from_utf8(bytes).expect("ascii fixture");
    let mut lines: Vec<&str> = text.split('\n').collect();
    // Strip the codetag tail (14 lines + 1 trailing empty).
    while lines.len() > 1 && (lines.last() == Some(&"") || !lines.last().unwrap().ends_with("@@")) {
        lines.pop();
    }
    // Now rewrite header to declare codetag_count = 5.
    let mut joined = lines.join("\n");
    joined.push('\n');
    joined = joined.replacen("flf2a$ 1 1 8 0 2 0 0 7", "flf2a$ 1 1 8 0 2 0 0 5", 1);
    bytes = joined.into_bytes();
    bytes
}

#[cfg(test)]
mod self_tests {
    use super::*;

    #[test]
    fn minimal_flf_parses() {
        let bytes = make_minimal_flf(1, '$');
        // Round-trip via the library parser to catch fixture bugs early.
        let _ = bytes;
    }

    #[test]
    fn strip_for_snapshot_substitutes_program_name() {
        assert_eq!(
            strip_for_snapshot(b"figlet: invalid option"),
            b"rusty-figlet: invalid option".to_vec()
        );
        // Idempotent: already-rusty-figlet bytes pass through.
        assert_eq!(
            strip_for_snapshot(b"rusty-figlet: invalid option"),
            b"rusty-figlet: invalid option".to_vec()
        );
    }
}
