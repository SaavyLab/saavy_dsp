//! saavy - Terminal synthesizer interface
//!
//! Run with: cargo run

mod app;
mod sequencer;
mod track;
mod ui;

use app::Saavy;
use saavy_dsp::{
    graph::{envelope::EnvNode, extensions::NodeExt, filter::FilterNode, oscillator::OscNode},
    pattern,
    sequencing::*,
};

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    // Define patterns
    let melody = pattern!(4/4 => [C4, Eb4, [F4, Eb4], F4]);
    let bass = pattern!(4/4 => [C2, _, G2, _]);

    // Build and run
    Saavy::new()
        .bpm(120.0)
        .track(
            "lead",
            melody.repeat(4),
            OscNode::sawtooth()
                .amplify(EnvNode::adsr(0.01, 0.1, 0.6, 0.2))
                .through(FilterNode::lowpass(2000.0)),
        )
        .track(
            "bass",
            bass.repeat(4),
            OscNode::square()
                .amplify(EnvNode::adsr(0.01, 0.05, 0.8, 0.1))
                .through(FilterNode::lowpass(400.0)),
        )
        .run()
}
