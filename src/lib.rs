pub mod dsp;
pub mod engine;
pub mod io;
pub mod patch;
pub mod tooling;

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
}

impl SaavyEngine {
    pub fn new(config: EngineConfig) -> Self {
        Self { config }
    }

    pub fn process_block(&mut self, _inputs: &io::AudioInput, _outputs: &mut io::AudioOutput) {
        let frames = self.config.block_size as usize;
        if _outputs.buffers.is_empty() {
            _outputs.buffers = vec![vec![0.0; frames]; 2];
        } else {
            for channel in &mut _outputs.buffers {
                channel.resize(frames, 0.0);
                for sample in channel.iter_mut() {
                    *sample = 0.0;
                }
            }
        }
    }

    pub fn schedule_event(&mut self, _event: io::midi::MidiEvent) {
        let _ = _event;
    }
}
