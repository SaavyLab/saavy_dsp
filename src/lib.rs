pub mod dsp;
pub mod graph; // Composable audio graph nodes
pub mod io;
pub mod sequencing; // Musical timing and patterns
pub mod synth; // Voice management and polyphony

pub const MAX_BLOCK_SIZE: usize = 2048;
pub(crate) const MIN_TIME: f32 = 1.0 / 48_000.0;
