//! `rusty-figlet` binary entry point.
//!
//! Resolves the compatibility mode (per AD-006), dispatches to the
//! clap-derive path (Default) or the hand-rolled Strict parser (Strict),
//! reads stdin with a 1 MiB cap (Clarifications Q6), and renders the
//! resulting banner(s) to stdout.

#![cfg(feature = "cli")]

use std::ffi::OsString;
use std::io::{self, BufRead, Read, Write};
use std::process::ExitCode;

#[cfg(feature = "completions")]
use clap::CommandFactory;
use clap::Parser;
#[cfg(feature = "strict-compat")]
use rusty_figlet::clamp_input_latin1;
use rusty_figlet::{
    Banner, CompatibilityMode, Figlet, FigletBuilder, FigletError, Font, JustifyFlag, JustifyFlags,
    LayoutFlag, LayoutFlags,
};

/// Maximum number of bytes consumed from stdin (1 MiB).
const STDIN_CAP_BYTES: usize = 1024 * 1024;

/// Process exit code used for argv / Strict-mode parse failures
/// (matches upstream `figlet(6)`'s getopt diagnostic exit).
const EXIT_USAGE: u8 = 2;

fn main() -> ExitCode {
    let argv: Vec<OsString> = std::env::args_os().collect();
    let argv0 = argv
        .first()
        .cloned()
        .unwrap_or_else(|| "rusty-figlet".into());
    let argv_tail: Vec<OsString> = argv.iter().skip(1).cloned().collect();

    let mode = resolve_mode(&argv_tail, &argv0);

    let result = match mode {
        #[cfg(feature = "strict-compat")]
        CompatibilityMode::Strict => run_strict(&argv_tail),
        #[cfg(not(feature = "strict-compat"))]
        CompatibilityMode::Strict => {
            // Strict mode requested but `strict-compat` leaf is disabled —
            // fall back to Default mode (the binary was built without the
            // upstream-byte-equal parser).
            eprintln!(
                "rusty-figlet: built without strict-compat leaf — falling back to default mode"
            );
            run_default(&argv_tail)
        }
        CompatibilityMode::Default => run_default(&argv_tail),
        _ => run_default(&argv_tail),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(BinError::Strict(stderr_line)) => {
            // Upstream-byte-equal stderr: write the pre-formatted
            // diagnostic verbatim (the `figlet:` -> `rusty-figlet:`
            // substitution is applied here so test snapshots route
            // through the same byte stream they capture from the binary).
            let mapped = stderr_line.replacen("figlet:", "rusty-figlet:", 1);
            eprintln!("{mapped}");
            ExitCode::from(EXIT_USAGE)
        }
        Err(BinError::Figlet(err)) => {
            eprintln!("rusty-figlet: {err}");
            ExitCode::from(EXIT_USAGE)
        }
    }
}

/// Binary-layer error union. Strict-mode parse errors carry their
/// pre-formatted upstream-style diagnostic so `main()` can write it
/// byte-equally to stderr; library errors take the standard
/// `rusty-figlet: <err>` shape. The `Strict` variant is dead when the
/// `strict-compat` leaf is disabled — the dispatch arm + the
/// `run_strict()` constructor are both `#[cfg(feature = "strict-compat")]`-gated.
enum BinError {
    #[allow(dead_code)]
    Strict(String),
    Figlet(FigletError),
}

impl From<FigletError> for BinError {
    fn from(err: FigletError) -> Self {
        Self::Figlet(err)
    }
}

/// Resolve the effective compatibility mode using the same precedence
/// ladder as `rusty_figlet::mode::resolve` (the library function is
/// crate-private; see Phase 2 T030). Keeps the binary self-contained.
fn resolve_mode(argv_tail: &[OsString], argv0: &std::ffi::OsStr) -> CompatibilityMode {
    use std::path::Path;

    let mut last: Option<bool> = None;
    for token in argv_tail {
        if let Some(s) = token.to_str() {
            match s {
                "--strict" => last = Some(true),
                "--no-strict" => last = Some(false),
                _ => {}
            }
        }
    }
    if let Some(b) = last {
        return if b {
            CompatibilityMode::Strict
        } else {
            CompatibilityMode::Default
        };
    }

    if let Ok(value) = std::env::var("RUSTY_FIGLET_STRICT") {
        let v = value.trim().to_ascii_lowercase();
        if matches!(v.as_str(), "1" | "true" | "yes") {
            return CompatibilityMode::Strict;
        }
    }

    let stem = Path::new(argv0).file_stem().and_then(|s| s.to_str());
    if matches!(stem, Some("figlet") | Some("figlet-alias")) {
        return CompatibilityMode::Strict;
    }

    CompatibilityMode::Default
}

