//! Pluck voice - percussive, quickly-decaying note.
//!
//! Plucks are short, punchy sounds that work great for melodies, arpeggios,
//! and rhythmic patterns. Think kalimba, harp, or synth pluck.
//!
//! # How It Works
//!
//! 1. Triangle wave for soft, bell-like tone
//! 2. Instant attack (1ms) for immediate response
//! 3. Medium decay (150ms) with no sustain
//! 4. Short release (100ms) for quick tail
//! 5. Bright filter lets harmonics through
//!
//! # Variations
//!
//! - Shorter decay (50-80ms) = more percussive, staccato
//! - Longer decay (200-300ms) = more bell-like, ringing
//! - Sine wave = purer, more mellow
//! - Square wave = more hollow, synthetic
//! - Lower filter = darker, softer attack

use crate::graph::{
    envelope::EnvNode,
    extensions::NodeExt,
    filter::FilterNode,
    oscillator::OscNode,
};

/// Create a pluck voice - percussive, fast-decaying.
///
/// Returns a node graph configured for punchy pluck sounds.
/// Responds to note pitch for melodic use.
pub fn pluck() -> impl crate::graph::GraphNode {
    OscNode::triangle()
        .amplify(EnvNode::adsr(0.001, 0.15, 0.0, 0.1)) // Instant attack, quick decay
        .through(FilterNode::lowpass(4000.0))           // Bright but not harsh
}
