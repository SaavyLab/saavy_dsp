//! Benchmarks for DSP primitives and real-world scenarios.
//!
//! Run with: cargo bench
//!
//! These benchmarks measure the performance of core DSP operations to ensure
//! they complete well within real-time audio deadlines.
//!
//! Reference timing at 48kHz sample rate:
//!   - 64 samples  = 1.33ms deadline
//!   - 128 samples = 2.67ms deadline
//!   - 256 samples = 5.33ms deadline
//!   - 512 samples = 10.67ms deadline
//!
//! Benchmark groups:
//!   - dsp/*        Low-level primitives (oscillator, filter, envelope, etc.)
//!   - scenarios/*  Real-world voice chains and multi-track mixes

use criterion::{criterion_group, criterion_main};

mod dsp;
mod scenarios;

/// Common buffer sizes used in audio applications.
pub const BLOCK_SIZES: &[usize] = &[64, 128, 256, 512];

criterion_group!(
    benches,
    // Low-level DSP primitives
    dsp::bench_amplify,
    dsp::bench_oscillator,
    dsp::bench_filter,
    dsp::bench_envelope,
    dsp::bench_distortion,
    dsp::bench_delay,
    dsp::bench_mix,
    dsp::bench_reverb,
    // Real-world scenarios
    scenarios::bench_voices,
    scenarios::bench_mix,
);
criterion_main!(benches);
