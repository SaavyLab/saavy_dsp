use crate::{
    graph::node::{GraphNode, RenderCtx},
    MAX_BLOCK_SIZE,
};

/*
Signal Multiplication (Amplify)
===============================

Amplify multiplies two signals together, sample by sample. This fundamental
operation enables two essential synthesis techniques: amplitude control
(volume shaping) and ring modulation (timbral effects).

How It Works:
-------------
For each sample: output = signal × modulator

  Signal:    [0.5,  0.8, -0.3,  0.9, ...]
  Modulator: [1.0,  0.5,  0.5,  0.0, ...]
  Output:    [0.5,  0.4, -0.15, 0.0, ...]

Common Use Cases:
-----------------

1. Envelope Control (most common):
   Multiply an oscillator by an envelope to shape its volume over time.

     let voice = OscNode::sawtooth().amplify(EnvNode::adsr(0.01, 0.1, 0.7, 0.3));

   - Oscillator outputs audio (-1.0 to 1.0)
   - Envelope outputs control signal (0.0 to 1.0)
   - Result: Audio that fades in/out with the envelope

2. Tremolo (amplitude modulation):
   Multiply by a slow LFO to create volume wobble.

     let tremolo = OscNode::sine().amplify(LfoNode::sine(5.0));

   - Creates a pulsing/throbbing effect
   - LFO rate controls wobble speed

3. Ring Modulation:
   Multiply two audio-rate signals for metallic, inharmonic tones.

     let ring_mod = OscNode::sine().amplify(OscNode::sine());

   - Creates sum and difference frequencies
   - Classic "robot voice" or bell-like sounds
   - Output frequencies: (f1 + f2) and (f1 - f2)

Amplify vs Mix vs Through:
--------------------------
- Amplify: Multiplies signals (envelope control, ring mod)
- Mix:     Blends signals additively (wet/dry, layering)
- Through: Chains source → effect (filtering, processing)

Choose Amplify when you need one signal to control another's amplitude.

Note on Activity:
-----------------
Amplify tracks activity through the modulator (not the signal), because the
typical use case is oscillator × envelope, where the envelope determines
when the voice is done (after release completes).
*/

pub struct Amplify<N, M> {
    pub signal: N,
    pub modulator: M,
    temp_buffer: Vec<f32>,
}

impl<N, M> Amplify<N, M> {
    pub fn new(signal: N, modulator: M) -> Self {
        Self {
            signal,
            modulator,
            temp_buffer: vec![0.0; MAX_BLOCK_SIZE],
        }
    }
}

impl<N: GraphNode, M: GraphNode> GraphNode for Amplify<N, M> {
    fn render_block(&mut self, out: &mut [f32], ctx: &RenderCtx) {
        // Render signal into output
        self.signal.render_block(out, ctx);

        // Slice temp buffer to match output size (RT-safe, no allocation)
        let frames = &mut self.temp_buffer[..out.len()];
        frames.fill(0.0);
        self.modulator.render_block(frames, ctx);

        // Multiply signal by modulator (ring modulation / amplitude control)
        for (o, m) in out.iter_mut().zip(frames.iter()) {
            *o *= *m;
        }
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
        // Intentionally not checking signal's activity
        // because it's an oscillator and that's constant
        self.modulator.is_active()
    }

    fn get_envelope_level(&self) -> Option<f32> {
        // Prefer the modulator's envelope (e.g., ADSR), otherwise fall back to the signal
        self.modulator
            .get_envelope_level()
            .or_else(|| self.signal.get_envelope_level())
    }
}
