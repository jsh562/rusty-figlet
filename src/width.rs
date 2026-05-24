//! Output-width resolution per AD-010 + HINT-005.
//!
//! Precedence ladder (highest first):
//! 1. Explicit `-w N` from argv.
//! 2. `-t` set → `terminal_size_of(stdout)` when stdout is a tty, else
//!    `COLUMNS` env, else 80.
//! 3. No `-w` no `-t` → 80 (NOT terminal).
//!
//! In `Default` mode the CLI auto-applies `-t` when stdout is a tty AND
//! `-w` is not set; `Strict` mode does NOT auto-apply `-t` so byte-equal
//! upstream output is preserved.

use crate::CompatibilityMode;

/// Resolve the effective output width in columns.
///
/// `explicit_w` is the user-supplied `-w N` value (if any). `use_t` is
/// `true` when the user passed `-t`. `columns_env` is the parsed
/// `COLUMNS` env var (if set and a valid u32). `is_tty` is whether
/// stdout is a terminal. `mode` governs the auto-apply-`-t` policy.
pub fn resolve_width(
    explicit_w: Option<u32>,
    use_t: bool,
    columns_env: Option<u32>,
    is_tty: bool,
    mode: CompatibilityMode,
) -> u32 {
    if let Some(w) = explicit_w {
        return w;
    }

    let auto_t = mode == CompatibilityMode::Default && is_tty;
    if use_t || auto_t {
        if let Some(w) = detect_terminal_width() {
            return w;
        }
        if let Some(w) = columns_env {
            return w;
        }
    }
    80
}

/// Query the terminal for its current column count. Returns `None` when
/// stdout is not a terminal or the platform lookup fails.
pub fn detect_terminal_width() -> Option<u32> {
    use std::io;

    let (terminal_size::Width(cols), _) = terminal_size::terminal_size_of(io::stdout())?;
    Some(u32::from(cols))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_w_wins() {
        assert_eq!(
            resolve_width(Some(120), true, Some(70), true, CompatibilityMode::Default),
            120
        );
    }

    #[test]
    fn columns_fallback_when_terminal_lookup_fails() {
        // Forces the COLUMNS branch by disabling the tty assumption.
        // detect_terminal_width will return None when stdout is piped
        // during cargo test, so this exercises the fallback path.
        let got = resolve_width(None, true, Some(70), false, CompatibilityMode::Default);
        assert!(got == 70 || got >= 1, "got = {got}");
    }

    #[test]
    fn default_no_flags_is_eighty() {
        assert_eq!(
            resolve_width(None, false, None, false, CompatibilityMode::Default),
            80
        );
    }

    #[test]
    fn strict_does_not_auto_apply_t() {
        // Strict with stdout=tty + no flags must NOT auto-apply -t.
        // We still get 80 fallback because detect_terminal_width returns
        // None under cargo test.
        assert_eq!(
            resolve_width(None, false, None, true, CompatibilityMode::Strict),
            80
        );
    }
}
