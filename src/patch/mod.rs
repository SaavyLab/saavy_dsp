#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct Patch {
    pub name: String,
    pub description: Option<String>,
    pub voices: Vec<VoiceLayer>,
    pub macros: Vec<MacroControl>,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct VoiceLayer {
    pub oscillator_stack: Vec<OscillatorDescriptor>,
    pub amplitude_envelope: EnvelopeDescriptor,
    pub filter: Option<FilterDescriptor>,
    pub modulation: Vec<ModulationRoute>,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct OscillatorDescriptor {
    pub waveform: super::dsp::oscillator::OscillatorWaveform,
    pub detune_cents: f32,
    pub gain: f32,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct EnvelopeDescriptor {
    pub attack_ms: f32,
    pub decay_ms: f32,
    pub sustain_level: f32,
    pub release_ms: f32,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct FilterDescriptor {
    pub filter_type: super::dsp::filter::FilterType,
    pub cutoff_hz: f32,
    pub resonance: f32,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct ModulationRoute {
    pub source: ModSource,
    pub target: ModTarget,
    pub amount: f32,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub enum ModSource {
    Envelope(String),
    Lfo(String),
    Macro(u8),
    Velocity,
    Aftertouch,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub enum ModTarget {
    OscillatorFrequency { index: usize },
    OscillatorPulseWidth { index: usize },
    FilterCutoff,
    FilterResonance,
    Amplitude,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct MacroControl {
    pub id: u8,
    pub name: String,
    pub min: f32,
    pub max: f32,
    pub default: f32,
}
