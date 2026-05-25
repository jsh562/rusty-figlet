//! Color depth detection + SGR emission for 24-bit truecolor and 256-color (E012 Phase 6).
//!
//! ## Capabilities
//!
//! - [`ColorDepth`] enum with three rungs (Truecolor → Color256 → Color16) per FR-010.
//! - [`ColorDepth::detect`] reads `COLORTERM` + isatty per spec Edge Cases.
//! - Truecolor SGR emission (`\x1b[38;2;R;G;Bm` / `\x1b[48;2;R;G;Bm`) gated behind
//!   `color-truecolor` per FR-008 (T036).
//! - 256-color SGR emission (`\x1b[38;5;Nm` / `\x1b[48;5;Nm`) gated behind
//!   `color-256` per FR-009 (T037).
//! - [`resolve_depth`] graceful downgrade with FIXED stderr warning string —
//!   no user bytes interpolated per FR-018 + spec Security Posture (T038).
//!
//! ## Security posture (FR-018, FR-029)
//!
//! When the requested color depth is unavailable AND warnings are not
//! suppressed, [`resolve_depth`] emits a FIXED stderr warning string. The
//! warning never includes the `$COLORTERM` value or any other byte that
//! originated from the environment — per spec Edge Cases this protects
//! against log injection from adversarial terminal-name strings.
//!
//! FR-029 zero-cost: when `suppress_warning = true` the warning branch
//! short-circuits BEFORE the format-args evaluation, so the cost of the
//! suppression path is a single `if` and one struct-tag comparison.
//!
//! ## Module entry points
//!
//! - [`ColorDepth::detect`] — env-var based detection (always available).
//! - [`emit_truecolor_fg`] / [`emit_truecolor_bg`] — typed-`Color` SGR
//!   builders (under `color-truecolor`).
//! - [`emit_256_fg`] / [`emit_256_bg`] — typed-index SGR builders
//!   (under `color-256`).
//! - [`resolve_depth`] — requested vs detected reconciliation + warning.

use std::env;

/// Color depth rung used by the SGR emitters and the `Figlet` render path.
///
/// Ordering: `Truecolor > Color256 > Color16`. `ColorDepth::detect` returns
/// the **highest** rung the current terminal advertises. `resolve_depth`
/// downgrades a `requested` rung to the `detected` rung when the terminal
/// cannot honor it, emitting a fixed warning unless suppressed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ColorDepth {
    /// 24-bit truecolor (`\x1b[38;2;R;G;Bm`). Advertised by `COLORTERM=truecolor`
    /// or `COLORTERM=24bit`. Highest rung.
    Truecolor,
    /// 256-color indexed palette (`\x1b[38;5;Nm`). Middle rung.
    Color256,
    /// 16-color ANSI named palette (`\x1b[30m..37m`, `\x1b[90m..97m`). Lowest
    /// rung; always available on any ANSI-compatible terminal.
    #[default]
    Color16,
}

impl ColorDepth {
    /// Detect the highest color depth the current terminal advertises.
    ///
    /// ## Cache contract (FR-031)
    ///
    /// This function is intended to be called **once** at builder time
    /// (`FigletBuilder::build`); the result is cached on the [`crate::Figlet`]
    /// renderer for its lifetime. The render path NEVER calls `detect` —
    /// invalidation is caller-driven only via [`crate::Figlet::set_color_depth`].
    /// Calling `detect` is O(1) + one syscall (isatty) + one env-var read.
    ///
    /// Detection rules (per spec Edge Cases + FR-010):
    ///
    /// 1. If `COLORTERM` is set to `"truecolor"` or `"24bit"` (case-sensitive,
    ///    matching the upstream toilet convention), return [`Self::Truecolor`].
    /// 2. Otherwise return [`Self::Color16`] — the safe lowest-common-denominator
    ///    rung that any ANSI-compatible terminal honors.
    ///
    /// 256-color is NOT auto-detected because no portable environment variable
    /// reliably signals 256-color support — `TERM=xterm-256color` is a hint but
    /// not a contract. Callers who want 256-color SHOULD pass it explicitly via
    /// [`resolve_depth`].
    ///
    /// The isatty probe is performed only when [`std::io::IsTerminal`] is
    /// available; non-TTY stdout (e.g., piped to a file) returns [`Self::Color16`]
    /// regardless of `COLORTERM` so redirected output is not polluted with
    /// 24-bit escapes that the consuming program won't strip.
    pub fn detect() -> Self {
        // Non-TTY stdout: never advertise truecolor (FR-010 Edge Case —
        // piped output should never carry RGB escapes that downstream
        // tools can't render).
        if !is_stdout_tty() {
            return Self::Color16;
        }
        match env::var("COLORTERM").as_deref() {
            Ok("truecolor") | Ok("24bit") => Self::Truecolor,
            _ => Self::Color16,
        }
    }
}

