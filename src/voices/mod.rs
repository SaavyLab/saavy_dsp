//! Pre-built voices for common sounds.
//!
//! Each voice is a ready-to-use node graph. Use these as starting points
//! for your own sounds, or study them to learn how different timbres are built.
//!
//! # Example
//!
//! ```ignore
//! use saavy_dsp::voices;
//!
//! let kick = voices::kick();
//! let snare = voices::snare();
//! let hihat = voices::hihat();
//! let bass = voices::bass();
//! let lead = voices::lead();
//! ```

mod bass;
mod crash;
mod hihat;
mod kick;
mod lead;
mod ride;
mod snare;

pub use bass::bass;
pub use crash::crash;
pub use hihat::hihat;
pub use kick::kick;
pub use lead::lead;
pub use ride::ride;
pub use snare::snare;
