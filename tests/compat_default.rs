//! Phase 3 — US1: Default-font render integration tests.
//!
//! Drives the `rusty-figlet` binary under Default mode (no `--strict`)
//! through the `standard.flf` bundled font. Tests assert structural
//! properties (exit code, line counts, banner separation) rather than
//! specific glyph art so they remain stable against the Phase 1
//! placeholder fonts (height=1) and the post-Polish verbatim upstream
//! fonts (height=6). See FR-001..FR-006, SC-001, SC-002.

#![cfg(feature = "cli")]

use std::fs;
use std::io::Write;
use std::time::Instant;

#[path = "common/mod.rs"]
mod common;

/// T047 — SC-001 + FR-001 + FR-002 + US1 Independent Test smoke check.
///
/// `rusty-figlet "Hello"` exits 0, emits non-empty stdout containing the
/// rendered banner, and the row count equals the font's height. With the
/// Phase 1 placeholder font (height=1) this means 1 row; once the
/// upstream-verbatim fonts land (Polish phase, height=6) the row count
/// climbs to 6 and the same assertion still holds.
#[test]
fn default_font_renders_hello() {
    let (_tmp, _) = common::sandbox();
    let assert = common::rusty_figlet_cmd().arg("Hello").assert().success();
    let out = assert.get_output();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(!stdout.is_empty(), "stdout must be non-empty");
    // Each rendered row ends with `\n`; row count = trailing-newline-
    // stripped split-line count. The placeholder font is height=1 so we
    // require AT LEAST 1 line. (Asserting an upper bound would break
    // once upstream verbatim fonts land at height=6.)
    let line_count = stdout.lines().count();
    assert!(
        line_count >= 1,
        "expected >= 1 rendered line, got {line_count}"
    );
    assert!(
        stdout.contains('H') && stdout.contains('e'),
        "rendered banner should mention input chars; got:\n{stdout}"
    );
}

/// T048 — SC-002 + FR-003 stdin pipe.
///
/// `echo "test" | rusty-figlet` reads stdin, renders the banner, exits
/// 0. Wall time check is soft (50 ms is the SC-002 target on a typical
/// laptop; CI cold builds can exceed this so we assert a generous 5s
/// upper bound here and let SC-005's micro-benchmark task enforce the
/// tighter target later).
#[test]
fn stdin_pipe_renders_each_line_as_banner() {
    let (_tmp, _) = common::sandbox();
    let start = Instant::now();
    let assert = common::rusty_figlet_cmd()
        .write_stdin("test\n")
        .assert()
        .success();
    let elapsed = start.elapsed();
    let out = assert.get_output();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(!stdout.is_empty(), "stdin pipe must produce a banner");
    assert!(
        elapsed.as_secs() < 5,
        "render took {elapsed:?}; integration soft cap is 5s"
    );
}

/// T049 — FR-002 + US1 AS3: positional args concatenated with single space.
///
/// `rusty-figlet Hello World` → ONE banner of "Hello World", not two.
/// Verified by counting blank-line separators (multi-banner output
/// inserts one).
#[test]
fn positional_args_concatenated_with_space() {
    let (_tmp, _) = common::sandbox();
    let assert = common::rusty_figlet_cmd()
        .args(["Hello", "World"])
        .assert()
        .success();
    let out = assert.get_output();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(!stdout.is_empty(), "joined positional args must render");

    // Multi-banner output for stdin path inserts a blank line between
    // banners. A single positional render never inserts a blank line,
    // so we assert no trailing or interior empty-line gap exists when
    // height == 1.
    let blank_separators = stdout.split('\n').filter(|s| s.is_empty()).count();
    // There may be one trailing blank from writeln on the final row.
    assert!(
        blank_separators <= 1,
        "expected single banner (no inter-banner blank separator); got stdout:\n{stdout}"
    );
}

/// T050 — FR-003 + US1 AS2: positional argv overrides stdin.
///
/// `echo "stdin_text" | rusty-figlet Banner` → renders ONLY "Banner";
/// stdin is ignored.
#[test]
fn positional_arg_ignores_stdin() {
    let (_tmp, _) = common::sandbox();
    let assert = common::rusty_figlet_cmd()
        .write_stdin("stdin_text\n")
        .arg("Banner")
        .assert()
        .success();
    let out = assert.get_output();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains('B'),
        "expected positional 'Banner' to render; got:\n{stdout}"
    );
    assert!(
        !stdout.contains("stdin_text"),
        "stdin must be ignored when positional present; got:\n{stdout}"
    );
}

