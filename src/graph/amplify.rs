use crate::{
    dsp::amplify::multiply_in_place,
    graph::node::{GraphNode, RenderCtx},
    MAX_BLOCK_SIZE,
};

/*
Amplify Node
============

Multiplies two graph nodes together, sample by sample. This is how you connect
an envelope to an oscillator, create tremolo, or do ring modulation.

When to Use Amplify
-------------------

Use `.amplify()` when one signal should CONTROL another's amplitude:

  // Envelope shapes oscillator volume (the classic synth voice)
  let voice = OscNode::sawtooth().amplify(EnvNode::adsr(0.01, 0.1, 0.7, 0.3));

  // LFO creates tremolo (volume wobble)
  let tremolo = OscNode::sine().amplify(LfoNode::sine(5.0));

  // Two oscillators create ring modulation (metallic sound)
  let ring_mod = OscNode::sine().amplify(OscNode::sine());


Amplify vs Mix vs Through
-------------------------

  .amplify()  →  Multiplies signals (one controls the other's volume)
  .mix()      →  Adds signals together (blending, layering)
  .through()  →  Chains source → processor (filtering, effects)

Quick rule: if you're connecting an envelope or LFO, you want `.amplify()`.


How It Works
------------

See `dsp/amplify.rs` for the implementation details, including:
- The multiplication math
- Attenuation and gain in decibels
- Ring modulation and aliasing considerations


Voice Activity
--------------

Amplify tracks activity through the MODULATOR, not the signal. Why?

The typical pattern is: oscillator.amplify(envelope)
- Oscillator runs forever (always "active")
- Envelope determines when the note is done (after release)

So we check the modulator to know when the voice can be recycled.
*/

/// Combines two graph nodes by multiplying their outputs sample-by-sample.
pub struct Amplify<S, M> {
    /// The primary signal source (e.g., oscillator)
    pub signal: S,
    /// The modulator that controls amplitude (e.g., envelope)
    pub modulator: M,
    /// Pre-allocated buffer for modulator output (avoids allocation in render)
    mod_buffer: Vec<f32>,
}

impl<S, M> Amplify<S, M> {
    pub fn new(signal: S, modulator: M) -> Self {
        Self {
            signal,
            modulator,
            mod_buffer: vec![0.0; MAX_BLOCK_SIZE],
        }
    }
}

impl<S: GraphNode, M: GraphNode> GraphNode for Amplify<S, M> {
    fn render_block(&mut self, out: &mut [f32], ctx: &RenderCtx) {
        // Render signal into output buffer
        self.signal.render_block(out, ctx);

        // Render modulator into temp buffer (no allocation - buffer is pre-sized)
        let mod_out = &mut self.mod_buffer[..out.len()];
        mod_out.fill(0.0);
        self.modulator.render_block(mod_out, ctx);

        // Multiply signal by modulator using dsp primitive
        multiply_in_place(out, mod_out);
    }

    fn note_on(&mut self, ctx: &RenderCtx) {
        self.signal.note_on(ctx);
        self.modulator.note_on(ctx);
    }

    fn note_off(&mut self, ctx: &RenderCtx) {
        self.signal.note_off(ctx);
        self.modulator.note_off(ctx);
    }

    fn is_active(&self) -> bool {
        // Check modulator, not signal - envelope determines when voice is done
        self.modulator.is_active()
    }

    fn get_envelope_level(&self) -> Option<f32> {
        // Prefer modulator's envelope level, fall back to signal's
        self.modulator
            .get_envelope_level()
            .or_else(|| self.signal.get_envelope_level())
    }
}
