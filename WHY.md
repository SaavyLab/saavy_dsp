# saavy_dsp (formerly "project reson")

## overview

An independent study in digital sound synthesis and signal processing.
It's not a product or a plugin — it's an exploration of how sound works under the hood, built through code.

## the focus

- Learn real-time DSP fundamentals by **building** small, verifiable tools
- Develop a **rust-based DSP core** capable of real synthesis (no soundfonts)
- Deepen understanding of audio at the physical + computational level
- Eventually integrate insights back into broader music projects (e.g. orpheus)

## motivation

Not everything has to serve a roadmap.
This exists because sound is interesting — and because building something that *makes* sound is a different kind of literacy than building something that *plays* it.

**Goals:**
- Reach a working, minimal synthesizer: oscillators, envelopes, filters, LFOs
- Understand the math behind filters, oscillators, and envelopes, not just use them
- Replace dependency on sample-based playback with procedural sound generation
- Publish open-source DSP crates that feel clean, modern, and idiomatic Rust

## non-goals

- No focus on UI/UX, VST shells, or end-user polish
- No premature optimization for mobile
- No "music theory" ambitions — this is physics + math, not pedagogy

## scope

- Core DSP primitives (`src/dsp/`) — math, oscillators, envelopes
- Graph architecture (`src/graph/`) — composable audio processing nodes
- Voice management (`src/synth/`) — polyphony, note handling, MIDI routing
- Real-time audio demo (`examples/cpal_demo.rs`) — interactive testing
- Future: filters, LFOs, parameter automation, FFI for Flutter integration

## learning trajectory

1. ✅ **waveforms:** sine, saw, square, noise — **DONE**
2. ✅ **envelopes:** ADSR with lock-free control — **DONE**
3. 🔄 **filters:** TPT SVF (lowpass, highpass, bandpass) — IN PROGRESS
4. ⏳ **modulation:** LFO routing + sample-accurate automation — TODO
5. ✅ **voice management:** note on/off, voice stealing — **DONE**
6. ✅ **integration:** real-time audio via CPAL — **DONE**
7. ⏳ **analysis tools:** visualize waveforms / spectra — TODO

## success criteria

- ✅ Sound output that feels "alive" (no aliasing, stable envelopes)
- ✅ Code that's idiomatic Rust and RT-safe (no allocs in callback)
- ✅ Clean examples demonstrating features
- 🔄 Open-source release under `SaavyLab` — publishing soon

## current milestone: ADSR + Polyphony

**What works:**
- ADSR envelope with all phases (Attack, Decay, Sustain, Release)
- Lock-free message passing for real-time control (`rtrb`)
- Polyphonic voice management (allocation, stealing, mixing)
- Graph-based composition (`.amplify()` combinator)
- Real-time audio callback via CPAL
- Comprehensive test coverage

**What's next:**
- TPT State Variable Filter (LP/HP/BP)
- LFO modulation sources
- MIDI keyboard input via `midir`
- More graph combinators (`.through()`, `.mix()`)

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

**Status:** Production-ready polyphony, ready for MIDI integration 🎹
