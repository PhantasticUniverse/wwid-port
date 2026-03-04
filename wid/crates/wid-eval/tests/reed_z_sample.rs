//! Reed Z-sample parity test.
//!
//! Tests `calc_z()` with `CalculatorParams::REED` against golden Z-samples
//! from the Java oracle (ReedZSampleDriver).
//!
//! Instrument: SampleChanter (double reed), Tuning: A3-ClosedFingering.
//! Validates reed mouthpiece transfer matrix + thick-flanged termination.

use wid_compile::compile;
use wid_eval::{CalculatorParams, calc_z};
use wid_physics::{PhysicalParameters, TemperatureType};
use wid_types::{parse_instrument_xml, parse_tuning_xml};

const INSTRUMENT_XML: &str = include_str!(
    "../../../../oracle/v2.6.0/ReedStudy/instruments/SampleChanter.xml"
);
const TUNING_XML: &str = include_str!(
    "../../../../oracle/v2.6.0/ReedStudy/tunings/A3-ClosedFingering.xml"
);
const ZSAMPLE_JSON: &str = include_str!(
    "../../../../golden/expected/RD-ZSAMPLE/z_samples.json"
);

#[derive(serde::Deserialize, Debug)]
struct ZSample {
    #[allow(dead_code)]
    note: String,
    frequency: f64,
    #[serde(rename = "zReal")]
    z_real: f64,
    #[serde(rename = "zImag")]
    z_imag: f64,
}

#[test]
fn reed_zsample_matches_golden() {
    let raw = parse_instrument_xml(INSTRUMENT_XML).unwrap();
    let inst = compile(&raw).unwrap();
    let tuning = parse_tuning_xml(TUNING_XML).unwrap();
    let params = PhysicalParameters::new(72.0, TemperatureType::F);
    let samples: Vec<ZSample> = serde_json::from_str(ZSAMPLE_JSON).unwrap();

    assert_eq!(
        samples.len(),
        tuning.fingerings.len(),
        "Z-sample count should match fingering count"
    );

    let mut max_re_err: f64 = 0.0;
    let mut max_im_err: f64 = 0.0;

    for (i, sample) in samples.iter().enumerate() {
        let fingering = &tuning.fingerings[i];
        let z = calc_z(&inst, sample.frequency, fingering, &params, &CalculatorParams::REED);

        // Tolerance: abs_err <= A + R * max(|expected|, |actual|)
        let a = 1.0; // absolute tolerance
        let r = 1e-6; // relative tolerance

        let tol_re = a + r * sample.z_real.abs().max(z.re.abs());
        let tol_im = a + r * sample.z_imag.abs().max(z.im.abs());

        let re_err = (z.re - sample.z_real).abs();
        let im_err = (z.im - sample.z_imag).abs();

        if re_err > max_re_err {
            max_re_err = re_err;
        }
        if im_err > max_im_err {
            max_im_err = im_err;
        }

        assert!(
            re_err <= tol_re,
            "Re(Z) mismatch at {}Hz ({}): expected {}, got {}, err {}, tol {}",
            sample.frequency, sample.note, sample.z_real, z.re, re_err, tol_re
        );
        assert!(
            im_err <= tol_im,
            "Im(Z) mismatch at {}Hz ({}): expected {}, got {}, err {}, tol {}",
            sample.frequency, sample.note, sample.z_imag, z.im, im_err, tol_im
        );
    }

    eprintln!(
        "Reed Z-sample parity: {} fingerings, max Re err = {:.6e}, max Im err = {:.6e}",
        samples.len(), max_re_err, max_im_err
    );
}
