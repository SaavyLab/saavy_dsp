# Session Handoff

**Date:** 2026-01-04
**Last working on:** Prototyping new Pattern API

---

## Summary

We're redesigning the sequencing API to be more ergonomic for quick musical sketches. The goal is a Strudel-inspired but Rust-native pattern syntax.

## What's Done

1. **Educational comments added** to graph primitives:
   - `src/graph/oscillator.rs` - waveform types, harmonics
   - `src/graph/envelope.rs` - ADSR stages, presets
   - `src/graph/filter.rs` - SVF, filter types, cutoff/resonance
   - `src/graph/amplify.rs` - signal multiplication, ring mod
   - `src/graph/through.rs` - serial signal chains

2. **Design doc created** for new Pattern API:
   - `docs/design/pattern-api.md` - full design with examples

3. **Note constants started**:
   - `src/sequencing/notes.rs` - C0-B8 with sharps/flats (CREATED, not yet wired up)

## In Progress

Adding `notes.rs` to module exports. Next step was:
```rust
// In src/sequencing/mod.rs, add:
pub mod notes;
pub use notes::*;
```

## What's Next (in order)

1. Wire up `notes.rs` in `mod.rs`
2. Create `Pattern` struct (`src/sequencing/pattern.rs`)
3. Implement `pattern!` macro (start simple)
4. Move TUI to `src/bin/saavy.rs`
5. Build sequencer playback engine

## Key Design Decisions

### Pattern API Syntax
```rust
// Time sig on pattern, brackets for subdivision
pattern!(4/4 => [C4, E4, G4, C5])

// Triplets = 3 things in brackets
pattern!(4/4 => [[C4, E4, G4], _, _, _])

// Rests with _
pattern!(4/4 => [C4, _, G4, _])

// Swing with @weight
pattern!(4/4 => [[C4@2, E4], [G4@2, B4]])
```

### Why time sig on Pattern (not Sequencer)
- Compound vs simple meter changes subdivision meaning
- Pattern is a self-contained musical phrase
- Different patterns can have different meters

### Hierarchy
```
Pattern (one cycle) → Sequence (chained patterns) → Sequencer (playback)
```

### v1 Scope (keep simple)
- Notes, rests, subdivisions, swing, time sig
- `.then()`, `.repeat()`, `layer!`
- NO dynamics, ties, pitch bend (v2)

## Files to Know

| File | Purpose |
|------|---------|
| `docs/design/pattern-api.md` | Full API design doc |
| `src/sequencing/notes.rs` | Note constants (just created) |
| `src/sequencing/sequence.rs` | Current (verbose) API - will be low-level layer |
| `examples/cpal_scope.rs` | TUI demo - will become `src/bin/saavy.rs` |

## Test the Current State

```bash
cargo build        # Should work
cargo test         # Should pass
cargo run --example cpal_scope  # TUI demo with synth
```

## Context

The project is `saavy_dsp` - an educational DSP toolkit for building synths. We're making it more fun/usable by:
1. Adding a first-class TUI binary
2. Building a real sequencer (not just hardcoded arpeggios)
3. Making the sequencing API ergonomic for quick sketches
