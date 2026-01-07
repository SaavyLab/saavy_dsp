//! Benchmarks for delay line operations.

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion};
use saavy_dsp::dsp::delay::DelayLine;

use crate::BLOCK_SIZES;

pub fn bench_delay(c: &mut Criterion) {
    let mut group = c.benchmark_group("dsp/delay");

    // Test with different delay times (in samples)
    let delay_times: &[usize] = &[
        480,    // 10ms at 48kHz
        4800,   // 100ms at 48kHz
        48000,  // 1 second at 48kHz
    ];

    for &size in BLOCK_SIZES {
        // Generate a test signal
        let input: Vec<f32> = (0..size)
            .map(|i| (i as f32 * 0.1).sin())
            .collect();

        for &delay_samples in delay_times {
            let delay_ms = delay_samples as f32 / 48.0;

            // Basic delay render (integer delay)
            let mut delay = DelayLine::new();
            let mut buffer = input.clone();
            group.bench_with_input(
                BenchmarkId::new(format!("render_{}ms", delay_ms as u32), size),
                &size,
                |b, _| {
                    b.iter(|| {
                        buffer.copy_from_slice(&input);
                        delay.render(black_box(&mut buffer), black_box(delay_samples));
                    })
                },
            );
        }

        // Interpolated read (fractional delay - used in chorus/flanger)
        let mut delay = DelayLine::new();
        // Pre-fill delay line
        for &sample in &input {
            delay.write(sample);
        }
        group.bench_with_input(
            BenchmarkId::new("read_interpolated", size),
            &size,
            |b, _| {
                b.iter(|| {
                    let mut sum = 0.0f32;
                    for i in 0..size {
                        // Simulate modulated delay time (chorus-like)
                        let delay_time = 480.0 + (i as f32 * 0.1).sin() * 48.0;
                        sum += delay.read_interpolated(black_box(delay_time));
                    }
                    sum
                })
            },
        );
    }

    group.finish();
}
