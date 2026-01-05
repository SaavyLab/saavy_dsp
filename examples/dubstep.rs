//! Dubstep / Brostep Example
//!
//! Classic early 2010s dubstep sounds featuring:
//! - "Wub" bass: LFO modulating filter cutoff for that iconic wobble
//! - Half-time drums: snare on beat 3 (not 2&4) for that heavy feel
//! - 140 BPM but feels like 70 due to half-time
//!
//! The "wub" is created by an LFO (low frequency oscillator) rapidly
//! opening and closing a low-pass filter on a harmonically rich sound.
//! Faster LFO = faster wobble. Square wave LFO = harder chops.
//!
//! Run with: cargo run --example dubstep

use saavy_dsp::{
    graph::{
        delay::DelayNode,
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

    // === DUBSTEP DRUMS (half-time feel) ===
    // At 140 BPM, the snare hits on beat 3, making it feel like 70 BPM

    // Kick on 1 and 3 (feels like just beat 1 in half-time)
    let kicks = pattern!(4/4 => [C2, _, C2, _]);

    // Snare on beat 3 only (the half-time signature)
    let snares = pattern!(4/4 => [_, _, C3, _]);

    // Hi-hats: eighth notes for drive
    let hats = pattern!(4/4 => [[C4, C4], [C4, C4], [C4, C4], [C4, C4]]);

    // === THE WUB BASS ===
    // This is where the magic happens!

    // Pattern: long sustained notes to let the wobble breathe
    // Using low notes for that sub-bass foundation
    let wub_pattern = pattern!(4/4 => [E1, _, _, _]); // Whole note wub
    let wub_pattern_2 = pattern!(4/4 => [G1, _, _, _]);
    let wub_pattern_3 = pattern!(4/4 => [A1, _, _, _]);

    // Build the wub bass voice:
    // 1. Rich harmonics from stacked oscillators
    // 2. LFO modulates filter cutoff for the "wub wub wub"
    // 3. Resonant lowpass for that squelchy character

    // Square wave LFO at 4 Hz = 4 wubs per second (eighth note wobble at 120bpm-ish feel)
    // Try different rates: 2 Hz = slower/heavier, 8 Hz = faster/more intense
    let wub_lfo = LfoNode::square(4.0);

    let wub_bass = OscNode::sawtooth()
        .mix(OscNode::square(), 0.5)  // Layer saw + square for thickness
        .through(
            FilterNode::lowpass(400.0)
                .with_resonance(0.7)  // Resonance for that squelchy character
                .modulate(wub_lfo, FilterParam::Cutoff, 600.0)  // LFO sweeps 400-1000 Hz
        )
        .amplify(EnvNode::adsr(0.01, 0.1, 0.8, 0.3))  // Quick attack, sustain for wobble
        .gain(4.0);  // Boost the wub - it's the star of the show!

    // === REESE BASS (alternative: detuned saws for that dark rumble) ===
    let reese_pattern = pattern!(4/4 => [_, E1, _, _]); // Off-beat hits

    // Classic reese: two slightly detuned saws create beating/movement
    let reese_bass = OscNode::sawtooth()
        .mix(OscNode::sawtooth(), 0.5)  // Would be better with detune, but this adds fatness
        .through(FilterNode::lowpass(300.0))
        .amplify(EnvNode::adsr(0.05, 0.2, 0.6, 0.4))
        .gain(2.5);  // Boost to sit with the wub

    // === LEAD STAB (for drops/accents) ===
    let stab_pattern = pattern!(4/4 => [[E4, _, _, _], [_, _, _, _], [G4, _, _, _], [_, _, _, _]]);

    let lead_stab = OscNode::square()
        .through(FilterNode::lowpass(2000.0))
        .amplify(EnvNode::adsr(0.001, 0.15, 0.0, 0.1))  // Super short, punchy
        .gain(0.6)  // Pull back so bass dominates
        .through(DelayNode::new(187.5, 0.3, 0.3));  // Delay synced-ish to tempo (187.5ms ≈ dotted 8th at 140)

    // === ARRANGEMENT ===
    // Intro: just drums
    // Drop: bring in the wub

    let drum_intro = kicks.clone().repeat(4);
    let full_drums = kicks.clone()
        .then(kicks.clone())
        .then(kicks.clone())
        .then(kicks.clone())
        .repeat(2);

    let snare_intro = pattern!(4/4 => [_, _, _, _]).repeat(4);  // Silent intro
    let full_snares = snares.clone().repeat(8);

    let hat_pattern = hats.clone().repeat(12);

    // Wub comes in after intro
    let wub_intro = pattern!(4/4 => [_, _, _, _]).repeat(4);  // Silent during intro
    let wub_drop = wub_pattern.clone()
        .then(wub_pattern.clone())
        .then(wub_pattern_2.clone())
        .then(wub_pattern_3.clone())
        .repeat(2);

    let reese_full = reese_pattern.clone().repeat(12);
    let stab_full = stab_pattern.clone().repeat(12);

    // === RUN IT ===
    Saavy::new()
        .bpm(140.0)  // Classic dubstep tempo
        .track("kick", drum_intro.concat(full_drums), voices::kick())
        .track("snare", snare_intro.concat(full_snares), voices::snare())
        .track("hats", hat_pattern, voices::hihat())
        .track("wub", wub_intro.concat(wub_drop), wub_bass)
        .track("reese", reese_full, reese_bass)
        .track("stab", stab_full, lead_stab)
        .run()
}