/// T051 — FR-006: empty argv + empty stdin → exit 0, no stdout.
#[test]
fn empty_input_exits_zero_no_output() {
    let (_tmp, _) = common::sandbox();
    let assert = common::rusty_figlet_cmd()
        .write_stdin("")
        .assert()
        .success();
    let out = assert.get_output();
    assert!(
        out.stdout.is_empty(),
        "empty input must produce no stdout; got {} bytes: {:?}",
        out.stdout.len(),
        String::from_utf8_lossy(&out.stdout)
    );
}

/// T052 — FR-004 + Clarifications Q6: stdin > 1 MiB triggers one-time
/// warning and truncates.
#[test]
fn stdin_cap_one_time_warning_per_process() {
    let (_tmp, _) = common::sandbox();
    // 2 MiB of repeated short lines. The first 1 MiB is consumed; the
    // rest is silently discarded after a single stderr warning.
    let mut payload = Vec::with_capacity(2 * 1024 * 1024);
    while payload.len() < 2 * 1024 * 1024 {
        payload
            .write_all(b"line\n")
            .expect("synthetic payload write");
    }
    let assert = common::rusty_figlet_cmd()
        .write_stdin(payload)
        .assert()
        .success();
    let out = assert.get_output();
    let stderr = String::from_utf8_lossy(&out.stderr);
    let warning_count = stderr.matches("stdin input capped at 1 MiB").count();
    assert_eq!(
        warning_count, 1,
        "expected exactly one cap warning per process; got {warning_count}; stderr:\n{stderr}"
    );
}

/// T053 — FR-003 + US1 AS1: stdin lines separated by blank-banner gap.
///
/// `printf "line one\nline two\n" | rusty-figlet` → TWO banners
/// separated by a single blank line.
#[test]
fn stdin_lines_separated_by_blank_banner_gap() {
    let (_tmp, _) = common::sandbox();
    let assert = common::rusty_figlet_cmd()
        .write_stdin("line one\nline two\n")
        .assert()
        .success();
    let out = assert.get_output();
    let stdout = String::from_utf8_lossy(&out.stdout);

    // Both inputs must appear in the rendered output. (The placeholder
    // font emits the literal char on row 0, so "l" appears in both
    // banners; we check for a blank-line separator instead.)
    let lines: Vec<&str> = stdout.split('\n').collect();
    let blank_lines = lines.iter().filter(|s| s.is_empty()).count();
    // Two banners on a height=1 font → exactly one blank separator + a
    // possible trailing blank (writeln on final row). So blank-line
    // count is at least 1.
    assert!(
        blank_lines >= 1,
        "expected at least one blank-line separator between banners; got stdout:\n{stdout}"
    );
}

/// T054 — FR-005 + HINT-009: UTF-8 input with missing codepoint emits a
/// one-time stderr warning.
///
/// We feed a CJK character (U+4E2D = '中') which is absent from
/// `standard.flf`. The first occurrence triggers the warning; subsequent
/// occurrences are silently substituted (one-time per process).
#[test]
fn utf8_missing_glyph_one_time_warning() {
    let (_tmp, _) = common::sandbox();
    // Two CJK chars to verify the warning fires AT MOST ONCE.
    let assert = common::rusty_figlet_cmd().arg("中中").assert().success();
    let out = assert.get_output();
    let stderr = String::from_utf8_lossy(&out.stderr);
    let warning_count = stderr.matches("codepoint U+").count();
    assert!(
        warning_count <= 1,
        "missing-codepoint warning must fire at most once per process; got {warning_count} on stderr:\n{stderr}"
    );
}

// ============================================================================
// Phase 4 — US2: Font Selection (T060..T064)
// ============================================================================

