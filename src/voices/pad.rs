//! Pad voice - sustained, atmospheric texture.
//!
//! Pads are the foundation of ambient and atmospheric music. They provide
//! a lush, evolving backdrop that fills sonic space without demanding attention.
//!
//! # How It Works
//!
//! 1. Two detuned sawtooth oscillators create width and movement
//! 2. Slow attack (300ms) for gradual fade-in
//! 3. High sustain keeps the sound alive while held
//! 4. Long release (500ms) for smooth fade-out
//! 5. Low-pass filter softens the brightness
//!
//! # Variations
//!
//! - More detune (20+ cents) = wider, more dramatic
//! - Less detune (5 cents) = subtle, cohesive
//! - Lower filter cutoff = darker, more ambient
//! - Add LFO to filter = evolving, animated texture

use crate::graph::{
    envelope::EnvNode,
    extensions::NodeExt,
    filter::FilterNode,
    oscillator::OscNode,
};

/// Create a pad voice - lush, sustained texture.
///
/// Returns a node graph configured for atmospheric pad sounds.
/// Responds to note pitch for melodic/harmonic use.
pub fn pad() -> impl crate::graph::GraphNode {
    OscNode::sawtooth()
        .mix(OscNode::sawtooth().with_detune(8.0), 0.5) // Slight detune for width
        .amplify(EnvNode::adsr(0.3, 0.1, 0.8, 0.5))    // Slow attack, long release
        .through(FilterNode::lowpass(2500.0))           // Soften the brightness
}
