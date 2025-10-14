# saavy_dsp

**a friendly rust library for making musical sounds from scratch.**

## what is this?

saavy_dsp gives you oscillators, envelopes, and filters to build synthesizers. It's not trying to be the most powerful DSP library—it's trying to be the most *approachable* one. Start with presets, tweak until it sounds good, and don't worry about the math unless you want to.

Good for:
- **Learning synthesis** without drowning in academic papers
- **Prototyping sounds** quickly in Rust
- **Building instruments** for games, apps, or VST plugins
- **Having fun** making weird noises

Not trying to replace: Max/MSP, SuperCollider, or production-grade synth engines. This is the *friendly* option.

## design principles

* **approachable**: presets and examples get you making sound in minutes
* **composable**: chain simple blocks into complex instruments
* **real-time safe**: no locks, no allocs on the audio thread
* **portable**: runs anywhere Rust runs (including embedded, eventually)

## what's implemented (current state)

* **oscillators**: sine, saw, square, noise
* **envelopes**: ADSR with lock-free control
* **graph architecture**: composable audio processing nodes
* **polyphony**: voice allocation, stealing, and mixing
* **sequencing**: musical timing with proper time signatures, no floats
* **real-time audio**: cpal-based interactive demo

## quickstart

### Polyphonic Synthesis (MIDI/Keyboard)

```rust
use rtrb::RingBuffer;
use saavy_dsp::{
    graph::{envelope::EnvNode, extensions::NodeExt, oscillator::OscNode},
    synth::{message::SynthMessage, poly::PolySynth},
};

fn main() {
    let sample_rate = 48_000.0;
    let (mut tx, rx) = RingBuffer::<SynthMessage>::new(64);

    // Design your sound (the "patch")
    let factory = || {
        let osc = OscNode::sine();
        let env = EnvNode::adsr(0.01, 0.1, 0.7, 0.3);
        osc.amplify(env)
    };

    // Create polyphonic synth
    let mut synth = PolySynth::new(sample_rate, 8, factory, rx);

    // Play notes
    let _ = tx.push(SynthMessage::NoteOn { note: 60, velocity: 100 });
    let _ = tx.push(SynthMessage::NoteOn { note: 64, velocity: 100 });

    // Render
    let mut buffer = vec![0.0; 256];
    synth.render_block(&mut buffer);
}
```

### Musical Sequencing

```rust
use saavy_dsp::sequencing::{Duration, Sequence};

fn main() {
    // Create a rhythm: "1, and-of-2, 4"
    let seq = Sequence::new(480) // 480 ppq (standard MIDI timing)
        .note(Duration::EIGHTH).with_note(36)       // Kick on 1
        .rest(Duration::EIGHTH)                      // and
        .rest(Duration::EIGHTH)                      // 2
        .note(Duration::EIGHTH).with_note(38)       // Snare on and-of-2
        .rest(Duration::QUARTER)                     // 3
        .note(Duration::QUARTER).with_note(36)      // Kick on 4
        .build()
        .unwrap();

    // All timing is integer ticks—no float drift
    // Supports compound meters (6/8, 9/8), tuplets, dotted notes
    // Bar validation catches rhythm errors at build time
}
```

## examples

* `examples/envelope_demo.rs` — ADSR phase visualization
* `examples/polyphony_demo.rs` — voice management and stealing
* `examples/simple_poly.rs` — basic polyphonic synthesis
* `examples/cpal_demo.rs` — real-time interactive audio (`--features cpal-demo`)

Run with:
```bash
cargo run --example envelope_demo
cargo run --example simple_poly
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
- `EnvNode` - ADSR envelope
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

## what's next?

The roadmap is loose. Current focus:
- **Filters** (lowpass, highpass, bandpass) — shape the tone
- **Presets** — factory sounds you can load and tweak
- **LFOs** — wobble things over time
- **Sequencing/timing** — play patterns, not just single notes

Long-term dreams: wavetables, effects (delay/reverb), better examples, maybe a fun terminal UI.

See [WHY.md](WHY.md) if you're curious about the "why" behind this project.

## license

[MIT](LICENSE)