/// Resolve a caller's requested color depth against the detected terminal
/// capability, downgrading gracefully when the terminal cannot honor the
/// request.
///
/// ## Downgrade matrix
///
/// | requested  | detected   | result     | warning?           |
/// |------------|------------|------------|--------------------|
/// | Truecolor  | Truecolor  | Truecolor  | no                 |
/// | Truecolor  | Color256   | Color256   | yes (if not suppr) |
/// | Truecolor  | Color16    | Color16    | yes (if not suppr) |
/// | Color256   | Truecolor  | Color256   | no                 |
/// | Color256   | Color256   | Color256   | no                 |
/// | Color256   | Color16    | Color16    | yes (if not suppr) |
/// | Color16    | *          | Color16    | no                 |
///
/// `suppress_warning = true` short-circuits BEFORE any format-args evaluation
/// per FR-029 (zero-cost when suppressed).
///
/// The emitted warning is a **fixed** string — no `$COLORTERM` bytes, no
/// terminal-name bytes — per FR-018 + spec Security Posture (defense against
/// log-injection from adversarial environment variables).
pub fn resolve_depth(
    requested: ColorDepth,
    detected: ColorDepth,
    suppress_warning: bool,
) -> ColorDepth {
    let (result, downgraded) = match (requested, detected) {
        (ColorDepth::Truecolor, ColorDepth::Truecolor) => (ColorDepth::Truecolor, false),
        (ColorDepth::Truecolor, ColorDepth::Color256) => (ColorDepth::Color256, true),
        (ColorDepth::Truecolor, ColorDepth::Color16) => (ColorDepth::Color16, true),
        (ColorDepth::Color256, ColorDepth::Truecolor) => (ColorDepth::Color256, false),
        (ColorDepth::Color256, ColorDepth::Color256) => (ColorDepth::Color256, false),
        (ColorDepth::Color256, ColorDepth::Color16) => (ColorDepth::Color16, true),
        (ColorDepth::Color16, _) => (ColorDepth::Color16, false),
    };
    if downgraded && !suppress_warning {
        // FR-018: FIXED string only. Never interpolate $COLORTERM or any
        // other byte that came from the environment.
        emit_downgrade_warning();
    }
    result
}

/// Emit the fixed downgrade warning to stderr. Kept as a separate function
/// so the format-args literal is a single static buffer the compiler can
/// fold.
#[cold]
#[inline(never)]
fn emit_downgrade_warning() {
    eprintln!(
        "rusty-figlet: requested color depth unavailable; downgrading to terminal capability"
    );
}

/// Best-effort stdout-isatty probe. Returns `true` when stdout is attached
/// to a terminal; `false` otherwise (piped, redirected, or detection failed).
///
/// Implemented via [`std::io::IsTerminal`] which is stable since Rust 1.70
/// and is fully cross-platform (Windows + Unix).
fn is_stdout_tty() -> bool {
    use std::io::IsTerminal;
    std::io::stdout().is_terminal()
}

// ---------------------------------------------------------------------------
// T036 — Truecolor SGR emission (gated behind `color-truecolor`).
// ---------------------------------------------------------------------------

/// Emit a 24-bit truecolor foreground SGR for the typed RGB triple.
///
/// The output bytes are `\x1b[38;2;R;G;Bm` per FR-008. The `(r, g, b)`
/// arguments are typed `u8` values — there is no path for user-controlled
/// bytes to flow into the escape sequence per spec Security Posture.
#[cfg(feature = "color-truecolor")]
#[must_use]
pub fn emit_truecolor_fg(r: u8, g: u8, b: u8) -> String {
    // Preallocated capacity: longest is `\x1b[38;2;255;255;255m` = 19 bytes.
    let mut s = String::with_capacity(20);
    s.push_str("\x1b[38;2;");
    push_u8(&mut s, r);
    s.push(';');
    push_u8(&mut s, g);
    s.push(';');
    push_u8(&mut s, b);
    s.push('m');
    s
}

/// Emit a 24-bit truecolor background SGR for the typed RGB triple.
///
/// The output bytes are `\x1b[48;2;R;G;Bm` per FR-008.
#[cfg(feature = "color-truecolor")]
#[must_use]
pub fn emit_truecolor_bg(r: u8, g: u8, b: u8) -> String {
    let mut s = String::with_capacity(20);
    s.push_str("\x1b[48;2;");
    push_u8(&mut s, r);
    s.push(';');
    push_u8(&mut s, g);
    s.push(';');
    push_u8(&mut s, b);
    s.push('m');
    s
}

// ---------------------------------------------------------------------------
// T037 — 256-color SGR emission (gated behind `color-256`).
// ---------------------------------------------------------------------------

