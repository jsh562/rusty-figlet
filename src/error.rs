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
///         FigletError::UnknownFilter { .. } => "unknown filter",
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

    /// The `tlf2a` magic header was missing or malformed.
    ///
    /// `found` is the first up-to-32 bytes of the file (per spec Security
    /// Posture — capped to prevent log spam from adversarial inputs).
    /// Raised by [`crate::tlf::parse_tlf`] when the magic prefix mismatches
    /// or when the numeric header fields are structurally invalid.
    #[error("invalid tlf header: found {found:?}")]
    InvalidTlfHeader {
        /// Up to 32 bytes of observed header for diagnostic display.
        found: Vec<u8>,
    },

    /// A `.tlf` file's glyph table failed to parse.
    ///
    /// `reason` is a short human description; `line` is the 1-indexed line
    /// number at which the parser detected the problem. Distinct from
    /// [`FigletError::FontParse`] because TLF carries different semantics
    /// (UTF-8 multicolumn glyphs, multicolor cell markers) and downstream
    /// callers may want to recover differently from each.
    #[error("tlf parse error at line {line}: {reason}")]
    TlfParse {
        /// Short human-readable description of the parse failure.
        reason: String,
        /// 1-indexed line number where the parse error was detected.
        line: u32,
    },

    /// A `-F <chain>` segment named a filter that is not in the supported
    /// set (or whose leaf is disabled at compile-time).
    ///
    /// `name` is the offending segment as supplied; `available` enumerates
    /// the valid filter names (in declaration order) so the CLI can emit a
    /// helpful diagnostic per FR-016 and spec Edge Cases. Raised by
    /// [`crate::filter::FilterChain::parse`].
    #[error("unknown filter: {name}; available: {available:?}")]
    UnknownFilter {
        /// Offending filter name from the `-F` chain.
        name: String,
        /// Valid filter names in declaration order.
        available: Vec<String>,
    },

    /// A CLI or library caller requested an export format whose leaf is
    /// disabled at compile time (FR-016 + Phase 7 / T046).
    ///
    /// `requested` is the user-supplied format name (e.g. `"html"`);
    /// `available` enumerates the format names whose leaves ARE enabled in
    /// this build so the CLI can produce a helpful diagnostic. Raised by
    /// [`crate::export::write_export`].
    #[error("unsupported export format: {requested}; available: {available:?}")]
    UnsupportedExportFormat {
        /// Offending export format name as supplied by the caller.
        requested: String,
        /// Format names whose leaves ARE compiled into this build.
        available: Vec<String>,
    },

    /// Strict-compat mode encountered input it cannot byte-equal-match
    /// against the documented target (`toilet 0.3-1` or `figlet 2.2.5`).
    ///
    /// `mode` identifies which strict-compat target was active; `detail` is
    /// a short human-readable description of the unmappable construct
    /// (e.g., `"TLF multicolor glyph not representable in toilet 16-color floor"`).
    ///
    /// Raised by [`crate::strict_toilet::strict_render`] (gated by the
    /// `toilet-strict-compat` leaf) and by future figlet-2.2.5 strict
    /// invariants when no upstream byte-equal mapping exists for a given
    /// input. The variant is feature-gated free — it is always compiled so
    /// library callers can `match` on it regardless of which strict-compat
    /// leaf is enabled at build time (per FR-016 + AD-005).
    #[error("strict-compat violation ({mode:?}): {detail}")]
    StrictCompatViolation {
        /// Which strict-compat target was active when the violation occurred.
        mode: StrictTarget,
        /// Short description of the unmappable construct.
        detail: String,
    },

    /// An internal invariant was violated. Indicates a bug in the library;
    /// please file an issue.
    #[error("internal error: {0}")]
    Internal(&'static str),
}

/// Strict-compat target identifier carried by
/// [`FigletError::StrictCompatViolation`] (E012 US6 — AD-005 + FR-016).
///
/// `Figlet225` is the existing `strict-compat` leaf (figlet 2.2.5 byte-equal
/// argv parser + diagnostics). `Toilet031` is the Phase 8
/// `toilet-strict-compat` leaf (toilet 0.3-1 byte-equal renderer + filter
/// chain + 16-color floor).
///
/// Marked `#[non_exhaustive]` so future targets (e.g., a frozen figlet 2.2.4
/// or a future toilet 0.4 line) remain non-breaking additions per AD-013.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StrictTarget {
    /// Byte-equal compatibility with upstream `toilet 0.3-1`.
    Toilet031,
    /// Byte-equal compatibility with upstream `figlet 2.2.5`.
    Figlet225,
}
