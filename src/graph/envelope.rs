use crate::{
    dsp::envelope::{Envelope, EnvelopeState},
    graph::node::{GraphNode, RenderCtx},
};

/*
ADSR Envelope Generator
=======================

An envelope shapes how a sound evolves over time. Without an envelope, a note
would instantly appear at full volume and instantly disappear - very unnatural.
The ADSR envelope is the most common type, modeling how acoustic instruments
behave.

The Four Stages:
----------------

  Level
    ^
  1 |      /\
    |     /  \___________
    |    /               \
    |   /                 \
  0 |__/                   \___
    +--[A]-[D]----[S]-----[R]---> Time
        │   │      │       │
        │   │      │       └── Release: fade out after key release
        │   │      └────────── Sustain: held level while key is down
        │   └───────────────── Decay: fall to sustain level
        └───────────────────── Attack: initial rise to peak

Attack (seconds): How quickly the sound reaches full volume.
  - 0.001-0.01: Percussive, immediate (drums, plucks)
  - 0.05-0.2:   Soft attack (pads, strings)
  - 0.5+:       Slow swells (ambient, cinematic)

Decay (seconds): How quickly it falls from peak to sustain level.
  - 0.01-0.1:   Snappy, punchy (plucks, keys)
  - 0.2-0.5:    Natural decay (piano-like)
  - 1.0+:       Slow fade (pads)

Sustain (0.0-1.0): The level held while the key is pressed.
  - 0.0:        No sustain (fully percussive)
  - 0.5-0.8:    Typical for sustained sounds
  - 1.0:        No decay at all (organ-like)

Release (seconds): How quickly the sound fades after key release.
  - 0.01-0.1:   Tight, staccato
  - 0.2-0.5:    Natural release
  - 1.0+:       Long tail (pads, reverb-like)

Common Presets:
---------------
  Pluck/Keys:  EnvNode::adsr(0.005, 0.2,  0.3, 0.1)   // Quick attack, short sustain
  Pad:         EnvNode::adsr(0.3,   0.5,  0.7, 0.8)   // Slow attack, long release
  Brass:       EnvNode::adsr(0.08,  0.1,  0.8, 0.15)  // Slightly soft attack
  Percussion:  EnvNode::adsr(0.001, 0.1,  0.0, 0.1)   // Instant attack, no sustain

How Envelopes Connect:
----------------------
Envelopes output a control signal (0.0 to 1.0) that multiplies another signal.
Use .amplify() to connect an envelope to an oscillator:

  let voice = OscNode::sawtooth().amplify(EnvNode::adsr(0.01, 0.1, 0.7, 0.3));

The envelope can also modulate filter cutoff for that classic "wah" effect:

  let filter = FilterNode::lowpass(500.0);
  let env = EnvNode::adsr(0.01, 0.2, 0.3, 0.1);
  // Use .modulate() to sweep the filter with the envelope
*/

pub struct EnvNode {
    env: Envelope,
}

impl EnvNode {
    pub fn new() -> Self {
        let env = Envelope::new();
        Self { env }
    }

    pub fn adsr(attack: f32, decay: f32, sustain: f32, release: f32) -> Self {
        let env = Envelope::adsr(attack, decay, sustain, release);
        Self { env }
    }

    /// Get the current envelope level (for visualization)
    pub fn level(&self) -> f32 {
        self.env.level()
    }

    /// Get the current envelope state (for visualization)
    pub fn state(&self) -> EnvelopeState {
        self.env.state()
    }
}

impl GraphNode for EnvNode {
    fn render_block(&mut self, out: &mut [f32], ctx: &RenderCtx) {
        self.env.render(out, ctx);
    }

    fn note_on(&mut self, ctx: &RenderCtx) {
        self.env.note_on(ctx);
    }

    fn note_off(&mut self, ctx: &RenderCtx) {
        self.env.note_off(ctx);
    }

    fn is_active(&self) -> bool {
        self.env.is_active()
    }

    fn get_envelope_level(&self) -> Option<f32> {
        Some(self.env.level())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_RATE: f32 = 1_000.0;

    fn ctx() -> RenderCtx {
        RenderCtx::from_freq(SAMPLE_RATE, 440.0, 100.0)
    }

    #[test]
    fn render_outputs_envelope_levels() {
        let mut node = EnvNode::adsr(0.01, 0.02, 0.5, 0.1);
        let mut buffer = vec![0.0; 128];
        let ctx = ctx();

        node.note_on(&ctx);
        node.render_block(&mut buffer, &ctx);

        assert!(buffer.iter().any(|&sample| sample > 0.0));
        assert!(buffer.iter().all(|&sample| sample <= 1.0));
    }

    #[test]
    fn level_and_state_reflect_internal_envelope() {
        let mut node = EnvNode::adsr(0.01, 0.02, 0.4, 0.1);
        let ctx = ctx();

        node.note_on(&ctx);
        let level_after_on = node.level();
        assert!(level_after_on >= 0.0);

        node.note_off(&ctx);
        let mut buffer = vec![0.0; 256];
        node.render_block(&mut buffer, &ctx);

        assert!(node.level() <= 1.0);
        assert!(matches!(node.state(), EnvelopeState::Release | EnvelopeState::Idle));
    }
}
