//! Benchmarks for complete voice chains.
//!
//! These test realistic signal paths as used in the examples,
//! from simple lead voices to complex modulated patches.

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion};
use saavy_dsp::graph::{
    envelope::EnvNode,
    extensions::NodeExt,
    filter::{FilterNode, FilterParam},
    lfo::LfoNode,
    node::{GraphNode, RenderCtx},
    oscillator::{OscNode, OscParam},
};
use saavy_dsp::voices;

use crate::BLOCK_SIZES;

pub fn bench_voices(c: &mut Criterion) {
    let mut group = c.benchmark_group("scenarios/voices");
    let ctx = RenderCtx::from_freq(48_000.0, 110.0, 100.0); // A2, typical bass note

    for &size in BLOCK_SIZES {
        let mut buffer = vec![0.0f32; size];

        // === SIMPLE VOICE ===
        // lead-style: sawtooth → envelope → filter
        // This is a baseline for what a typical voice costs
        let mut lead = OscNode::sawtooth()
            .amplify(EnvNode::adsr(0.01, 0.1, 0.6, 0.2))
            .through(FilterNode::lowpass(2500.0));
        lead.note_on(&ctx);

        group.bench_with_input(BenchmarkId::new("lead", size), &size, |b, _| {
            b.iter(|| {
                lead.render_block(black_box(&mut buffer), black_box(&ctx));
            })
        });

        // === SUBTRACTIVE BASS ===
        // bass-style: square → envelope → filter (lower cutoff)
        let mut bass = OscNode::square()
            .amplify(EnvNode::adsr(0.01, 0.1, 0.7, 0.15))
            .through(FilterNode::lowpass(500.0));
        bass.note_on(&ctx);

        group.bench_with_input(BenchmarkId::new("bass", size), &size, |b, _| {
            b.iter(|| {
                bass.render_block(black_box(&mut buffer), black_box(&ctx));
            })
        });

        // === MODULATED VOICE ===
        // kick-style: oscillator with pitch envelope modulation
        // Tests the modulation overhead
        let pitch_env = EnvNode::adsr(0.001, 0.08, 0.0, 0.0);
        let mut kick = OscNode::sine()
            .with_frequency(50.0)
            .modulate(pitch_env, OscParam::Frequency, 100.0)
            .amplify(EnvNode::adsr(0.001, 0.15, 0.0, 0.05))
            .through(FilterNode::lowpass(200.0));
        kick.note_on(&ctx);

        group.bench_with_input(BenchmarkId::new("kick_modulated", size), &size, |b, _| {
            b.iter(|| {
                kick.render_block(black_box(&mut buffer), black_box(&ctx));
            })
        });

        // === ACID BASS ===
        // techno-style: sawtooth → filter (with LFO modulation) → envelope
        // This is a more complex chain with continuous LFO modulation
        let filter_lfo = LfoNode::triangle(2.0); // Faster LFO for benchmark visibility
        let mut acid = OscNode::sawtooth()
            .through(
                FilterNode::lowpass(400.0)
                    .with_resonance(0.8)
                    .modulate(filter_lfo, FilterParam::Cutoff, 800.0),
            )
            .amplify(EnvNode::adsr(0.001, 0.1, 0.6, 0.1));
        acid.note_on(&ctx);

        group.bench_with_input(BenchmarkId::new("acid_bass", size), &size, |b, _| {
            b.iter(|| {
                acid.render_block(black_box(&mut buffer), black_box(&ctx));
            })
        });

        // === LAYERED PAD ===
        // Two oscillators mixed together → envelope → filter
        let mut pad = OscNode::sawtooth()
            .mix(OscNode::square(), 0.5)
            .amplify(EnvNode::adsr(0.1, 0.2, 0.8, 0.5))
            .through(FilterNode::lowpass(1500.0));
        pad.note_on(&ctx);

        group.bench_with_input(BenchmarkId::new("pad_layered", size), &size, |b, _| {
            b.iter(|| {
                pad.render_block(black_box(&mut buffer), black_box(&ctx));
            })
        });

        // === PRE-BUILT VOICES ===
        // Test the actual voices from the voices module
        let mut kick_voice = voices::kick();
        kick_voice.note_on(&ctx);
        group.bench_with_input(BenchmarkId::new("voices::kick", size), &size, |b, _| {
            b.iter(|| {
                kick_voice.render_block(black_box(&mut buffer), black_box(&ctx));
            })
        });

        let mut lead_voice = voices::lead();
        lead_voice.note_on(&ctx);
        group.bench_with_input(BenchmarkId::new("voices::lead", size), &size, |b, _| {
            b.iter(|| {
                lead_voice.render_block(black_box(&mut buffer), black_box(&ctx));
            })
        });

        let mut hihat_voice = voices::hihat();
        hihat_voice.note_on(&ctx);
        group.bench_with_input(BenchmarkId::new("voices::hihat", size), &size, |b, _| {
            b.iter(|| {
                hihat_voice.render_block(black_box(&mut buffer), black_box(&ctx));
            })
        });
    }

    group.finish();
}
