//! Benchmarks for oscillator waveform generation.

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion};
use saavy_dsp::dsp::oscillator::OscillatorBlock;
use saavy_dsp::graph::node::RenderCtx;

use crate::BLOCK_SIZES;

pub fn bench_oscillator(c: &mut Criterion) {
    let mut group = c.benchmark_group("dsp/oscillator");
    let ctx = RenderCtx::from_freq(48_000.0, 440.0, 100.0);

    for &size in BLOCK_SIZES {
        let mut buffer = vec![0.0f32; size];

        // Sine - uses sin() transcendental function
        let mut osc = OscillatorBlock::sine();
        group.bench_with_input(BenchmarkId::new("sine", size), &size, |b, _| {
            b.iter(|| {
                osc.render(black_box(&mut buffer), black_box(&ctx));
            })
        });

        // Sawtooth - simple linear ramp
        let mut osc = OscillatorBlock::sawtooth();
        group.bench_with_input(BenchmarkId::new("sawtooth", size), &size, |b, _| {
            b.iter(|| {
                osc.render(black_box(&mut buffer), black_box(&ctx));
            })
        });

        // Square - branch per sample
        let mut osc = OscillatorBlock::square();
        group.bench_with_input(BenchmarkId::new("square", size), &size, |b, _| {
            b.iter(|| {
                osc.render(black_box(&mut buffer), black_box(&ctx));
            })
        });

        // Triangle - absolute value
        let mut osc = OscillatorBlock::triangle();
        group.bench_with_input(BenchmarkId::new("triangle", size), &size, |b, _| {
            b.iter(|| {
                osc.render(black_box(&mut buffer), black_box(&ctx));
            })
        });

        // Noise - xorshift PRNG
        let mut osc = OscillatorBlock::noise();
        group.bench_with_input(BenchmarkId::new("noise", size), &size, |b, _| {
            b.iter(|| {
                osc.render(black_box(&mut buffer), black_box(&ctx));
            })
        });
    }

    group.finish();
}
