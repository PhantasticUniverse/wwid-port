use wid_compile::compile;
use wid_eval::{CalculatorParams, calc_z};
use wid_physics::{PhysicalParameters, TemperatureType};
use wid_types::{parse_instrument_xml, parse_tuning_xml};

fn main() {
    let inst_xml = include_str!("../../oracle/v2.6.0/ReedStudy/instruments/SampleChanter.xml");
    let tun_xml = include_str!("../../oracle/v2.6.0/ReedStudy/tunings/A3-ClosedFingering.xml");
    let raw = parse_instrument_xml(inst_xml).unwrap();
    let compiled = compile(&raw).unwrap();
    let tuning = parse_tuning_xml(tun_xml).unwrap();
    let params = PhysicalParameters::new(72.0, TemperatureType::F);

    println!("=== SampleChanter compilation ===");
    println!("Mouthpiece position: {}", compiled.mouthpiece.position);
    println!("Mouthpiece bore_diameter: {}", compiled.mouthpiece.bore_diameter);
    println!("Mouthpiece beta: {}", compiled.mouthpiece.beta);
    println!("Mouthpiece type: {:?}", compiled.mouthpiece.mouthpiece_type);
    println!("Headspace sections: {}", compiled.mouthpiece.headspace.len());
    println!("Components: {}", compiled.components.len());
    for (i, c) in compiled.components.iter().enumerate() {
        match c {
            wid_compile::Component::Bore(b) => println!("  [{i}] Bore len={:.6} lr={:.6} rr={:.6}", b.length, b.left_radius, b.right_radius),
            wid_compile::Component::Hole(h) => println!("  [{i}] Hole pos={:.6} d={:.6} h={:.6} bore_d={:.6}", h.position, h.diameter, h.height, h.bore_diameter),
        }
    }

    let fingering = &tuning.fingerings[0];
    println!("\n=== First fingering: {} @ {} Hz ===", fingering.note.name, fingering.note.frequency.unwrap());
    println!("Open holes: {:?}", fingering.open_holes);

    // Scan Im(Z) from 185 to 210 Hz
    println!("\n=== Im(Z) scan ===");
    let mut prev_im = f64::NAN;
    for i in 0..51 {
        let f = 185.0 + i as f64 * 0.5;
        let z = calc_z(&compiled, f, fingering, &params, &CalculatorParams::REED);
        let crossed = !prev_im.is_nan() && prev_im * z.im < 0.0;
        println!("f={:.1} Hz: Re(Z)={:.8}, Im(Z)={:.8}{}", f, z.re, z.im, if crossed { " *** CROSSING ***" } else { "" });
        prev_im = z.im;
    }
}
