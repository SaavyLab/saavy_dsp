# Pattern API Design

This document captures the design for a new pattern-based sequencing API, replacing the current verbose `Sequence` builder.

## Goals

1. **Quick sketches** - Optimized for fun, fast musical ideas (not full songs)
2. **Obvious** - Code should clearly represent the music
3. **Combined pitch + rhythm** - Notes carry their timing implicitly via position
4. **DAW-like mental model** - Structured, visual, not magic strings

## Core Concepts

### Pattern

A `Pattern` is a single cycle of notes with a time signature. It's a self-contained musical phrase.

```rust
let arp = pattern!(4/4 => [C4, E4, G4, C5]);
```

**Why time signature lives on Pattern:**
- Compound vs simple meter changes how subdivisions work
- A pattern is a complete musical thought with its own feel
- Different patterns can have different meters in the same piece
- The sequencer handles tempo/playback, not musical structure

### Subdivision (Brackets)

Brackets subdivide a slot equally among their contents. This is borrowed from Strudel/TidalCycles but expressed in Rust syntax.

```rust
// Each top-level item gets one beat
pattern!(4/4 => [C4, E4, G4, C5])  // 4 quarter notes

// Brackets subdivide a beat
pattern!(4/4 => [C4, [E4, G4], C5, _])  // quarter, 2 eighths, quarter, rest

// Nested brackets subdivide further
pattern!(4/4 => [C4, [E4, [G4, B4]], _, _])  // quarter, eighth, 2 sixteenths, rests
```

### Triplets and Tuplets

Triplets happen naturally - put 3 things in a bracket:

```rust
// Three notes in one beat = triplet
pattern!(4/4 => [[C4, E4, G4], _, _, _])  // triplet quarter notes, then rests

// Quintuplet: 5 notes in one beat
pattern!(4/4 => [[C4, D4, E4, F4, G4], _, _, _])
```

### Rests

Use `_` for rests:

```rust
pattern!(4/4 => [C4, _, G4, _])  // quarter, rest, quarter, rest
pattern!(4/4 => [[C4, _, E4], _, _, _])  // triplet with rest in middle
```

### Swing / Uneven Subdivisions

Use `@weight` to give a note more time within its group:

```rust
// Swing feel: first note gets 2 parts, second gets 1
pattern!(4/4 => [[C4@2, E4], [G4@2, B4], _, _])

// This creates a 2:1 ratio (like jazz swing)
// C4 gets 2/3 of the beat, E4 gets 1/3
```

### Compound Meter

Time signature determines what "one beat" means:

```rust
// 4/4: beat = quarter note, subdivides in 2s
pattern!(4/4 => [C4, E4, G4, C5])  // 4 quarter notes

// 6/8: beat = dotted quarter, subdivides in 3s
pattern!(6/8 => [C4, E4])  // 2 dotted quarters = 1 bar

// 6/8 with subdivision
pattern!(6/8 => [[C4, E4, G4], [C5, B4, G4]])  // 2 beats, each subdivided into 3
```

## Composition

### Chaining Patterns

```rust
let intro = pattern!(4/4 => [C4, _, _, _]);
let main = pattern!(4/4 => [C4, E4, G4, C5]);

// Chain into a sequence
let song = intro.then(main).then(main);

// Or repeat
let verse = main.repeat(4);
```

### Layering (Multiple Voices)

```rust
let bass = pattern!(4/4 => [C2, _, C2, _]);
let lead = pattern!(4/4 => [C4, E4, G4, C5]);

// Layer plays patterns simultaneously
let combined = layer![bass, lead];
```

## Playback

### Sequencer

The `Sequencer` handles tempo and playback:

```rust
Sequencer::new(120.0)  // BPM
    .play(arp)
    .looping(true)
```

### Multi-track

```rust
Sequencer::new(120.0)
    .track(bass_pattern, bass_synth)
    .track(lead_pattern, lead_synth)
    .looping(true)
```

## Note Constants

For readability, we provide note constants:

```rust
// Octave 4 (middle C = C4 = MIDI 60)
pub const C4: u8 = 60;
pub const D4: u8 = 62;
pub const E4: u8 = 64;
pub const F4: u8 = 65;
pub const G4: u8 = 67;
pub const A4: u8 = 69;
pub const B4: u8 = 71;

// Sharps/flats
pub const Cs4: u8 = 61;  // C#4 / Db4
pub const Db4: u8 = 61;
// ... etc for all notes, all octaves (0-8)
```

## Macro Expansion

The `pattern!` macro expands to:

```rust
// This:
pattern!(4/4 => [C4, [E4, G4], _, C5])

// Becomes approximately:
Pattern::new(TimeSignature::new(4, 4))
    .slot(Note(C4))
    .slot(Subdivision(vec![Note(E4), Note(G4)]))
    .slot(Rest)
    .slot(Note(C5))
    .build()
```

## Comparison to Current API

### Before (verbose)

```rust
Sequence::new(480)
    .note(Duration::QUARTER).with_note(60)
    .note(Duration::EIGHTH).with_note(64)
    .note(Duration::EIGHTH).with_note(67)
    .rest(Duration::QUARTER)
    .note(Duration::QUARTER).with_note(72)
    .bars(1)
    .time_signature(TimeSignature::FOUR_FOUR)
    .build()?
```

### After (concise)

```rust
pattern!(4/4 => [C4, [E4, G4], _, C5])
```

## Migration Path

The existing `Sequence`, `Duration`, and `TimeSignature` types remain as the low-level building blocks. The new `Pattern` and `pattern!` macro are a higher-level API built on top.

```
pattern! macro
     │
     ▼
  Pattern (new)
     │
     ▼
  Sequence (existing) ─── uses ──► Duration (existing)
     │                              TimeSignature (existing)
     ▼
  SequenceEvent (existing)
```

## Open Questions

1. **How to handle patterns of different lengths in a layer?**
   - Loop shorter patterns? Error? Stretch?

2. **Should we support polyrhythms?**
   - e.g., 3 against 4

3. **What about dynamics/velocity?**
   - Could use `C4!ff` or `C4~80` syntax, but adds complexity

4. **Pitch bend, CC, other MIDI events?**
   - Out of scope for v1, but design should allow extension

## Implementation Plan

1. Define note constants (`src/sequencing/notes.rs`)
2. Define `Pattern` struct and `PatternSlot` enum
3. Implement `pattern!` macro with basic syntax
4. Add subdivision support (brackets)
5. Add swing support (@weight)
6. Add composition methods (`.then()`, `.repeat()`)
7. Build `Sequencer` playback engine
8. Connect to TUI visualization
