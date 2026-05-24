//! Hand-rolled Strict-mode argv parser per AD-007.
//!
//! Mirrors upstream `figlet(6)` getopt diagnostics byte-for-byte (modulo
//! the `figlet:` → `rusty-figlet:` program-name substitution applied by
//! the test harness): excluded short flags surface
//! `figlet: invalid option -- '<char>'`; excluded long flags surface
//! `figlet: unrecognized option '<flag>'`.
//!
//! Last-wins semantics for repeated flags (`-c`/`-l`/`-r`, layout flags,
//! `-w`) per FR-022 + FR-023.

use std::ffi::OsString;
use std::path::PathBuf;

use crate::error::FigletError;

/// Set of single-letter flags that take a following argument.
const ARG_TAKING_SHORTS: &[char] = &['f', 'd', 'w', 'm', 'C', 'I'];

/// Excluded (forbidden) short flags in Strict mode per FR-042 + FR-046.
const EXCLUDED_SHORTS: &[char] = &['L', 'R', 'I', 'N', 'C'];

/// Excluded long flags in Strict mode per FR-043 + FR-045.
const EXCLUDED_LONGS: &[&str] = &[
    "--info-dump",
    "--no-controlfile",
    "--color",
    "--rainbow",
    "--left-to-right",
    "--right-to-left",
];

/// Outcome of [`parse_argv`] on success — the resolved argument bag.
#[derive(Debug, Default, Clone)]
pub struct StrictArgs {
    /// Resolved font (last-wins).
    pub font: Option<String>,
    /// Repeated `-d` font dirs.
    pub font_dirs: Vec<PathBuf>,
    /// Resolved width (last-wins).
    pub width: Option<u32>,
    /// `-t` flag set.
    pub use_terminal_width: bool,
    /// Resolved justify (last-wins between `-c`/`-l`/`-r`/`-x`).
    pub justify: Option<JustifyKind>,
    /// Resolved layout (last-wins between `-k`/`-W`/`-S`/`-s`/`-o`/`-m`).
    pub layout: Option<LayoutKind>,
    /// Paragraph (`-p`) or normal (`-n`) mode flag — last-wins.
    pub paragraph: Option<bool>,
    /// Positional message tokens, concatenated with single space at
    /// render time per FR-002.
    pub message: Vec<String>,
}

/// Resolved justify-class flag for Strict mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JustifyKind {
    /// `-c`
    Center,
    /// `-l`
    Left,
    /// `-r`
    Right,
    /// `-x`
    FontDefault,
}

/// Resolved layout-class flag for Strict mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutKind {
    /// `-k`
    Kerning,
    /// `-W`
    FullWidth,
    /// `-S`
    ForceSmush,
    /// `-s`
    DefaultSmush,
    /// `-o`
    OverlapOnly,
    /// `-m N`
    Explicit(i32),
}

/// Strict-mode parse error. Carries the formatted, byte-equal upstream
/// diagnostic for emission on stderr.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StrictError {
    /// Excluded short flag (e.g. `-L`); message is
    /// `figlet: invalid option -- 'X'`.
    InvalidShortOption {
        /// The offending short-flag character.
        ch: char,
        /// Pre-formatted upstream-style stderr message (no trailing newline).
        message: String,
    },
    /// Excluded long flag (e.g. `--info-dump`); message is
    /// `figlet: unrecognized option '--info-dump'`.
    UnrecognizedLongOption {
        /// The offending long-flag token (including the leading `--`).
        flag: String,
        /// Pre-formatted upstream-style stderr message (no trailing newline).
        message: String,
    },
    /// A short flag that takes an argument was supplied with no value.
    MissingArgument {
        /// The short-flag character missing its argument.
        ch: char,
        /// Pre-formatted upstream-style stderr message (no trailing newline).
        message: String,
    },
}

impl StrictError {
    /// Return the pre-formatted upstream-style stderr message.
    pub fn message(&self) -> &str {
        match self {
            Self::InvalidShortOption { message, .. }
            | Self::UnrecognizedLongOption { message, .. }
            | Self::MissingArgument { message, .. } => message,
        }
    }
}

impl From<StrictError> for FigletError {
    fn from(err: StrictError) -> Self {
        FigletError::Internal(match err {
            StrictError::InvalidShortOption { .. } => "strict: invalid short option",
            StrictError::UnrecognizedLongOption { .. } => "strict: unrecognized long option",
            StrictError::MissingArgument { .. } => "strict: missing argument",
        })
    }
}

