//! Tom drum voice.
//!
//! A pitched drum similar to kick but at a higher frequency range.
//! Great for fills, tribal patterns, and melodic percussion.
//!
//! # How It Works
//!
//! 1. Sine oscillator with fixed frequency (ignores note pitch)
//! 2. Pitch envelope: starts ~350Hz, drops to ~150Hz over ~60ms
//! 3. Amplitude envelope with instant attack, medium decay
//! 4. Low-pass filter smooths the sound
//!
//! The pitch sweep gives the tom its characteristic "boing" quality.
//! Higher base frequency than kick creates a distinct pitched drum sound.
//!
//! # Variations
//!
//! - Higher base frequency (200-300Hz) = high tom
//! - Lower base frequency (100-150Hz) = floor tom
//! - Longer decay = more resonant, tribal feel
//! - Shorter decay = tighter, more controlled

use crate::graph::{
    envelope::EnvNode,
    extensions::NodeExt,
    filter::FilterNode,
    oscillator::{OscNode, OscParam},
};

/// Create a tom drum voice.
///
/// Returns a node graph configured for pitched tom sounds.
/// The note pitch is ignored - toms use a fixed frequency with pitch envelope.
pub fn tom() -> impl crate::graph::GraphNode {
    // Pitch envelope: fast attack, ~60ms decay for pitch sweep
    let pitch_env = EnvNode::adsr(0.001, 0.06, 0.0, 0.0);

    // Sine wave with fixed frequency + pitch modulation
    // Base: 150Hz, Depth: +200Hz (so starts at 350Hz)
    OscNode::sine()
        .with_frequency(150.0)
        .modulate(pitch_env, OscParam::Frequency, 200.0)
        // Amplitude envelope: instant attack, ~120ms decay
        .amplify(EnvNode::adsr(0.001, 0.12, 0.0, 0.05))
        // Low-pass to keep it smooth
        .through(FilterNode::lowpass(400.0))
}
