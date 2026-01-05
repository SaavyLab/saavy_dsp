//! Lead voice.
//!
//! A bright, cutting lead sound using a sawtooth wave. Sawtooth waves
//! contain all harmonics (both odd and even), making them ideal for
//! leads that need to cut through a mix.
//!
//! # How It Works
//!
//! 1. Sawtooth oscillator provides full harmonic spectrum
//! 2. Low-pass filter tames the brightness (adjustable)
//! 3. Amplitude envelope with some sustain for held notes
//! 4. Moderate cutoff lets the harmonics sing
//!
//! # Variations
//!
//! - Add second oscillator detuned = thicker, "supersaw" sound
//! - Modulate filter with LFO = wah-wah effect
//! - Shorter decay, no sustain = plucky lead
//! - Add delay = spacey lead

use crate::graph::{envelope::EnvNode, extensions::NodeExt, filter::FilterNode, oscillator::OscNode};

/// Create a lead voice.
///
/// Returns a node graph configured for bright, singing lead sounds.
/// Responds to note pitch for playing melodies.
pub fn lead() -> impl crate::graph::GraphNode {
    // Sawtooth for bright, harmonically rich sound
    OscNode::sawtooth()
        // Envelope with sustain for held notes
        .amplify(EnvNode::adsr(0.01, 0.1, 0.6, 0.2))
        // Filter to tame brightness while keeping presence
        .through(FilterNode::lowpass(2500.0))
}