/// Format an unknown-flag diagnostic per FR-042 / FR-043.
///
/// `token` must include the leading `-` for short flags or `--` for
/// long flags. Returns the byte-equal upstream string (no trailing
/// newline). The program-name prefix is the literal `figlet:` — the
/// test harness substitutes `rusty-figlet:` before snapshot comparison.
pub fn format_unknown_flag(token: &str) -> String {
    if let Some(long) = token.strip_prefix("--") {
        format!("figlet: unrecognized option '--{long}'")
    } else if let Some(rest) = token.strip_prefix('-') {
        let ch = rest.chars().next().unwrap_or('?');
        format!("figlet: invalid option -- '{ch}'")
    } else {
        format!("figlet: unrecognized option '{token}'")
    }
}

/// Parse `argv` (NOT including `argv[0]`) into a [`StrictArgs`]. Stops
/// at the first excluded/unknown flag and returns [`StrictError`] with
/// the upstream-format diagnostic.
pub fn parse_argv(argv: &[OsString]) -> Result<StrictArgs, StrictError> {
    let mut args = StrictArgs::default();
    let mut i = 0usize;
    // After `--`, all remaining tokens are positional.
    let mut positional_only = false;

    while i < argv.len() {
        let token = match argv[i].to_str() {
            Some(s) => s.to_owned(),
            None => {
                args.message.push(argv[i].to_string_lossy().into_owned());
                i += 1;
                continue;
            }
        };

        if positional_only {
            args.message.push(token);
            i += 1;
            continue;
        }

        if token == "--" {
            positional_only = true;
            i += 1;
            continue;
        }

        if let Some(long) = token.strip_prefix("--") {
            // We intentionally do NOT support any long flag in Strict
            // mode beyond `--strict` / `--no-strict` (already consumed
            // by mode::resolve before we get here). Every other long
            // form is rejected with the upstream "unrecognized option"
            // format.
            if long == "strict" || long == "no-strict" {
                i += 1;
                continue;
            }
            // Excluded longs and any other long form are both rejected
            // with the same upstream diagnostic.
            let _ = EXCLUDED_LONGS;
            return Err(StrictError::UnrecognizedLongOption {
                flag: token.clone(),
                message: format_unknown_flag(&token),
            });
        }

        if let Some(short_body) = token.strip_prefix('-').filter(|s| !s.is_empty()) {
            // Grouped shorts handled char-by-char; the first arg-taking
            // short in a group consumes the remainder of the token as
            // its value (or the next argv token if the group ends).
            let chars: Vec<char> = short_body.chars().collect();
            let mut idx = 0usize;
            while idx < chars.len() {
                let ch = chars[idx];

                if EXCLUDED_SHORTS.contains(&ch) {
                    let token_str = format!("-{ch}");
                    return Err(StrictError::InvalidShortOption {
                        ch,
                        message: format_unknown_flag(&token_str),
                    });
                }

                if ARG_TAKING_SHORTS.contains(&ch) {
                    let value = if idx + 1 < chars.len() {
                        chars[idx + 1..].iter().collect::<String>()
                    } else {
                        i += 1;
                        match argv.get(i).and_then(|os| os.to_str()).map(str::to_owned) {
                            Some(v) => v,
                            None => {
                                let msg = format!("figlet: option requires an argument -- '{ch}'");
                                return Err(StrictError::MissingArgument { ch, message: msg });
                            }
                        }
                    };
                    apply_short_with_value(&mut args, ch, &value);
                    idx = chars.len();
                    continue;
                }

                match ch {
                    'c' => args.justify = Some(JustifyKind::Center),
                    'l' => args.justify = Some(JustifyKind::Left),
                    'r' => args.justify = Some(JustifyKind::Right),
                    'x' => args.justify = Some(JustifyKind::FontDefault),
                    'k' => args.layout = Some(LayoutKind::Kerning),
                    'W' => args.layout = Some(LayoutKind::FullWidth),
                    'S' => args.layout = Some(LayoutKind::ForceSmush),
                    's' => args.layout = Some(LayoutKind::DefaultSmush),
                    'o' => args.layout = Some(LayoutKind::OverlapOnly),
                    't' => args.use_terminal_width = true,
                    'p' => args.paragraph = Some(true),
                    'n' => args.paragraph = Some(false),
                    other => {
                        let token_str = format!("-{other}");
                        return Err(StrictError::InvalidShortOption {
                            ch: other,
                            message: format_unknown_flag(&token_str),
                        });
                    }
                }
                idx += 1;
            }
            i += 1;
            continue;
        }

        // Positional.
        args.message.push(token);
        i += 1;
    }

    Ok(args)
}