/// Emit a 256-color indexed foreground SGR (`\x1b[38;5;Nm`) per FR-009.
///
/// `n` is a typed `u8` (0..=255); no path exists for user bytes.
#[cfg(feature = "color-256")]
#[must_use]
pub fn emit_256_fg(n: u8) -> String {
    // Longest is `\x1b[38;5;255m` = 11 bytes.
    let mut s = String::with_capacity(12);
    s.push_str("\x1b[38;5;");
    push_u8(&mut s, n);
    s.push('m');
    s
}

/// Emit a 256-color indexed background SGR (`\x1b[48;5;Nm`) per FR-009.
#[cfg(feature = "color-256")]
#[must_use]
pub fn emit_256_bg(n: u8) -> String {
    let mut s = String::with_capacity(12);
    s.push_str("\x1b[48;5;");
    push_u8(&mut s, n);
    s.push('m');
    s
}

/// Internal: push a `u8` decimal representation onto an existing `String`
/// without going through `format!` — keeps the SGR emit path allocation-
/// free beyond the single output buffer.
#[cfg(any(feature = "color-truecolor", feature = "color-256"))]
fn push_u8(s: &mut String, n: u8) {
    if n >= 100 {
        s.push(((n / 100) + b'0') as char);
        s.push((((n / 10) % 10) + b'0') as char);
        s.push(((n % 10) + b'0') as char);
    } else if n >= 10 {
        s.push(((n / 10) + b'0') as char);
        s.push(((n % 10) + b'0') as char);
    } else {
        s.push((n + b'0') as char);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_color16() {
        assert_eq!(ColorDepth::default(), ColorDepth::Color16);
    }

    #[test]
    fn resolve_truecolor_to_truecolor_no_warning() {
        let r = resolve_depth(ColorDepth::Truecolor, ColorDepth::Truecolor, false);
        assert_eq!(r, ColorDepth::Truecolor);
    }

    #[test]
    fn resolve_truecolor_to_color16_downgrades() {
        let r = resolve_depth(ColorDepth::Truecolor, ColorDepth::Color16, true);
        assert_eq!(r, ColorDepth::Color16);
    }

    #[test]
    fn resolve_truecolor_to_color256_downgrades() {
        let r = resolve_depth(ColorDepth::Truecolor, ColorDepth::Color256, true);
        assert_eq!(r, ColorDepth::Color256);
    }

    #[test]
    fn resolve_color256_to_truecolor_uses_color256() {
        let r = resolve_depth(ColorDepth::Color256, ColorDepth::Truecolor, false);
        assert_eq!(r, ColorDepth::Color256);
    }

    #[test]
    fn resolve_color16_always_color16() {
        let r = resolve_depth(ColorDepth::Color16, ColorDepth::Truecolor, false);
        assert_eq!(r, ColorDepth::Color16);
    }

    #[cfg(feature = "color-truecolor")]
    #[test]
    fn truecolor_fg_emits_canonical_sgr() {
        let s = emit_truecolor_fg(255, 128, 0);
        assert_eq!(s, "\x1b[38;2;255;128;0m");
    }

    #[cfg(feature = "color-truecolor")]
    #[test]
    fn truecolor_bg_emits_canonical_sgr() {
        let s = emit_truecolor_bg(0, 0, 0);
        assert_eq!(s, "\x1b[48;2;0;0;0m");
    }

    #[cfg(feature = "color-truecolor")]
    #[test]
    fn truecolor_no_extra_chars() {
        // Defense against accidentally leaking environment bytes into
        // the SGR sequence. The only non-printable byte permitted is
        // the ESC (0x1B) introducer; every other char must be ASCII
        // and printable.
        let s = emit_truecolor_fg(1, 2, 3);
        for ch in s.chars() {
            assert!(ch.is_ascii(), "non-ASCII byte in SGR: {ch:?}");
            assert!(
                ch == '\x1b' || !ch.is_control(),
                "control byte other than ESC in SGR: {ch:?}"
            );
        }
    }

    #[cfg(feature = "color-256")]
    #[test]
    fn color256_fg_emits_canonical_sgr() {
        let s = emit_256_fg(196);
        assert_eq!(s, "\x1b[38;5;196m");
    }

    #[cfg(feature = "color-256")]
    #[test]
    fn color256_bg_emits_canonical_sgr() {
        let s = emit_256_bg(21);
        assert_eq!(s, "\x1b[48;5;21m");
    }

    #[cfg(feature = "color-256")]
    #[test]
    fn color256_edge_indices() {
        assert_eq!(emit_256_fg(0), "\x1b[38;5;0m");
        assert_eq!(emit_256_fg(255), "\x1b[38;5;255m");
    }
}
