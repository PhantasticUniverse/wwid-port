//! Per-study-model acoustic parameters that flow through evaluation.
//!
//! These constants differ between study models (NAF, Whistle, Flute, Reed)
//! and affect hole sizing, finger position adjustments, end termination,
//! mouthpiece model, and tuning strategy.

/// Which mouthpiece impedance model to use.
///
/// - `DefaultFipple` — transfer matrix model used by NAF
///   (upstream: `DefaultFippleMouthpieceCalculator`)
/// - `SimpleFipple` — empirical window impedance model used by Whistle
///   (upstream: `SimpleFippleMouthpieceCalculator`)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouthpieceModel {
    /// NAF: headspace compliance + fipple factor transfer matrix.
    DefaultFipple,
    /// Whistle: empirical Z_window with headspace transmission in parallel.
    SimpleFipple,
}

/// Acoustic calculator parameters that vary by study model.
#[derive(Debug, Clone, Copy)]
pub struct CalculatorParams {
    /// Hole size multiplier applied to tone hole calculations.
    /// NAF: 0.9605 (reduced hole size), others: 1.0.
    pub hole_size_mult: f64,

    /// Finger position adjustment for closed-hole calculations.
    /// NAF: 0.0, Whistle/Flute: 0.010.
    pub finger_adjustment: f64,

    /// Whether to use unflanged end termination.
    /// NAF: false (thick flanged), Whistle/Flute: true (unflanged).
    pub unflanged_end: bool,

    /// Which mouthpiece impedance model to use.
    pub mouthpiece_model: MouthpieceModel,

    /// Blowing level (0–10) for LinearV tuner velocity interpolation.
    /// Only used with `SimpleFipple` mouthpiece model. Default: 5.
    pub blowing_level: u8,
}

impl CalculatorParams {
    /// NAF study model parameters.
    ///
    /// - `hole_size_mult`: 0.9605 (from `NAFCalculator` → `DefaultHoleCalculator`)
    /// - `finger_adjustment`: 0.0
    /// - `unflanged_end`: false (uses `ThickFlangedOpenEnd`)
    /// - `mouthpiece_model`: DefaultFipple (`DefaultFippleMouthpieceCalculator`)
    /// - `blowing_level`: 5 (unused for NAF)
    pub const NAF: Self = Self {
        hole_size_mult: 0.9605,
        finger_adjustment: 0.0,
        unflanged_end: false,
        mouthpiece_model: MouthpieceModel::DefaultFipple,
        blowing_level: 5,
    };

    /// Whistle study model parameters.
    ///
    /// - `hole_size_mult`: 1.0 (from `WhistleCalculator` → `DefaultHoleCalculator` with no scale)
    /// - `finger_adjustment`: 0.010 (from `WhistleCalculator` constructor's `DefaultHoleCalculator`)
    /// - `unflanged_end`: true (uses `UnflangedEndCalculator`)
    /// - `mouthpiece_model`: SimpleFipple (`SimpleFippleMouthpieceCalculator`)
    /// - `blowing_level`: 5 (from `LinearVInstrumentTuner` default constructor)
    pub const WHISTLE: Self = Self {
        hole_size_mult: 1.0,
        finger_adjustment: 0.010,
        unflanged_end: true,
        mouthpiece_model: MouthpieceModel::SimpleFipple,
        blowing_level: 5,
    };

    /// Flute study model parameters.
    ///
    /// Identical to Whistle — `FluteStudyModel extends WhistleStudyModel`.
    /// The mouthpiece difference (EmbouchureHole vs Fipple) is handled
    /// internally by `calc_z_window` parameter extraction.
    pub const FLUTE: Self = Self {
        hole_size_mult: 1.0,
        finger_adjustment: 0.010,
        unflanged_end: true,
        mouthpiece_model: MouthpieceModel::SimpleFipple,
        blowing_level: 5,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn naf_params_match_old_constants() {
        assert_eq!(CalculatorParams::NAF.hole_size_mult, 0.9605);
        assert_eq!(CalculatorParams::NAF.finger_adjustment, 0.0);
        assert!(!CalculatorParams::NAF.unflanged_end);
        assert_eq!(CalculatorParams::NAF.mouthpiece_model, MouthpieceModel::DefaultFipple);
    }

    #[test]
    fn whistle_params() {
        assert_eq!(CalculatorParams::WHISTLE.hole_size_mult, 1.0);
        assert_eq!(CalculatorParams::WHISTLE.finger_adjustment, 0.010);
        assert!(CalculatorParams::WHISTLE.unflanged_end);
        assert_eq!(CalculatorParams::WHISTLE.mouthpiece_model, MouthpieceModel::SimpleFipple);
        assert_eq!(CalculatorParams::WHISTLE.blowing_level, 5);
    }
}
