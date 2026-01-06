//! Clap voice - punchy, bright hand clap.
//!
//! A classic electronic clap sound. Uses filtered noise with a sharp
//! envelope to create that punchy, snappy character.
//!
//! # How It Works
//!
//! 1. White noise source
//! 2. Bandpass filter focuses the frequency range (~1.5kHz center)
//! 3. Quick envelope with slight attack (5ms) for "clap" feel
//! 4. Short decay, no sustain for punchy character
//! 5. Boosted gain to cut through the mix
//!
//! The bandpass filter is key - it removes both the low rumble and
//! ultra-high hiss, leaving the characteristic "crack" frequencies.
//!
//! # Variations
//!
//! - Higher bandpass center = thinner, more "crack"
//! - Lower bandpass center = fuller, more "thwack"
//! - Longer decay = more reverberant room feel
//! - Add slight reverb/delay for depth

use crate::graph::{
    envelope::EnvNode,
    extensions::NodeExt,
    filter::FilterNode,
    oscillator::OscNode,
};

/// Create a clap voice.
///
/// Returns a node graph configured for punchy clap sounds.
/// The note pitch is ignored - claps are unpitched percussion.
pub fn clap() -> impl crate::graph::GraphNode {
    OscNode::noise()
        // Bandpass focuses on the "crack" frequencies
        .through(FilterNode::bandpass(1500.0))
        // Quick envelope with slight attack for clap character
        .amplify(EnvNode::adsr(0.005, 0.08, 0.0, 0.1))
        // Boost to cut through mix
        .gain(1.5)
}
