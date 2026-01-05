use crate::{graph::node::RenderCtx, MIN_TIME};

/*
ADSR Envelope Implementation
============================

This module implements a linear ADSR envelope generator - the workhorse of
synthesizer amplitude control.

Vocabulary
----------

  level       The envelope's current output value (0.0 to 1.0). This multiplies
              the audio signal to control its amplitude over time.

  stage       Which phase of the envelope we're in: Idle, Attack, Decay,
              Sustain, or Release. A state machine governs transitions.

  gate        The note on/off signal. Gate high (note_on) triggers Attack.
              Gate low (note_off) triggers Release from wherever we are.

  increment   How much `level` changes per sample. Calculated from the stage
              duration and sample rate.

  sample_rate Samples per second (e.g., 48000). Converts time in seconds to
              time in samples.


The Shape: Linear Ramps
-----------------------

  Level
    1.0 ┐     ╱╲
        │    ╱  ╲___________
    S   │   ╱               ╲
        │  ╱                 ╲
    0.0 └─╱───────────────────╲──→ Time
        Attack Decay  Sustain  Release
         (A)   (D)      (S)      (R)

We use LINEAR ramps (straight lines) rather than exponential curves.

Linear pros:  Simple, predictable, CPU-cheap
Linear cons:  Doesn't match how acoustic sounds decay (exponential)

Many classic analog synths used linear envelopes. Exponential envelopes
sound more "natural" but linear is fine for learning and sounds punchy.


The Math: Time to Increment
---------------------------

The key calculation converts a time duration into a per-sample increment:

    increment = target_change / (time_seconds * sample_rate)

Example: Attack of 0.1 seconds at 48kHz
  - We need level to go from 0.0 → 1.0 (change of 1.0)
  - Total samples = 0.1 * 48000 = 4800 samples
  - increment = 1.0 / 4800 ≈ 0.000208

Each sample, we do: level += increment
After 4800 samples: level = 4800 * 0.000208 ≈ 1.0 ✓


The State Machine
-----------------

    ┌──────────────────────────────────────────────────────┐
    │                                                      │
    │   ┌──────┐  note_on   ┌────────┐  level=1   ┌─────┐ │
    │   │ Idle │ ─────────→ │ Attack │ ─────────→ │Decay│ │
    │   └──────┘            └────────┘            └─────┘ │
    │       ↑                    │                   │    │
    │       │                    │ note_off          │    │
    │       │                    ↓                   ↓    │
    │       │               ┌─────────┐  level=S  ┌─────┐ │
    │       │               │ Release │ ←──────── │ Sus │ │
    │       │               └─────────┘  note_off └─────┘ │
    │       │                    │                        │
    │       │    level=0         │                        │
    │       └────────────────────┘                        │
    │                                                      │
    └──────────────────────────────────────────────────────┘

Key behavior: note_off triggers Release from ANY stage (Attack, Decay, or
Sustain). Release always starts from the CURRENT level, not the sustain level.
This prevents clicks when releasing during attack.


Implementation Notes
--------------------

We calculate increment fresh each sample rather than caching it. This:
  - Handles sample_rate changes gracefully
  - Keeps code simple (no cache invalidation)
  - Has negligible performance cost (one division per sample)

Release is special: we snapshot the starting level and total samples at
note_off time, then interpolate linearly. This ensures we hit exactly 0.0.
*/

/// The current stage of the envelope state machine.
/// Renamed from "State" to "Stage" to avoid confusion with Rust's state terminology.
#[derive(Debug, Clone, Copy)]
pub enum EnvelopeState {
    Idle,    // Gate low, envelope inactive, level = 0
    Attack,  // Gate just went high, ramping up to 1.0
    Decay,   // Reached peak, ramping down to sustain level
    Sustain, // Holding at sustain level while gate is high
    Release, // Gate went low, ramping down to 0
}

pub struct Envelope {
    // ADSR parameters (set once, define the envelope shape)
    attack_time: f32,  // seconds to ramp 0 → 1
    decay_time: f32,   // seconds to ramp 1 → sustain
    sustain_level: f32, // level to hold (0.0 - 1.0)
    release_time: f32, // seconds to ramp current → 0

    // Runtime state (changes every sample)
    stage: EnvelopeState, // current stage of the state machine
    level: f32,           // current output value (0.0 - 1.0)

    // Decay bookkeeping
    decay_start_level: f32, // level when decay began (usually 1.0)

    // Release bookkeeping (we pre-calculate at note_off for precision)
    release_start_level: f32, // level when release began
    release_total_samples: u32, // total samples for release phase
    release_elapsed_samples: u32, // samples elapsed since release began
}

impl Envelope {
    pub fn new() -> Self {
        Self {
            attack_time: 0.01,   // 10ms default
            decay_time: 0.1,     // 100ms default
            sustain_level: 0.7,  // 70% level default
            release_time: 0.3,   // 300ms default

            stage: EnvelopeState::Idle,
            level: 0.0,
            decay_start_level: 0.0,
            release_start_level: 0.0,
            release_total_samples: 1,
            release_elapsed_samples: 0,
        }
    }

    pub fn adsr(attack: f32, decay: f32, sustain: f32, release: f32) -> Self {
        Self {
            attack_time: attack.max(MIN_TIME),
            decay_time: decay.max(MIN_TIME),
            sustain_level: sustain.clamp(0.0, 1.0),
            release_time: release.max(MIN_TIME),

            stage: EnvelopeState::Idle,
            level: 0.0,
            decay_start_level: 0.0,
            release_start_level: 0.0,
            release_total_samples: 1,
            release_elapsed_samples: 0,
        }
    }

