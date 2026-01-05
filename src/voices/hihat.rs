//! Hi-hat voice (closed).
//!
//! A tight, short burst of filtered noise. Closed hi-hats are the backbone
//! of most drum patterns, providing rhythmic drive.
//!
//! # How It Works
//!
//! 1. White noise provides the "metallic" character
//! 2. High-pass filter removes low frequencies (hi-hats are bright)
//! 3. Very short envelope for that tight "tss" sound
//!
//! # Variations
//!
//! - Longer decay = open hi-hat
//! - Lower filter = darker, jazzier hat
//! - Add bandpass = more "ringy" metallic tone

use crate::graph::{envelope::EnvNode, extensions::NodeExt, filter::FilterNode, oscillator::OscNode};

/// Create a closed hi-hat voice.
///
/// Returns a node graph configured for tight, bright hi-hat sounds.
pub fn hihat() -> impl crate::graph::GraphNode {
    OscNode::noise()
        .amplify(EnvNode::adsr(0.001, 0.05, 0.0, 0.03))
        .through(FilterNode::highpass(7000.0))
}
