//! Air properties and acoustic wave parameters.
//!
//! [`PhysicalParameters`] implements the CIPM-2007 model for moist air
//! (density, viscosity, thermal conductivity, speed of sound) and derives
//! the wave number, alpha constant, and bore impedance needed for acoustic
//! modelling.
//!
//! [`SimplePhysicalParameters`] is a simplified model with polynomial
//! approximations, used internally by the fipple mouthpiece calculator.

use num_complex::Complex64;
use std::f64::consts::PI;

// ── Universal constants ──────────────────────────────────────────

const R: f64 = 8.314472; // Universal gas constant, J/(mol·K)
const MA0: f64 = 28.960745; // Standard molar mass of CO2-free dry air, kg/kmol
const MCO2: f64 = 44.0100; // Molar mass of CO2, kg/kmol
const MO2: f64 = 31.9988; // Molar mass of O2, kg/kmol
const MV: f64 = 18.01527; // Molar mass of water vapour, kg/kmol

/// Temperature unit for constructing [`PhysicalParameters`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemperatureType {
    /// Celsius
    C,
    /// Fahrenheit
    F,
}

/// Convert a temperature to Celsius.
fn to_celsius(temperature: f64, temp_type: TemperatureType) -> f64 {
    match temp_type {
        TemperatureType::C => temperature,
        TemperatureType::F => (temperature + 40.0) * 5.0 / 9.0 - 40.0,
    }
}

/// CIPM-2007 air property model.
///
/// Computes speed of sound, density, viscosity, thermal conductivity,
/// specific heat ratio, and derived acoustic parameters from temperature,
/// pressure, humidity, and CO2 concentration.
///
/// Default conditions: 72 °F (~22.2 °C), 101.325 kPa, 45% RH, 390 ppm CO2.
#[derive(Debug, Clone)]
pub struct PhysicalParameters {
    temperature: f64,     // °C
    pressure: f64,        // kPa
    humidity: f64,        // % of saturation
    x_co2: f64,           // mol/mol
    x_v: f64,             // mol/mol (water vapour molar fraction)
    rho: f64,             // kg/m³
    eta: f64,             // Pa·s (dynamic viscosity)
    specific_heat: f64,   // J/(kg·K)
    gamma: f64,           // cp/cv
    kappa: f64,           // W/(m·K) (thermal conductivity)
    prandtl: f64,         // dimensionless
    speed_of_sound: f64,  // m/s
    epsilon_constant: f64, // loss factor constant
    alpha_constant: f64,  // attenuation constant
    wave_number_1: f64,   // 2π/c (wave number at 1 Hz)
}

impl Default for PhysicalParameters {
    fn default() -> Self {
        Self::new(72.0, TemperatureType::F)
    }
}

impl PhysicalParameters {
    /// Create with default pressure (101.325 kPa), humidity (45%), and CO2 (390 ppm).
    pub fn new(temperature: f64, temp_type: TemperatureType) -> Self {
        Self::with_all(temperature, temp_type, 101.325, 45.0, 0.000390)
    }

    /// Create with full control over all environmental parameters.
    pub fn with_all(
        temperature: f64,
        temp_type: TemperatureType,
        pressure_kpa: f64,
        rel_humidity_pct: f64,
        x_co2: f64,
    ) -> Self {
        let celsius = to_celsius(temperature, temp_type);
        let mut params = Self {
            temperature: celsius,
            pressure: pressure_kpa,
            humidity: rel_humidity_pct,
            x_co2,
            x_v: 0.0,
            rho: 0.0,
            eta: 0.0,
            specific_heat: 0.0,
            gamma: 0.0,
            kappa: 0.0,
            prandtl: 0.0,
            speed_of_sound: 0.0,
            epsilon_constant: 0.0,
            alpha_constant: 0.0,
            wave_number_1: 0.0,
        };
        params.set_properties();
        params
    }

