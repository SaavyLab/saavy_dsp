pub mod midi;

#[derive(Debug, Default)]
pub struct AudioInput {
    pub buffers: Vec<Vec<f32>>,
}

#[derive(Debug, Default)]
pub struct AudioOutput {
    pub buffers: Vec<Vec<f32>>,
}
