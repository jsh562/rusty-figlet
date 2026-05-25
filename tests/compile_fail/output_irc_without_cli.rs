//! Compile-fail: same shape as `output_html_without_cli.rs` for IRC.
fn main() {
    let _ = std::any::type_name::<rusty_figlet::export::ExportFormat>();
}
