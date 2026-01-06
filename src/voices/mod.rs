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
//! // Drums
//! let kick = voices::kick();
//! let snare = voices::snare();
//! let hihat = voices::hihat();
//! let openhat = voices::openhat();
//! let clap = voices::clap();
//! let tom = voices::tom();
//!
//! // Melodic
//! let bass = voices::bass();
//! let lead = voices::lead();
//! let pad = voices::pad();
//! let pluck = voices::pluck();
//! ```

mod bass;
mod clap;
mod crash;
mod hihat;
mod kick;
mod lead;
mod openhat;
mod pad;
mod pluck;
mod ride;
mod snare;
mod tom;

pub use bass::bass;
pub use clap::clap;
pub use crash::crash;
pub use hihat::hihat;
pub use kick::kick;
pub use lead::lead;
pub use openhat::openhat;
pub use pad::pad;
pub use pluck::pluck;
pub use ride::ride;
pub use snare::snare;
pub use tom::tom;
