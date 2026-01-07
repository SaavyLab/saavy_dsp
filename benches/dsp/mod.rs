//! Benchmarks for low-level DSP primitives.

mod amplify;
mod delay;
mod distortion;
mod envelope;
mod filter;
mod mix;
mod oscillator;
mod reverb;

pub use amplify::bench_amplify;
pub use delay::bench_delay;
pub use distortion::bench_distortion;
pub use envelope::bench_envelope;
pub use filter::bench_filter;
pub use mix::bench_mix;
pub use oscillator::bench_oscillator;
pub use reverb::bench_reverb;
