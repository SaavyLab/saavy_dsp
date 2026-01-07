//! Benchmarks for state-variable filter.

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion};
use saavy_dsp::dsp::filter::SVFilter;
use saavy_dsp::graph::node::RenderCtx;

use crate::BLOCK_SIZES;

pub fn bench_filter(c: &mut Criterion) {
    let mut group = c.benchmark_group("dsp/filter");
    let ctx = RenderCtx::from_freq(48_000.0, 440.0, 100.0);

    for &size in BLOCK_SIZES {
        // Generate a test signal (sawtooth-like ramp)
        let input: Vec<f32> = (0..size)
            .map(|i| (i as f32 / size as f32) * 2.0 - 1.0)
            .collect();

        // Lowpass filter
        let mut filter = SVFilter::lowpass(1000.0);
        filter.set_resonance(0.5);
        let mut buffer = input.clone();
        group.bench_with_input(BenchmarkId::new("lowpass", size), &size, |b, _| {
            b.iter(|| {
                buffer.copy_from_slice(&input);
                filter.render(black_box(&mut buffer), black_box(&ctx));
            })
        });

        // Highpass filter
        let mut filter = SVFilter::highpass(1000.0);
        filter.set_resonance(0.5);
        let mut buffer = input.clone();
        group.bench_with_input(BenchmarkId::new("highpass", size), &size, |b, _| {
            b.iter(|| {
                buffer.copy_from_slice(&input);
                filter.render(black_box(&mut buffer), black_box(&ctx));
            })
        });

        // Bandpass filter
        let mut filter = SVFilter::bandpass(1000.0);
        filter.set_resonance(0.5);
        let mut buffer = input.clone();
        group.bench_with_input(BenchmarkId::new("bandpass", size), &size, |b, _| {
            b.iter(|| {
                buffer.copy_from_slice(&input);
                filter.render(black_box(&mut buffer), black_box(&ctx));
            })
        });

        // Notch filter
        let mut filter = SVFilter::notch(1000.0);
        filter.set_resonance(0.5);
        let mut buffer = input.clone();
        group.bench_with_input(BenchmarkId::new("notch", size), &size, |b, _| {
            b.iter(|| {
                buffer.copy_from_slice(&input);
                filter.render(black_box(&mut buffer), black_box(&ctx));
            })
        });
    }

    group.finish();
}
