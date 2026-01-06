//! Open hi-hat voice.
//!
//! A sustained, ringing hi-hat sound. While the closed hi-hat (hihat.rs)
//! has a tight, quick decay, the open hat lets the noise ring out longer.
//!
//! # How It Works
//!
//! 1. White noise source (metallic texture)
//! 2. Longer envelope with sustain (150ms decay, 20% sustain, 250ms release)
//! 3. High-pass filter removes low-end rumble
//! 4. Band-limiting high-pass prevents harsh ultra-highs
//!
//! The sustained envelope is what distinguishes open from closed hat.
//!
//! # Variations
//!
//! - Longer sustain = more open, "washy" feel
//! - Higher high-pass cutoff = thinner, more "tsss"
//! - Lower high-pass cutoff = fuller, more body
//! - Add bandpass resonance for more metallic character

use crate::graph::{
    envelope::EnvNode,
    extensions::NodeExt,
    filter::FilterNode,
    oscillator::OscNode,
};

/// Create an open hi-hat voice.
///
/// Returns a node graph configured for sustained hi-hat sounds.
/// The note pitch is ignored - hi-hats are unpitched percussion.
pub fn openhat() -> impl crate::graph::GraphNode {
    OscNode::noise()
        // Longer envelope than closed hat - sustains and rings
        .amplify(EnvNode::adsr(0.001, 0.15, 0.2, 0.25))
        // High-pass removes low-end, keeps it bright
        .through(FilterNode::highpass(7000.0))
        // Gentle low-pass tames harsh ultra-highs
        .through(FilterNode::lowpass(12000.0))
}
