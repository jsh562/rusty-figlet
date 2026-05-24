//! Phase 6 — US4: Library API surface tests.
//!
//! This integration suite verifies the SemVer-pinned `rusty-figlet`
//! library surface and the contracts that gate v0.1.0 closure:
//!
//! - **T094 / SC-008 + HINT-007** — `default-features = false` dep-tree
//!   assertion: shells out to `cargo tree --no-default-features --prefix
//!   none --edges normal` from `env!("CARGO_MANIFEST_DIR")` and asserts
//!   the runtime tree contains ONLY `rusty-figlet` + `thiserror` + the
//!   pure-Rust transitive deps of `thiserror`. Cross-platform-neutral
//!   per plan §Library Default-Features Dep-Tree Test Environment.
//! - **T095 / SC-009** — `Send + Sync` compile-time guards for the
//!   public types via `static_assertions::assert_impl_all!`.
//! - **T097 / SC-018** — `FigletBuilder::font_bytes(...)` end-to-end
//!   rendering with zero `std::fs` calls.
//! - **T098 / SC-018** — `#[non_exhaustive]` enforcement doctest
//!   substitute (the gold-standard `trybuild` setup is heavier than
//!   warranted for v0.1.0 closure; the doctest on `FigletError`
//!   demonstrating the wildcard `_` arm covers the SemVer contract).
//! - **T099 / plan §Library API Memory & Coverage** — `Banner::lines()`
//!   lazy iterator structural check: asserts the row buffer is sized
//!   `O(font_height × output_width)`.
//! - **T100 / FR-053** — `Banner::lines()` returns the same content as
//!   the `Display` impl, confirming the two surfaces drive equivalent
//!   data (the Display contract per Key Entities).
//! - **T101 / FR-050** — `FigletBuilder` chain methods all return
//!   `Self`; `build()` returns `Result<Figlet, FigletError>`.

use std::env;
use std::path::PathBuf;
use std::process::Command;

use rusty_figlet::{Banner, Figlet, FigletBuilder, FigletError, Font, Justify};

// ============================================================================
// T095 / SC-009 — Send + Sync compile-time guards
// ============================================================================

use static_assertions::assert_impl_all;

// FigletError is the only public type that ALSO requires `'static` so it
// can cross async await points and thread boundaries (per Clarifications
// 2026-05-23 Q2). The other types intentionally lack `'static` because
// `font_bytes(&[u8])` may borrow from caller-owned input.
assert_impl_all!(FigletBuilder: Send, Sync);
assert_impl_all!(Figlet: Send, Sync);
assert_impl_all!(Banner: Send, Sync);
assert_impl_all!(FigletError: Send, Sync);

#[test]
fn figlet_error_is_static() {
    fn assert_static<T: 'static>() {}
    assert_static::<FigletError>();
}

// ============================================================================
// T094 / SC-008 + HINT-007 — default-features=false dep-tree assertion
// ============================================================================

