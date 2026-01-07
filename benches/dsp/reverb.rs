//! Benchmarks for reverb processing.

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion};
use saavy_dsp::dsp::reverb::SchroederReverb;

use crate::BLOCK_SIZES;

pub fn bench_reverb(c: &mut Criterion) {
    let mut group = c.benchmark_group("dsp/reverb");

    let sample_rate = 48_000.0;

    for &size in BLOCK_SIZES {
        // Generate a test signal (impulse-like with some content)
        let input: Vec<f32> = (0..size)
            .map(|i| {
                if i < 10 {
                    1.0 - (i as f32 / 10.0) // Initial impulse
                } else {
                    (i as f32 * 0.05).sin() * 0.1 // Quiet tail
                }
            })
            .collect();

        // Small room (short reverb)
        let mut reverb = SchroederReverb::new(sample_rate);
        reverb.set_room_size(0.3);
        reverb.set_damping(0.5);
        group.bench_with_input(BenchmarkId::new("small_room", size), &size, |b, _| {
            b.iter(|| {
                let mut sum = 0.0f32;
                for &sample in &input {
                    sum += reverb.process(black_box(sample));
                }
                sum
            })
        });

        // Large room (long reverb)
        let mut reverb = SchroederReverb::new(sample_rate);
        reverb.set_room_size(0.9);
        reverb.set_damping(0.3);
        group.bench_with_input(BenchmarkId::new("large_room", size), &size, |b, _| {
            b.iter(|| {
                let mut sum = 0.0f32;
                for &sample in &input {
                    sum += reverb.process(black_box(sample));
                }
                sum
            })
        });

        // High damping (dark reverb)
        let mut reverb = SchroederReverb::new(sample_rate);
        reverb.set_room_size(0.5);
        reverb.set_damping(0.9);
        group.bench_with_input(BenchmarkId::new("high_damping", size), &size, |b, _| {
            b.iter(|| {
                let mut sum = 0.0f32;
                for &sample in &input {
                    sum += reverb.process(black_box(sample));
                }
                sum
            })
        });
    }

    group.finish();
}
