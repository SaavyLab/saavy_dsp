//! Low-level DSP primitives used by the higher level graph nodes.
//!
//! These components are allocation-free and realtime-safe, making them safe to
//! embed directly inside voice structs. They intentionally stay focused on the
//! signal-processing math so graph combinators can layer on orchestration and
//! modulation.

/// Time-domain delay line with optional interpolation.
pub mod delay;
/// Attack/decay/sustain/release envelope generator.
pub mod envelope;
/// State-variable filter implementation with multiple responses.
pub mod filter;
/// Oscillator waveforms and noise sources.
pub mod oscillator;

pub use envelope::EnvelopeState;
