//! Composable building blocks for constructing audio-processing graphs.
//!
//! Graph nodes wrap the low-level DSP primitives with ergonomics needed for
//! instrument design: note events, modulation, and block-based rendering. The
//! `extensions` module adds fluent helpers so patches can be authored with a
//! clear, chainable API.

/// Multiply two signals together (amplitude or ring modulation).
pub mod amplify;
/// Feedback delay effect with realtime-safe modulation.
pub mod delay;
/// Envelope generator node exposing ADSR state.
pub mod envelope;
/// Fluent combinators (`.amplify()`, `.mix()`, etc.).
pub mod extensions;
/// Topology-preserving filter node with multiple responses.
pub mod filter;
/// Low frequency oscillators for parameter modulation.
pub mod lfo;
/// Linear wet/dry mixing for parallel graphs.
pub mod mix;
/// Connect modulation sources to node parameters.
pub mod modulate;
/// Core traits shared by all graph nodes.
pub mod node;
/// Audio-band oscillators and noise sources.
pub mod oscillator;
/// Serial chaining of two nodes (source → effect).
pub mod through;
