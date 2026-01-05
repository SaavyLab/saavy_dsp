//! Kick drum voice.
//!
//! A classic synthesized kick drum using a sine wave with pitch envelope.
//! The pitch starts high and quickly drops to the fundamental, creating
//! the characteristic "punch" of an electronic kick.
//!
//! # How It Works
//!
//! 1. Sine oscillator provides the body (pure, deep tone)
//! 2. Very fast pitch envelope: starts ~150Hz, drops to ~50Hz
//! 3. Amplitude envelope with instant attack, quick decay
//! 4. Low-pass filter removes any harshness
//!
//! # Variations
//!
//! - Longer decay = boomy 808-style kick
//! - Higher start pitch = more "click" attack
//! - Add noise burst at start = more acoustic character

use crate::graph::{envelope::EnvNode, extensions::NodeExt, filter::FilterNode, oscillator::OscNode};

/// Create a kick drum voice.
///
/// Returns a node graph configured for punchy electronic kick sounds.
/// The note pitch is mostly ignored - kicks are tuned by the voice itself.
pub fn kick() -> impl crate::graph::GraphNode {
    // Sine wave for the body - clean, deep tone
    OscNode::sine()
        // Fast amplitude envelope: punch with quick decay
        .amplify(EnvNode::adsr(0.001, 0.15, 0.0, 0.05))
        // Low-pass to keep it smooth
        .through(FilterNode::lowpass(200.0))
}