fn run_default(argv_tail: &[OsString]) -> Result<(), BinError> {
    let cli = BinCli::parse();

    // T131 + FR-060 + US7 AS1: `completions <shell>` short-circuits the
    // render pipeline. Generates the requested shell's completion script
    // via clap_complete (against the same `BinCli` surface the user
    // interacts with) and exits 0. Default mode only — Strict mode
    // rejects the subcommand in `run_strict` per FR-063 + US7 AS3.
    // Gated by the `completions` leaf (v0.2+).
    #[cfg(feature = "completions")]
    if let Some(BinSubcommand::Completions { shell }) = cli.subcommand {
        let mut cmd = BinCli::command();
        let name = cmd.get_name().to_string();
        clap_complete::generate(shell, &mut cmd, name, &mut io::stdout());
        return Ok(());
    }

    // T120 + T123: resolve color choice from --color, NO_COLOR env, and
    // stdout TTY status per FR-030 + FR-032 + AD-011. NO_COLOR (any
    // non-empty value) wins over --color=always per FR-032.
    // Gated by the `color` leaf (v0.2+).
    #[cfg(feature = "color")]
    let no_color_env = std::env::var_os("NO_COLOR")
        .map(|v| !v.is_empty())
        .unwrap_or(false);
    #[cfg(feature = "color")]
    let color_choice = match cli.color {
        BinColorChoice::Auto => rusty_figlet::color::ColorChoice::Auto,
        BinColorChoice::Always => rusty_figlet::color::ColorChoice::Always,
        BinColorChoice::Never => rusty_figlet::color::ColorChoice::Never,
    };
    #[cfg(feature = "color")]
    let stdout_is_tty = is_stdout_tty();
    #[cfg(feature = "color")]
    let use_color = rusty_figlet::color::should_color(color_choice, no_color_env, stdout_is_tty);
    // T121 + T123: rainbow is only active when color is also active. We
    // build the palette lazily after rendering when we know the actual
    // banner width per HINT-006. Gated by `rainbow` leaf.
    #[cfg(all(feature = "color", feature = "rainbow"))]
    let rainbow_active = use_color && cli.rainbow;
    // When `color` is enabled but `rainbow` is not, the rainbow palette
    // path is unreachable.
    #[cfg(all(feature = "color", not(feature = "rainbow")))]
    let rainbow_active: bool = false;

    // FR-046 + Clarifications Q7: `-C`/`-N` are accepted-but-ignored in
    // Default mode; emit a one-time stderr warning per process per
    // Clarifications Q6 so scripted callers see exactly one notice even
    // when many lines are rendered.
    if cli._control_file.is_some() || cli._no_controlfile {
        warn_control_file_ignored();
    }

    // T109: collect layout / justify / paragraph flag occurrences in
    // argv order so we can apply last-wins semantics per FR-022 +
    // FR-023. clap doesn't preserve declaration order across separate
    // bool flags, so we do a second pass over argv ourselves.
    let occurrences = collect_flag_occurrences(argv_tail);

    let font = map_font(cli.font.as_deref())?;
    let mut builder = FigletBuilder::new().font(font);
    if !cli.font_dirs.is_empty() {
        builder = builder.font_dirs(cli.font_dirs.clone());
    }

    // T106 + AD-010: width precedence ladder.
    // `-w N` from cli > `-t` (auto-applied in Default mode when stdout
    // is a tty) > 80. The `-t` auto-apply branch is gated by the
    // `terminal-width` leaf (v0.2+); without it, only an explicit `-w`
    // value or the 80-column fallback applies.
    #[cfg(feature = "terminal-width")]
    let width = {
        let columns_env = std::env::var("COLUMNS")
            .ok()
            .and_then(|s| s.parse::<u32>().ok());
        let is_tty = is_stdout_tty_default();
        rusty_figlet::resolve_width_for(
            cli.width,
            cli._use_terminal_width,
            columns_env,
            is_tty,
            CompatibilityMode::Default,
        )
    };
    #[cfg(not(feature = "terminal-width"))]
    let width = cli.width.unwrap_or(80);
    builder = builder.width(width);

    // T109: wire layout flags (FR-023).
    builder = builder.layout(occurrences.layout_flags.clone());
    // T109: wire justify (FR-022).
    let justify = rusty_figlet::resolve_justify_for(&occurrences.justify_flags);
    builder = builder.justify(justify);

    let figlet = builder.build()?;

    // T123: when color is active, route through `termcolor::StandardStream`
    // so per-character SGR escapes reach stdout via the same sink used by
    // `output::write_banner`. When color is suppressed, the plain
    // `io::stdout().lock()` path is used and bytes are byte-identical to
    // the non-color path (per SC-013 `--color=never` byte-identity).
    // Gated by the `color` leaf (v0.2+).
    #[cfg(feature = "color")]
    if use_color {
        // Force `termcolor::ColorChoice::Always` because we already
        // resolved TTY + NO_COLOR ourselves via `should_color`; letting
        // termcolor re-detect would double-suppress on piped stdout when
        // the user passed `--color=always`.
        let mut stream = termcolor::StandardStream::stdout(termcolor::ColorChoice::Always);

        // FR-002 vs FR-003 precedence (color path).
        if !cli.message.is_empty() {
            let text = cli.message.join(" ");
            let banner = figlet.render(&text)?;
            write_banner_with_color(&banner, rainbow_active, &mut stream)?;
            return Ok(());
        }

        let stdin = io::stdin();
        let mut handle = stdin.lock();
        let text = read_stdin_capped(&mut handle)?;
        if text.is_empty() {
            return Ok(());
        }
        let paragraph = occurrences.paragraph;
        if paragraph {
            render_paragraph_color(&figlet, &text, rainbow_active, &mut stream)?;
        } else {
            render_normal_color(&figlet, &text, rainbow_active, &mut stream)?;
        }
        return Ok(());
    }

    let stdout = io::stdout();
    let mut out = stdout.lock();

    // FR-002 vs FR-003 precedence: positional argv message wins; stdin
    // is consumed only when no positional was supplied.
    if !cli.message.is_empty() {
        let text = cli.message.join(" ");
        let banner = figlet.render(&text)?;
        write_banner_lines(&banner, &mut out)?;
        return Ok(());
    }

    // FR-003: read stdin per line. FR-006: empty input → exit 0 no output.
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    let text = read_stdin_capped(&mut handle)?;
    if text.is_empty() {
        return Ok(());
    }

    // T108: paragraph mode per FR-026. Last `-p`/`-n` wins; default is
    // `-n` (each newline-terminated line as a separate banner).
    let paragraph = occurrences.paragraph;
    if paragraph {
        render_paragraph_mode(&figlet, &text, &mut out)?;
    } else {
        render_normal_mode(&figlet, &text, &mut out)?;
    }

    Ok(())
}

