pub mod dsp;
pub mod graph; // Composable audio graph nodes
pub mod sequencing; // Musical timing and patterns
pub mod voices; // Pre-built voices (kick, snare, bass, lead)

pub const MAX_BLOCK_SIZE: usize = 2048;
pub const MAX_DELAY_SAMPLES: usize = 192_000; // ~2 seconds at 96kHz, ~4 seconds at 48kHz
pub(crate) const MIN_TIME: f32 = 1.0 / 48_000.0;
