package com.widgolden;

import com.wwidesigner.geometry.Instrument;

/// Applies programmatic mutations to an Instrument for testing null handling
/// and edge cases. Used by NAF-FF-01 scenarios to test fipple factor and
/// windway height null behavior.
public class InstrumentOverrides {

    /// Set the fipple factor on the instrument's mouthpiece.
    /// Pass null to test null-fipple-factor behavior.
    public static void setFippleFactor(Instrument instrument, Double value) {
        instrument.getMouthpiece().getFipple().setFippleFactor(value);
    }

    /// Set the windway height on the instrument's mouthpiece.
    /// Pass null to test null-windway-height behavior.
    public static void setWindwayHeight(Instrument instrument, Double value) {
        instrument.getMouthpiece().getFipple().setWindwayHeight(value);
    }
}
