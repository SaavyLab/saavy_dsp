# saavy_dsp

learn digital audio by building synthesizers in rust.

## what it is

saavy_dsp is a hands-on dsp education disguised as a synth toolkit. the code itself is the textbook—open any source file and find clear explanations of the concepts alongside working implementations. tinker with `main.rs`, wonder how envelopes work, read `envelope.rs`, and suddenly adsr makes sense.

```
cargo run
```

that's it. you'll see a timeline, an oscilloscope, and hear two voices playing. now open `src/bin/saavy/main.rs` and start changing things.

it's a good match when you want to:
- **learn dsp** by building, not just reading theory
- **understand synthesis** from the source—every file explains what it does and why
- **experiment freely** with a tui that shows you what's happening in real-time
- **build intuition** for oscillators, envelopes, filters, and sequencing

## guiding principles

- **code is the curriculum**: every source file teaches the concept it implements
- **tinker-first**: make sound immediately, understand later (or as you go)
- **see what you hear**: the tui visualizes audio so you can connect code to sound
- **real-time safe**: lock-free, allocation-free audio paths—learn good habits from the start

## status snapshot

- **tui**: timeline, oscilloscope, transport controls—see your patterns play
- **oscillators**: sine, saw, square, triangle, noise
- **envelopes**: adsr with lock-free parameter control
- **filters**: tpt state variable filter (lowpass, highpass, bandpass, notch)
- **modulation**: lfo with multiple waveforms, modulatable delay
- **graph architecture**: modular processing nodes with trait-based composition
- **graph combinators**: `.amplify()`, `.through()`, `.modulate()`
- **sequencing**: sample-accurate playback with proper time signatures, tuplets, and dotted values
- **pattern api**: concise macro syntax for musical patterns with subdivisions and triplets
- **polyphony**: voice allocation, stealing, and mixing for keyboard/midi use cases

## quickstart

### the tinkering workflow

1. run `cargo run` to see the tui and hear the default pattern
2. open `src/bin/saavy/main.rs` in your editor
3. change a note, add a track, swap an oscillator
4. run again and hear/see the difference

```rust
// src/bin/saavy/main.rs - your playground
Saavy::new()
    .bpm(120.0)
    .track(
        "lead",
        pattern!(4/4 => [C4, E4, G4, C5]).repeat(4),
        OscNode::sawtooth()
            .amplify(EnvNode::adsr(0.01, 0.1, 0.6, 0.2))
            .through(FilterNode::lowpass(2000.0)),
    )
    .run()
```

want to understand envelopes? open `src/graph/envelope.rs`. filters? `src/graph/filter.rs`. the code explains itself.

### musical patterns

```rust
use saavy_dsp::pattern;
use saavy_dsp::sequencing::*;

fn main() {
    // concise pattern syntax - brackets subdivide time
    let arp = pattern!(4/4 => [C4, E4, G4, C5]);           // 4 quarter notes
    let groove = pattern!(4/4 => [C4, [E4, G4], C5, _]);   // quarter, 2 eighths, quarter, rest
    let triplet = pattern!(4/4 => [[C4, E4, G4], _, _, _]); // triplet on beat 1

    // chain patterns together
    let song = arp.repeat(2)
        .then(groove)
        .then(triplet);

    // convert to sequence for playback
    let seq = song.to_sequence(480); // 480 ppq
}
```

the pattern api is built on top of the lower-level `Sequence` builder, which gives you fine-grained control when you need it:

```rust
use saavy_dsp::sequencing::{Duration, Sequence};

fn main() {
    // explicit timing control
    let seq = Sequence::new(480)
        .note(Duration::EIGHTH).with_note(36)       // kick on 1
        .rest(Duration::EIGHTH)                      // and
        .rest(Duration::EIGHTH)                      // 2
        .note(Duration::EIGHTH).with_note(38)       // snare on and-of-2
        .rest(Duration::QUARTER)                     // 3
        .note(Duration::QUARTER).with_note(36)      // kick on 4
        .build()
        .unwrap();
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

## running

```bash
cargo run                         # tui with timeline, oscilloscope, and pattern playback
cargo run --example cpal_scope    # standalone oscilloscope with chorus effect
cargo run --example envelope_demo # visualize adsr phases
cargo run --example simple_poly   # basic polyphonic synth
cargo run --example pattern_player # pattern api demo
```

the `src/bin/saavy/` binary is your main playground. the examples show specific concepts in isolation.

## cargo features

- `default = []` — minimal build, no optional features
- `cpal-demo` — enables the realtime demo via `cpal` and `crossterm`
- `serde` — (de)serialization hooks for the future preset system
- `analysis` — pulls in `rustfft` for analysis tooling

## architecture at a glance

```
src/
├── bin/saavy/    # tui binary - your playground (start here!)
├── dsp/          # low-level primitives (oscillators, envelopes, filters)
├── graph/        # composable nodes with educational docs (read these!)
├── sequencing/   # musical durations, sequences, and timing helpers
├── synth/        # polyphonic synth internals (voice allocation, messaging)
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
- **voice profiles**: ready-made patches (kick, snare, bass, lead) you can study and tweak
- **midi integration**: keyboard input via `midir` for live performance
- **more visualizations**: per-track envelopes, spectrum analyzer

longer-term dreams include wavetables, reverb, and a daw-lite terminal experience for composing full tracks.

## license

[MIT](LICENSE)
