//! Bass voice.
//!
//! A classic subtractive bass using a square wave filtered down.
//! Square waves have only odd harmonics, giving a hollow, woody character
//! that works well for bass lines.
//!
//! # How It Works
//!
//! 1. Square wave oscillator provides rich harmonic content
//! 2. Low-pass filter removes upper harmonics (subtractive synthesis)
//! 3. Amplitude envelope shapes the note dynamics
//! 4. Low cutoff frequency keeps it deep and warm
//!
//! # Variations
//!
//! - Higher cutoff = more aggressive, "acid" bass
//! - Add filter envelope = classic 303-style bass
//! - Use sawtooth instead = brighter, more present bass
//! - Longer attack = swelling bass pad

use crate::graph::{envelope::EnvNode, extensions::NodeExt, filter::FilterNode, oscillator::OscNode};

/// Create a bass voice.
///
/// Returns a node graph configured for deep, punchy bass sounds.
/// Responds to note pitch for playing bass lines.
pub fn bass() -> impl crate::graph::GraphNode {
    // Square wave for hollow, woody character
    OscNode::square()
        // Snappy envelope for rhythmic bass
        .amplify(EnvNode::adsr(0.01, 0.1, 0.7, 0.15))
        // Low-pass filter keeps it deep
        .through(FilterNode::lowpass(500.0))
}