    /// Compute all derived properties from stored temperature/pressure/humidity/CO2.
    fn set_properties(&mut self) {
        let kelvin = 273.15 + self.temperature;
        let pascal = 1000.0 * self.pressure;

        // Enhancement factor (CIPM-2007)
        let enhancement = 1.00062 + 3.14e-5 * self.pressure
            + 5.6e-7 * self.temperature * self.temperature;

        // Saturated vapour pressure in kPa (CIPM-2007)
        let psv = 0.001
            * (1.2378847e-5 * kelvin * kelvin - 1.9121316e-2 * kelvin + 33.93711047
                - 6.3431645e3 / kelvin)
                .exp();

        // Molar fraction of water vapour (CIPM-2007)
        self.x_v = 0.01 * self.humidity * enhancement * psv / self.pressure;

        // Compressibility factor (CIPM-2007)
        let t = self.temperature;
        let xv = self.x_v;
        let compressibility = 1.0
            - pascal / kelvin
                * (1.58123e-6 - 2.9331e-8 * t + 1.1043e-10 * t * t
                    + (5.707e-6 - 2.051e-8 * t) * xv
                    + (1.9898e-4 - 2.376e-6 * t) * xv * xv)
            + (pascal / kelvin) * (pascal / kelvin) * (1.83e-11 - 0.765e-8 * xv * xv);

        // Molar masses
        let ma = MA0 + (MCO2 - MO2) * self.x_co2; // dry air
        let m = (1.0 - xv) * ma + xv * MV; // moist air

        // Specific gas constant of humid air, J/(kg·K)
        let ra = R / (0.001 * m);

        // Mass fractions
        let qv = xv * MV / m; // water
        let qco2 = self.x_co2 * MCO2 / m; // CO2

        // Air density
        self.rho = self.pressure * 1e3 / (compressibility * ra * kelvin);

        // Dynamic viscosity (Sutherland + Tsilingiris mixing)
        let eta_air = 1.4592e-6 * kelvin.powf(1.5) / (kelvin + 109.10);
        let eta_vapour = 8.058131868e-6 + self.temperature * 4.000549451e-8;
        let eta_ratio = (eta_air / eta_vapour).sqrt();
        let humidity_ratio = xv / (1.0 - xv);
        let phi_av = 0.5 * (1.0 + eta_ratio * (MV / ma).powf(0.25)).powi(2)
            / (2.0 * (1.0 + ma / MV)).sqrt();
        let phi_va = 0.5 * (1.0 + (ma / MV).powf(0.25) / eta_ratio).powi(2)
            / (2.0 * (1.0 + MV / ma)).sqrt();
        self.eta = eta_air / (1.0 + phi_av * humidity_ratio)
            + humidity_ratio * eta_vapour / (humidity_ratio + phi_va);

        // Isobaric specific heat (Tsilingiris, with air reduced by 2 J/kg·K)
        let cp_air = 1032.0
            + kelvin
                * (-0.284887
                    + kelvin * (0.7816818e-3 + kelvin * (-0.4970786e-6 + kelvin * 0.1077024e-9)));
        let cp_vapour =
            1869.10989 + self.temperature * (-0.2578421578 + self.temperature * 1.941058941e-2);
        let cp_co2 = 817.02 + self.temperature * (1.0562 - self.temperature * 6.67e-4);
        self.specific_heat = cp_air * (1.0 - qv - qco2) + cp_vapour * qv + cp_co2 * qco2;

        // Specific heat ratio
        self.gamma = self.specific_heat / (self.specific_heat - ra);

        // Thermal conductivity (Sutherland + Tsilingiris mixing)
        let kappa_air = 2.3340e-3 * kelvin.powf(1.5) / (kelvin + 164.54);
        let kappa_vapour = 0.01761758242
            + self.temperature * (5.558941059e-5 + self.temperature * 1.663336663e-7);
        self.kappa = kappa_air / (1.0 + phi_av * humidity_ratio)
            + humidity_ratio * kappa_vapour / (humidity_ratio + phi_va);

        // Prandtl number
        self.prandtl = self.eta * self.specific_heat / self.kappa;

        // Speed of sound
        self.speed_of_sound = (self.gamma * compressibility * ra * kelvin).sqrt();

        // Loss constants
        self.epsilon_constant = 1.0 / (2.0 * PI.sqrt()) * (self.eta / self.rho).sqrt()
            * (1.0 + (self.gamma - 1.0) / self.prandtl.sqrt());
        self.alpha_constant =
            (self.eta / (2.0 * self.rho * self.speed_of_sound)).sqrt()
                * (1.0 + (self.gamma - 1.0) / self.prandtl.sqrt());
        self.wave_number_1 = 2.0 * PI / self.speed_of_sound;
    }

