//! Crash cymbal voice.
//!
//! A big, explosive cymbal hit. Crashes mark transitions and accents,
//! washing over the mix with bright, sustained noise.
//!
//! # How It Works
//!
//! 1. White noise for full-spectrum metallic character
//! 2. High-pass filter keeps it bright and airy
//! 3. Long decay lets it wash and fade naturally
//!
//! # Variations
//!
//! - Shorter decay = "choke" effect
//! - Lower filter = darker, trashier crash
//! - Mix with sine for more "gong" character

use crate::graph::{envelope::EnvNode, extensions::NodeExt, filter::FilterNode, oscillator::OscNode};

/// Create a crash cymbal voice.
///
/// Returns a node graph configured for explosive crash sounds.
pub fn crash() -> impl crate::graph::GraphNode {
    OscNode::noise()
        .amplify(EnvNode::adsr(0.001, 0.8, 0.05, 0.5))
        .through(FilterNode::highpass(3000.0))
}