/// T123: per-banner color writer. Builds the rainbow palette sized to
/// the actual banner's MAX line width (per HINT-006 — NOT the `-w`
/// budget) and routes through [`rusty_figlet::output::write_banner`].
/// When `rainbow_active` is false the banner is written verbatim with
/// no SGR decoration so the output remains identical to the non-color
/// path under `--color=auto` non-TTY (rainbow off) scenarios.
/// Gated by the `color` leaf (v0.2+).
#[cfg(feature = "color")]
fn write_banner_with_color<W: termcolor::WriteColor>(
    banner: &Banner,
    rainbow_active: bool,
    out: &mut W,
) -> Result<(), BinError> {
    let cfg = if rainbow_active {
        #[cfg(feature = "rainbow")]
        {
            let max_width = banner.lines().map(|l| l.chars().count()).max().unwrap_or(0) as u32;
            Some(rusty_figlet::output::ColorConfig {
                rainbow_palette: Some(rusty_figlet::color::rainbow_palette(max_width)),
            })
        }
        #[cfg(not(feature = "rainbow"))]
        {
            let _ = banner;
            None
        }
    } else {
        None
    };
    rusty_figlet::output::write_banner(banner, cfg.as_ref(), out).map_err(FigletError::from)?;
    Ok(())
}

/// T123 color variant of [`render_normal_mode`]. Gated by the `color` leaf.
#[cfg(feature = "color")]
fn render_normal_color<W: termcolor::WriteColor>(
    figlet: &Figlet,
    text: &str,
    rainbow_active: bool,
    out: &mut W,
) -> Result<(), BinError> {
    let mut first_banner = true;
    for line in text.split('\n') {
        if line.is_empty() {
            continue;
        }
        if !first_banner {
            writeln!(out).map_err(FigletError::from)?;
        }
        let banner = figlet.render(line)?;
        write_banner_with_color(&banner, rainbow_active, out)?;
        first_banner = false;
    }
    Ok(())
}