    // ── Getters ──────────────────────────────────────────────────

    pub fn temperature(&self) -> f64 {
        self.temperature
    }
    pub fn pressure(&self) -> f64 {
        self.pressure
    }
    pub fn humidity(&self) -> f64 {
        self.humidity
    }
    pub fn x_co2(&self) -> f64 {
        self.x_co2
    }
    pub fn epsilon_constant(&self) -> f64 {
        self.epsilon_constant
    }
    pub fn speed_of_sound(&self) -> f64 {
        self.speed_of_sound
    }
    pub fn rho(&self) -> f64 {
        self.rho
    }
    pub fn eta(&self) -> f64 {
        self.eta
    }
    pub fn gamma(&self) -> f64 {
        self.gamma
    }
    pub fn specific_heat(&self) -> f64 {
        self.specific_heat
    }
    pub fn alpha_constant(&self) -> f64 {
        self.alpha_constant
    }

    // ── Derived acoustic quantities ──────────────────────────────

    /// Wave number (real, lossless) for a given frequency.
    pub fn calc_wave_number(&self, freq: f64) -> f64 {
        freq * self.wave_number_1
    }

    /// Frequency from wave number.
    pub fn calc_frequency(&self, wave_number: f64) -> f64 {
        wave_number / self.wave_number_1
    }

    /// Wave impedance of a bore of nominal radius `r`, in kg/(m^4·s).
    pub fn calc_z0(&self, radius: f64) -> f64 {
        self.rho * self.speed_of_sound / (PI * radius * radius)
    }

    /// Epsilon: dimensionless loss adjustment from wave number and tube radius.
    pub fn get_epsilon(&self, wave_number: f64, radius: f64) -> f64 {
        self.alpha_constant / (radius * wave_number.sqrt())
    }

    /// Epsilon from frequency (rather than wave number).
    pub fn get_epsilon_from_f(&self, frequency: f64, radius: f64) -> f64 {
        self.epsilon_constant / (radius * frequency.sqrt())
    }

    /// Complex wave number with viscothermal losses.
    ///
    /// Returns `j*k + (1+j)*alpha` where `alpha = alpha_constant * sqrt(k) / r`.
    pub fn get_complex_wave_number(&self, wave_number: f64, radius: f64) -> Complex64 {
        let alpha = (1.0 / radius) * wave_number.sqrt() * self.alpha_constant;
        Complex64::new(0.0, 1.0) * wave_number + Complex64::new(1.0, 1.0) * alpha
    }
}

// ── SimplePhysicalParameters ────────────────────────────────────

/// Simplified air property model with polynomial approximations.
///
/// Used exclusively by the fipple mouthpiece calculator. Properties are
/// linearized around a reference temperature of 26.85 °C with fixed
/// relative humidity of 0.45 and pressure of 101 kPa.
#[derive(Debug, Clone)]
pub struct SimplePhysicalParameters {
    temperature: f64,    // °C
    rho: f64,            // kg/m³
    eta: f64,            // Pa·s
    mu: f64,             // dynamic viscosity variant
    gamma: f64,          // cp/cv
    nu: f64,             // Prandtl number
    specific_heat: f64,  // J/(kg·K)
    speed_of_sound: f64, // m/s
    wave_number_1: f64,  // 2π/c
    alpha_constant: f64,
}

/// Fixed relative humidity used by SimplePhysicalParameters.
const SIMPLE_RELATIVE_HUMIDITY: f64 = 0.45;

impl Default for SimplePhysicalParameters {
    fn default() -> Self {
        Self::new(72.0, TemperatureType::F)
    }
}

