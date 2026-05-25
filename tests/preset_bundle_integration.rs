//! E012 Phase 10 — preset bundle exact-membership assertion (T070).
//!
//! Parses the crate's own `Cargo.toml` and asserts:
//!
//! 1. `figlet-toilet-compat` contains EXACTLY the v0.3.0 toilet-parity set
//!    (`cli`, `color`, `rainbow`, `tlf-parser`, `filter-crop`, `filter-gay`,
//!    `filter-metal`, `filter-flip`, `filter-flop`, `filter-rotate`,
//!    `filter-border`) — no more, no less. Fails on missing OR extra leaves.
//! 2. `figlet-color` retains v0.2.x semantics: `cli`, `color`, `rainbow`.
//! 3. `full` umbrella enumerates the 19-leaf v0.3.0 surface.
//!
//! Per plan §preset-bundles + FR-013 + SC-010 (figlet-toilet-compat is the
//! canonical anchor for the portfolio-wide toilet-parity convention).

use std::collections::BTreeSet;

fn parse_features() -> toml::Value {
    let cargo_toml =
        std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml")).unwrap();
    cargo_toml.parse::<toml::Value>().unwrap()
}

fn bundle_set(name: &str) -> BTreeSet<String> {
    let v = parse_features();
    let features = v
        .get("features")
        .and_then(|f| f.as_table())
        .unwrap_or_else(|| panic!("Cargo.toml missing [features]"));
    let arr = features
        .get(name)
        .and_then(|x| x.as_array())
        .unwrap_or_else(|| panic!("Cargo.toml [features] missing `{name}`"));
    arr.iter()
        .filter_map(|v| v.as_str().map(|s| s.to_owned()))
        .collect()
}

#[test]
fn figlet_toilet_compat_composes_toilet_parity_leaves_exactly() {
    let expected: BTreeSet<String> = [
        "cli",
        "color",
        "rainbow",
        "tlf-parser",
        "filter-crop",
        "filter-gay",
        "filter-metal",
        "filter-flip",
        "filter-flop",
        "filter-rotate",
        "filter-border",
    ]
    .iter()
    .map(|&s| s.to_owned())
    .collect();
    let actual = bundle_set("figlet-toilet-compat");
    assert_eq!(
        actual, expected,
        "figlet-toilet-compat must compose the v0.3.0 toilet-parity set exactly"
    );
}

#[test]
fn figlet_color_retains_v02x_semantics() {
    let expected: BTreeSet<String> = ["cli", "color", "rainbow"]
        .iter()
        .map(|&s| s.to_owned())
        .collect();
    let actual = bundle_set("figlet-color");
    assert_eq!(
        actual, expected,
        "figlet-color must retain v0.2.x semantics (cli + color + rainbow) per AD-010"
    );
}

#[test]
fn full_umbrella_enumerates_v030_19_leaves() {
    let expected: BTreeSet<String> = [
        // v0.2.x leaves (6)
        "cli",
        "color",
        "rainbow",
        "terminal-width",
        "completions",
        "strict-compat",
        // v0.3.0 leaves (13 — TLF + 7 filters + 2 color depth + 3 output + strict-toilet)
        "tlf-parser",
        "filter-crop",
        "filter-gay",
        "filter-metal",
        "filter-flip",
        "filter-flop",
        "filter-rotate",
        "filter-border",
        "color-truecolor",
        "color-256",
        "output-html",
        "output-irc",
        "output-svg",
        "toilet-strict-compat",
    ]
    .iter()
    .map(|&s| s.to_owned())
    .collect();
    let actual = bundle_set("full");
    assert_eq!(
        actual, expected,
        "full umbrella must enumerate all v0.3.0 leaves"
    );
}

#[test]
fn version_is_030() {
    let v = parse_features();
    let pkg = v.get("package").and_then(|p| p.as_table()).unwrap();
    let version = pkg.get("version").and_then(|x| x.as_str()).unwrap();
    assert_eq!(version, "0.3.0");
}
