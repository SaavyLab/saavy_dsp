# saavy_dsp (formerly "project reson")

## overview

an independent study in digital sound synthesis with two intertwined goals:
1. to build an **approachable, creative toolkit** for making music in rust—an "op-1 of dsps"
2. to ground that toolkit in a **living implementation** of classic dsp principles

it's not a product—it's an exploration of how a robust technical foundation can unlock playful, intuitive sound design.

## the focus

- **for the creator**: build a fluent, composable api that makes sound design feel like play
- **for the learner**: provide clear rust implementations where each component demonstrates foundational dsp concepts
- **unite the two**: create a library where you can seamlessly move between playfully designing a sound and deeply understanding the physics and math that make it work

## motivation

great tools are often a blend of simplicity and depth. this project exists to find that balance. the "op-1" spirit drives the user-facing api—making it fun, immediate, and inspiring. the "living guidebook" spirit drives the core—making it stable, correct, and educational.

you don't have to choose between a tool that's easy to use and a tool that's easy to understand. we're building both, in one place.

**Goals:**
- reach a working, minimal synthesizer with an intuitive, chainable api
- ensure every component is a clear, verifiable implementation of classic dsp principles
- replace dependency on sample-based playback with procedural sound generation
- publish open-source dsp crates that feel clean, modern, and idiomatic rust

## non-goals

- no focus on ui/ux, vst shells, or end-user polish (yet)
- no premature optimization for mobile
- no "music theory" ambitions — this is physics + math through a creative lens

## scope

- Core DSP primitives (`src/dsp/`) — math, oscillators, envelopes
- Graph architecture (`src/graph/`) — composable audio processing nodes
- Voice management (`src/synth/`) — polyphony, note handling, MIDI routing
- Real-time audio demo (`examples/cpal_demo.rs`) — interactive testing
- Future: filters, LFOs, parameter automation, FFI for Flutter integration

## learning trajectory

1. ✅ **waveforms:** sine, saw, square, triangle, noise — **DONE**
2. ✅ **envelopes:** ADSR with lock-free control — **DONE**
3. ✅ **filters:** TPT SVF (lowpass, highpass, bandpass, notch) — **DONE**
4. ✅ **modulation:** LFO routing + modulatable delay — **DONE**
5. ✅ **voice management:** note on/off, voice stealing — **DONE**
6. ✅ **integration:** real-time audio via CPAL — **DONE**
7. 🔄 **analysis tools:** oscilloscope visualization working — **DONE**, spectrum analyzer TODO

## success criteria

- ✅ Sound output that feels "alive" (no aliasing, stable envelopes)
- ✅ Code that's idiomatic Rust and RT-safe (no allocs in callback)
- ✅ Clean examples demonstrating features
- 🔄 Open-source release under `SaavyLab` — publishing soon

## current milestone: Complete Subtractive Synthesis Engine

**What works:**
- Complete oscillator set (sine, saw, square, triangle, noise)
- ADSR envelope with all phases (Attack, Decay, Sustain, Release)
- TPT State Variable Filter with modulatable cutoff/resonance (LP/HP/BP/Notch)
- LFO with multiple waveforms for modulation
- Modulatable delay line
- Lock-free message passing for real-time control (`rtrb`)
- Polyphonic voice management (allocation, stealing, mixing)
- Graph combinators: `.amplify()`, `.through()`, `.modulate()`
- Real-time audio callback via CPAL
- Oscilloscope visualization (`cpal_scope.rs`)
- Comprehensive test coverage

**What's next:**
- `.mix()` combinator for parallel signal routing
- Effect helpers (chorus/flanger as delay+LFO combinations)
- Voice profiles (kick, snare, bass, lead presets)
- MIDI keyboard input via `midir`
- Track/Arrangement system for pattern-based composition

## why it matters

**For reputation:** Demonstrates full-stack audio literacy
**For curiosity:** Demystifies the craft of sound itself
**For the long term:** Lays groundwork to replace sample-based playback in future apps

> "we build the lyre ourselves."

---

## architectural decisions

### Why `graph/` instead of `dsp_fluent/`?
"Graph" is industry-standard terminology (audio graphs, processing graphs). It better communicates the node-based architecture.

### Why lock-free instead of mutex?
Real-time audio threads cannot block. A mutex can cause priority inversion, leading to audio glitches. We use `rtrb` (SPSC ring buffer) for non-blocking message passing.

### Why separate `Voice` and `PolySynth`?
Clear separation of concerns:
- `Voice` = single-voice chain (oscillator + envelope)
- `PolySynth` = manages multiple voices, handles allocation/stealing/mixing

This scales from monophonic to polyphonic without architectural changes.

### Why no high-level `Engine` API yet?
We're building bottom-up. The low-level pieces (graph nodes, voices) are solid. High-level APIs will emerge once usage patterns stabilize.

---

**Status:** Complete subtractive synthesis engine with modulation, ready for effect chains and MIDI integration 🎹
