//! Benchmarks for multi-track mixing scenarios.
//!
//! These simulate rendering multiple voices simultaneously,
//! like a full arrangement with drums, bass, and synths.

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion};
use saavy_dsp::graph::node::{GraphNode, RenderCtx};
use saavy_dsp::voices;

use crate::BLOCK_SIZES;

/// Render multiple voices and sum them (simulating a mixer)
fn render_and_mix(
    voices: &mut [Box<dyn GraphNode>],
    buffer: &mut [f32],
    scratch: &mut [f32],
    ctx: &RenderCtx,
) {
    buffer.fill(0.0);

    for voice in voices.iter_mut() {
        scratch.fill(0.0);
        voice.render_block(scratch, ctx);

        // Sum into output buffer
        for (out, &sample) in buffer.iter_mut().zip(scratch.iter()) {
            *out += sample;
        }
    }

    // Normalize to prevent clipping (simple division by voice count)
    let scale = 1.0 / voices.len() as f32;
    for sample in buffer.iter_mut() {
        *sample *= scale;
    }
}

pub fn bench_mix(c: &mut Criterion) {
    let mut group = c.benchmark_group("scenarios/mix");

    for &size in BLOCK_SIZES {
        let mut buffer = vec![0.0f32; size];
        let mut scratch = vec![0.0f32; size];
        let ctx = RenderCtx::from_freq(48_000.0, 110.0, 100.0);

        // === MINIMAL: 2 tracks (kick + bass) ===
        let mut minimal: Vec<Box<dyn GraphNode>> = vec![
            Box::new(voices::kick()),
            Box::new(voices::bass()),
        ];
        for v in minimal.iter_mut() {
            v.note_on(&ctx);
        }

        group.bench_with_input(BenchmarkId::new("2_track", size), &size, |b, _| {
            b.iter(|| {
                render_and_mix(
                    black_box(&mut minimal),
                    black_box(&mut buffer),
                    black_box(&mut scratch),
                    black_box(&ctx),
                );
            })
        });

        // === DRUMS: 4 tracks (kick, snare, hihat, clap) ===
        let mut drums: Vec<Box<dyn GraphNode>> = vec![
            Box::new(voices::kick()),
            Box::new(voices::snare()),
            Box::new(voices::hihat()),
            Box::new(voices::clap()),
        ];
        for v in drums.iter_mut() {
            v.note_on(&ctx);
        }

        group.bench_with_input(BenchmarkId::new("4_track_drums", size), &size, |b, _| {
            b.iter(|| {
                render_and_mix(
                    black_box(&mut drums),
                    black_box(&mut buffer),
                    black_box(&mut scratch),
                    black_box(&ctx),
                );
            })
        });

        // === FULL ARRANGEMENT: 6 tracks (like techno example) ===
        // kick, snare, hihat, bass, lead, pad
        let mut full: Vec<Box<dyn GraphNode>> = vec![
            Box::new(voices::kick()),
            Box::new(voices::snare()),
            Box::new(voices::hihat()),
            Box::new(voices::bass()),
            Box::new(voices::lead()),
            Box::new(voices::pad()),
        ];
        for v in full.iter_mut() {
            v.note_on(&ctx);
        }

        group.bench_with_input(BenchmarkId::new("6_track_full", size), &size, |b, _| {
            b.iter(|| {
                render_and_mix(
                    black_box(&mut full),
                    black_box(&mut buffer),
                    black_box(&mut scratch),
                    black_box(&ctx),
                );
            })
        });

        // === DENSE: 8 tracks (stress test) ===
        let mut dense: Vec<Box<dyn GraphNode>> = vec![
            Box::new(voices::kick()),
            Box::new(voices::snare()),
            Box::new(voices::hihat()),
            Box::new(voices::openhat()),
            Box::new(voices::bass()),
            Box::new(voices::lead()),
            Box::new(voices::pad()),
            Box::new(voices::pluck()),
        ];
        for v in dense.iter_mut() {
            v.note_on(&ctx);
        }

        group.bench_with_input(BenchmarkId::new("8_track_dense", size), &size, |b, _| {
            b.iter(|| {
                render_and_mix(
                    black_box(&mut dense),
                    black_box(&mut buffer),
                    black_box(&mut scratch),
                    black_box(&ctx),
                );
            })
        });
    }

    group.finish();
}
