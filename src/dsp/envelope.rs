pub struct AdsrEnvelope;

impl AdsrEnvelope {
    pub fn new() -> Self {
        Self
    }

    pub fn apply(&mut self, _buffer: &mut [f32]) {
        todo!("apply ADSR envelope to buffer")
    }
}
