//! Library error type for `rusty-figlet`.
//!
//! [`FigletError`] is the unified error type returned by every fallible
//! public API in this crate. It is marked `#[non_exhaustive]` so that
//! additive variants in future releases remain non-breaking under SemVer
//! (per AD-013). Downstream consumers that pattern-match on the enum MUST
//! include a wildcard `_` arm.
//!
//! `Send + Sync + 'static` is guaranteed at compile time (SC-009) so the
//! error works across async `await` points and thread boundaries.

use std::io;
use std::path::PathBuf;

/// All fallible operations in `rusty-figlet` return `Result<T, FigletError>`.
///
/// The enum is `#[non_exhaustive]` (per AD-013) so additive variants in
/// future minor releases do NOT constitute a breaking change. Downstream
/// matches MUST include a wildcard `_` arm:
///
/// ```rust
/// use rusty_figlet::FigletError;
/// fn describe(err: &FigletError) -> &'static str {
///     match err {
///         FigletError::FontNotFound { .. } => "missing font",
///         FigletError::FontParse { .. } => "bad font file",
///         FigletError::Io(_) => "io error",
///         FigletError::WidthTooNarrow { .. } => "width too narrow",
///         FigletError::Internal(_) => "internal error",
///         _ => "unknown",
///     }
/// }
/// ```
///
/// `Error::source()` returns `Some(&io::Error)` ONLY for the [`FigletError::Io`]
/// variant; all other variants are leaf errors and return `None` from
/// `source()`. `FontParse { line }` is 1-indexed and matches the convention
/// used by upstream `figlet(6)` parse-error stderr messages.
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum FigletError {
    /// The requested font name (or path) could not be located.
    ///
    /// `name` is the user-supplied identifier; `searched` is the ordered
    /// list of paths the resolver consulted, suitable for displaying in a
    /// diagnostic message.
    #[error("font not found: {name}; searched {searched:?}")]
    FontNotFound {
        /// Font name or path the user supplied (e.g. `"slant"`, `"./my.flf"`).
        name: String,
        /// Ordered list of paths inspected during font resolution.
        searched: Vec<PathBuf>,
    },

    /// A `.flf` file failed to parse.
    ///
    /// `reason` is a short human description (e.g. `"bad signature"`,
    /// `"missing endmark"`); `line` is the 1-indexed line number at which
    /// the parser detected the problem.
    #[error("font parse error at line {line}: {reason}")]
    FontParse {
        /// Short human-readable description of the parse failure.
        reason: String,
        /// 1-indexed line number where the parse error was detected.
        line: u32,
    },

    /// Underlying I/O failure (file read, stdin, stdout).
    ///
    /// `Error::source()` returns the wrapped [`io::Error`] for this variant.
    #[error("io error: {0}")]
    Io(#[from] io::Error),

    /// The requested width is too narrow to render the requested glyph(s).
    ///
    /// `needed` is the minimum width a single glyph requires; `given` is
    /// the width the caller supplied.
    #[error("width too narrow: needed {needed}, given {given}")]
    WidthTooNarrow {
        /// Minimum width required by the widest glyph.
        needed: u32,
        /// Width supplied by the caller.
        given: u32,
    },

    /// An internal invariant was violated. Indicates a bug in the library;
    /// please file an issue.
    #[error("internal error: {0}")]
    Internal(&'static str),
}
