use saavy_dsp::{io::AudioOutput, EngineConfig, SaavyEngine};

#[test]
fn renders_silence_with_empty_scene() {
    let config = EngineConfig::default();
    let mut engine = SaavyEngine::new(config);
    let mut output = AudioOutput::default();
    engine.process_block(&Default::default(), &mut output);
    assert!(output.buffers.iter().all(|channel| channel.iter().all(|sample| sample.abs() < f32::EPSILON)));
}
