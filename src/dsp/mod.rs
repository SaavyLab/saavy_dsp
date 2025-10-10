pub mod envelope;
pub mod filter;
pub mod oscillator;

use envelope::AdsrEnvelope;
use filter::FilterBlock;
use oscillator::OscillatorBlock;

pub struct VoiceChain {
    pub oscillator: OscillatorBlock,
    pub envelope: AdsrEnvelope,
    pub filter: FilterBlock,
}
