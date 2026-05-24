//! Strict-mode activation resolver per FR-040 + AD-006.
//!
//! Precedence ladder (highest → lowest):
//!
//! 1. `--no-strict` overrides every lower-precedence source.
//! 2. `--strict` flag on the command line.
//! 3. `RUSTY_FIGLET_STRICT=1` environment variable.
//! 4. `argv[0]` basename ∈ {`figlet`, `figlet-alias`}.
//! 5. Default mode.
//!
//! When both `--strict` and `--no-strict` appear together, last-wins on
//! the command line (Clarifications Q8).

use std::ffi::{OsStr, OsString};
use std::path::Path;

use crate::CompatibilityMode;

/// A captured snapshot of env vars + argv[0] so callers can dependency-
/// inject the inputs to [`resolve`] for testability.
#[derive(Debug, Clone, Default)]
pub struct EnvSnapshot {
    /// Value of `RUSTY_FIGLET_STRICT` (if set).
    pub rusty_figlet_strict: Option<String>,
}

impl EnvSnapshot {
    /// Capture from the current process environment.
    pub fn capture() -> Self {
        Self {
            rusty_figlet_strict: std::env::var("RUSTY_FIGLET_STRICT").ok(),
        }
    }
}

/// One occurrence of the `--strict` / `--no-strict` toggle in argv
/// order so last-wins can be applied (Clarifications Q8).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrictToggle {
    /// `--strict` was observed.
    On,
    /// `--no-strict` was observed.
    Off,
}

/// Scan a raw argv slice for `--strict` / `--no-strict` tokens in
/// occurrence order. The hand-rolled scanner is intentionally narrow:
/// it does NOT attempt to validate the broader CLI surface — that
/// belongs to clap (Default) or [`crate::strict`] (Strict).
pub fn collect_strict_toggles(argv: &[OsString]) -> Vec<StrictToggle> {
    let mut out = Vec::new();
    for token in argv {
        if let Some(s) = token.to_str() {
            match s {
                "--strict" => out.push(StrictToggle::On),
                "--no-strict" => out.push(StrictToggle::Off),
                _ => {}
            }
        }
    }
    out
}

/// Resolve the effective compatibility mode given the full argv slice,
/// the captured environment, and `argv[0]`.
pub fn resolve(args: &[OsString], env: &EnvSnapshot, argv0: &OsStr) -> CompatibilityMode {
    // (1) command-line toggle has highest precedence (last-wins between
    // them). If the user typed any --strict / --no-strict, that single
    // last token is authoritative.
    let toggles = collect_strict_toggles(args);
    if let Some(last) = toggles.last() {
        return match last {
            StrictToggle::On => CompatibilityMode::Strict,
            StrictToggle::Off => CompatibilityMode::Default,
        };
    }

    // (2) RUSTY_FIGLET_STRICT env var.
    if let Some(value) = env.rusty_figlet_strict.as_deref() {
        if is_truthy(value) {
            return CompatibilityMode::Strict;
        }
    }

    // (3) argv[0] basename match.
    if argv0_matches_figlet(argv0) {
        return CompatibilityMode::Strict;
    }

    CompatibilityMode::Default
}

fn is_truthy(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes"
    )
}

fn argv0_matches_figlet(argv0: &OsStr) -> bool {
    let stem = Path::new(argv0).file_stem().and_then(OsStr::to_str);
    matches!(stem, Some("figlet") | Some("figlet-alias"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_env() -> EnvSnapshot {
        EnvSnapshot::default()
    }

    fn s(v: &str) -> OsString {
        OsString::from(v)
    }

    #[test]
    fn strict_flag_alone_returns_strict() {
        let argv = vec![s("--strict")];
        assert_eq!(
            resolve(&argv, &empty_env(), OsStr::new("rusty-figlet")),
            CompatibilityMode::Strict
        );
    }

    #[test]
    fn env_alone_returns_strict() {
        let env = EnvSnapshot {
            rusty_figlet_strict: Some("1".into()),
        };
        assert_eq!(
            resolve(&[], &env, OsStr::new("rusty-figlet")),
            CompatibilityMode::Strict
        );
    }

    #[test]
    fn argv0_basename_figlet_returns_strict() {
        assert_eq!(
            resolve(&[], &empty_env(), OsStr::new("figlet")),
            CompatibilityMode::Strict
        );
        // With .exe suffix (Windows).
        assert_eq!(
            resolve(&[], &empty_env(), OsStr::new("figlet.exe")),
            CompatibilityMode::Strict
        );
    }

    #[test]
    fn no_strict_overrides_env() {
        let env = EnvSnapshot {
            rusty_figlet_strict: Some("1".into()),
        };
        let argv = vec![s("--no-strict")];
        assert_eq!(
            resolve(&argv, &env, OsStr::new("rusty-figlet")),
            CompatibilityMode::Default
        );
    }

    #[test]
    fn no_strict_overrides_argv0() {
        let argv = vec![s("--no-strict")];
        assert_eq!(
            resolve(&argv, &empty_env(), OsStr::new("figlet")),
            CompatibilityMode::Default
        );
    }

    #[test]
    fn last_wins_strict_then_no_strict() {
        let argv = vec![s("--strict"), s("--no-strict")];
        assert_eq!(
            resolve(&argv, &empty_env(), OsStr::new("rusty-figlet")),
            CompatibilityMode::Default
        );
    }

    #[test]
    fn last_wins_no_strict_then_strict() {
        let argv = vec![s("--no-strict"), s("--strict")];
        assert_eq!(
            resolve(&argv, &empty_env(), OsStr::new("rusty-figlet")),
            CompatibilityMode::Strict
        );
    }

    #[test]
    fn truthy_env_values() {
        for v in ["1", "true", "TRUE", "yes", "Yes"] {
            let env = EnvSnapshot {
                rusty_figlet_strict: Some(v.into()),
            };
            assert_eq!(
                resolve(&[], &env, OsStr::new("rusty-figlet")),
                CompatibilityMode::Strict,
                "value {v:?}"
            );
        }
    }

    #[test]
    fn empty_or_falsy_env_is_default() {
        for v in ["", "0", "false", "no"] {
            let env = EnvSnapshot {
                rusty_figlet_strict: Some(v.into()),
            };
            assert_eq!(
                resolve(&[], &env, OsStr::new("rusty-figlet")),
                CompatibilityMode::Default,
                "value {v:?}"
            );
        }
    }
}
