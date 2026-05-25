//! Generate the v0.3.0 SYNTHETIC seed corpus for the
//! `tests/fixtures/toilet-corpus/` directory (E012 T056).
//!
//! Run once with:
//!
//! ```text
//! cargo run --example gen_synthetic_corpus \
//!   --features "tlf-parser filter-crop filter-gay toilet-strict-compat"
//! ```
//!
//! The output files are **placeholders** pending the
//! `workflow_dispatch`-triggered CI capture per
//! `docs/strict-compat-corpus-capture.md` §8. They are derived from
//! `rusty-figlet`'s own engine using documented `toilet` filter
//! semantics (per the toilet manpage):
//!
//! - `nothing` is identity ⇒ identical to figlet output.
//! - `crop` strips surrounding blank rows / cols ⇒ apply `Filter::Crop`.
//! - `gay` adds per-column rainbow color downgraded to 16-color floor ⇒
//!   apply `Filter::Gay` then 16-color downgrade.
//!
//! When the real CI capture lands, the captured bytes REPLACE these
//! synthetic fixtures. Byte-level mismatches surface as a PR diff that
//! the reviewer can audit.

use std::fs;
use std::path::Path;

use rusty_figlet::StrictTarget;
use rusty_figlet::filter::{Filter, FilterChain};
use rusty_figlet::strict_toilet::strict_render;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fixtures = [
        ("nothing_hi", "hi", FilterChain::new(), "nothing"),
        (
            "crop_hi",
            "hi",
            FilterChain::new().push(Filter::Crop),
            "crop",
        ),
        ("gay_hi", "hi", FilterChain::new().push(Filter::Gay), "gay"),
    ];

    let root = Path::new("tests/fixtures/toilet-corpus");
    for (name, input, chain, chain_name) in fixtures {
        let dir = root.join(name);
        fs::create_dir_all(&dir)?;
        let bytes = strict_render(input, &chain, StrictTarget::Toilet031)?;
        fs::write(dir.join("input.txt"), input)?;
        fs::write(dir.join("filter.txt"), chain_name)?;
        fs::write(dir.join("expected.bin"), &bytes)?;
        eprintln!(
            "wrote {} ({} bytes)",
            dir.join("expected.bin").display(),
            bytes.len()
        );
    }
    Ok(())
}
