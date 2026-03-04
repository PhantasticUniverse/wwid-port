//! Acoustic calculations for woodwind instruments.
//!
//! This crate provides transfer matrix and state vector calculations for
//! the components of a compiled instrument: bore sections, toneholes,
//! termination, and mouthpiece.
//!
//! - [`tube`] — Lossy cylinder/cone transfer matrices, radiation impedance (Silva 2008)
//! - [`bore`] — Bore section transfer matrix (conical or cylindrical)
//! - [`hole`] — Tonehole T-network model (Lefebvre & Scavone 2012)
//! - [`termination`] — Thick-flanged open end with reflection coefficient
//! - [`mouthpiece`] — Fipple mouthpiece: headspace, fipple factor, window impedance
//! - [`simple_fipple`] — SimpleFipple mouthpiece: empirical window impedance (Whistle/Flute)
//! - [`simple_reed`] — Reed mouthpiece: linear reactance model (single/double/lip reed)

pub mod bore;
pub mod hole;
pub mod mouthpiece;
pub mod simple_fipple;
pub mod simple_reed;
pub mod termination;
pub mod tube;