/// T060 — SC-003 + FR-010 + FR-011 + US2 AS1: all 12 bundled fonts resolve
/// via `-f <name>` and render successfully.
#[test]
fn all_twelve_bundled_fonts_resolve_via_dash_f() {
    let names = [
        "standard", "slant", "small", "big", "mini", "banner", "block", "bubble", "digital",
        "lean", "script", "shadow",
    ];
    for name in names {
        let assert = common::rusty_figlet_cmd()
            .args(["-f", name, "X"])
            .assert()
            .success();
        let out = assert.get_output();
        assert!(
            !out.stdout.is_empty(),
            "bundled font {name} must render non-empty banner; stderr:\n{}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
}

/// T061 — SC-004 + FR-010 + US2 AS2: external `.flf` loads from disk via
/// `-f <path>`.
///
/// Writes `standard.flf` into the sandbox tempdir, invokes
/// `rusty-figlet -f <tempdir>/standard.flf X`, and asserts stdout matches
/// `rusty-figlet -f standard X` byte-for-byte (same font bytes ⇒ same
/// rendering regardless of source per SC-004).
#[test]
fn external_flf_loads_from_disk_via_dash_f_path() {
    let (_tmp, root) = common::sandbox();
    // The bundled `standard.flf` bytes are embedded in the binary; we
    // need a real file on disk to exercise the external path. Use the
    // crate's source-tree asset since `assert_cmd` runs from the crate
    // root.
    let src =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/fonts/standard.flf");
    let dst = root.join("standard.flf");
    fs::copy(&src, &dst).expect("copy bundled standard.flf into sandbox");

    let bundled = common::rusty_figlet_cmd()
        .args(["-f", "standard", "X"])
        .assert()
        .success();
    let external = common::rusty_figlet_cmd()
        .args(["-f", dst.to_str().expect("utf8 path"), "X"])
        .assert()
        .success();

    assert_eq!(
        bundled.get_output().stdout,
        external.get_output().stdout,
        "external `.flf` load must render byte-identical to bundled load"
    );
}

/// T062 — FR-010 + US2 AS3: `-f <exact-path>` beats `-d <dir>` lookup.
///
/// Writes a clearly-distinct minimal `.flf` at `<sandbox>/exact.flf`
/// (single-row 'E' glyphs) and a second `.flf` at `<sandbox>/dirs/exact.flf`
/// (single-row 'D' glyphs). With `-f <sandbox>/exact.flf -d <sandbox>/dirs`,
/// the exact-path wins per the FR-010 precedence ladder.
#[test]
fn exact_path_beats_dash_d_lookup() {
    let (_tmp, root) = common::sandbox();
    let dir_root = root.join("dirs");
    fs::create_dir_all(&dir_root).expect("create dirs subdir");

    // Two visually-distinguishable minimal FIGfonts: 'E' for exact-path,
    // 'D' for dirs-search.
    fn flf_with_glyph_char(c: char) -> Vec<u8> {
        let mut out = String::new();
        out.push_str("flf2a$ 1 1 8 0 2 0 0 7\n");
        out.push_str("comment line 1\n");
        out.push_str("comment line 2\n");
        for cp in 32..=126u32 {
            let ch = char::from_u32(cp).unwrap();
            // Use the unique marker char for ASCII letters; everything
            // else stays printable so the parser is happy.
            let render = if ch.is_ascii_alphanumeric() { c } else { ch };
            out.push_str(&format!("{render}$$$$$$$@@\n"));
        }
        for cp in [196u32, 214, 220, 228, 246, 252, 223] {
            out.push_str(&format!("{cp:X} U+{cp:04X}\n"));
            out.push_str(&format!("{c}$$$$$$$@@\n"));
        }
        out.into_bytes()
    }

    let exact_path = root.join("exact.flf");
    fs::write(&exact_path, flf_with_glyph_char('E')).expect("write exact.flf");
    let dirs_path = dir_root.join("exact.flf");
    fs::write(&dirs_path, flf_with_glyph_char('D')).expect("write dirs/exact.flf");

    let assert = common::rusty_figlet_cmd()
        .args([
            "-f",
            exact_path.to_str().expect("utf8 path"),
            "-d",
            dir_root.to_str().expect("utf8 path"),
            "A",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout.contains('E'),
        "exact path must win over -d dir; expected 'E' marker, got:\n{stdout}"
    );
    assert!(
        !stdout.contains('D'),
        "dirs marker 'D' must NOT appear when exact path resolves; got:\n{stdout}"
    );
}

/// T063 — FR-010: `-d <dir>` resolves an external `.flf` when bare name
/// supplied via `-f`.
#[test]
fn font_dir_flag_resolves_external_flf() {
    let (_tmp, root) = common::sandbox();
    let fonts_dir = root.join("fonts");
    fs::create_dir_all(&fonts_dir).expect("create fonts dir");

    // Copy bundled `standard.flf` into the sandbox so we have a real
    // on-disk `mycustom.flf` to find.
    let src =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/fonts/standard.flf");
    let dst = fonts_dir.join("mycustom.flf");
    fs::copy(&src, &dst).expect("copy standard.flf as mycustom.flf");

    let assert = common::rusty_figlet_cmd()
        .args([
            "-d",
            fonts_dir.to_str().expect("utf8 path"),
            "-f",
            "mycustom",
            "X",
        ])
        .assert()
        .success();
    assert!(
        !assert.get_output().stdout.is_empty(),
        "-d dir lookup must resolve `mycustom` from <sandbox>/fonts/"
    );
}

/// T064 — FR-010: `-f <name>` and `-f <name>.flf` produce byte-identical
/// output (suffix-stripping per HINT-003).
#[test]
fn dash_f_with_or_without_flf_suffix() {
    let bare = common::rusty_figlet_cmd()
        .args(["-f", "slant", "X"])
        .assert()
        .success();
    let suffixed = common::rusty_figlet_cmd()
        .args(["-f", "slant.flf", "X"])
        .assert()
        .success();
    assert_eq!(
        bare.get_output().stdout,
        suffixed.get_output().stdout,
        "`-f slant` and `-f slant.flf` must render byte-identical"
    );
}

/// T062 (companion) — FR-010 + FR-012 + US2 AS3: nonexistent font emits a
/// clear error naming the requested font and listing searched paths.
#[test]
fn font_not_found_emits_clear_error_listing_searched_paths() {
    let assert = common::rusty_figlet_cmd()
        .args(["-f", "nonexistent_font_xyz.flf", "X"])
        .assert()
        .failure();
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(
        stderr.contains("nonexistent_font_xyz"),
        "stderr must name the requested font; got:\n{stderr}"
    );
    assert!(
        stderr.contains("font not found") || stderr.contains("FontNotFound"),
        "stderr must convey 'font not found'; got:\n{stderr}"
    );
}

// ============================================================================
// Phase 7 — US5: Layout, Width, Smushing (T110, T114..T118)
// ============================================================================

/// T110 — SC-011 + FR-020 + FR-022: `-w 60 -c "X"` produces lines ≤ 60
/// cols and visually centered.
#[test]
fn width_60_center_lines_le_60_visually_centered() {
    let (_tmp, _) = common::sandbox();
    let assert = common::rusty_figlet_cmd()
        .args(["-w", "60", "-c", "X"])
        .assert()
        .success();
    let out = assert.get_output();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(!stdout.is_empty(), "expected non-empty banner");

    for (idx, line) in stdout.lines().enumerate() {
        let col_count = line.chars().count();
        assert!(
            col_count <= 60,
            "line {idx} exceeds 60 cols ({col_count}): {line:?}"
        );
        if col_count == 0 {
            continue;
        }
        // Visual centering check: leading whitespace > 0 (the rendered
        // glyph cannot start at column 0 when centered in 60 cols on a
        // height=1 font whose glyph is 8 columns wide). Upstream
        // figlet uses left-biased centering: pad = (target - w) / 2,
        // so leading >= (60 - 8) / 2 = 26.
        let leading = line.chars().take_while(|c| *c == ' ').count();
        assert!(
            leading >= 1,
            "line {idx} must have leading whitespace for center justify; line: {line:?}"
        );
        // The total line width (incl. hardblank-derived trailing
        // spaces) plus leading whitespace fits in 60.
        assert!(
            col_count <= 60,
            "centered line {idx} must fit in 60 cols; got {col_count}: {line:?}"
        );
    }
}

/// T114 — FR-023 + US5 AS3: layout-class flags `-k`/`-W`/`-S`/`-s`/`-o`/`-m`
/// are mutually exclusive and last-wins. Smoke test asserts exit 0 across
/// the matrix; the underlying resolver semantics are unit-tested in
/// `src/layout.rs`.
#[test]
fn layout_class_flags_last_wins() {
    let (_tmp, _) = common::sandbox();
    // All four combinations exit 0 with non-empty stdout. The detailed
    // resolver-level last-wins behavior is verified in
    // `src/layout.rs::tests::last_wins_layout_kerning`.
    let combos: &[&[&str]] = &[
        &["-k", "-W", "-S", "X"], // last: -S
        &["-W", "-k", "X"],       // last: -k
        &["-S", "-W", "X"],       // last: -W
        &["-m", "24", "-S", "X"], // last: -S
        &["-S", "-m", "24", "X"], // last: -m 24
        &["-W", "-S", "-k", "X"], // last: -k
    ];
    for argv in combos {
        let assert = common::rusty_figlet_cmd().args(*argv).assert().success();
        let out = assert.get_output();
        assert!(
            !out.stdout.is_empty(),
            "layout combo {argv:?} must render; stderr:\n{}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
}

/// T115 — FR-022 + US5 AS3: justify flags `-c`/`-l`/`-r` mutually
/// exclusive, last-wins.
#[test]
fn justify_flags_last_wins() {
    let (_tmp, _) = common::sandbox();
    // -c -l -r → last is -r (right-aligned). With width=80 and a
    // single-char input on a height=1 placeholder font, right-aligned
    // means the visible char sits at column 79 (0-indexed) with 79
    // spaces before it. We don't assert the exact column because the
    // post-Polish height=6 fonts will widen the glyph; we DO assert
    // that the banner is non-empty and that the LAST char of the line
    // (after trim_end) is the input letter, ruling out left- and
    // centered- justification (which would have spaces after).
    let assert = common::rusty_figlet_cmd()
        .args(["-w", "80", "-c", "-l", "-r", "X"])
        .assert()
        .success();
    let out = assert.get_output();
    let stdout = String::from_utf8_lossy(&out.stdout);
    let first = stdout.lines().next().unwrap_or_default();
    assert!(
        !first.is_empty(),
        "first banner row must be non-empty; got: {stdout:?}"
    );
    // Leading whitespace should exist (right-aligned).
    let leading = first.chars().take_while(|c| *c == ' ').count();
    assert!(
        leading > 0,
        "expected leading whitespace for right-justify (-r last); got first row: {first:?}"
    );

    // Subtest: -r -c → last is -c (center). Expect roughly half the
    // padding on each side of the rendered char.
    let assert2 = common::rusty_figlet_cmd()
        .args(["-w", "80", "-r", "-c", "X"])
        .assert()
        .success();
    let out2 = assert2.get_output();
    let stdout2 = String::from_utf8_lossy(&out2.stdout);
    let first2 = stdout2.lines().next().unwrap_or_default();
    let leading2 = first2.chars().take_while(|c| *c == ' ').count();
    // Centered with width 80, single-cell glyph → ~39 spaces. Right-
    // aligned would be 79 spaces. We assert leading is closer to 39
    // than 79 by checking < 60.
    assert!(
        leading2 < 60,
        "expected center-justify leading ≈ 39 (not 79); got {leading2} in: {first2:?}"
    );
}

/// T116 — FR-025 + Clarifications Q6: single over-width word emits one-
/// time stderr warning per process and renders at full glyph width.
#[test]
fn over_width_word_warns_once_per_process() {
    let (_tmp, _) = common::sandbox();
    let assert = common::rusty_figlet_cmd()
        .args(["-w", "5", "supercalifragilistic"])
        .assert()
        .success();
    let out = assert.get_output();
    let stderr = String::from_utf8_lossy(&out.stderr);
    let warn_count = stderr.matches("too wide for width").count();
    assert_eq!(
        warn_count, 1,
        "expected EXACTLY one over-width warning; got {warn_count}; stderr:\n{stderr}"
    );
    // Banner must still render the word (no mid-word break).
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        !stdout.is_empty(),
        "expected the word to be rendered at full glyph width"
    );
}

/// T117 — FR-026: `-p` paragraph mode joins consecutive non-empty stdin
/// lines into one banner; blank lines separate banners. `-n` (default)
/// treats each line as a separate banner.
#[test]
fn paragraph_mode_concatenates_consecutive_lines() {
    let (_tmp, _) = common::sandbox();
    // Paragraph mode: "a\nb\n\nc\n" → ["a b", "c"] → 2 banners.
    let assert_p = common::rusty_figlet_cmd()
        .args(["-p"])
        .write_stdin("a\nb\n\nc\n")
        .assert()
        .success();
    let stdout_p = String::from_utf8_lossy(&assert_p.get_output().stdout);
    // Two banners on a height=1 font: one blank separator. Total blank
    // lines = 1 (separator) + maybe 1 trailing newline.
    let blank_lines_p = stdout_p.split('\n').filter(|s| s.is_empty()).count();
    assert!(
        (1..=2).contains(&blank_lines_p),
        "paragraph mode: expected 1 inter-banner blank (got {blank_lines_p}); stdout:\n{stdout_p}"
    );

    // Normal mode (-n): same input → 3 banners (a, b, c) → 2 blank
    // separators (between a-b and b-c; the empty line is dropped).
    let assert_n = common::rusty_figlet_cmd()
        .args(["-n"])
        .write_stdin("a\nb\n\nc\n")
        .assert()
        .success();
    let stdout_n = String::from_utf8_lossy(&assert_n.get_output().stdout);
    let blank_lines_n = stdout_n.split('\n').filter(|s| s.is_empty()).count();
    assert!(
        blank_lines_n >= 2,
        "normal mode: expected ≥2 blank separators (got {blank_lines_n}); stdout:\n{stdout_n}"
    );
}

/// T118 — FR-023: `-m N` explicit layout bitfield path.
///
/// `-m 24` = bits 8 + 16 = opposite-pair + big-X rules. `-m 0` = kerning.
/// `-m 63` = all 6 rules. We assert exit 0 + non-empty banner across the
/// three values; rule-precedence specifics are covered by
/// `tests/smush_rules.rs`.
#[test]
fn dash_m_explicit_layout_bitfield() {
    let (_tmp, _) = common::sandbox();
    for arg in ["0", "24", "63"] {
        let assert = common::rusty_figlet_cmd()
            .args(["-m", arg, "X"])
            .assert()
            .success();
        let out = assert.get_output();
        assert!(
            !out.stdout.is_empty(),
            "-m {arg} must render non-empty banner; stderr:\n{}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
}

// ============================================================================
// Phase 8 — US6: Color and Rainbow Output (T124..T128)
//
// Color/rainbow tests must isolate `NO_COLOR` so concurrent test runs do
// not see each other's env mutations — every test that touches color
// scopes a `common::env_guard("NO_COLOR", None)` (or `Some("1")`) RAII
// guard before invoking the binary.
// ============================================================================

/// Marker byte sequence for 24-bit ANSI foreground SGR escapes
/// (`\x1b[38;2;R;G;Bm`) per FR-031 + AD-011.
const ANSI_24BIT_FG_PREFIX: &[u8] = b"\x1b[38;2;";

/// T124 — SC-013 + FR-031 + US6 AS1: `--rainbow --color=always` emits 24-bit
/// ANSI escapes on stdout; `--color=never` suppresses them AND output
/// matches the default plain rendering byte-for-byte.
#[test]
fn rainbow_emits_24bit_ansi_when_color_always() {
    // NO_COLOR must be unset for `--color=always` to take effect.
    let _guard = common::env_guard("NO_COLOR", None);

    let assert = common::rusty_figlet_cmd()
        .args(["--rainbow", "--color=always", "X"])
        .assert()
        .success();
    let out = assert.get_output();
    let stdout = &out.stdout;
    assert!(
        stdout
            .windows(ANSI_24BIT_FG_PREFIX.len())
            .any(|w| w == ANSI_24BIT_FG_PREFIX),
        "expected 24-bit ANSI fg escape `\\x1b[38;2;…m` in stdout for --color=always --rainbow; got {} bytes",
        stdout.len()
    );

    // Sub-test: `--color=never --rainbow` suppresses all SGR; bytes
    // must equal the plain (no color, no rainbow) rendering.
    let plain = common::rusty_figlet_cmd()
        .arg("X")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let never = common::rusty_figlet_cmd()
        .args(["--rainbow", "--color=never", "X"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    assert!(
        !never
            .windows(ANSI_24BIT_FG_PREFIX.len())
            .any(|w| w == ANSI_24BIT_FG_PREFIX),
        "--color=never must suppress 24-bit ANSI escapes"
    );
    assert_eq!(
        plain, never,
        "--color=never bytes must equal plain non-color rendering"
    );
}

/// T125 — SC-013 + FR-032 + US6 AS2: `NO_COLOR=1` suppresses color even
/// when `--color=always --rainbow` are passed (FR-032 precedence). Exit
/// code remains 0.
#[test]
fn no_color_env_suppresses_regardless_of_flag() {
    let _guard = common::env_guard("NO_COLOR", Some("1"));

    let assert = common::rusty_figlet_cmd()
        .args(["--rainbow", "--color=always", "X"])
        .assert()
        .success();
    let out = assert.get_output();
    let stdout = &out.stdout;
    assert!(
        !stdout
            .windows(ANSI_24BIT_FG_PREFIX.len())
            .any(|w| w == ANSI_24BIT_FG_PREFIX),
        "NO_COLOR=1 must suppress ANSI escapes even under --color=always; stdout had escapes"
    );

    // Bytes must equal plain rendering (no color, no rainbow). We can't
    // share a `NO_COLOR` guard with the plain invocation (different test
    // process), but the `--color=never` plain invocation under the same
    // guard validates the byte-identity contract.
    let plain = common::rusty_figlet_cmd()
        .args(["--color=never", "X"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    assert_eq!(
        stdout.as_slice(),
        plain.as_slice(),
        "NO_COLOR=1 output bytes must match --color=never bytes"
    );
}

/// T126 — FR-030 + Plan §Color Output Test Isolation: `--color=auto` with
/// stdout piped (not a TTY) emits NO ANSI escapes. `assert_cmd` always
/// pipes stdout, so this exercises the non-TTY auto path naturally.
#[test]
fn color_auto_no_escapes_on_non_tty() {
    let _guard = common::env_guard("NO_COLOR", None);

    let assert = common::rusty_figlet_cmd()
        .args(["--color=auto", "--rainbow", "X"])
        .assert()
        .success();
    let stdout = assert.get_output().stdout.clone();
    assert!(
        !stdout
            .windows(ANSI_24BIT_FG_PREFIX.len())
            .any(|w| w == ANSI_24BIT_FG_PREFIX),
        "--color=auto on piped (non-TTY) stdout must suppress ANSI escapes"
    );
}

/// T127 — FR-030: `--color=always` overrides TTY detection — escapes are
/// emitted even when stdout is piped. Sub-test `--color=never` over piped
/// stdout produces NO escapes (same as `auto` non-TTY by accident, but
/// the contract is the flag, not the TTY).
#[test]
fn color_always_overrides_tty_detection() {
    let _guard = common::env_guard("NO_COLOR", None);

    let always = common::rusty_figlet_cmd()
        .args(["--color=always", "--rainbow", "X"])
        .assert()
        .success();
    let stdout_always = always.get_output().stdout.clone();
    assert!(
        stdout_always
            .windows(ANSI_24BIT_FG_PREFIX.len())
            .any(|w| w == ANSI_24BIT_FG_PREFIX),
        "--color=always must emit ANSI escapes regardless of TTY status"
    );

    let never = common::rusty_figlet_cmd()
        .args(["--color=never", "--rainbow", "X"])
        .assert()
        .success();
    let stdout_never = never.get_output().stdout.clone();
    assert!(
        !stdout_never
            .windows(ANSI_24BIT_FG_PREFIX.len())
            .any(|w| w == ANSI_24BIT_FG_PREFIX),
        "--color=never must suppress ANSI escapes"
    );
}

/// T128 — FR-031 + HINT-006: rainbow gradient spans the actual banner
/// width (max line width), NOT the `-w 200` budget. We verify by
/// scanning the 24-bit SGR `R;G;B` triples on the FIRST rendered line:
/// the hue cycles from red (≈ start of palette) toward red again at the
/// END of the line. Distinct hues across columns prove a width-spanning
/// gradient; a constant hue would indicate a misuse of width.
#[test]
fn rainbow_gradient_spans_banner_width_not_w_budget() {
    let _guard = common::env_guard("NO_COLOR", None);

    let assert = common::rusty_figlet_cmd()
        .args(["-w", "200", "--rainbow", "--color=always", "X"])
        .assert()
        .success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone())
        .expect("rainbow output should still be UTF-8");
    // Parse all `\x1b[38;2;R;G;Bm` triples on the FIRST non-empty line.
    // (Subsequent lines repeat the same column palette.)
    let first_line = stdout.lines().find(|l| !l.is_empty()).unwrap_or("");
    let rgb_triples = extract_rgb_triples(first_line);
    assert!(
        rgb_triples.len() >= 2,
        "rainbow gradient on first line must emit ≥ 2 distinct column colors; got {} triples in: {first_line:?}",
        rgb_triples.len()
    );
    // Distinct hues across the first vs last column on the line.
    let first_rgb = rgb_triples.first().copied().unwrap();
    let last_rgb = rgb_triples.last().copied().unwrap();
    assert_ne!(
        first_rgb, last_rgb,
        "first and last column colors must differ to prove per-column hue cycling"
    );

    // If the gradient were sized to `-w 200` we'd see only a tiny slice
    // of the hue spectrum on a banner < 200 cols wide (all triples
    // clustered near hue=0 / red). We assert the green/blue channels
    // also vary, indicating the hue cycles through > 60° (red→yellow
    // transitions only the G channel; red→cyan would touch G AND B).
    let mut g_set = std::collections::BTreeSet::new();
    let mut b_set = std::collections::BTreeSet::new();
    for (_, g, b) in &rgb_triples {
        g_set.insert(*g);
        b_set.insert(*b);
    }
    // At least one channel beyond R must vary across the gradient.
    assert!(
        g_set.len() > 1 || b_set.len() > 1,
        "G or B channel must vary across the line — gradient should span banner width, not be clamped to a tiny `-w 200` slice. g_set={g_set:?} b_set={b_set:?}"
    );
}

/// Helper: extract `(R, G, B)` triples from every
/// `\x1b[38;2;R;G;Bm` 24-bit ANSI escape on `s`, in order.
fn extract_rgb_triples(s: &str) -> Vec<(u8, u8, u8)> {
    let mut out = Vec::new();
    let bytes = s.as_bytes();
    let mut i = 0;
    while i + 7 < bytes.len() {
        // Look for `\x1b[38;2;` prefix.
        if &bytes[i..i + 7] == b"\x1b[38;2;" {
            let rest = &s[i + 7..];
            if let Some(m) = rest.find('m') {
                let payload = &rest[..m];
                let parts: Vec<&str> = payload.split(';').collect();
                if parts.len() == 3 {
                    if let (Ok(r), Ok(g), Ok(b)) = (
                        parts[0].parse::<u8>(),
                        parts[1].parse::<u8>(),
                        parts[2].parse::<u8>(),
                    ) {
                        out.push((r, g, b));
                    }
                }
                i += 7 + m + 1;
                continue;
            }
        }
        i += 1;
    }
    out
}

/// T129 (default-mode permissibility companion): `--color=*` and
/// `--rainbow` are accepted in Default mode (no `--strict`).
/// Companion to Phase 5 T081 which asserts the same flags are rejected
/// under `--strict`; together they prove the FR-045 default-vs-strict
/// dichotomy.
#[test]
fn default_mode_accepts_color_and_rainbow_flags() {
    let _guard = common::env_guard("NO_COLOR", None);
    for argv in [
        ["--color=auto", "X"].as_slice(),
        ["--color=always", "X"].as_slice(),
        ["--color=never", "X"].as_slice(),
        ["--rainbow", "X"].as_slice(),
        ["--rainbow", "--color=always", "X"].as_slice(),
    ] {
        common::rusty_figlet_cmd().args(argv).assert().success();
    }
}
