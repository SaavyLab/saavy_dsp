//! Benchmarks for signal mixing operations.

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion};
use saavy_dsp::dsp::mix;

use crate::BLOCK_SIZES;

pub fn bench_mix(c: &mut Criterion) {
    let mut group = c.benchmark_group("dsp/mix");

    for &size in BLOCK_SIZES {
        // Generate test signals
        let signal_a: Vec<f32> = (0..size)
            .map(|i| (i as f32 * 0.1).sin())
            .collect();
        let signal_b: Vec<f32> = (0..size)
            .map(|i| (i as f32 * 0.15).cos())
            .collect();
        let mut output = vec![0.0f32; size];

        // Linear crossfade mix (separate output buffer)
        group.bench_with_input(BenchmarkId::new("crossfade", size), &size, |b, _| {
            b.iter(|| {
                mix::mix(
                    black_box(&signal_a),
                    black_box(&signal_b),
                    black_box(0.5),
                    black_box(&mut output),
                );
            })
        });

        // In-place mix
        let mut buffer_a = signal_a.clone();
        group.bench_with_input(BenchmarkId::new("crossfade_in_place", size), &size, |b, _| {
            b.iter(|| {
                buffer_a.copy_from_slice(&signal_a);
                mix::mix_in_place(black_box(&mut buffer_a), black_box(&signal_b), black_box(0.5));
            })
        });

        // Sum (no weighting)
        group.bench_with_input(BenchmarkId::new("sum", size), &size, |b, _| {
            b.iter(|| {
                mix::sum(
                    black_box(&signal_a),
                    black_box(&signal_b),
                    black_box(&mut output),
                );
            })
        });

        // Sum in-place
        let mut buffer_a = signal_a.clone();
        group.bench_with_input(BenchmarkId::new("sum_in_place", size), &size, |b, _| {
            b.iter(|| {
                buffer_a.copy_from_slice(&signal_a);
                mix::sum_in_place(black_box(&mut buffer_a), black_box(&signal_b));
            })
        });

        // Dry/wet mixing (common for effects)
        let dry = signal_a.clone();
        let mut wet = signal_b.clone();
        group.bench_with_input(BenchmarkId::new("dry_wet", size), &size, |b, _| {
            b.iter(|| {
                wet.copy_from_slice(&signal_b);
                mix::apply_dry_wet(black_box(&dry), black_box(&mut wet), black_box(0.3));
            })
        });
    }

    group.finish();
}
