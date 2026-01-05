//! Snare drum voice.
//!
//! A synthesized snare combining a tonal body with noise for the "snare" rattle.
//! Real snares have metal wires stretched across the bottom head that buzz
//! when the drum is struck - we simulate this with filtered noise.
//!
//! # How It Works
//!
//! 1. Triangle wave provides the tonal "body" (the drum head sound)
//! 2. Noise provides the "snare" rattle character
//! 3. Both go through amplitude envelopes
//! 4. Band-pass filter shapes the noise to sound more like wire buzz
//!
//! # Variations
//!
//! - More noise = trashy, lo-fi snare
//! - Less noise = more "tom" like
//! - Higher filter = brighter, snappier
//! - Longer decay = looser snare sound

use crate::graph::{envelope::EnvNode, extensions::NodeExt, filter::FilterNode, oscillator::OscNode};

/// Create a snare drum voice.
///
/// Returns a node graph configured for snappy electronic snare sounds.
/// Combines tonal body with noise for the characteristic snare rattle.
pub fn snare() -> impl crate::graph::GraphNode {
    // Noise for the snare rattle, band-pass filtered
    let rattle = OscNode::noise()
        .amplify(EnvNode::adsr(0.001, 0.12, 0.0, 0.08))
        .through(FilterNode::bandpass(3000.0));

    // Triangle for the tonal body
    let body = OscNode::triangle()
        .amplify(EnvNode::adsr(0.001, 0.08, 0.0, 0.05))
        .through(FilterNode::lowpass(400.0));

    // Mix body and rattle (more rattle than body for snare character)
    body.mix(rattle, 0.7)
}
