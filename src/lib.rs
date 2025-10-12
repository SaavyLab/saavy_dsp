pub mod dsp;
pub mod dsp_fluent;
pub mod engine;
pub mod io;
pub mod patch;
pub mod tooling;

use dsp::oscillator::{OscillatorBlock, OscillatorWaveform};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct EngineConfig {
    pub sample_rate: u32,
    pub block_size: u32,
    pub max_voices: usize,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            sample_rate: 48_000,
            block_size: 512,
            max_voices: 64,
        }
    }
}

pub struct SaavyEngine {
    config: EngineConfig,
    oscillator: OscillatorBlock,
    amplitude: f32,
}

impl SaavyEngine {
    pub fn new(config: EngineConfig) -> Self {
        let oscillator = OscillatorBlock::new(440.0, config.sample_rate as f32, OscillatorWaveform::Sine);
        Self {
            config,
            oscillator,
            amplitude: 0.2,
        }
    }

    pub fn process_block(&mut self, _inputs: &io::AudioInput, _outputs: &mut io::AudioOutput) {
        let frames = self.config.block_size as usize;
        if _outputs.buffers.is_empty() {
            _outputs.buffers = vec![vec![0.0; frames]; 2];
        }

        for channel in &mut _outputs.buffers {
            channel.resize(frames, 0.0);
        }

        self.oscillator.render(&mut _outputs.buffers[0], self.amplitude);

        if _outputs.buffers.len() > 1 {
            let (head, tail) = _outputs.buffers.split_at_mut(1);
            let reference = &head[0];
            for channel in tail {
                channel.copy_from_slice(reference);
            }
        }
    }

    pub fn schedule_event(&mut self, _event: io::midi::MidiEvent) {
        let _ = _event;
    }

    pub fn set_waveform(&mut self, waveform: OscillatorWaveform) {
        self.oscillator.set_waveform(waveform);
    }

    pub fn set_frequency(&mut self, frequency: f32) {
        self.oscillator.set_frequency(frequency);
    }

    pub fn set_amplitude(&mut self, amplitude: f32) {
        self.amplitude = amplitude.clamp(0.0, 1.0);
    }

    pub fn set_block_size(&mut self, block_size: u32) {
        self.config.block_size = block_size;
    }
}
