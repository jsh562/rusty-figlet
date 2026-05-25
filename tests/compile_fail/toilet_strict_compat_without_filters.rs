//! Compile-fail: strict-compat mode references `FilterChain`; enabling
//! `toilet-strict-compat` without any filter leaf compiles but produces
//! a chain whose only valid filter is `Nothing`.
fn main() {
    let _ = std::any::type_name::<rusty_figlet::filter::FilterChain>();
}
