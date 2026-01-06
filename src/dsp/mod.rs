//! Low-level DSP primitives used by the higher level graph nodes.
//!
//! Each module here contains:
//! - Educational documentation explaining HOW the algorithm works
//! - The core math and implementation
//! - Stateless functions or minimal-state structs
//!
//! For high-level explanations of WHAT each concept is and how to use it,
//! see the corresponding module in `graph/`.

/// Signal multiplication for amplitude control and ring modulation.
pub mod amplify;
/// Time-domain delay line with optional interpolation.
pub mod delay;
/// Waveshaping distortion (soft clip, hard clip, foldback).
pub mod distortion;
/// Attack/decay/sustain/release envelope generator.
pub mod envelope;
/// State-variable filter implementation with multiple responses.
pub mod filter;
/// Low frequency oscillator concepts (control-rate vs audio-rate).
pub mod lfo;
/// Signal mixing and crossfading.
pub mod mix;
/// Parameter modulation (LFO → filter cutoff, etc).
pub mod modulate;
/// Oscillator waveforms and noise sources.
pub mod oscillator;
/// Reverb via comb and allpass filter networks.
pub mod reverb;
/// Serial signal chain concepts.
pub mod through;

pub use envelope::EnvelopeState;
pub use oscillator::Waveform;
