use saavy_dsp::{io::AudioOutput, EngineConfig, SaavyEngine};

fn main() {
    let config = EngineConfig::default();
    let mut engine = SaavyEngine::new(config);
    let mut output = AudioOutput::default();
    engine.process_block(&Default::default(), &mut output);
}
