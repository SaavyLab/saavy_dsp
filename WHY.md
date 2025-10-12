# project reson (working title)

## overview
_reson_ is an independent study in digital sound synthesis and signal processing.  
it’s not a product or a plugin — it’s an exploration of how sound works under the hood, built through code.

the focus:  
- learn real-time dsp fundamentals by **building** small, verifiable tools  
- develop a **rust-based dsp core** capable of real synthesis (no soundfonts)  
- deepen understanding of audio at the physical + computational level  
- eventually integrate insights back into broader music projects (e.g. orpheus)

## motivation
not everything has to serve a roadmap.  
this exists because sound is interesting — and because building something that *makes* sound is a different kind of literacy than building something that *plays* it.

**goals:**
- reach a working, minimal synthesizer: two oscillators, one filter, envelopes, lfo  
- understand the math behind filters, oscillators, and envelopes, not just use them  
- replace dependency on sample-based playback with procedural sound generation  
- publish open-source dsp crates that feel clean, modern, and idiomatic rust

## non-goals
- no focus on ui/ux, vst shells, or end-user polish  
- no premature optimization for mobile  
- no “music theory” ambitions — this is physics + math, not pedagogy  

## scope
- core dsp engine (`dsp-core`) — math, traits, processing graph  
- oscillators (`oscillators`) — sine, saw, square, noise, polyblep experiments  
- filters (`filters`) — tpt svf, ladder, one-pole studies  
- modulation (`mod`) — envelopes, lfos, param smoothing  
- engine prototype (`reson-engine`) — basic voice management, note on/off  
- ffi binding (future) — for integration into flutter or other systems  

## learning trajectory
1. **waveforms:** implement sine, saw, square, triangle  
2. **envelopes:** ad / adsr curves, param smoothing  
3. **filters:** one-pole, tpt svf, ladder, drive  
4. **modulation:** lfo routing + sample-accurate automation  
5. **voice management:** note on/off, voice stealing, glide  
6. **integration:** realtime audio via cpal or portaudio demo  
7. **analysis tools:** visualize waveforms / spectra (python sidecar or rust egui)  

## success criteria
- sound output that feels “alive” (no aliasing, stable envelopes)  
- code that’s idiomatic rust and RT-safe (no allocs in callback)  
- clean readmes + audio demos for each milestone  
- eventual open-source release under `SaavyLab`

## why it matters
for reputation: demonstrates full-stack audio literacy  
for curiosity: demystifies the craft of sound itself  
for the long term: lays groundwork to replace sample-based playback in future apps  

> “we build the lyre ourselves.”

---