/// T123 color variant of [`render_paragraph_mode`]. Gated by the `color` leaf.
#[cfg(feature = "color")]
fn render_paragraph_color<W: termcolor::WriteColor>(
    figlet: &Figlet,
    text: &str,
    rainbow_active: bool,
    out: &mut W,
) -> Result<(), BinError> {
    let mut paragraphs: Vec<String> = Vec::new();
    let mut current: Vec<&str> = Vec::new();
    for line in text.split('\n') {
        if line.is_empty() {
            if !current.is_empty() {
                paragraphs.push(current.join(" "));
                current.clear();
            }
        } else {
            current.push(line);
        }
    }
    if !current.is_empty() {
        paragraphs.push(current.join(" "));
    }

    let mut first_banner = true;
    for para in &paragraphs {
        if !first_banner {
            writeln!(out).map_err(FigletError::from)?;
        }
        let banner = figlet.render(para)?;
        write_banner_with_color(&banner, rainbow_active, out)?;
        first_banner = false;
    }
    Ok(())
}

/// T108: normal (`-n`) newline handling. Each non-empty line is a
/// separate banner; banners are separated by a single blank line.
fn render_normal_mode<W: Write>(figlet: &Figlet, text: &str, out: &mut W) -> Result<(), BinError> {
    let mut first_banner = true;
    for line in text.split('\n') {
        if line.is_empty() {
            continue;
        }
        if !first_banner {
            writeln!(out).map_err(FigletError::from)?;
        }
        let banner = figlet.render(line)?;
        write_banner_lines(&banner, out)?;
        first_banner = false;
    }
    Ok(())
}

/// T108: paragraph (`-p`) newline handling per FR-026. Consecutive
/// non-empty lines are joined with a single space into one banner;
/// blank lines (`\n\n`) separate banners.
fn render_paragraph_mode<W: Write>(
    figlet: &Figlet,
    text: &str,
    out: &mut W,
) -> Result<(), BinError> {
    let mut paragraphs: Vec<String> = Vec::new();
    let mut current: Vec<&str> = Vec::new();
    for line in text.split('\n') {
        if line.is_empty() {
            if !current.is_empty() {
                paragraphs.push(current.join(" "));
                current.clear();
            }
        } else {
            current.push(line);
        }
    }
    if !current.is_empty() {
        paragraphs.push(current.join(" "));
    }

    let mut first_banner = true;
    for para in &paragraphs {
        if !first_banner {
            writeln!(out).map_err(FigletError::from)?;
        }
        let banner = figlet.render(para)?;
        write_banner_lines(&banner, out)?;
        first_banner = false;
    }
    Ok(())
}

/// Stdout-tty detection via `std::io::IsTerminal`. No extra dep —
/// always available under the `cli` umbrella. Returns false when stdout
/// is piped. Used by the `color` (NO_COLOR + --color=auto) and
/// `terminal-width` (-t auto-detect) leaves; when both are disabled the
/// function is dead, so gate it on whichever leaf is enabled.
#[cfg(any(feature = "color", feature = "terminal-width"))]
fn is_stdout_tty() -> bool {
    use std::io::IsTerminal;
    io::stdout().is_terminal()
}

/// Alias used by the `terminal-width` leaf's `-t` auto-detect branch.
/// Kept as a separate symbol so the leaf-disabled build's dead-code
/// linter doesn't flag the helper.
#[cfg(feature = "terminal-width")]
fn is_stdout_tty_default() -> bool {
    is_stdout_tty()
}

/// Per-argv-occurrence layout / justify / paragraph flag collector
/// (T109). Walks the post-`argv[0]` tail once, recording each layout-
/// class and justify-class flag in argv order; long-form `--…`
/// equivalents are also recognised. After `--`, every token is treated
/// as positional and skipped. Unknown tokens are skipped (clap will
/// surface the diagnostic).
#[derive(Debug, Default)]
struct Occurrences {
    layout_flags: LayoutFlags,
    justify_flags: JustifyFlags,
    /// Last-wins `-p` (true) vs `-n` (false). Default false (normal).
    paragraph: bool,
}

