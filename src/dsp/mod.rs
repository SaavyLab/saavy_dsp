pub mod envelope;
pub mod filter;
pub mod oscillator;

use envelope::Envelope;
use filter::FilterBlock;
use oscillator::OscillatorBlock;

pub struct VoiceChain {
    pub oscillator: OscillatorBlock,
    pub envelope: Envelope,
    pub filter: FilterBlock,
}
