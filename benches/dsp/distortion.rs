//! Benchmarks for waveshaping distortion.

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion};
use saavy_dsp::dsp::distortion;

use crate::BLOCK_SIZES;

pub fn bench_distortion(c: &mut Criterion) {
    let mut group = c.benchmark_group("dsp/distortion");

    for &size in BLOCK_SIZES {
        // Generate a test signal (sine-like values)
        let input: Vec<f32> = (0..size)
            .map(|i| (i as f32 * 0.1).sin())
            .collect();

        // Soft clip - smooth saturation
        let mut buffer = input.clone();
        group.bench_with_input(BenchmarkId::new("soft_clip", size), &size, |b, _| {
            b.iter(|| {
                buffer.copy_from_slice(&input);
                distortion::soft_clip_buffer(black_box(&mut buffer), black_box(4.0));
            })
        });

        // Hard clip - abrupt limiting
        let mut buffer = input.clone();
        group.bench_with_input(BenchmarkId::new("hard_clip", size), &size, |b, _| {
            b.iter(|| {
                buffer.copy_from_slice(&input);
                distortion::hard_clip_buffer(black_box(&mut buffer), black_box(2.0), black_box(0.8));
            })
        });

        // Foldback - iterative folding (potentially more expensive)
        let mut buffer = input.clone();
        group.bench_with_input(BenchmarkId::new("foldback", size), &size, |b, _| {
            b.iter(|| {
                buffer.copy_from_slice(&input);
                distortion::foldback_buffer(black_box(&mut buffer), black_box(3.0), black_box(0.5));
            })
        });
    }

    group.finish();
}