fn apply_short_with_value(args: &mut StrictArgs, ch: char, value: &str) {
    match ch {
        'f' => args.font = Some(value.to_owned()),
        'd' => args.font_dirs.push(PathBuf::from(value)),
        'w' => {
            if let Ok(n) = value.parse::<u32>() {
                args.width = Some(n);
            }
        }
        'm' => {
            if let Ok(n) = value.parse::<i32>() {
                args.layout = Some(LayoutKind::Explicit(n));
            }
        }
        // Excluded shorts that take a value are caught earlier; reach
        // here only via the ARG_TAKING_SHORTS allow-list above.
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(s: &[&str]) -> Result<StrictArgs, StrictError> {
        let argv: Vec<OsString> = s.iter().map(|&v| OsString::from(v)).collect();
        parse_argv(&argv)
    }

    #[test]
    fn empty_argv_ok() {
        let got = parse(&[]).unwrap();
        assert!(got.message.is_empty());
    }

    #[test]
    fn single_positional_collected() {
        let got = parse(&["hello"]).unwrap();
        assert_eq!(got.message, vec!["hello".to_owned()]);
    }

    #[test]
    fn dash_f_with_separate_value() {
        let got = parse(&["-f", "slant", "X"]).unwrap();
        assert_eq!(got.font.as_deref(), Some("slant"));
        assert_eq!(got.message, vec!["X".to_owned()]);
    }

    #[test]
    fn dash_f_with_attached_value() {
        let got = parse(&["-fslant", "X"]).unwrap();
        assert_eq!(got.font.as_deref(), Some("slant"));
    }

    #[test]
    #[allow(non_snake_case)]
    fn excluded_short_L_rejected() {
        let err = parse(&["-L", "X"]).unwrap_err();
        match err {
            StrictError::InvalidShortOption { ch, message } => {
                assert_eq!(ch, 'L');
                assert_eq!(message, "figlet: invalid option -- 'L'");
            }
            other => panic!("expected InvalidShortOption, got {other:?}"),
        }
    }

    #[test]
    #[allow(non_snake_case)]
    fn excluded_short_C_rejected() {
        let err = parse(&["-C", "file", "X"]).unwrap_err();
        match err {
            StrictError::InvalidShortOption { ch, .. } => assert_eq!(ch, 'C'),
            other => panic!("expected InvalidShortOption, got {other:?}"),
        }
    }

    #[test]
    fn excluded_long_info_dump_rejected() {
        let err = parse(&["--info-dump", "X"]).unwrap_err();
        match err {
            StrictError::UnrecognizedLongOption { flag, message } => {
                assert_eq!(flag, "--info-dump");
                assert_eq!(message, "figlet: unrecognized option '--info-dump'");
            }
            other => panic!("expected UnrecognizedLongOption, got {other:?}"),
        }
    }

    #[test]
    fn excluded_long_color_rejected() {
        let err = parse(&["--color=always", "X"]).unwrap_err();
        match err {
            StrictError::UnrecognizedLongOption { .. } => {}
            other => panic!("expected UnrecognizedLongOption, got {other:?}"),
        }
    }

    #[test]
    fn last_wins_justify_flags() {
        let got = parse(&["-c", "-l", "-r", "X"]).unwrap();
        assert_eq!(got.justify, Some(JustifyKind::Right));
    }

    #[test]
    fn last_wins_layout_flags() {
        let got = parse(&["-k", "-W", "-S", "X"]).unwrap();
        assert_eq!(got.layout, Some(LayoutKind::ForceSmush));
    }

    #[test]
    fn double_dash_makes_rest_positional() {
        let got = parse(&["--", "-S", "-f"]).unwrap();
        assert_eq!(got.message, vec!["-S".to_owned(), "-f".to_owned()]);
    }

    #[test]
    fn format_unknown_flag_shapes() {
        assert_eq!(format_unknown_flag("-L"), "figlet: invalid option -- 'L'");
        assert_eq!(
            format_unknown_flag("--rainbow"),
            "figlet: unrecognized option '--rainbow'"
        );
    }
}
