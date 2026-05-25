//! Verify that the bundled placeholder `.tlf` files parse cleanly via the
//! v0.3.0 TLF parser (E012 Phase 3 — T013 validation).
//!
//! These tests exercise `Figlet::from_tlf_bytes` end-to-end with the
//! actual on-disk placeholder bytes embedded via `include_bytes!`, so a
//! regression in either the parser or the placeholder generator triggers
//! a CI failure.

#![cfg(feature = "tlf-parser")]

use rusty_figlet::Figlet;

const MONO9_TLF: &[u8] = include_bytes!("../assets/fonts/mono9.tlf");
const FUTURE_TLF: &[u8] = include_bytes!("../assets/fonts/future.tlf");
const PAGGA_TLF: &[u8] = include_bytes!("../assets/fonts/pagga.tlf");

#[test]
fn mono9_placeholder_parses() {
    let _figlet = Figlet::from_tlf_bytes(MONO9_TLF).expect("mono9 parses");
}

#[test]
fn future_placeholder_parses() {
    let _figlet = Figlet::from_tlf_bytes(FUTURE_TLF).expect("future parses");
}

#[test]
fn pagga_placeholder_parses() {
    let _figlet = Figlet::from_tlf_bytes(PAGGA_TLF).expect("pagga parses");
}

#[test]
fn mono9_renders_text() {
    let figlet = Figlet::from_tlf_bytes(MONO9_TLF).expect("parses");
    let banner = figlet.render("hi").expect("renders");
    assert!(banner.height() >= 1);
}
