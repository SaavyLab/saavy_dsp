//! Debussy's Première Arabesque (1888) - Public Domain
//!
//! A simplified arrangement demonstrating:
//! - Flowing arpeggiated patterns
//! - Polyrhythms: triplets in RH against eighths in LH
//! - Layered voices creating a piano-like texture
//!
//! Run with: cargo run --example arabesque

use saavy_dsp::{
    graph::{envelope::EnvNode, extensions::NodeExt, filter::FilterNode, oscillator::OscNode},
    pattern,
    runtime::Saavy,
    sequencing::*,
};

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    // --- Section A: Opening Arpeggios (measures 1-4) ---
    // Flowing sixteenth-note arpeggios in E major
    let arp_rh_1 = pattern!(4/4 => [
        [E4, Gs4, B4, E5], [Gs4, B4, E5, Gs5], [B4, E5, Gs5, B5], [E5, Gs5, B5, E6]
    ]);
    let arp_rh_2 = pattern!(4/4 => [
        [Ds5, Gs5, B5, Ds6], [B4, Ds5, Gs5, B5], [Gs4, B4, Ds5, Gs5], [E4, Gs4, B4, E5]
    ]);
    let arp_lh = pattern!(4/4 => [E2, _, E2, _]);

    // --- Section B: Triplets over Eighths (the polyrhythm section) ---
    // RH: Triplet figures - 3 notes per beat
    let triplets_rh = pattern!(4/4 => [
        [E5, Gs5, B5], [E5, Gs5, B5], [Ds5, Fs5, B5], [Ds5, Fs5, B5]
    ]);
    let triplets_rh_2 = pattern!(4/4 => [
        [Cs5, E5, A5], [Cs5, E5, A5], [B4, Ds5, Gs5], [B4, Ds5, Gs5]
    ]);
    // LH: Steady eighth notes - 2 notes per beat (3-against-2 polyrhythm)
    let eighths_lh = pattern!(4/4 => [
        [E3, B3], [E3, B3], [B2, Fs3], [B2, Fs3]
    ]);
    let eighths_lh_2 = pattern!(4/4 => [
        [A2, E3], [A2, E3], [Gs2, Ds3], [Gs2, Ds3]
    ]);

    // --- Section C: Return to arpeggios ---
    let arp_rh_3 = pattern!(4/4 => [
        [E4, B4, E5, Gs5], [Gs4, E5, Gs5, B5], [B4, Gs5, B5, E6], [E5, B5, E6, Gs6]
    ]);

    // --- Assemble the piece ---
    let rh_part = arp_rh_1
        .clone()
        .then(arp_rh_2.clone())
        .then(arp_rh_1.clone())
        .then(arp_rh_2.clone())
        // Triplet section (polyrhythm!)
        .then(triplets_rh.clone())
        .then(triplets_rh_2.clone())
        .then(triplets_rh.clone())
        .then(triplets_rh_2.clone())
        // Return
        .then(arp_rh_3)
        .then(arp_rh_1);

    let lh_part = arp_lh
        .clone()
        .repeat(4)
        // Eighth note section (against triplets = polyrhythm)
        .then(eighths_lh.clone())
        .then(eighths_lh_2.clone())
        .then(eighths_lh)
        .then(eighths_lh_2)
        // Return
        .then(arp_lh.clone())
        .then(arp_lh);

    // --- Voices ---
    // Soft, bell-like voice for the melody/arpeggios
    let rh_voice = OscNode::triangle()
        .amplify(EnvNode::adsr(0.01, 0.3, 0.3, 0.4))
        .through(FilterNode::lowpass(3000.0));

    // Warm bass for the left hand
    let lh_voice = OscNode::sine()
        .mix(OscNode::triangle(), 0.3)
        .amplify(EnvNode::adsr(0.01, 0.2, 0.5, 0.3))
        .through(FilterNode::lowpass(800.0));

    // --- Run with TUI ---
    Saavy::new()
        .bpm(72.0) // Andantino con moto
        .track("rh", rh_part, rh_voice)
        .track("lh", lh_part, lh_voice)
        .run()
}
