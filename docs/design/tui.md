# TUI Design

The TUI (`src/bin/saavy.rs`) is the primary interface for saavy_dsp - not just a demo, but the canonical way to run and interact with the DSP toolkit.

## Vision

A DAW-lite terminal experience: pattern sequencing meets live visualization, all in your terminal.

```
┌─ saavy ──────────────────────────────────────────────────────────────────────┐
│  BPM: 120   4/4   ▶ Playing                                    [?] help      │
├──────────────────────────────────────────────────────────────────────────────┤
│  TRACKS                    │ 1   2   3   4 │ 1   2   3   4 │ 1   2   3   4   │
│  ─────────────────────────────────────────────────────────────────────────── │
│  lead    ░░▓▓░░▓▓░░▓▓░░▓▓  │ C4  E4  G4  C5│ C4  E4  G4  C5│ ▓▓░░▓▓░░        │
│  bass    ▓▓░░░░░░▓▓░░░░░░  │ C2  _   C2  _ │ C2  _   C2  _ │ ▓▓░░░░░░        │
│  drums   ▓▓░░▓░▓▓░░▓░▓▓░░  │ K   H   S   H │ K   H   S   H │ ▓▓░░▓░░░        │
│                        ▲ playhead                                            │
├──────────────────────────────────────────────────────────────────────────────┤
│  TRACK: lead                                                                 │
│  ┌─ Oscilloscope ─────┐ ┌─ Spectrum ──────┐ ┌─ ADSR ──────────┐              │
│  │     ╭──╮   ╭──╮    │ │ ▁▂▄▆█▆▄▂▁      │ │   /\            │              │
│  │ ───╯    ╰─╯    ╰── │ │ ▁▂▃▄▅▆▇█▇▆▅▄▃▂▁│ │  /  \___        │              │
│  │                    │ │                 │ │ /       \       │              │
│  └────────────────────┘ └─────────────────┘ └─────────────────┘              │
├──────────────────────────────────────────────────────────────────────────────┤
│  [space] play/pause  [←→] seek  [↑↓] select track  [e] edit  [q] quit        │
└──────────────────────────────────────────────────────────────────────────────┘
```

## Core Views

### 1. Timeline View (top)
- Horizontal scrolling pattern grid
- Multiple tracks stacked vertically
- Playhead indicator showing current position
- Beat/bar markers
- Visual density showing note activity (░▓ blocks)
- Expandable to show actual note names

### 2. Track Visualizers (bottom)
- **Oscilloscope**: Real-time waveform of track output
- **Spectrum Analyzer**: FFT frequency display
- **ADSR Envelope**: Current envelope state with stage indicator
- **Filter Response** (optional): Cutoff/resonance curve

### 3. Transport Bar
- BPM (editable)
- Time signature
- Play/pause/stop state
- Current position (bar:beat:tick)

## Interaction Model

### Navigation
- `↑/↓` - Select track
- `←/→` - Seek through timeline (when paused) or scroll view (when playing)
- `Tab` - Cycle through visualizer focus
- `Enter` - Expand/collapse track detail

### Playback
- `Space` - Play/pause
- `Home` - Return to start
- `.` - Stop and reset

### Editing (future)
- `e` - Enter edit mode for selected track
- `i` - Insert pattern
- `d` - Delete pattern
- `c` - Copy pattern
- `v` - Paste pattern

## Architecture

```
src/bin/saavy.rs
├── main()              - Setup, event loop
├── app.rs              - App state, mode management
├── ui/
│   ├── mod.rs          - Layout, frame rendering
│   ├── timeline.rs     - Pattern grid widget
│   ├── visualizer.rs   - Scope/spectrum/ADSR widgets
│   ├── transport.rs    - BPM/time sig bar
│   └── help.rs         - Help overlay
└── audio/
    ├── engine.rs       - cpal setup, audio thread
    └── bridge.rs       - UI ↔ audio communication (ring buffers)
```

## Implementation Phases

### Phase 1: Foundation
- Move `cpal_scope.rs` to `src/bin/saavy.rs`
- Clean up into modular structure
- Basic transport controls

### Phase 2: Timeline
- Single-track pattern display
- Playhead visualization
- Beat grid

### Phase 3: Multi-track
- Track list with selection
- Per-track visualizers
- Track mute/solo

### Phase 4: Editing
- Pattern editing mode
- Copy/paste
- Real-time pattern updates

## Technical Notes

### Audio-UI Sync
- Use `rtrb` ring buffers for lock-free communication
- Audio thread pushes visualization data (waveform samples, FFT bins)
- UI thread reads at frame rate (~60fps)
- Playhead position tracked via atomic counter

### Visualization Data Flow
```
Audio Thread                    UI Thread
───────────────────────────────────────────
render samples ──┬──> output buffer
                 │
                 └──> viz ring buffer ──> read at 60fps
                                          │
                      position atomic <───┘ (for playhead)
```

### Dependencies
- `ratatui` - TUI framework
- `crossterm` - Terminal backend
- `cpal` - Audio I/O
- `rustfft` - Spectrum analysis
- `rtrb` - Lock-free queues

## Open Questions

1. **Pattern editing UX**: Modal vim-style? Direct manipulation? Both?
2. **Color scheme**: Adapt to terminal theme or fixed palette?
3. **Mouse support**: Worth adding for timeline scrubbing?
4. **MIDI input**: Show incoming MIDI in timeline?
