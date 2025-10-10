# saavy dsp

**low-level, realtime-safe dsp primitives for musical sound.** rust-first. allocation-free on the audio thread. dependency-light.

## overview

saavy dsp provides oscillators, envelopes, filters, modulation, and a small voice engine you can compose into instruments or offline renders. it’s not a plugin, not a host, and not opinionated about your ui. just dsp.

## design principles

* **rt-safe**: no locks, no allocs in `process()`. bounded work per block.
* **deterministic**: sample-accurate event scheduling; snapshot tests for audio equivalence.
* **composable**: small traits + plain structs; bring your own graph.
* **portable**: stable rust; runs anywhere you can run rust.
* **austere**: few footguns, fewer dependencies.

## what’s here (mvp scope)

* oscillators: sine, triangle, saw/square (bandlimited), noise
* envelopes: adsr (segment-accurate), simple mseg (later)
* filters: tpt svf (lp/bp/hp), gentle drive
* modulation: lfo + envelope routing into osc/filter params
* voices: fixed-pool polyphony + voice-steal policy
* render paths: offline render; optional realtime demo (feature-gated)

## quickstart

```rust
use saavy_dsp::{Engine, Patch, RenderConfig};

fn main() {
    let sr = 48_000.0;
    let mut eng = Engine::new(RenderConfig::new(sr, 128));
    let patch = Patch::basic_synth(); // 2 osc -> svf -> amp

    let voice = eng.new_voice(&patch);
    eng.note_on(voice, 60, 0.8);   // middle c
    let mut left = vec![0.0f32; sr as usize]; // 1 second mono
    eng.render(&mut left);

    // write wav, compare hash, do science
}
```

## features (planned, not promised)

* wavetables + oversampling/anti-aliasing paths
* waveshaping + simple delay/chorus
* automation lanes + tempo sync
* golden-audio regression harness

## examples

* `examples/offline_bounce.rs` — render a patch to a wav (no io deps)
* `examples/cpal_demo.rs` — optional sound-out (`--features cpal-demo`)

## cargo features

* `cpal-demo` — builds the realtime demo via `cpal`
* `serde` — enable (de)serialization for patches
* `analysis` — extras used by tests/benches (e.g., `rustfft`); not in core

## testing

* unit + property tests for primitives
* audio snapshot tests (goldens) to guard regressions
* benchmarks via `criterion` (gated, non-rt thread)

## status

early. the core is being carved with testable slices first; api surface may shift.

## license

mit/apl2. pick your poison.