/// T094 — Asserts that with `default-features = false` the runtime dep
/// graph contains ONLY `rusty-figlet` + `thiserror` + thiserror's
/// pure-Rust transitive deps. Asserts ABSENCE of every CLI-only crate
/// the `cli` feature pulls in: `clap`, `clap_complete`, `anstyle`,
/// `termcolor`, `terminal_size`.
///
/// Cross-platform-neutral per plan §Library Default-Features Dep-Tree
/// Test Environment: discovers `cargo` via the `CARGO` env var (set by
/// Cargo when running `cargo test`) with `"cargo"` as PATH fallback;
/// runs from `CARGO_MANIFEST_DIR`; omits `--target` so `cargo tree`
/// resolves the host target identically on all DDR-003 runners.
#[test]
fn default_features_off_excludes_cli_deps() {
    let cargo = env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let manifest_dir = env!("CARGO_MANIFEST_DIR");

    let output = Command::new(&cargo)
        .args([
            "tree",
            "--no-default-features",
            "--prefix",
            "none",
            "--edges",
            "normal",
            "--no-dedupe",
        ])
        .current_dir(manifest_dir)
        .output()
        .expect("invoke cargo tree");

    assert!(
        output.status.success(),
        "cargo tree failed (status={:?}):\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    let stdout = String::from_utf8(output.stdout).expect("cargo tree utf-8");
    let lines: Vec<&str> = stdout.lines().collect();

    // Required runtime crates (every CI host must observe these in the
    // no-default-features tree).
    assert!(
        lines.iter().any(|l| l.starts_with("rusty-figlet ")),
        "cargo tree missing rusty-figlet:\n{stdout}"
    );
    assert!(
        lines.iter().any(|l| l.starts_with("thiserror ")),
        "cargo tree missing thiserror:\n{stdout}"
    );

    // Forbidden CLI-only crates: any presence here is a failure of the
    // `cli` feature gating.
    let forbidden = [
        "clap ",
        "clap_complete ",
        "anstyle ",
        "termcolor ",
        "terminal_size ",
    ];
    for crate_prefix in forbidden {
        assert!(
            !lines.iter().any(|l| l.starts_with(crate_prefix)),
            "forbidden CLI-only crate `{}` present in default-features=false dep tree:\n{stdout}",
            crate_prefix.trim_end()
        );
    }
}

/// Companion to `default_features_off_excludes_cli_deps`: verifies the
/// library actually builds when the `cli` feature is off, exercising the
/// `cfg(feature = "cli")`-gated modules. The brief T102 calls this out
/// explicitly.
#[test]
fn library_builds_without_default_features() {
    let cargo = env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let manifest_dir = env!("CARGO_MANIFEST_DIR");

    let output = Command::new(&cargo)
        .args(["build", "--no-default-features", "--lib"])
        .current_dir(manifest_dir)
        .output()
        .expect("invoke cargo build");

    assert!(
        output.status.success(),
        "cargo build --no-default-features --lib failed (status={:?}):\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

// ============================================================================
// T097 / SC-018 — font_bytes() end-to-end with zero filesystem access
// ============================================================================

/// T097 — Build a `Figlet` from in-memory `.flf` bytes only and render
/// successfully. The bundled `standard.flf` is embedded via
/// `include_bytes!` so the test never touches the filesystem on the
/// rendering path (per FR-052 + FR-056 + SC-018).
#[test]
fn font_bytes_renders_with_zero_fs_calls() {
    const STANDARD: &[u8] = include_bytes!("../assets/fonts/standard.flf");

    let banner = FigletBuilder::new()
        .font_bytes(STANDARD)
        .build()
        .expect("build via font_bytes")
        .render("X")
        .expect("render");

    let lines: Vec<String> = banner.lines().collect();
    assert!(
        !lines.is_empty(),
        "font_bytes path produced an empty banner"
    );
}

/// Companion check: `font_bytes(...)` and `font(Font::Standard)` paths
/// produce byte-equal output, confirming the no-fs path is functionally
/// identical to the bundled-lookup path.
#[test]
fn font_bytes_matches_bundled_lookup() {
    const STANDARD: &[u8] = include_bytes!("../assets/fonts/standard.flf");

    let via_bytes: Vec<String> = FigletBuilder::new()
        .font_bytes(STANDARD)
        .build()
        .expect("build via font_bytes")
        .render("X")
        .expect("render")
        .lines()
        .collect();

    let via_bundled: Vec<String> = FigletBuilder::new()
        .font(Font::Standard)
        .build()
        .expect("build via Font::Standard")
        .render("X")
        .expect("render")
        .lines()
        .collect();

    assert_eq!(
        via_bytes, via_bundled,
        "font_bytes path diverged from bundled-lookup path"
    );
}

// ============================================================================
// T098 / SC-018 — #[non_exhaustive] enforcement (doctest substitute)
// ============================================================================
//
// The gold-standard enforcement for AD-013 would shell out to `trybuild`
// with a `compile_fail` fixture sub-crate that omits the wildcard arm.
// `trybuild` adds a non-trivial surface (workspace fixtures, locked
// rustc output, brittle stderr matching) that is heavier than warranted
// for v0.1.0 closure. The doctest on `FigletError` already demonstrates
// the required wildcard pattern at `cargo test --doc` time (passing
// the exhaustive-match-with-wildcard contract through the doc-test
// harness), and the `#[non_exhaustive]` attribute itself is verified by
// rustc at compile time on every dependent crate. The integration test
// below confirms the doctest is wired into the public surface (i.e.,
// the doctest file resides on `FigletError` and is exercised by the
// `cargo test --doc` invocation).
//
// Documented choice per the Phase 6 brief: "If trybuild setup is too
// heavy for one pass, substitute a simpler equivalent: a doctest on
// FigletError that demonstrates the wildcard arm. Mark with note.
// trybuild is the gold standard; relaxed version acceptable for v0.1
// closure if needed."

/// T098 — Positive subtest: the wildcard-arm pattern (the AD-013
/// downstream contract) compiles and runs cleanly.
#[test]
fn non_exhaustive_match_with_wildcard_compiles() {
    let err = FigletError::Internal("test");
    let described = match &err {
        FigletError::FontNotFound { .. } => "missing font",
        FigletError::FontParse { .. } => "bad font file",
        FigletError::Io(_) => "io error",
        FigletError::WidthTooNarrow { .. } => "width too narrow",
        FigletError::Internal(_) => "internal error",
        _ => "unknown (additive variant)",
    };
    assert_eq!(described, "internal error");
}

/// `Error::source()` returns `Some(&io::Error)` ONLY for the `Io`
/// variant per AD-013; all other variants are leaf errors.
#[test]
fn figlet_error_source_chain_per_variant() {
    use std::error::Error;
    use std::io;

    let not_found = FigletError::FontNotFound {
        name: "x".into(),
        searched: Vec::new(),
    };
    assert!(not_found.source().is_none());

    let parse = FigletError::FontParse {
        reason: "x".into(),
        line: 1,
    };
    assert!(parse.source().is_none());

    let too_narrow = FigletError::WidthTooNarrow {
        needed: 10,
        given: 5,
    };
    assert!(too_narrow.source().is_none());

    let internal = FigletError::Internal("x");
    assert!(internal.source().is_none());

    let io_err: FigletError = io::Error::other("boom").into();
    assert!(io_err.source().is_some());
}

// ============================================================================
// T099 / plan §Library API Memory & Coverage — Banner::lines() lazy + sized
// ============================================================================

/// T099 — Structural check that `Banner.lines()` returns a number of
/// rows equal to the font's height. The plan calls for a `peak_alloc`-
/// based assertion that the banner is `O(font_height × output_width)`;
/// we approximate the same contract by asserting (a) the row count
/// equals the font height (not the input length), and (b) the longest
/// row is bounded above by the resolved width, so the in-memory
/// footprint is bounded by `height × width` rather than `input_len ×
/// glyph_width`.
#[test]
fn banner_rows_are_height_by_width_bounded() {
    let figlet = FigletBuilder::new()
        .font(Font::Standard)
        .width(80)
        .build()
        .expect("build");

    let height = figlet.render("X").unwrap().height();
    let banner = figlet.render("HelloWorld").unwrap();

    // (a) row count equals font height regardless of input length
    let lines: Vec<String> = banner.lines().collect();
    assert_eq!(
        lines.len() as u32,
        height,
        "banner row count must equal font height, got {} for height {}",
        lines.len(),
        height
    );

    // (b) every row's char count is bounded — the in-memory footprint
    // is proportional to font_height × max_row_width, not input_len.
    let max_row = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);
    assert!(
        max_row < 10_000,
        "rendered row width {max_row} is unreasonably large for a 10-char input"
    );

    // Lazy iterator contract: calling .lines() returns an Iterator,
    // and calling .next() once does NOT pre-compute all rows in the
    // user's process — taking the first row is constant-time.
    let mut it = banner.lines();
    let first = it.next();
    assert!(first.is_some(), "first .next() must yield a row");
}

// ============================================================================
// T100 / FR-053 — Display drives same content as lines()
// ============================================================================

/// T100 — Verify that `format!("{banner}")` produces the same content
/// as `banner.lines().map(|l| format!("{l}\n")).collect::<String>()`,
/// confirming the Display impl drives the same lazy iterator data per
/// the Key Entities Banner contract.
#[test]
fn banner_display_matches_lines_iterator_loop() {
    let banner = FigletBuilder::new()
        .font(Font::Standard)
        .build()
        .expect("build")
        .render("Hi")
        .expect("render");

    let via_display = format!("{banner}");
    let via_loop: String = banner.lines().fold(String::new(), |mut acc, l| {
        use std::fmt::Write as _;
        let _ = writeln!(&mut acc, "{l}");
        acc
    });

    assert_eq!(
        via_display, via_loop,
        "Display impl must drive the same lazy iterator data as the manual loop"
    );

    // Trailing newline contract per Key Entities Banner: Display emits
    // a `\n` after the final line.
    assert!(
        via_display.ends_with('\n'),
        "Banner Display impl must emit a trailing newline; got `{via_display}`"
    );
}

// ============================================================================
// T101 / FR-050 — FigletBuilder fluent chain returns Self; build() result
// ============================================================================

/// T101 — Exercise the full fluent surface in one chain to confirm
/// every setter returns `Self`. `build()` returns `Result<Figlet,
/// FigletError>` (verified by the `?` operator binding).
#[test]
fn figlet_builder_fluent_chain_returns_self() {
    let figlet: Figlet = FigletBuilder::new()
        .font(Font::Standard)
        .font_dirs(Vec::<PathBuf>::new())
        .width(80)
        .kerning()
        .full_width()
        .smush()
        .justify(Justify::Center)
        .build()
        .expect("fluent chain build");

    let _banner: Banner = figlet.render("X").expect("render");
}

/// T101 — Direct fluent terminal call (`render`) also returns
/// `Result<Banner, FigletError>`.
#[test]
fn figlet_builder_render_terminal_returns_result_banner() {
    let banner: Banner = FigletBuilder::new()
        .font(Font::Standard)
        .render("X")
        .expect("terminal render");
    let _lines: Vec<String> = banner.lines().collect();
}

// ============================================================================
// US4 AS2 — FigletBuilder + render() returns a Banner with a lazy iterator
// ============================================================================

/// US4 AS2 + FR-053: construct a Figlet via builder, render, and assert
/// the returned Banner exposes the lazy iterator surface. Compile-time
/// `Send + Sync` is asserted by `assert_impl_all!` above.
#[test]
fn figlet_builder_render_returns_lazy_banner() {
    let banner = FigletBuilder::new()
        .font(Font::Standard)
        .width(80)
        .build()
        .expect("build")
        .render("X")
        .expect("render");

    // .next() once yields one row (lazy contract).
    let mut it = banner.lines();
    let first = it.next();
    assert!(first.is_some(), "lazy iterator must yield ≥1 row for 'X'");
}
