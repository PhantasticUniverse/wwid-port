//! Acoustic calculations for woodwind instruments.
//!
//! This crate provides transfer matrix and state vector calculations for
//! the components of a compiled instrument: bore sections, toneholes,
//! termination, and mouthpiece.
//!
//! Each module ports a specific Java calculator class from WIDesigner v2.6.0:
//!
//! - [`tube`] — `Tube.java`: cylinder/cone matrices, radiation impedance
//! - [`bore`] — `SimpleBoreSectionCalculator`: bore section TM
//! - [`hole`] — `DefaultHoleCalculator`: Lefebvre-Scavone 2012 tonehole model
//! - [`termination`] — `ThickFlangedOpenEndCalculator`: open end with flange
//! - [`mouthpiece`] — `DefaultFippleMouthpieceCalculator`: fipple (NAF) mouthpiece

pub mod bore;
pub mod hole;
pub mod mouthpiece;
pub mod termination;
pub mod tube;
