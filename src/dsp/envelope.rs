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

pub struct AdsrEnvelope {
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,

    state: EnvelopeState,
    current_level: f32,
    release_samples: u32,
    release_progress: u32,
    release_start: f32,
    sample_rate: f32,
}

impl AdsrEnvelope {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            attack: 0.01, // 10ms
            decay: 0.1,   // 100ms
            sustain: 0.7, // 70% level
            release: 0.3, // 300ms

            state: EnvelopeState::Idle,
            current_level: 0.0,
            release_samples: 1,
            release_progress: 0,
            release_start: 0.0,
            sample_rate
        }
    }

    pub fn with_params(sample_rate: f32, attack: f32, decay: f32, sustain: f32, release: f32) -> Self {
        Self {
            attack,
            decay,
            sustain,
            release,

            state: EnvelopeState::Idle,
            current_level: 0.0,
            release_samples: 1,
            release_progress: 0,
            release_start: 0.0,
            sample_rate
        }
    }

    pub fn note_on(&mut self) {
        self.state = EnvelopeState::Attack;
        self.release_progress = 0;
    } 

    pub fn note_off(&mut self) {
        // Only transition to release if we're not already idle
        if !matches!(self.state, EnvelopeState::Idle)  {
            self.release_start = self.current_level;
            self.release_samples = (self.release * self.sample_rate).round().max(1.0) as u32;
            self.release_progress = 0;
            self.state = EnvelopeState::Release;
        }
    }

    pub fn next_sample(&mut self) -> f32 {
        match self.state {
            EnvelopeState::Idle => self.current_level = 0.0,
            EnvelopeState::Attack => {
                // Ramp up from 0.0 to 1.0 over attack time
                let increment = 1.0 / (self.attack * self.sample_rate);
                self.current_level += increment;

                if self.current_level >= 1.0 {
                    self.current_level = 1.0;
                    self.state = EnvelopeState::Decay;
                }
            },
            EnvelopeState::Decay => {
                // Ramp down from 1.0 to 0.0 over decay time
                let target = self.sustain;
                let decrement = (1.0 - target) / (self.decay * self.sample_rate);
                self.current_level -= decrement;

                if self.current_level <= target {
                    self.current_level = target;
                    self.state = EnvelopeState::Sustain;
                }
            },
            EnvelopeState::Sustain => {
                // Hold at sustain level until note_off
                self.current_level = self.sustain;
            },
            EnvelopeState::Release => {
                if self.release == 0.0 {
                    self.current_level = 0.0;
                    self.state = EnvelopeState::Idle;
                } else {
                    let release_samples = self.release_samples.max(1);
                    let progress = self.release_progress.min(release_samples);
                    let decrement = if release_samples == 0 { self.release_start } else { self.release_start / release_samples as f32 };
                    self.current_level = (self.release_start - decrement * progress as f32).max(0.0);
                    self.release_progress = self.release_progress.saturating_add(1);

                    if self.release_progress >= release_samples {
                        self.current_level = 0.0;
                        self.state = EnvelopeState::Idle;
                    }
                }
            }
        }

        self.current_level.max(0.0).min(1.0)
    }

    pub fn apply(&mut self, buffer: &mut [f32]) {
        for sample in buffer.iter_mut() {
            *sample *= self.next_sample();
        }
    }
    
    pub fn render(&mut self, buffer: &mut [f32]) {
        for sample in buffer.iter_mut() {
            *sample = self.next_sample();
        }
    }

    pub fn is_active(&self) -> bool {
        !matches!(self.state, EnvelopeState::Idle)
    }

    pub fn reset(&mut self) {
        self.state = EnvelopeState::Idle;
        self.current_level = 0.0;
        self.release_progress = 0;
        self.release_start = 0.0;
    }
}
