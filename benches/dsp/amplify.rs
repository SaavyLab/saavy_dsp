//! Benchmarks for signal amplification primitives.

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion};
use saavy_dsp::dsp::amplify;

use crate::BLOCK_SIZES;

pub fn bench_amplify(c: &mut Criterion) {
    let mut group = c.benchmark_group("dsp/amplify");

    for &size in BLOCK_SIZES {
        // Pre-allocate buffers
        let signal: Vec<f32> = (0..size)
            .map(|i| (i as f32 / size as f32) * 2.0 - 1.0)
            .collect();
        let modulator: Vec<f32> = (0..size).map(|i| i as f32 / size as f32).collect();
        let mut output = vec![0.0f32; size];

        group.bench_with_input(BenchmarkId::new("multiply", size), &size, |b, _| {
            b.iter(|| {
                amplify::multiply(
                    black_box(&signal),
                    black_box(&modulator),
                    black_box(&mut output),
                )
            })
        });

        let mut signal_copy = signal.clone();
        group.bench_with_input(
            BenchmarkId::new("multiply_in_place", size),
            &size,
            |b, _| {
                b.iter(|| {
                    signal_copy.copy_from_slice(&signal);
                    amplify::multiply_in_place(black_box(&mut signal_copy), black_box(&modulator))
                })
            },
        );

        let mut signal_copy = signal.clone();
        group.bench_with_input(BenchmarkId::new("apply_gain", size), &size, |b, _| {
            b.iter(|| {
                signal_copy.copy_from_slice(&signal);
                amplify::apply_gain(black_box(&mut signal_copy), black_box(0.5))
            })
        });
    }

    group.finish();
}
