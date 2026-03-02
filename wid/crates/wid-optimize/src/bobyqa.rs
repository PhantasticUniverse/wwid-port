//! Re-export of the standalone `bobyqa` crate.
//!
//! BOBYQA lives in its own crate (`crates/bobyqa/`) for potential
//! open-source release as a standalone Rust library.

pub use bobyqa::{BobyqaProgress, BobyqaResult, bobyqa_minimize, bobyqa_minimize_with_callback};
