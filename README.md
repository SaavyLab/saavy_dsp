# saavy_dsp

an approachable rust dsp toolkit for crafting musical voices from scratch.

## what it does

saavy_dsp keeps the essentials close at hand: oscillators, envelopes, sequencing, and a small graph system for wiring them together. start with presets, tweak until it sounds right, and dive deeper only when you want to.

it's a good match when you want to:
- learn synthesis without parsing a textbook
- prototype new sounds in rust with quick feedback loops
- build instruments for games, apps, or vst plugins
- explore sound design just for the joy of it

not trying to unseat max/msp, supercollider, or your favorite production synth—this stays in the friendly lane.

## guiding principles

- **approachable**: start making sound in minutes with ready-to-use patches and examples
- **composable**: small, predictable nodes that snap together into bigger voices
- **real-time safe**: lock-free, allocation-free audio paths
- **portable**: pure rust core that runs wherever rust does (embedded targets are on the roadmap)

## status snapshot

- **oscillators**: sine, saw, square, noise
- **envelopes**: adsr with lock-free parameter control
- **graph architecture**: modular processing nodes with trait-based composition
- **polyphony**: voice allocation, stealing, and mixing baked in
- **sequencing**: integer timing with proper time signatures, tuplets, and dotted values
- **real-time audio**: `cpal`-powered interactive demo

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

## examples

- `examples/envelope_demo.rs` — visualize adsr phases
- `examples/polyphony_demo.rs` — inspect voice allocation and stealing
- `examples/simple_poly.rs` — build a basic polyphonic synth voice
- `examples/cpal_demo.rs` — run the real-time interactive demo (`--features cpal-demo`)

run with:

```bash
cargo run --example envelope_demo
cargo run --example simple_poly
cargo run --features cpal-demo --example cpal_demo
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
- `OscNode` – oscillators (sine, saw, square, noise)
- `EnvNode` – adsr envelope
- `Amplify` – multiplies two signals for mixing or modulation

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

currently exploring:
- **filters**: lowpass, highpass, and bandpass shapes for sculpting tone
- **presets**: ready-made patches you can tweak and share
- **lfos**: modulation sources for movement over time
- **sequencing tools**: richer pattern building beyond single notes

longer-term dreams include wavetables, effects (delay/reverb), richer examples, and maybe a playful terminal ui.

## license

[MIT](LICENSE)
