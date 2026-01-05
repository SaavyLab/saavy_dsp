//! Ride cymbal voice.
//!
//! A shimmering, sustained cymbal sound. Rides have more body and sustain
//! than hi-hats, providing a flowing rhythmic texture.
//!
//! # How It Works
//!
//! 1. White noise for metallic character
//! 2. Band-pass filter emphasizes the "ping" frequencies
//! 3. Longer envelope with some sustain for shimmer
//!
//! # Variations
//!
//! - Shorter decay = more staccato, jazz ride
//! - Bell sound = add sine wave at ~3kHz mixed in
//! - Darker = lower filter frequency

use crate::graph::{envelope::EnvNode, extensions::NodeExt, filter::FilterNode, oscillator::OscNode};

/// Create a ride cymbal voice.
///
/// Returns a node graph configured for shimmering ride sounds.
pub fn ride() -> impl crate::graph::GraphNode {
    OscNode::noise()
        .amplify(EnvNode::adsr(0.001, 0.3, 0.1, 0.2))
        .through(FilterNode::bandpass(5000.0))
}