    /// Gate high: start the attack phase from zero.
    ///
    /// This resets the envelope for a clean retrigger - essential for
    /// repeated notes to sound distinct rather than "tied together".
    pub fn note_on(&mut self, _ctx: &RenderCtx) {
        self.level = 0.0; // Reset to zero for clean attack
        self.stage = EnvelopeState::Attack;
        self.release_elapsed_samples = 0;
    }

    /// Gate low: start the release phase from current level.
    pub fn note_off(&mut self, ctx: &RenderCtx) {
        if matches!(self.stage, EnvelopeState::Idle) {
            return;
        }

        // Snapshot current level - we'll interpolate from here to 0
        self.release_start_level = self.level;

        // Pre-calculate total samples for release (avoids division each sample)
        if self.release_time <= MIN_TIME {
            self.release_total_samples = 1;
        } else {
            self.release_total_samples =
                (self.release_time * ctx.sample_rate).round().max(1.0) as u32;
        }

        self.release_elapsed_samples = 0;
        self.stage = EnvelopeState::Release;
    }

    /// Advance the envelope by one sample. Called once per sample.
    pub fn next_sample(&mut self, ctx: &RenderCtx) {
        match self.stage {
            EnvelopeState::Idle => {
                self.level = 0.0;
            }

            EnvelopeState::Attack => {
                // increment = 1.0 / (attack_time * sample_rate)
                // This gives us the per-sample step to reach 1.0 in attack_time seconds
                let increment = 1.0 / (self.attack_time * ctx.sample_rate);
                self.level += increment;

                if self.level >= 1.0 {
                    self.level = 1.0;
                    self.decay_start_level = 1.0;
                    self.stage = EnvelopeState::Decay;
                }
            }

            EnvelopeState::Decay => {
                // Ramp from decay_start_level down to sustain_level
                let target = self.sustain_level;
                let total_drop = self.decay_start_level - target;
                let decrement = total_drop / (self.decay_time * ctx.sample_rate);
                self.level -= decrement;

                if self.level <= target {
                    self.level = target;
                    self.stage = EnvelopeState::Sustain;
                }
            }

            EnvelopeState::Sustain => {
                // Hold at sustain level until gate goes low
                self.level = self.sustain_level;
            }

            EnvelopeState::Release => {
                // Linear interpolation from release_start_level to 0
                // level = start * (1 - elapsed/total)
                let progress = self.release_elapsed_samples as f32
                    / self.release_total_samples as f32;
                self.level = (self.release_start_level * (1.0 - progress)).max(0.0);

                self.release_elapsed_samples = self.release_elapsed_samples.saturating_add(1);

                if self.release_elapsed_samples >= self.release_total_samples {
                    self.level = 0.0;
                    self.stage = EnvelopeState::Idle;
                }
            }
        }

        debug_assert!((0.0..=1.0).contains(&self.level));
    }

    /// Render a block of envelope values into the buffer.
    pub fn render(&mut self, buffer: &mut [f32], ctx: &RenderCtx) {
        for sample in buffer.iter_mut() {
            self.next_sample(ctx);
            *sample = self.level;
        }
    }

    /// Returns true if the envelope is producing output (not idle).
    pub fn is_active(&self) -> bool {
        !matches!(self.stage, EnvelopeState::Idle)
    }

    /// Reset to idle state.
    pub fn reset(&mut self) {
        self.stage = EnvelopeState::Idle;
        self.level = 0.0;
        self.decay_start_level = 0.0;
        self.release_elapsed_samples = 0;
        self.release_start_level = 0.0;
    }

    /// Get the current envelope level (0.0 to 1.0)
    pub fn level(&self) -> f32 {
        self.level
    }

    /// Get the current envelope stage
    pub fn state(&self) -> EnvelopeState {
        self.stage
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::node::RenderCtx;

    const SAMPLE_RATE: f32 = 1_000.0;

    fn render_samples(env: &mut Envelope, samples: usize) {
        let ctx = RenderCtx::from_freq(SAMPLE_RATE, 440.0, 1.0);
        for _ in 0..samples {
            env.next_sample(&ctx);
        }
    }

    #[test]
    fn attack_reaches_full_level() {
        let mut env = Envelope::adsr(0.01, 0.1, 0.7, 0.2);
        let ctx = RenderCtx::from_freq(SAMPLE_RATE, 220.0, 1.0);

        env.note_on(&ctx);
        render_samples(&mut env, (0.01 * SAMPLE_RATE) as usize);

        assert!(env.level() > 0.99, "expected attack to reach full level");
        assert!(!matches!(env.state(), EnvelopeState::Attack));
    }

    #[test]
    fn sustain_holds_target_level() {
        let sustain = 0.6;
        let mut env = Envelope::adsr(0.01, 0.05, sustain, 0.2);
        let ctx = RenderCtx::from_freq(SAMPLE_RATE, 440.0, 1.0);

        env.note_on(&ctx);
        let attack_decay_samples = ((0.01 + 0.05) * SAMPLE_RATE) as usize + 5;
        render_samples(&mut env, attack_decay_samples);

        assert!(matches!(env.state(), EnvelopeState::Sustain));
        assert!((env.level() - sustain).abs() < 0.05, "sustain level should be held");
    }

    #[test]
    fn release_falls_back_to_idle() {
        let release = 0.03;
        let mut env = Envelope::adsr(0.01, 0.05, 0.5, release);
        let ctx = RenderCtx::from_freq(SAMPLE_RATE, 440.0, 1.0);

        env.note_on(&ctx);
        render_samples(&mut env, (0.02 * SAMPLE_RATE) as usize);

        env.note_off(&ctx);
        render_samples(&mut env, (release * SAMPLE_RATE) as usize + 2);

        assert!(env.level() <= 0.001, "release should fall back to zero");
        assert!(matches!(env.state(), EnvelopeState::Idle));
    }
}
