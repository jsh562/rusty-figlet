//! Compile-fail: `color-256` declares `color` as a dependency in
//! Cargo.toml (`color-256 = ["color"]`). Enabling 256-color without
//! also enabling `color` is a Cargo configuration error.
fn main() {
    let _ = std::any::type_name::<rusty_figlet::ColorDepth>();
}