fn collect_flag_occurrences(argv_tail: &[OsString]) -> Occurrences {
    let mut occ = Occurrences::default();
    let mut i = 0usize;
    while i < argv_tail.len() {
        let Some(tok) = argv_tail[i].to_str() else {
            i += 1;
            continue;
        };
        if tok == "--" {
            break;
        }
        if let Some(long) = tok.strip_prefix("--") {
            // Support `--name=value` form.
            let (name, value) = match long.find('=') {
                Some(eq) => (&long[..eq], Some(&long[eq + 1..])),
                None => (long, None),
            };
            match name {
                "center" => occ.justify_flags.flags.push(JustifyFlag::Center),
                "left" => occ.justify_flags.flags.push(JustifyFlag::Left),
                "right" => occ.justify_flags.flags.push(JustifyFlag::Right),
                "font-default-justify" => occ.justify_flags.flags.push(JustifyFlag::FontDefault),
                "kerning" => occ.layout_flags.flags.push(LayoutFlag::Kerning),
                "full-width" => occ.layout_flags.flags.push(LayoutFlag::FullWidth),
                "force-smush" => occ.layout_flags.flags.push(LayoutFlag::ForceSmush),
                "smush" => occ.layout_flags.flags.push(LayoutFlag::FontDefaultSmush),
                "overlap" => occ.layout_flags.flags.push(LayoutFlag::OverlapOnly),
                "layout-mode" => {
                    let v = value.map(str::to_owned).or_else(|| {
                        argv_tail
                            .get(i + 1)
                            .and_then(|os| os.to_str().map(str::to_owned))
                    });
                    if value.is_none() {
                        i += 1;
                    }
                    if let Some(s) = v {
                        if let Ok(n) = s.parse::<i32>() {
                            occ.layout_flags.flags.push(LayoutFlag::Explicit(n));
                        }
                    }
                }
                "paragraph" => occ.paragraph = true,
                "normal" => occ.paragraph = false,
                // Skip arg-taking longs whose value follows in the next
                // token to keep the index aligned with the underlying
                // clap parse.
                "font" | "fontdir" | "width" | "control-file" | "color" => {
                    if value.is_none() {
                        i += 1; // consume the next token as the value
                    }
                }
                _ => {}
            }
            i += 1;
            continue;
        }
        if let Some(short_body) = tok.strip_prefix('-').filter(|s| !s.is_empty()) {
            let chars: Vec<char> = short_body.chars().collect();
            let mut idx = 0usize;
            while idx < chars.len() {
                let ch = chars[idx];
                match ch {
                    'c' => occ.justify_flags.flags.push(JustifyFlag::Center),
                    'l' => occ.justify_flags.flags.push(JustifyFlag::Left),
                    'r' => occ.justify_flags.flags.push(JustifyFlag::Right),
                    'x' => occ.justify_flags.flags.push(JustifyFlag::FontDefault),
                    'k' => occ.layout_flags.flags.push(LayoutFlag::Kerning),
                    'W' => occ.layout_flags.flags.push(LayoutFlag::FullWidth),
                    'S' => occ.layout_flags.flags.push(LayoutFlag::ForceSmush),
                    's' => occ.layout_flags.flags.push(LayoutFlag::FontDefaultSmush),
                    'o' => occ.layout_flags.flags.push(LayoutFlag::OverlapOnly),
                    'p' => occ.paragraph = true,
                    'n' => occ.paragraph = false,
                    'm' => {
                        // Value is rest-of-token or next argv token.
                        let value = if idx + 1 < chars.len() {
                            let v: String = chars[idx + 1..].iter().collect();
                            idx = chars.len();
                            Some(v)
                        } else {
                            i += 1;
                            argv_tail
                                .get(i)
                                .and_then(|os| os.to_str().map(str::to_owned))
                        };
                        if let Some(s) = value {
                            if let Ok(n) = s.parse::<i32>() {
                                occ.layout_flags.flags.push(LayoutFlag::Explicit(n));
                            }
                        }
                    }
                    'f' | 'd' | 'w' | 'C' => {
                        // Arg-taking shorts: consume next token if no
                        // attached value (similar handling to `-m`).
                        if idx + 1 >= chars.len() {
                            i += 1;
                        }
                        idx = chars.len();
                    }
                    _ => {}
                }
                idx += 1;
            }
            i += 1;
            continue;
        }
        // Positional token — skip (clap collects these).
        i += 1;
    }
    occ
}

fn write_banner_lines<W: Write>(banner: &Banner, out: &mut W) -> Result<(), FigletError> {
    for line in banner.lines() {
        writeln!(out, "{line}").map_err(FigletError::from)?;
    }
    Ok(())
}

