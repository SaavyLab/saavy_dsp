//! Synthwave / Retrowave Example
//!
//! 80s-inspired synth sounds featuring:
//! - Big, detuned pad chords (wide stereo feel)
//! - Arpeggiated bass line
//! - Punchy gated drums
//! - Bright lead stabs
//!
//! Synthwave captures the nostalgic sound of 80s synth music,
//! with lush pads, driving bass, and that characteristic
//! "neon-lit nighttime drive" vibe.
//!
//! Run with: cargo run --example synthwave

use saavy_dsp::{
    graph::{
        chorus::ChorusNode,
        delay::DelayNode,
        envelope::EnvNode,
        extensions::NodeExt,
        filter::FilterNode,
        oscillator::OscNode,
    },
    pattern,
    runtime::Saavy,
    sequencing::*,
    voices,
};

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    // === BIG PAD ===
    // Detuned saws for that wide, lush synthwave sound
    let pad_voice = OscNode::sawtooth()
        .mix(OscNode::sawtooth().with_detune(10.0), 0.5)  // Detune for width
        .mix(OscNode::sawtooth().with_detune(-8.0), 0.33) // More width
        .amplify(EnvNode::adsr(0.4, 0.2, 0.7, 0.6))       // Slow attack, long release
        .through(FilterNode::lowpass(3000.0))
        .through(ChorusNode::new(0.7, 2.5, 0.3))          // Subtle chorus for shimmer
        .gain(0.5);

    // Am - F - C - E progression (classic synthwave)
    let pad_am = pattern!(4/4 => [A3, _, _, _]);
    let pad_f  = pattern!(4/4 => [F3, _, _, _]);
    let pad_c  = pattern!(4/4 => [C4, _, _, _]);
    let pad_e  = pattern!(4/4 => [E3, _, _, _]);

    let pad_progression = pad_am.then(pad_f).then(pad_c).then(pad_e).repeat(3);

    // === ARPEGGIATED BASS ===
    // Driving arpeggio pattern
    let bass_voice = OscNode::sawtooth()
        .through(FilterNode::lowpass(800.0).with_resonance(0.3))
        .amplify(EnvNode::adsr(0.005, 0.1, 0.5, 0.1))
        .gain(0.9);

    // Sixteenth note arpeggio pattern
    let bass_am = pattern!(4/4 => [[A1, A2, E2, A2], [A1, A2, E2, A2], [A1, A2, E2, A2], [A1, A2, E2, A2]]);
    let bass_f  = pattern!(4/4 => [[F1, F2, C2, F2], [F1, F2, C2, F2], [F1, F2, C2, F2], [F1, F2, C2, F2]]);
    let bass_c  = pattern!(4/4 => [[C2, C3, G2, C3], [C2, C3, G2, C3], [C2, C3, G2, C3], [C2, C3, G2, C3]]);
    let bass_e  = pattern!(4/4 => [[E1, E2, B1, E2], [E1, E2, B1, E2], [E1, E2, B1, E2], [E1, E2, B1, E2]]);

    let bass_arp = bass_am.then(bass_f).then(bass_c).then(bass_e).repeat(3);

    // === LEAD STAB ===
    // Bright, punchy lead for accents
    let lead_voice = OscNode::square()
        .through(FilterNode::lowpass(4000.0))
        .amplify(EnvNode::adsr(0.001, 0.15, 0.0, 0.1))
        .through(DelayNode::new(250.0, 0.25, 0.3))  // Tempo-synced delay
        .gain(0.4);

    // Sparse stabs on the off-beats
    let lead_pattern = pattern!(4/4 => [_, E4, _, _])
        .then(pattern!(4/4 => [_, _, _, _]))
        .then(pattern!(4/4 => [_, G4, _, _]))
        .then(pattern!(4/4 => [_, _, _, _]))
        .repeat(3);

    // === DRUMS ===
    // Four-on-floor kick with gated feel
    let kick_pattern = pattern!(4/4 => [C2, C2, C2, C2]).repeat(12);  // Four on the floor
    let snare_pattern = pattern!(4/4 => [_, C3, _, C3]).repeat(12);   // 2 and 4
    let hat_pattern = pattern!(4/4 => [[C4, C4], [C4, C4], [C4, C4], [C4, C4]]).repeat(12);

    // === RUN IT ===
    Saavy::new()
        .bpm(110.0)  // Classic synthwave tempo
        .track("pad", pad_progression, pad_voice)
        .track("bass", bass_arp, bass_voice)
        .track("lead", lead_pattern, lead_voice)
        .track("kick", kick_pattern, voices::kick())
        .track("snare", snare_pattern, voices::snare())
        .track("hats", hat_pattern, voices::hihat())
        .run()
}
