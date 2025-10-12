use saavy_dsp::{io::AudioOutput, EngineConfig, SaavyEngine};

#[test]
fn renders_silence_with_empty_scene() {
    let config = EngineConfig::default();
    let mut engine = SaavyEngine::new(config);
    let mut output = AudioOutput::default();
    engine.process_block(&Default::default(), &mut output);
    let samples: Vec<f32> = output
        .buffers
        .iter()
        .flat_map(|c| c.iter())
        .copied()
        .collect();
    assert!(samples.iter().any(|s| s.abs() > 0.0));
    assert!(samples.iter().all(|s| s.abs() <= 1.0));
}
