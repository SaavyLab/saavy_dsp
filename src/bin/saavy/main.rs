//! saavy - Terminal synthesizer interface
//!
//! Run with: cargo run

use saavy_dsp::{
    graph::{envelope::EnvNode, extensions::NodeExt, filter::FilterNode, oscillator::OscNode},
    pattern,
    runtime::Saavy,
    sequencing::*,
    voices,
};

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    // Define patterns
    let melody = pattern!(4/4 => [C4, Eb4, [F4, Eb4], F4]);
    let bassline = pattern!(4/4 => [C2, _, G2, _]);
    let kicks = pattern!(4/4 => [C2, _, C2, _]);
    let snares = pattern!(4/4 => [_, C3, _, C3]);
    let hats = pattern!(4/4 => [[C4, C4], [C4, C4], [C4, C4], [C4, C4]]);

    let lead_node = OscNode::sawtooth()
        .amplify(EnvNode::adsr(0.01, 0.1, 0.6, 0.2))
        .through(FilterNode::lowpass(2500.0));

    // Build and run
    Saavy::new()
        .bpm(120.0)
        .track("lead", melody.repeat(4), lead_node)
        .track("bass", bassline.repeat(4), voices::bass())
        .track("kick", kicks.repeat(4), voices::kick())
        .track("snare", snares.repeat(4), voices::snare())
        .track("hihat", hats.repeat(4), voices::hihat())
        .run()
}
