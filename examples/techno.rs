//! Techno Example
//!
//! Driving, hypnotic techno featuring:
//! - Four-on-floor kick (kick on every beat)
//! - Acid-style bass with resonant filter
//! - Minimal hi-hats for momentum
//! - Build/drop energy with filter sweeps
//!
//! Techno is all about the groove and hypnotic repetition.
//! The resonant filter on the bass creates that classic
//! "acid" sound made famous by the Roland TB-303.
//!
//! Run with: cargo run --example techno

use saavy_dsp::{
    graph::{
        envelope::EnvNode,
        extensions::NodeExt,
        filter::{FilterNode, FilterParam},
        lfo::LfoNode,
        oscillator::OscNode,
    },
    pattern,
    runtime::Saavy,
    sequencing::*,
    voices,
};

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    // === ACID BASS ===
    // The heart of techno - resonant filter creates that squelchy sound
    // LFO slowly sweeps the filter for evolving texture

    let filter_lfo = LfoNode::triangle(0.1);  // Slow sweep over ~10 seconds

    let acid_bass = OscNode::sawtooth()
        .through(
            FilterNode::lowpass(400.0)
                .with_resonance(0.8)  // High resonance for acid squelch
                .modulate(filter_lfo, FilterParam::Cutoff, 800.0)  // Sweep 400-1200 Hz
        )
        .amplify(EnvNode::adsr(0.001, 0.1, 0.6, 0.1))
        .gain(1.5);

    // Simple, hypnotic bass pattern
    let bass_pattern = pattern!(4/4 => [[E1, E1], [E1, _], [E1, E1], [E1, _]])
        .repeat(12);

    // === STAB SYNTH ===
    // Occasional stabs for tension
    let stab_voice = OscNode::square()
        .mix(OscNode::sawtooth(), 0.3)
        .through(FilterNode::lowpass(2000.0))
        .amplify(EnvNode::adsr(0.001, 0.08, 0.0, 0.05))
        .gain(0.6);

    let stab_pattern = pattern!(4/4 => [_, _, E3, _])
        .then(pattern!(4/4 => [_, _, _, _]))
        .then(pattern!(4/4 => [_, _, G3, _]))
        .then(pattern!(4/4 => [_, _, _, _]))
        .repeat(3);

    // === DRUMS ===
    // Four-on-floor kick - the backbone of techno
    let kick_pattern = pattern!(4/4 => [C2, C2, C2, C2]).repeat(12);

    // Clap on 2 and 4 (instead of snare for that electronic feel)
    let clap_pattern = pattern!(4/4 => [_, C3, _, C3]).repeat(12);

    // Off-beat hats for momentum
    let hat_pattern = pattern!(4/4 => [_, C4, _, C4]).repeat(12);

    // Open hat on the "&" of 4 for groove
    let open_hat_pattern = pattern!(4/4 => [_, _, _, [_, C4]]).repeat(12);

    // === RUN IT ===
    Saavy::new()
        .bpm(130.0)  // Classic techno tempo
        .track("kick", kick_pattern, voices::kick())
        .track("clap", clap_pattern, voices::clap())
        .track("hats", hat_pattern, voices::hihat())
        .track("open", open_hat_pattern, voices::openhat())
        .track("bass", bass_pattern, acid_bass)
        .track("stab", stab_pattern, stab_voice)
        .run()
}
