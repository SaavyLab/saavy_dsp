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
    let lead = pattern!(4/4 => [C4, E4, G4, C5]);
    let bass = pattern!(4/4 => [C2, _, C2, _]);

    // Build and run
    Saavy::new()
        .bpm(120.0)
        .track(
            "lead",
            lead.repeat(4),
            OscNode::sawtooth()
                .amplify(EnvNode::adsr(0.01, 0.1, 0.6, 0.2))
                .through(FilterNode::lowpass(2000.0)),
        )
        .track(
            "bass",
            bass.repeat(4),
            OscNode::sine()
                .amplify(EnvNode::adsr(0.05, 0.2, 0.8, 0.3))
                .through(FilterNode::lowpass(500.0)),
        )
        .run()
}