impl SimplePhysicalParameters {
    /// Create from temperature and unit.
    pub fn new(temperature: f64, temp_type: TemperatureType) -> Self {
        let celsius = to_celsius(temperature, temp_type);
        let speed_of_sound = calculate_speed_of_sound(celsius, SIMPLE_RELATIVE_HUMIDITY);

        let kelvin = 273.15 + celsius;
        let eta = 3.648e-6 * (1.0 + 0.0135003 * kelvin);

        let delta_t = celsius - 26.85;
        let rho = 1.1769 * (1.0 - 0.00335 * delta_t);
        let mu = 1.8460e-5 * (1.0 + 0.00250 * delta_t);
        let gamma = 1.4017 * (1.0 - 0.00002 * delta_t);
        let nu = 0.8410 * (1.0 - 0.00020 * delta_t);

        let wave_number_1 = 2.0 * PI / speed_of_sound;
        let alpha_constant =
            (mu / (2.0 * rho * speed_of_sound)).sqrt() * (1.0 + (gamma - 1.0) / nu);

        Self {
            temperature: celsius,
            rho,
            eta,
            mu,
            gamma,
            nu,
            specific_heat: 0.0, // not computed in upstream
            speed_of_sound,
            wave_number_1,
            alpha_constant,
        }
    }

    /// Create from a full [`PhysicalParameters`] (uses only its temperature).
    pub fn from_physical(params: &PhysicalParameters) -> Self {
        Self::new(params.temperature(), TemperatureType::C)
    }

    pub fn temperature(&self) -> f64 {
        self.temperature
    }
    pub fn speed_of_sound(&self) -> f64 {
        self.speed_of_sound
    }
    pub fn rho(&self) -> f64 {
        self.rho
    }
    pub fn eta(&self) -> f64 {
        self.eta
    }
    pub fn gamma(&self) -> f64 {
        self.gamma
    }
    pub fn alpha_constant(&self) -> f64 {
        self.alpha_constant
    }

    /// Wave number (real, lossless) for a given frequency.
    pub fn calc_wave_number(&self, freq: f64) -> f64 {
        freq * self.wave_number_1
    }

    /// Frequency from wave number.
    pub fn calc_frequency(&self, wave_number: f64) -> f64 {
        wave_number / self.wave_number_1
    }

    /// Wave impedance of a bore of nominal radius `r`.
    pub fn calc_z0(&self, radius: f64) -> f64 {
        self.rho * self.speed_of_sound / (PI * radius * radius)
    }
}

