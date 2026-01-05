//! Runtime for playing patterns with a TUI.
//!
//! This module provides the `Saavy` builder for creating and running
//! synthesizer applications with a terminal UI.
//!
//! # Example
//!
//! ```ignore
//! use saavy_dsp::{runtime::Saavy, pattern, sequencing::*, voices};
//!
//! fn main() -> color_eyre::Result<()> {
//!     Saavy::new()
//!         .bpm(120.0)
//!         .track("lead", pattern!(4/4 => [C4, E4, G4, C5]), voices::lead())
//!         .track("bass", pattern!(4/4 => [C2, _, G2, _]), voices::bass())
//!         .run()
//! }
//! ```

mod app;
mod sequencer;
mod track;
mod ui;

pub use app::{IntoSequence, Saavy};
