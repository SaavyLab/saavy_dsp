//! Benchmarks for ADSR envelope generator.

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion};
use saavy_dsp::dsp::envelope::Envelope;
use saavy_dsp::graph::node::RenderCtx;

use crate::BLOCK_SIZES;

pub fn bench_envelope(c: &mut Criterion) {
    let mut group = c.benchmark_group("dsp/envelope");
    let ctx = RenderCtx::from_freq(48_000.0, 440.0, 100.0);

    for &size in BLOCK_SIZES {
        let mut buffer = vec![0.0f32; size];

        // Attack phase (ramping up)
        let mut env = Envelope::adsr(0.1, 0.1, 0.7, 0.3);
        env.note_on(&ctx);
        group.bench_with_input(BenchmarkId::new("attack", size), &size, |b, _| {
            b.iter(|| {
                env.render(black_box(&mut buffer), black_box(&ctx));
            })
        });

        // Sustain phase (holding steady)
        let mut env = Envelope::adsr(0.001, 0.001, 0.7, 0.3);
        env.note_on(&ctx);
        // Advance past attack/decay
        for _ in 0..200 {
            env.next_sample(&ctx);
        }
        group.bench_with_input(BenchmarkId::new("sustain", size), &size, |b, _| {
            b.iter(|| {
                env.render(black_box(&mut buffer), black_box(&ctx));
            })
        });

        // Release phase (ramping down)
        let mut env = Envelope::adsr(0.001, 0.001, 0.7, 0.1);
        env.note_on(&ctx);
        for _ in 0..200 {
            env.next_sample(&ctx);
        }
        env.note_off(&ctx);
        group.bench_with_input(BenchmarkId::new("release", size), &size, |b, _| {
            b.iter(|| {
                env.render(black_box(&mut buffer), black_box(&ctx));
            })
        });
    }

    group.finish();
}
