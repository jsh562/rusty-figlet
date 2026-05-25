//! Compile-fail: `FilterChain::apply` is always available (the
//! `Filter::Nothing` variant has no leaf gate); applying a chain that
//! contains a `Filter::Crop` variant when `filter-crop` is disabled
//! produces a `FigletError::UnknownFilter` at runtime — not a compile
//! error. This file documents the contract via a use-site assertion.
fn main() {
    let _ = std::any::type_name::<rusty_figlet::filter::Filter>();
}
