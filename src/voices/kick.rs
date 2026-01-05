//! Kick drum voice.
//!
//! A classic synthesized kick drum using a sine wave with pitch envelope.
//! The pitch starts high and quickly drops to the fundamental, creating
//! the characteristic "punch" of an electronic kick.
//!
//! # How It Works
//!
//! 1. Sine oscillator with fixed base frequency (ignores note pitch)
//! 2. Pitch envelope: starts ~150Hz, drops to ~50Hz over ~80ms
//! 3. Amplitude envelope with instant attack, quick decay
//! 4. Low-pass filter removes any harshness
//!
//! The pitch sweep is what gives the kick its "punch" - the initial high
//! frequency creates the attack transient, then it drops to the fundamental
//! for the body of the sound.
//!
//! # Variations
//!
//! - Longer pitch decay = more "thump" (808-style)
//! - Shorter pitch decay = tighter, punchier
//! - Higher start frequency = more "click" attack
//! - Add noise burst at start = more acoustic character

use crate::graph::{
    envelope::EnvNode,
    extensions::NodeExt,
    filter::FilterNode,
    oscillator::{OscNode, OscParam},
};

/// Create a kick drum voice.
///
/// Returns a node graph configured for punchy electronic kick sounds.
/// The note pitch is ignored - kicks use a fixed frequency with pitch envelope.
pub fn kick() -> impl crate::graph::GraphNode {
    // Pitch envelope: fast attack, ~80ms decay to sweep frequency down
    // Output goes 0 -> 1 (attack) -> 0 (decay), scaled by depth
    let pitch_env = EnvNode::adsr(0.001, 0.08, 0.0, 0.0);

    // Sine wave with fixed frequency + pitch modulation
    // Base: 50Hz (fundamental), Depth: +100Hz (so starts at 150Hz)
    OscNode::sine()
        .with_frequency(50.0)
        .modulate(pitch_env, OscParam::Frequency, 100.0)
        // Amplitude envelope: instant attack, ~150ms decay
        .amplify(EnvNode::adsr(0.001, 0.15, 0.0, 0.05))
        // Low-pass to keep it smooth
        .through(FilterNode::lowpass(200.0))
}
