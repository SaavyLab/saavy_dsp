use saavy_dsp::{io::AudioOutput, EngineConfig, SaavyEngine};

fn main() {
    let config = EngineConfig::default();
    let mut engine = SaavyEngine::new(config);
    engine.set_frequency(220.0);
    let mut output = AudioOutput::default();
    engine.process_block(&Default::default(), &mut output);

    println!("Rendered {} samples", output.buffers[0].len());
}
