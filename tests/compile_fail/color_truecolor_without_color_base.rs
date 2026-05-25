//! Compile-fail: `color-truecolor` declares `color` as a dependency in
//! Cargo.toml (`color-truecolor = ["color"]`). Enabling truecolor without
//! also enabling `color` is a Cargo configuration error.
fn main() {
    let _ = std::any::type_name::<rusty_figlet::ColorDepth>();
}
