//! Real-world scenario benchmarks.
//!
//! These benchmarks model actual usage patterns from the examples,
//! testing complete voice chains and multi-track rendering.

mod mix;
mod voices;

pub use mix::bench_mix;
pub use voices::bench_voices;
