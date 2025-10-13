# saavy_dsp

**low-level, realtime-safe dsp primitives for musical sound.** rust-first. allocation-free on the audio thread. dependency-light.

## overview

saavy_dsp provides oscillators, envelopes, and a graph-based architecture for building synthesizers. It's not a plugin, not a host, and not opinionated about your ui. just dsp.

## design principles

* **rt-safe**: no locks, no allocs in audio callback. lock-free message passing via `rtrb`.
* **deterministic**: sample-accurate rendering; regression tests for audio correctness.
* **composable**: graph nodes + fluent API; build complex instruments from simple blocks.
* **portable**: stable rust; runs anywhere you can run rust.
* **austere**: few footguns, minimal dependencies.

## what's implemented (current state)

* **oscillators**: sine, saw, square, noise
* **envelopes**: ADSR with lock-free control
* **graph architecture**: composable audio processing nodes
* **polyphony**: voice allocation, stealing, and mixing
* **real-time audio**: cpal-based interactive demo

## quickstart

### Basic Graph Composition

```rust
use saavy_dsp::graph::{
    envelope::EnvNode,
    extensions::NodeExt,
    node::GraphNode,
    oscillator::OscNode,
};

fn main() {
    let sample_rate = 48_000.0;

    // Create graph nodes
    let mut env = EnvNode::with_params(sample_rate, 0.01, 0.1, 0.7, 0.3);
    env.note_on();
    let mut synth = OscNode::sine(440.0, sample_rate).amplify(env);

    // Render audio
    let mut buffer = vec![0.0; 128];
    synth.render_block(&mut buffer);
}
```

### Polyphonic Synthesis

```rust
use rtrb::RingBuffer;
use saavy_dsp::synth::{message::SynthMessage, poly::PolySynth};

fn main() {
    let sample_rate = 48_000.0;
    let (mut tx, rx) = RingBuffer::<SynthMessage>::new(64);

    let mut synth = PolySynth::new(
        sample_rate,
        8,    // max voices
        rx,
        0.01, // attack
        0.1,  // decay
        0.7,  // sustain
        0.3,  // release
    );

    // Send MIDI events
    let _ = tx.push(SynthMessage::NoteOn { note: 60, velocity: 100 });

    // Render
    let mut buffer = vec![0.0; 256];
    synth.render_block(&mut buffer);
}
```

## examples

* `examples/fluent_demo.rs` — basic graph composition
* `examples/graph_basics.rs` — multiple graph patterns
* `examples/envelope_demo.rs` — ADSR phase visualization
* `examples/polyphony_demo.rs` — voice management and stealing
* `examples/cpal_demo.rs` — real-time interactive audio (`--features cpal-demo`)

Run with:
```bash
cargo run --example fluent_demo
cargo run --features cpal-demo --example cpal_demo
```

## cargo features

* `default = []` — minimal build, no optional features
* `cpal-demo` — builds the realtime demo via `cpal` and `crossterm`
* `serde` — enable (de)serialization for future preset system
* `analysis` — `rustfft` for future analysis tools

## architecture

```
src/
├── dsp/          # Low-level primitives (Envelope, OscillatorBlock)
├── graph/        # Composable graph nodes (OscNode, EnvNode, Amplify)
├── synth/        # Voice management (Voice, PolySynth, message types)
└── io/           # MIDI conversion, audio I/O types
```

### Key Concepts

**Graph Nodes**: Building blocks that implement `GraphNode` trait
- `OscNode` - Oscillator (sine, saw, square, noise)
- `EnvNode` / `SharedEnvNode` - ADSR envelope
- `Amplify` - Multiplies two signals (ring modulation)

**Polyphony**: Fixed voice pool with automatic allocation
- Voice stealing (oldest releasing voice)
- Lock-free MIDI message queue
- Efficient mixing of active voices

## testing

```bash
cargo test                    # Run all tests
cargo test --lib              # Unit tests only
cargo test --test '*'         # Integration tests only
```

Current test coverage:
* Unit tests for oscillator phase wrapping and sine accuracy
* Integration tests for PolySynth (rendering, voice management, note off)

## status

**Current milestone**: ADSR envelopes + polyphony ✅

**Next up**:
- Filters (TPT SVF: lowpass, highpass, bandpass)
- LFO modulation
- MIDI keyboard input via `midir`

See [WHY.md](WHY.md) for project vision and learning goals.

## northstar: fluent API (future)

This is the aspirational syntax we're building towards:

```rust
// Not yet implemented, but coming soon
let synth = OscNode::sine(440.0, sr)
    .amplify(envelope)
    .through(filter)   // <- Not implemented yet
    .mix(0.5);         // <- Not implemented yet
```

Currently working: `.amplify()` combinator

## license

[MIT](LICENSE)
