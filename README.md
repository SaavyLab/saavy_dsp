# saavy_dsp

an approachable rust dsp toolkit for crafting musical voices from scratch.

## what it is

saavy_dsp is a synthesizer toolkit designed for play and exploration, built on a foundation of classic dsp principles. it keeps the essentials close at hand—oscillators, envelopes, and filters—and gives you a clean, composable api to snap them together. it's your sonic lego set, with a textbook in the box.

it's a good match when you want to:
- **prototype new sounds** in rust with a quick, intuitive workflow
- **learn synthesis** not just by reading, but by building and experimenting
- **build instruments** for games, apps, or vst plugins with confidence
- **trust your tools**, knowing they're a clear, direct implementation of foundational dsp concepts

this is your sketchbook for sound—a place to explore, learn, and create, backed by solid engineering.

## guiding principles

- **approachable & playful**: start making sound in minutes with a fluent, chainable api
- **built on solid foundations**: every component is a clear implementation of classic dsp principles
- **composable**: small, predictable nodes that snap together into bigger, more expressive voices
- **real-time safe**: lock-free, allocation-free audio paths for glitch-free creativity

## status snapshot

- **oscillators**: sine, saw, square, triangle, noise
- **envelopes**: adsr with lock-free parameter control
- **filters**: tpt state variable filter (lowpass, highpass, bandpass, notch)
- **modulation**: lfo with multiple waveforms, modulatable delay
- **graph architecture**: modular processing nodes with trait-based composition
- **graph combinators**: `.amplify()`, `.through()`, `.modulate()`
- **polyphony**: voice allocation, stealing, and mixing baked in
- **sequencing**: integer timing with proper time signatures, tuplets, and dotted values
- **real-time audio**: `cpal`-powered interactive demo with oscilloscope visualization

## quickstart

### polyphonic synthesis (midi/keyboard)

```rust
use rtrb::RingBuffer;
use saavy_dsp::{
    graph::{envelope::EnvNode, extensions::NodeExt, oscillator::OscNode},
    synth::{message::SynthMessage, poly::PolySynth},
};

fn main() {
    let sample_rate = 48_000.0;
    let (mut tx, rx) = RingBuffer::<SynthMessage>::new(64);

    // design your sound (the "patch")
    let factory = || {
        let osc = OscNode::sine();
        let env = EnvNode::adsr(0.01, 0.1, 0.7, 0.3);
        osc.amplify(env)
    };

    // create a polyphonic synth
    let mut synth = PolySynth::new(sample_rate, 8, factory, rx);

    // play notes
    let _ = tx.push(SynthMessage::NoteOn { note: 60, velocity: 100 });
    let _ = tx.push(SynthMessage::NoteOn { note: 64, velocity: 100 });

    // render a block of audio
    let mut buffer = vec![0.0; 256];
    synth.render_block(&mut buffer);
}
```

### musical sequencing

```rust
use saavy_dsp::sequencing::{Duration, Sequence};

fn main() {
    // create a rhythm: "1, and-of-2, 4"
    let seq = Sequence::new(480) // 480 ppq (standard MIDI timing)
        .note(Duration::EIGHTH).with_note(36)       // kick on 1
        .rest(Duration::EIGHTH)                      // and
        .rest(Duration::EIGHTH)                      // 2
        .note(Duration::EIGHTH).with_note(38)       // snare on and-of-2
        .rest(Duration::QUARTER)                     // 3
        .note(Duration::QUARTER).with_note(36)      // kick on 4
        .build()
        .unwrap();

    // timing is integer ticks only—no float drift
    // supports compound meters (6/8, 9/8), tuplets, dotted notes
    // bar validation catches rhythm errors at build time
}
```

## building effects

the composable graph api makes classic effects simple to build. here are a few common patterns:

### chorus

creates an ensemble effect with slow delay modulation:

```rust
let lfo = LfoNode::sine(1.0);                    // slow modulation (1 Hz)
let delay = DelayNode::new(15.0, 0.0, 0.5);      // short delay, no feedback
let chorus = delay.modulate(lfo, DelayParam::DelayTime, 10.0);

// in a voice:
osc.amplify(env).through(chorus)
```

### flanger

faster, shorter modulation with feedback for a "jet plane" sweep:

```rust
let lfo = LfoNode::sine(0.5);                    // medium-speed sweep
let delay = DelayNode::new(5.0, 0.6, 0.7);       // very short, high feedback
let flanger = delay.modulate(lfo, DelayParam::DelayTime, 3.0);

osc.amplify(env).through(flanger)
```

### vibrato

pitch wobble via delay time modulation:

```rust
let lfo = LfoNode::sine(5.0);                    // faster vibrato (5 Hz)
let delay = DelayNode::new(0.0, 0.0, 1.0);       // wet only, no feedback
let vibrato = delay.modulate(lfo, DelayParam::DelayTime, 2.0);

osc.amplify(env).through(vibrato)
```

these are just starting points—tweak the parameters or combine multiple effects to create your own signature sounds.

## examples

- `examples/envelope_demo.rs` — visualize adsr phases
- `examples/polyphony_demo.rs` — inspect voice allocation and stealing
- `examples/simple_poly.rs` — build a basic polyphonic synth voice
- `examples/cpal_scope.rs` — real-time oscilloscope visualization (includes chorus effect)
- `examples/cpal_demo.rs` — run the real-time interactive demo (`--features cpal-demo`)

run with:

```bash
cargo run --example envelope_demo
cargo run --example simple_poly
cargo run --features cpal-demo --example cpal_scope
```

## cargo features

- `default = []` — minimal build, no optional features
- `cpal-demo` — enables the realtime demo via `cpal` and `crossterm`
- `serde` — (de)serialization hooks for the future preset system
- `analysis` — pulls in `rustfft` for analysis tooling

## architecture at a glance

```
src/
├── dsp/          # low-level primitives (envelopes, oscillator blocks)
├── graph/        # composable nodes (OscNode, EnvNode, Amplify)
├── sequencing/   # musical durations, sequences, and timing helpers
├── synth/        # polyphonic synth internals (voice, poly, messaging)
└── io/           # midi conversion and audio io types
```

### key concepts

**graph nodes**: building blocks that implement the `GraphNode` trait
- `OscNode` – oscillators (sine, saw, square, triangle, noise)
- `EnvNode` – adsr envelope
- `FilterNode` – tpt svf with modulatable cutoff and resonance
- `LfoNode` – low-frequency oscillator for modulation
- `DelayNode` – modulatable delay line
- `Amplify` – multiplies two signals for mixing or modulation
- `Through` – serial signal processing
- `Modulate` – applies modulation to node parameters

**polyphony**: fixed voice pool with automatic allocation
- voice stealing (oldest releasing voice)
- lock-free midi message queue
- efficient mixing of active voices

## testing

```bash
cargo test                    # run all tests
cargo test --lib              # unit tests only
cargo test --test '*'         # integration tests only
```

coverage today includes oscillator phase wrapping, sine accuracy, and polysynth render/voice management integration tests.

## roadmap

next up:
- **voice profiles**: ready-made patches (kick, snare, bass, lead) you can tweak and share
- **midi integration**: keyboard input via `midir` for live performance
- **track/arrangement system**: pattern-based sequencing beyond single notes

longer-term dreams include wavetables, reverb, richer examples, and maybe a playful terminal ui.

## license

[MIT](LICENSE)
