use crate::{graph::node::RenderCtx, MIN_TIME};

/*
Level
  1.0 ┐     ╱╲________
      │    ╱  ╲       ╲
  0.7 │   ╱    ╲_______╲___
      │  ╱              ╲  ╲
  0.0 └─╱────────────────╲──╲─→ Time
      Attack Decay Sustain Release
       (A)   (D)    (S)     (R)

Attack:  Ramp from 0 → 1             (time in seconds)
Decay:   Ramp from 1 → sustain level (time in seconds)
Sustain: Hold at level               (0.0 → 1.0)
Release: Ramp from current → 0       (time in seconds)

If attack = 0.1s and sample_rate = 48000:
increment = 1.0 / (0.1 * 48000) = 1.0 / 4800 ≈ 0.000208 per sample
After 4800 samples (0.1s), level reaches 1.0
*/

#[derive(Debug, Clone, Copy)]
pub enum EnvelopeState {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

pub struct Envelope {
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,

    state: EnvelopeState,
    current_level: f32,
    decay_start: f32,
    release_samples: u32,
    release_progress: u32,
    release_start: f32,
    release_step: f32,
}

impl Envelope {
    pub fn new() -> Self {
        Self {
            attack: 0.01, // 10ms
            decay: 0.1,   // 100ms
            sustain: 0.7, // 70% level
            release: 0.3, // 300ms

            state: EnvelopeState::Idle,
            current_level: 0.0,
            decay_start: 0.0,
            release_samples: 1,
            release_progress: 0,
            release_step: 0.0,
            release_start: 0.0,
        }
    }

    pub fn adsr(attack: f32, decay: f32, sustain: f32, release: f32) -> Self {
        Self {
            attack: attack.max(MIN_TIME),
            decay: decay.max(MIN_TIME),
            sustain: sustain.clamp(0.0, 1.0),
            release: release.max(MIN_TIME),

            state: EnvelopeState::Idle,
            decay_start: 0.0,
            current_level: 0.0,
            release_samples: 1,
            release_progress: 0,
            release_step: 0.0,
            release_start: 0.0,
        }
    }

    pub fn note_on(&mut self, _ctx: &RenderCtx) {
        self.state = EnvelopeState::Attack;
        self.release_progress = 0;
    }

    pub fn note_off(&mut self, ctx: &RenderCtx) {
        if matches!(self.state, EnvelopeState::Idle) {
            return;
        }

        self.release_start = self.current_level;
        if self.release <= MIN_TIME {
            self.release_samples = 1;
            self.release_step = self.release_start;
        } else {
            self.release_samples = (self.release * ctx.sample_rate).round().max(1.0) as u32;
            self.release_step = self.release_start / self.release_samples as f32;
        }

        self.release_progress = 0;
        self.state = EnvelopeState::Release;
    }

    pub fn next_sample(&mut self, ctx: &RenderCtx) {
        match self.state {
            EnvelopeState::Idle => self.current_level = 0.0,
            EnvelopeState::Attack => {
                // Ramp up from 0.0 to 1.0 over attack time
                let increment = 1.0 / (self.attack * ctx.sample_rate);
                self.current_level += increment;

                if self.current_level >= 1.0 {
                    self.current_level = 1.0;
                    self.decay_start = self.current_level.max(self.sustain);
                    self.state = EnvelopeState::Decay;
                }
            }
            EnvelopeState::Decay => {
                // Ramp down from 1.0 to 0.0 over decay time
                let target = self.sustain;
                let decrement = (self.decay_start - target) / (self.decay * ctx.sample_rate);
                self.current_level -= decrement;

                if self.current_level <= target {
                    self.current_level = target;
                    self.state = EnvelopeState::Sustain;
                }
            }
            EnvelopeState::Sustain => {
                // Hold at sustain level until note_off
                self.current_level = self.sustain;
            }
            EnvelopeState::Release => {
                self.current_level = (self.release_start
                    - self.release_step * self.release_progress as f32)
                    .max(0.0);
                self.release_progress = self.release_progress.saturating_add(1);

                if self.release_progress >= self.release_samples {
                    self.current_level = 0.0;
                    self.state = EnvelopeState::Idle;
                }
            }
        }

        debug_assert!((0.0..=1.0).contains(&self.current_level));
    }

    pub fn render(&mut self, buffer: &mut [f32], ctx: &RenderCtx) {
        for sample in buffer.iter_mut() {
            self.next_sample(ctx);
            *sample = self.current_level;
        }
    }

    pub fn is_active(&self) -> bool {
        !matches!(self.state, EnvelopeState::Idle)
    }

    pub fn reset(&mut self) {
        self.state = EnvelopeState::Idle;
        self.current_level = 0.0;
        self.decay_start = 0.0;
        self.release_progress = 0;
        self.release_start = 0.0;
    }

    /// Get the current envelope level (0.0 to 1.0)
    pub fn level(&self) -> f32 {
        self.current_level
    }

    /// Get the current envelope state
    pub fn state(&self) -> EnvelopeState {
        self.state
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