/// Strict-mode dispatch per T071 + FR-040..FR-046. Parses argv via the
/// hand-rolled [`rusty_figlet::strict::parse_argv`] surface, formats
/// excluded-flag / unrecognized-flag diagnostics byte-equally with
/// upstream `figlet(6)`, Latin-1-clamps the rendered input per FR-044,
/// and routes everything to stdout with the same line-per-banner contract
/// as Default mode (sans color/rainbow per FR-045).
/// Gated by the `strict-compat` leaf (v0.2+).
#[cfg(feature = "strict-compat")]
fn run_strict(argv_tail: &[OsString]) -> Result<(), BinError> {
    use rusty_figlet::strict;

    // (1) Hand-rolled argv parse. The hand-rolled scanner emits upstream-
    // byte-equal diagnostics; we forward the pre-formatted message to
    // stderr via `BinError::Strict` so the test harness can byte-compare
    // against captured upstream output (after the `figlet:` ->
    // `rusty-figlet:` program-name substitution that `main()` applies).
    let args = match strict::parse_argv(argv_tail) {
        Ok(a) => a,
        Err(err) => return Err(BinError::Strict(err.message().to_owned())),
    };

    // (2) US7 AS3: `completions <shell>` is a Rust-native subcommand;
    // Strict mode rejects it with the upstream-format unrecognized-
    // option diagnostic per FR-063.
    if let Some(first) = args.message.first() {
        if first == "completions" {
            let msg = strict::format_unknown_flag("completions");
            return Err(BinError::Strict(msg));
        }
    }

    // (3) Build the Figlet renderer. Font lookup follows the same
    // bundled-then-external ladder as Default mode; HINT-005 explicitly
    // forbids Strict-mode auto-`-t`, so width defaults stay at 80.
    let font = map_font(args.font.as_deref())?;
    let mut builder = FigletBuilder::new().font(font);
    if !args.font_dirs.is_empty() {
        builder = builder.font_dirs(args.font_dirs.clone());
    }

    // T109 (Strict path): width precedence per AD-010 + HINT-005.
    // Strict mode does NOT auto-apply `-t` even when stdout is a tty.
    // The `terminal-width` leaf gates the auto-detect branch; without it,
    // only the explicit `-w` value (or 80 fallback) applies.
    #[cfg(feature = "terminal-width")]
    let width = {
        let columns_env = std::env::var("COLUMNS")
            .ok()
            .and_then(|s| s.parse::<u32>().ok());
        let is_tty = is_stdout_tty();
        rusty_figlet::resolve_width_for(
            args.width,
            args.use_terminal_width,
            columns_env,
            is_tty,
            CompatibilityMode::Strict,
        )
    };
    #[cfg(not(feature = "terminal-width"))]
    let width = args.width.unwrap_or(80);
    builder = builder.width(width);

    // T109 (Strict path): wire layout + justify from the hand-rolled
    // Strict parser. The Strict parser already applies last-wins per
    // FR-022 + FR-023; we translate its single resolved value back into
    // a `LayoutFlags` / `JustifyFlags` of length 1 so the renderer sees
    // the same shape as Default mode.
    let mut layout_flags = LayoutFlags::default();
    if let Some(kind) = args.layout {
        layout_flags.flags.push(match kind {
            rusty_figlet::strict::LayoutKind::Kerning => LayoutFlag::Kerning,
            rusty_figlet::strict::LayoutKind::FullWidth => LayoutFlag::FullWidth,
            rusty_figlet::strict::LayoutKind::ForceSmush => LayoutFlag::ForceSmush,
            rusty_figlet::strict::LayoutKind::DefaultSmush => LayoutFlag::FontDefaultSmush,
            rusty_figlet::strict::LayoutKind::OverlapOnly => LayoutFlag::OverlapOnly,
            rusty_figlet::strict::LayoutKind::Explicit(n) => LayoutFlag::Explicit(n),
        });
    }
    builder = builder.layout(layout_flags);

    let mut justify_flags = JustifyFlags::default();
    if let Some(kind) = args.justify {
        justify_flags.flags.push(match kind {
            rusty_figlet::strict::JustifyKind::Center => JustifyFlag::Center,
            rusty_figlet::strict::JustifyKind::Left => JustifyFlag::Left,
            rusty_figlet::strict::JustifyKind::Right => JustifyFlag::Right,
            rusty_figlet::strict::JustifyKind::FontDefault => JustifyFlag::FontDefault,
        });
    }
    let justify = rusty_figlet::resolve_justify_for(&justify_flags);
    builder = builder.justify(justify);

    let figlet = builder.build()?;

    let stdout = io::stdout();
    let mut out = stdout.lock();

    // (4) Render with FR-044 Latin-1-clamped input. Positional message
    // wins (FR-002); empty argv message falls through to capped stdin
    // (FR-003 + FR-004 + FR-006).
    if !args.message.is_empty() {
        let text = args.message.join(" ");
        render_latin1(&figlet, &text, &mut out)?;
        return Ok(());
    }

    let stdin = io::stdin();
    let mut handle = stdin.lock();
    let text = read_stdin_capped(&mut handle)?;
    if text.is_empty() {
        return Ok(());
    }

    // T108: paragraph mode per FR-026 (Strict path mirrors Default).
    let paragraph = args.paragraph.unwrap_or(false);
    if paragraph {
        let mut paragraphs: Vec<String> = Vec::new();
        let mut current: Vec<&str> = Vec::new();
        for line in text.split('\n') {
            if line.is_empty() {
                if !current.is_empty() {
                    paragraphs.push(current.join(" "));
                    current.clear();
                }
            } else {
                current.push(line);
            }
        }
        if !current.is_empty() {
            paragraphs.push(current.join(" "));
        }

        let mut first_banner = true;
        for para in &paragraphs {
            if !first_banner {
                writeln!(out).map_err(FigletError::from)?;
            }
            render_latin1(&figlet, para, &mut out)?;
            first_banner = false;
        }
    } else {
        let mut first_banner = true;
        for line in text.split('\n') {
            if line.is_empty() {
                continue;
            }
            if !first_banner {
                writeln!(out).map_err(FigletError::from)?;
            }
            render_latin1(&figlet, line, &mut out)?;
            first_banner = false;
        }
    }

    Ok(())
}