/// Owen Cramer polynomial speed-of-sound approximation (JASA 1993).
///
/// Hardcoded for pressure = 101 kPa. Uses Yang Yili's formulation.
fn calculate_speed_of_sound(ambient_temp_c: f64, relative_humidity: f64) -> f64 {
    let p: f64 = 101000.0;
    let a: [f64; 16] = [
        331.5024, 0.603055, -0.000528, 51.471935, 0.1495874, -0.000782, -1.82e-7, 3.73e-8,
        -2.93e-10, -85.20931, -0.228525, 5.91e-5, -2.835149, -2.15e-13, 29.179762, 0.000486,
    ];

    let t_k = ambient_temp_c + 273.15;
    let f = 1.00062 + 0.0000000314 * p + 0.00000056 * ambient_temp_c * ambient_temp_c;
    let psv =
        (0.000012811805 * t_k * t_k - 0.019509874 * t_k + 34.04926034 - 6353.6311 / t_k).exp();
    let xw = relative_humidity * f * psv / p;
    let c_inner = 331.45 - a[0] - p * a[6] - a[13] * p * p;
    let c_sqrt = (a[9] * a[9] + 4.0 * a[14] * c_inner).sqrt();
    let xc = (-a[9] - c_sqrt) / (2.0 * a[14]);

    let t = ambient_temp_c;
    a[0] + a[1] * t
        + a[2] * t * t
        + (a[3] + a[4] * t + a[5] * t * t) * xw
        + (a[6] + a[7] * t + a[8] * t * t) * p
        + (a[9] + a[10] * t + a[11] * t * t) * xc
        + a[12] * xw * xw
        + a[13] * p * p
        + a[14] * xc * xc
        + a[15] * xw * p * xc
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;

    // Golden reference values from Java oracle at 72°F, 101.325 kPa, 45% RH, 390 ppm CO2
    const GOLDEN_SPEED_OF_SOUND: f64 = 345.16416347142905;
    const GOLDEN_RHO: f64 = 1.190098732335034;
    const GOLDEN_ETA: f64 = 1.8187599550799636e-5;
    const GOLDEN_GAMMA: f64 = 1.3993223735405296;
    const GOLDEN_SPECIFIC_HEAT: f64 = 1010.4534527147047;
    const GOLDEN_ALPHA_CONSTANT: f64 = 2.1900771817689785e-4;
    const GOLDEN_TEMP_C: f64 = 22.22222222222222;
    const GOLDEN_WAVE_NUMBER_440: f64 = 8.009526560795058;
    const GOLDEN_Z0_BORE: f64 = 1441210.5846022002;
    const GOLDEN_BORE_RADIUS: f64 = 0.00952501543050833;

    #[test]
    fn default_temperature_is_72f() {
        let p = PhysicalParameters::default();
        assert_abs_diff_eq!(p.temperature(), GOLDEN_TEMP_C, epsilon = 1e-10);
    }

    #[test]
    fn speed_of_sound_matches_java() {
        let p = PhysicalParameters::default();
        assert_abs_diff_eq!(p.speed_of_sound(), GOLDEN_SPEED_OF_SOUND, epsilon = 1e-8);
    }

    #[test]
    fn rho_matches_java() {
        let p = PhysicalParameters::default();
        assert_abs_diff_eq!(p.rho(), GOLDEN_RHO, epsilon = 1e-10);
    }

    #[test]
    fn eta_matches_java() {
        let p = PhysicalParameters::default();
        assert_abs_diff_eq!(p.eta(), GOLDEN_ETA, epsilon = 1e-16);
    }

    #[test]
    fn gamma_matches_java() {
        let p = PhysicalParameters::default();
        assert_abs_diff_eq!(p.gamma(), GOLDEN_GAMMA, epsilon = 1e-10);
    }

    #[test]
    fn specific_heat_matches_java() {
        let p = PhysicalParameters::default();
        assert_abs_diff_eq!(p.specific_heat(), GOLDEN_SPECIFIC_HEAT, epsilon = 1e-7);
    }

    #[test]
    fn alpha_constant_matches_java() {
        let p = PhysicalParameters::default();
        assert_abs_diff_eq!(p.alpha_constant(), GOLDEN_ALPHA_CONSTANT, epsilon = 1e-14);
    }

    #[test]
    fn wave_number_at_440_matches_java() {
        let p = PhysicalParameters::default();
        let wn = p.calc_wave_number(440.0);
        assert_abs_diff_eq!(wn, GOLDEN_WAVE_NUMBER_440, epsilon = 1e-10);
    }

    #[test]
    fn z0_at_bore_radius_matches_java() {
        let p = PhysicalParameters::default();
        let z0 = p.calc_z0(GOLDEN_BORE_RADIUS);
        assert_abs_diff_eq!(z0, GOLDEN_Z0_BORE, epsilon = 1.0);
    }

    #[test]
    fn frequency_round_trips_wave_number() {
        let p = PhysicalParameters::default();
        let freq = 440.0;
        let wn = p.calc_wave_number(freq);
        let recovered = p.calc_frequency(wn);
        assert_abs_diff_eq!(recovered, freq, epsilon = 1e-10);
    }

    #[test]
    fn complex_wave_number_has_positive_imaginary() {
        let p = PhysicalParameters::default();
        let wn = p.calc_wave_number(440.0);
        let cwn = p.get_complex_wave_number(wn, 0.01);
        // Real part = alpha (positive loss)
        assert!(cwn.re > 0.0);
        // Imaginary part = wave_number (positive propagation)
        assert!(cwn.im > 0.0);
    }

    #[test]
    fn epsilon_decreases_with_larger_radius() {
        let p = PhysicalParameters::default();
        let wn = p.calc_wave_number(440.0);
        let eps_small = p.get_epsilon(wn, 0.005);
        let eps_large = p.get_epsilon(wn, 0.010);
        assert!(eps_small > eps_large);
    }

    // ── SimplePhysicalParameters tests ──────────────────────────

    #[test]
    fn simple_default_is_72f() {
        let sp = SimplePhysicalParameters::default();
        assert_abs_diff_eq!(sp.temperature(), GOLDEN_TEMP_C, epsilon = 1e-10);
    }

    #[test]
    fn simple_speed_of_sound_reasonable() {
        let sp = SimplePhysicalParameters::default();
        // Should be close to the full model but not identical (different formula)
        assert!((sp.speed_of_sound() - GOLDEN_SPEED_OF_SOUND).abs() < 1.0);
    }

    #[test]
    fn simple_rho_reasonable() {
        let sp = SimplePhysicalParameters::default();
        // Within 1% of the CIPM model
        assert!((sp.rho() - GOLDEN_RHO).abs() / GOLDEN_RHO < 0.01);
    }

    #[test]
    fn simple_from_physical_matches_direct() {
        let pp = PhysicalParameters::default();
        let sp_from = SimplePhysicalParameters::from_physical(&pp);
        let sp_direct = SimplePhysicalParameters::new(72.0, TemperatureType::F);
        assert_abs_diff_eq!(sp_from.speed_of_sound(), sp_direct.speed_of_sound(), epsilon = 1e-10);
        assert_abs_diff_eq!(sp_from.rho(), sp_direct.rho(), epsilon = 1e-10);
        assert_abs_diff_eq!(sp_from.alpha_constant(), sp_direct.alpha_constant(), epsilon = 1e-14);
    }

    #[test]
    fn simple_z0_matches_full_approximately() {
        let pp = PhysicalParameters::default();
        let sp = SimplePhysicalParameters::default();
        let z0_full = pp.calc_z0(GOLDEN_BORE_RADIUS);
        let z0_simple = sp.calc_z0(GOLDEN_BORE_RADIUS);
        // Within 1%
        assert!((z0_full - z0_simple).abs() / z0_full < 0.01);
    }

    // ── Temperature conversion tests ────────────────────────────

    #[test]
    fn fahrenheit_to_celsius_32f_is_0c() {
        assert_abs_diff_eq!(to_celsius(32.0, TemperatureType::F), 0.0, epsilon = 1e-10);
    }

    #[test]
    fn fahrenheit_to_celsius_212f_is_100c() {
        assert_abs_diff_eq!(to_celsius(212.0, TemperatureType::F), 100.0, epsilon = 1e-10);
    }

    #[test]
    fn celsius_passthrough() {
        assert_abs_diff_eq!(to_celsius(22.5, TemperatureType::C), 22.5, epsilon = 1e-10);
    }

    // ── 20°C golden values (Java app effective default) ──────────
    //
    // The Java app's OptimizationPreferences.DEFAULT_TEMPERATURE = 20
    // overrides the PhysicalParameters constructor default of 72°F.
    // These tests confirm our model matches at the temperature users
    // actually see when running WIDesigner.

    const GOLDEN_20C_SPEED_OF_SOUND: f64 = 343.786643259173161;
    const GOLDEN_20C_RHO: f64 = 1.199836215699805;
    const GOLDEN_20C_ETA: f64 = 1.809734419892756e-5;
    const GOLDEN_20C_GAMMA: f64 = 1.399533676343388;
    const GOLDEN_20C_ALPHA: f64 = 2.180438087240154e-4;
    const GOLDEN_20C_EPSILON: f64 = 1.612866142128732e-3;

    #[test]
    fn params_at_20c_speed_of_sound() {
        let p = PhysicalParameters::new(20.0, TemperatureType::C);
        assert_abs_diff_eq!(p.speed_of_sound(), GOLDEN_20C_SPEED_OF_SOUND, epsilon = 1e-8);
    }

    #[test]
    fn params_at_20c_density() {
        let p = PhysicalParameters::new(20.0, TemperatureType::C);
        assert_abs_diff_eq!(p.rho(), GOLDEN_20C_RHO, epsilon = 1e-10);
    }

    #[test]
    fn params_at_20c_viscosity() {
        let p = PhysicalParameters::new(20.0, TemperatureType::C);
        assert_abs_diff_eq!(p.eta(), GOLDEN_20C_ETA, epsilon = 1e-16);
    }

    #[test]
    fn params_at_20c_gamma() {
        let p = PhysicalParameters::new(20.0, TemperatureType::C);
        assert_abs_diff_eq!(p.gamma(), GOLDEN_20C_GAMMA, epsilon = 1e-10);
    }

    #[test]
    fn params_at_20c_alpha_constant() {
        let p = PhysicalParameters::new(20.0, TemperatureType::C);
        assert_abs_diff_eq!(p.alpha_constant(), GOLDEN_20C_ALPHA, epsilon = 1e-14);
    }

    #[test]
    fn params_at_20c_epsilon_constant() {
        let p = PhysicalParameters::new(20.0, TemperatureType::C);
        assert_abs_diff_eq!(p.epsilon_constant(), GOLDEN_20C_EPSILON, epsilon = 1e-14);
    }

    // ── Humidity variation tests (20°C, varying RH) ────────────
    //
    // Humidity affects vapour mole fraction, which shifts density,
    // speed of sound, and viscosity. These tests verify the CIPM
    // model responds correctly to humidity changes.

    // 20°C, 101.325 kPa, 20% RH, 390 ppm CO2
    const GOLDEN_RH20_SPEED: f64 = 3.434752507143849e2;
    const GOLDEN_RH20_RHO: f64 = 1.202456663608775e0;
    const GOLDEN_RH20_ETA: f64 = 1.815858103182552e-5;
    const GOLDEN_RH20_EPSILON: f64 = 1.614887161433479e-3;

    // 20°C, 101.325 kPa, 80% RH, 390 ppm CO2
    const GOLDEN_RH80_SPEED: f64 = 3.442236679296437e2;
    const GOLDEN_RH80_RHO: f64 = 1.196174645522774e0;
    const GOLDEN_RH80_ETA: f64 = 1.801177031067394e-5;
    const GOLDEN_RH80_EPSILON: f64 = 1.610038947624210e-3;

    fn params_at_rh(rh: f64) -> PhysicalParameters {
        PhysicalParameters::with_all(20.0, TemperatureType::C, 101.325, rh, 390e-6)
    }

    #[test]
    fn humidity_20pct_speed_of_sound() {
        assert_abs_diff_eq!(params_at_rh(20.0).speed_of_sound(), GOLDEN_RH20_SPEED, epsilon = 1e-8);
    }

    #[test]
    fn humidity_20pct_density() {
        assert_abs_diff_eq!(params_at_rh(20.0).rho(), GOLDEN_RH20_RHO, epsilon = 1e-10);
    }

    #[test]
    fn humidity_20pct_viscosity() {
        assert_abs_diff_eq!(params_at_rh(20.0).eta(), GOLDEN_RH20_ETA, epsilon = 1e-16);
    }

    #[test]
    fn humidity_20pct_epsilon() {
        assert_abs_diff_eq!(params_at_rh(20.0).epsilon_constant(), GOLDEN_RH20_EPSILON, epsilon = 1e-14);
    }

    #[test]
    fn humidity_80pct_speed_of_sound() {
        assert_abs_diff_eq!(params_at_rh(80.0).speed_of_sound(), GOLDEN_RH80_SPEED, epsilon = 1e-8);
    }

    #[test]
    fn humidity_80pct_density() {
        assert_abs_diff_eq!(params_at_rh(80.0).rho(), GOLDEN_RH80_RHO, epsilon = 1e-10);
    }

    #[test]
    fn humidity_80pct_viscosity() {
        assert_abs_diff_eq!(params_at_rh(80.0).eta(), GOLDEN_RH80_ETA, epsilon = 1e-16);
    }

    #[test]
    fn humidity_80pct_epsilon() {
        assert_abs_diff_eq!(params_at_rh(80.0).epsilon_constant(), GOLDEN_RH80_EPSILON, epsilon = 1e-14);
    }

    #[test]
    fn higher_humidity_increases_speed_of_sound() {
        // Water vapour is lighter than dry air → higher humidity → faster sound
        let low = params_at_rh(20.0).speed_of_sound();
        let high = params_at_rh(80.0).speed_of_sound();
        assert!(high > low, "80% RH should give faster sound than 20% RH");
    }

    #[test]
    fn higher_humidity_decreases_density() {
        // Water vapour is lighter than dry air → higher humidity → lower density
        let low = params_at_rh(20.0).rho();
        let high = params_at_rh(80.0).rho();
        assert!(high < low, "80% RH should give lower density than 20% RH");
    }
}
