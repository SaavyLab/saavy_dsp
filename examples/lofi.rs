//! Lo-fi Hip Hop Example
//!
//! Chill, downtempo vibes featuring:
//! - Dusty, filtered keys (warm triangle waves)
//! - Lazy drums with swing feel
//! - Simple chord progression
//! - Heavy low-pass filtering for that "vinyl" character
//!
//! Lo-fi hip hop is characterized by its imperfect, nostalgic sound.
//! The heavy filtering removes high frequencies, creating warmth.
//! Swing timing makes the drums feel relaxed and human.
//!
//! Run with: cargo run --example lofi

use saavy_dsp::{
    graph::{
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

    // === DUSTY KEYS ===
    // Warm, mellow sound with heavy filtering
    let keys_voice = OscNode::triangle()
        .mix(OscNode::sine(), 0.3)  // Add some sine warmth
        .amplify(EnvNode::adsr(0.02, 0.3, 0.4, 0.4))
        .through(FilterNode::lowpass(1200.0))  // Heavy filtering for lo-fi warmth
        .gain(0.8);

    // Simple chord progression: Am - F - C - G (in root position, simplified)
    // Using single notes for simplicity
    let keys_am = pattern!(4/4 => [A3, _, E4, _]);
    let keys_f  = pattern!(4/4 => [F3, _, C4, _]);
    let keys_c  = pattern!(4/4 => [C3, _, G3, _]);
    let keys_g  = pattern!(4/4 => [G3, _, D4, _]);

    let keys_progression = keys_am.then(keys_f).then(keys_c).then(keys_g).repeat(3);

    // === BASS ===
    // Deep, round bass following the chord roots
    let bass_voice = OscNode::sine()
        .mix(OscNode::triangle(), 0.2)
        .amplify(EnvNode::adsr(0.01, 0.15, 0.6, 0.2))
        .through(FilterNode::lowpass(400.0))
        .gain(1.2);

    let bass_am = pattern!(4/4 => [A1, _, _, _]);
    let bass_f  = pattern!(4/4 => [F1, _, _, _]);
    let bass_c  = pattern!(4/4 => [C2, _, _, _]);
    let bass_g  = pattern!(4/4 => [G1, _, _, _]);

    let bass_line = bass_am.then(bass_f).then(bass_c).then(bass_g).repeat(3);

    // === LAZY DRUMS ===
    // Relaxed pattern with simple groove

    // Kick on 1 and 3
    let kick_pattern = pattern!(4/4 => [C2, _, C2, _]).repeat(12);

    // Snare on 2 and 4 (backbeat)
    let snare_pattern = pattern!(4/4 => [_, C3, _, C3]).repeat(12);

    // Closed hats - simple eighths
    let hat_pattern = pattern!(4/4 => [[C4, C4], [C4, C4], [C4, C4], [C4, C4]]).repeat(12);

    // === RUN IT ===
    Saavy::new()
        .bpm(82.0)  // Slow, chill tempo
        .track("keys", keys_progression, keys_voice)
        .track("bass", bass_line, bass_voice)
        .track("kick", kick_pattern, voices::kick())
        .track("snare", snare_pattern, voices::snare())
        .track("hats", hat_pattern, voices::hihat())
        .run()
}