/// Render `text` after Latin-1-clamping it per FR-044, then write the
/// banner rows to `out`. The clamp produces a `Vec<u8>` whose bytes map
/// to Unicode codepoints 0..=255 (Latin-1 round-trips through Unicode),
/// which the figfont codepoint lookup indexes verbatim.
/// Gated by the `strict-compat` leaf because `clamp_input_latin1` is only
/// useful in the Strict-mode dispatch path.
#[cfg(feature = "strict-compat")]
fn render_latin1<W: Write>(figlet: &Figlet, text: &str, out: &mut W) -> Result<(), FigletError> {
    let clamped = clamp_input_latin1(text);
    let s: String = clamped.into_iter().map(char::from).collect();
    let banner = figlet.render(&s)?;
    write_banner_lines(&banner, out)
}

fn map_font(name: Option<&str>) -> Result<Font, FigletError> {
    let Some(raw) = name else {
        return Ok(Font::Standard);
    };
    let bare = raw.strip_suffix(".flf").unwrap_or(raw);
    Ok(match bare {
        "standard" => Font::Standard,
        "slant" => Font::Slant,
        "small" => Font::Small,
        "big" => Font::Big,
        "mini" => Font::Mini,
        "banner" => Font::Banner,
        "block" => Font::Block,
        "bubble" => Font::Bubble,
        "digital" => Font::Digital,
        "lean" => Font::Lean,
        "script" => Font::Script,
        "shadow" => Font::Shadow,
        _ => Font::External(std::path::PathBuf::from(raw)),
    })
}

/// Read up to 1 MiB from `handle` and return the resulting UTF-8 string
/// (lossy for invalid bytes). Emits a one-time stderr warning when the
/// cap is reached (FR-004 + Clarifications Q6).
fn read_stdin_capped<R: BufRead>(handle: &mut R) -> Result<String, FigletError> {
    let mut buf: Vec<u8> = Vec::with_capacity(8 * 1024);
    // `take(cap + 1)` lets us cheaply detect truncation: any read beyond
    // the cap yields one extra byte we then discard.
    let mut limited = handle.take(STDIN_CAP_BYTES as u64 + 1);
    limited.read_to_end(&mut buf).map_err(FigletError::from)?;

    let truncated = buf.len() > STDIN_CAP_BYTES;
    if truncated {
        buf.truncate(STDIN_CAP_BYTES);
        warn_stdin_cap();
    }
    let text = String::from_utf8_lossy(&buf).into_owned();
    Ok(text.trim_end_matches('\n').to_owned())
}

use std::sync::OnceLock;

static STDIN_CAP_WARNED: OnceLock<()> = OnceLock::new();
static CONTROL_FILE_WARNED: OnceLock<()> = OnceLock::new();

fn warn_stdin_cap() {
    if STDIN_CAP_WARNED.set(()).is_ok() {
        eprintln!("rusty-figlet: stdin input capped at 1 MiB; remaining input discarded");
    }
}

