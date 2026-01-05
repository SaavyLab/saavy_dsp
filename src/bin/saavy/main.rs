//! saavy - Terminal synthesizer interface
//!
//! Run with: cargo run

mod app;
mod sequencer;
mod track;
mod ui;

use app::Saavy;
use saavy_dsp::{pattern, sequencing::*, voices};

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    // Define patterns
    let melody = pattern!(4/4 => [C4, Eb4, [F4, Eb4], F4]);
    let bassline = pattern!(4/4 => [C2, _, G2, _]);
    let kicks = pattern!(4/4 => [C2, _, C2, _]);
    let snares = pattern!(4/4 => [_, C3, _, C3]);
    let hats = pattern!(4/4 => [[C4, C4], [C4, C4], [C4, C4], [C4, C4]]);

    // Build and run
    Saavy::new()
        .bpm(120.0)
        .track("bass", bassline.repeat(4), voices::bass())
        .track("kick", kicks.repeat(4), voices::kick())
        .track("snare", snares.repeat(4), voices::snare())
        .track("hihat", hats.repeat(4), voices::hihat())
        .run()
}