/// One-time stderr warning per Clarifications Q6 + FR-046 + HINT-010:
/// `-C`/`-N` are accepted-but-ignored in Default mode (Strict mode
/// rejects them per FR-042). Renders input as-is — no transliteration —
/// per Clarifications Q7. Subsequent invocations of `-C`/`-N` in the
/// same process are silently ignored after the first warning.
fn warn_control_file_ignored() {
    if CONTROL_FILE_WARNED.set(()).is_ok() {
        eprintln!("rusty-figlet: control files not yet implemented; ignoring -C/-N");
    }
}

/// Tri-state `--color` value local to the binary; mapped to
/// `rusty_figlet::color::ColorChoice` for `should_color` resolution.
/// Gated by the `color` leaf (v0.2+).
#[cfg(feature = "color")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, clap::ValueEnum)]
#[value(rename_all = "lower")]
enum BinColorChoice {
    /// Auto-detect from TTY status.
    Auto,
    /// Always emit color (still suppressed by NO_COLOR per FR-032).
    Always,
    /// Never emit color.
    Never,
}

#[derive(Debug, clap::Parser)]
#[command(
    name = "rusty-figlet",
    version,
    about = "Render ASCII-art banners from text"
)]
struct BinCli {
    #[arg(short = 'f', long = "font", value_name = "FONT")]
    font: Option<String>,
    #[arg(short = 'd', long = "fontdir", value_name = "DIR")]
    font_dirs: Vec<std::path::PathBuf>,
    #[arg(short = 'w', long = "width", value_name = "INT")]
    width: Option<u32>,
    #[arg(short = 't', long = "terminal-width")]
    _use_terminal_width: bool,
    #[arg(short = 'c', long = "center")]
    _center: bool,
    #[arg(short = 'l', long = "left")]
    _left: bool,
    #[arg(short = 'r', long = "right")]
    _right: bool,
    #[arg(short = 'x', long = "font-default-justify")]
    _justify_default: bool,
    #[arg(short = 'k', long = "kerning")]
    _kerning: bool,
    #[arg(short = 'W', long = "full-width")]
    _full_width: bool,
    #[arg(short = 'S', long = "force-smush")]
    _force_smush: bool,
    #[arg(short = 's', long = "smush")]
    _default_smush: bool,
    #[arg(short = 'o', long = "overlap")]
    _overlap: bool,
    #[arg(
        short = 'm',
        long = "layout-mode",
        value_name = "INT",
        allow_hyphen_values = true
    )]
    _explicit_layout: Option<i32>,
    #[arg(short = 'p', long = "paragraph")]
    _paragraph: bool,
    #[arg(short = 'n', long = "normal")]
    _normal: bool,
    #[arg(short = 'C', long = "control-file", value_name = "FILE")]
    _control_file: Option<std::path::PathBuf>,
    #[arg(short = 'N', long = "no-controlfile")]
    _no_controlfile: bool,
    #[cfg(feature = "color")]
    #[arg(long = "color", value_name = "WHEN", value_enum, default_value_t = BinColorChoice::Auto)]
    color: BinColorChoice,
    #[cfg(feature = "rainbow")]
    #[arg(long = "rainbow")]
    rainbow: bool,
    #[arg(long = "strict")]
    _strict: bool,
    #[arg(long = "no-strict")]
    _no_strict: bool,
    #[arg(value_name = "MESSAGE", trailing_var_arg = true)]
    message: Vec<String>,

    /// Subcommands (e.g. `completions <shell>`). FR-060 + FR-063 +
    /// US7 AS1 — emits a shell-completion script to stdout for one of
    /// `bash`/`zsh`/`fish`/`powershell` and exits 0. Default mode only;
    /// Strict mode rejects it with the upstream "unrecognized option"
    /// diagnostic per FR-063 (wired in `run_strict`).
    /// Gated by the `completions` leaf (v0.2+).
    #[cfg(feature = "completions")]
    #[command(subcommand)]
    subcommand: Option<BinSubcommand>,
}

/// Top-level subcommands emitted by the rusty-figlet binary. Currently
/// limited to `completions <shell>` (FR-060). Strict mode rejects every
/// subcommand because upstream `figlet 2.2.5` does not have them.
/// Gated by the `completions` leaf (v0.2+).
#[cfg(feature = "completions")]
#[derive(Debug, clap::Subcommand)]
enum BinSubcommand {
    /// Emit a shell-completion script for the named shell to stdout
    /// (FR-060 + US7 AS1). Generates the script via
    /// `clap_complete::generate` against the binary's own CLI surface.
    Completions {
        /// Shell to generate completions for. One of `bash`, `zsh`,
        /// `fish`, `powershell`.
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
}
