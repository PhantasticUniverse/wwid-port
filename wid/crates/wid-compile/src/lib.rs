//! Instrument compilation: raw XML model to acoustic component chain.
//!
//! The [`compile`] function converts an [`InstrumentRaw`] (from XML) into an
//! [`InstrumentCompiled`] whose component chain (bore sections interleaved
//! with toneholes) is ready for acoustic evaluation.
//!
//! # Compilation steps
//!
//! 1. Convert all dimensions to metres
//! 2. Sort bore points by position (ascending)
//! 3. Extract headspace: bore sections above the mouthpiece position
//! 4. Sort holes by position, interleave bore sections between them
//! 5. Set termination diameter from last bore point
//!
//! The result is a flat component list where bore sections alternate with
//! holes, all in ascending position order and all below the mouthpiece.

use wid_types::InstrumentRaw;

/// Minimum bore section length (metres). Prevents zero-length sections.
pub const MINIMUM_CONE_LENGTH: f64 = 0.00001;

/// A compiled instrument ready for acoustic evaluation.
#[derive(Debug, Clone)]
pub struct InstrumentCompiled {
    pub name: String,
    pub mouthpiece: CompiledMouthpiece,
    pub components: Vec<Component>,
    pub termination: CompiledTermination,
}

/// A component in the compiled instrument chain.
#[derive(Debug, Clone)]
pub enum Component {
    Bore(BoreSection),
    Hole(CompiledHole),
}

/// A conical or cylindrical bore section.
#[derive(Debug, Clone)]
pub struct BoreSection {
    pub length: f64,
    pub left_radius: f64,
    pub right_radius: f64,
    pub right_bore_position: f64,
}

/// A tonehole with its interpolated bore diameter.
#[derive(Debug, Clone)]
pub struct CompiledHole {
    pub name: Option<String>,
    pub position: f64,
    pub diameter: f64,
    pub height: f64,
    pub bore_diameter: f64,
    pub inner_curvature_radius: Option<f64>,
    // Key fields omitted for now (NAF instruments don't use keys)
}

/// Compiled mouthpiece with headspace bore sections.
#[derive(Debug, Clone)]
pub struct CompiledMouthpiece {
    pub position: f64,
    pub bore_diameter: f64,
    pub headspace: Vec<BoreSection>,
    pub mouthpiece_type: MouthpieceType,
    /// Loop gain factor (Auvray, 2012): G = gain_factor * freq * rho / |Z|.
    /// Computed from beta, windway_height, window_length, window_width.
    /// None when beta or windway_height is absent.
    pub gain_factor: Option<f64>,
    /// Mouthpiece beta parameter (dimensionless).
    /// Used by reed instruments in the reactance calculation.
    /// Defaults to 0.0 if not set in the instrument XML.
    pub beta: f64,
}

/// The kind of mouthpiece, with its specific parameters (in metres).
#[derive(Debug, Clone)]
pub enum MouthpieceType {
    Fipple {
        window_length: f64,
        window_width: f64,
        fipple_factor: Option<f64>,
        window_height: Option<f64>,
        windway_length: Option<f64>,
        windway_height: Option<f64>,
    },
    EmbouchureHole {
        length: f64,
        width: f64,
        height: f64,
        airstream_length: f64,
        airstream_height: f64,
    },
    /// Reed mouthpiece (single, double, or lip reed).
    ///
    /// Uses a linear reactance model: `X = alpha * 1e-3 * freq + beta`.
    /// For lip reeds, beta sign is negated in the impedance calculation.
    /// Matches Java `SimpleReedMouthpieceCalculator`.
    SimpleReed {
        /// Reed-specific reactance coefficient (from XML alpha parameter).
        alpha: f64,
        /// Whether this is a lip reed (negates beta in impedance calc).
        is_lip_reed: bool,
    },
}

/// End termination of the bore.
#[derive(Debug, Clone)]
pub struct CompiledTermination {
    pub flange_diameter: f64,
    pub bore_diameter: f64,
    pub bore_position: f64,
}

/// Compilation errors.
#[derive(Debug, Clone)]
pub struct CompileError {
    pub messages: Vec<String>,
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for msg in &self.messages {
            writeln!(f, "{msg}")?;
        }
        Ok(())
    }
}

impl std::error::Error for CompileError {}

/// Compile a raw instrument into a component chain ready for evaluation.
///
/// Converts all dimensions to metres, validates the instrument geometry,
/// and builds the interleaved bore-section/hole component list.
pub fn compile(raw: &InstrumentRaw) -> Result<InstrumentCompiled, CompileError> {
    // Validate first
    let errors = validate(raw);
    if !errors.is_empty() {
        return Err(CompileError { messages: errors });
    }

    let m = raw.length_type.to_metres();

    // Convert bore points to metres and sort by position
    let mut bore_points: Vec<BorePointM> = raw
        .bore_points
        .iter()
        .map(|bp| BorePointM {
            position: bp.bore_position * m,
            diameter: bp.bore_diameter * m,
        })
        .collect();
    bore_points.sort_by(|a, b| a.position.partial_cmp(&b.position).unwrap());

    // Convert holes to metres and sort by position
    let mut holes: Vec<HoleM> = raw
        .holes
        .iter()
        .map(|h| HoleM {
            name: h.name.clone(),
            position: h.bore_position * m,
            diameter: h.diameter * m,
            height: h.height * m,
            inner_curvature_radius: h.inner_curvature_radius.map(|r| r * m),
            bore_diameter: 0.0, // filled during compilation
        })
        .collect();
    holes.sort_by(|a, b| a.position.partial_cmp(&b.position).unwrap());

    let mouthpiece_position = raw.mouthpiece.position * m;
    let flange_diameter = raw.termination.flange_diameter * m;

    let mut components: Vec<Component> = Vec::new();

    // ── Step 1: Process mouthpiece (extract headspace) ──────────

    // Create bore sections up to mouthpiece position
    make_sections(&mut bore_points, mouthpiece_position, &mut components);

    // Interpolate bore diameter at mouthpiece and create section there
    let mp_bore_diameter = process_position(&mut bore_points, mouthpiece_position, &mut components);

    // Extract headspace: all bore sections with right_bore_position <= mouthpiece_position
    let mut headspace = Vec::new();
    components.retain(|c| {
        if let Component::Bore(bs) = c {
            if bs.right_bore_position <= mouthpiece_position {
                headspace.push(bs.clone());
                return false;
            }
        }
        true
    });

    // ── Step 2: Process termination ─────────────────────────────

    let last_point = bore_points.last().unwrap();
    let term_bore_diameter = last_point.diameter;
    let term_bore_position = last_point.position;

    // ── Step 3: Process holes (interleave with bore sections) ───

    for hole in &mut holes {
        make_sections(&mut bore_points, hole.position, &mut components);
        hole.bore_diameter = process_position(&mut bore_points, hole.position, &mut components);
        components.push(Component::Hole(CompiledHole {
            name: hole.name.clone(),
            position: hole.position,
            diameter: hole.diameter,
            height: hole.height,
            bore_diameter: hole.bore_diameter,
            inner_curvature_radius: hole.inner_curvature_radius,
        }));
    }

    // ── Step 4: Remaining bore sections after last hole ─────────

    let last_pos = bore_points.last().map(|p| p.position).unwrap_or(0.0) + 1.0;
    make_sections(&mut bore_points, last_pos, &mut components);

    // ── Build mouthpiece ────────────────────────────────────────

    let mouthpiece_type = build_mouthpiece_type(&raw.mouthpiece, m);

    let gain_factor = compute_gain_factor(&raw.mouthpiece, &mouthpiece_type);

    let beta = raw.mouthpiece.beta.unwrap_or(0.0);

    Ok(InstrumentCompiled {
        name: raw.name.clone(),
        mouthpiece: CompiledMouthpiece {
            position: mouthpiece_position,
            bore_diameter: mp_bore_diameter,
            headspace,
            mouthpiece_type,
            gain_factor,
            beta,
        },
        components,
        termination: CompiledTermination {
            flange_diameter,
            bore_diameter: term_bore_diameter,
            bore_position: term_bore_position,
        },
    })
}

// ── Instrument mutation API ──────────────────────────────────────

/// Read the fipple factor from a raw instrument.
pub fn get_fipple_factor(raw: &InstrumentRaw) -> Option<f64> {
    raw.mouthpiece.fipple.as_ref()?.fipple_factor
}

/// Set the fipple factor on a raw instrument.
pub fn set_fipple_factor(raw: &mut InstrumentRaw, value: f64) {
    if let Some(ref mut fipple) = raw.mouthpiece.fipple {
        fipple.fipple_factor = Some(value);
    }
}

/// Read the window height from a raw instrument (in metres).
///
/// For fipple instruments: returns `fipple.window_height * length_unit`.
/// For embouchure hole instruments: returns `embouchure_hole.height * length_unit`.
pub fn get_window_height(raw: &InstrumentRaw) -> Option<f64> {
    let m = raw.length_type.to_metres();
    if let Some(ref fipple) = raw.mouthpiece.fipple {
        return fipple.window_height.map(|h| h * m);
    }
    if let Some(ref emb) = raw.mouthpiece.embouchure_hole {
        return Some(emb.height * m);
    }
    None
}

/// Set the window height on a raw instrument (value in metres).
///
/// For fipple instruments: sets `fipple.window_height`.
/// For embouchure hole instruments: sets `embouchure_hole.height`.
pub fn set_window_height(raw: &mut InstrumentRaw, value_metres: f64) {
    let m = raw.length_type.to_metres();
    if let Some(ref mut fipple) = raw.mouthpiece.fipple {
        fipple.window_height = Some(value_metres / m);
    } else if let Some(ref mut emb) = raw.mouthpiece.embouchure_hole {
        emb.height = value_metres / m;
    }
}

/// Read the airstream length from a raw instrument (in metres).
///
/// For embouchure hole instruments: returns `embouchure_hole.airstream_length * length_unit`.
/// For fipple instruments: returns `fipple.window_length * length_unit`
/// (airstream length is analogous to window length for fipple mouthpieces).
///
/// Matches Java `AirstreamLengthObjectiveFunction.getGeometryPoint()`.
pub fn get_airstream_length(raw: &InstrumentRaw) -> Option<f64> {
    let m = raw.length_type.to_metres();
    if let Some(ref emb) = raw.mouthpiece.embouchure_hole {
        return Some(emb.airstream_length * m);
    }
    if let Some(ref fipple) = raw.mouthpiece.fipple {
        return Some(fipple.window_length * m);
    }
    None
}

/// Set the airstream length on a raw instrument (value in metres).
///
/// For embouchure hole instruments: sets `embouchure_hole.airstream_length`.
/// For fipple instruments: sets `fipple.window_length`.
///
/// Matches Java `AirstreamLengthObjectiveFunction.setGeometryPoint()`.
pub fn set_airstream_length(raw: &mut InstrumentRaw, value_metres: f64) {
    let m = raw.length_type.to_metres();
    if let Some(ref mut emb) = raw.mouthpiece.embouchure_hole {
        emb.airstream_length = value_metres / m;
    } else if let Some(ref mut fipple) = raw.mouthpiece.fipple {
        fipple.window_length = value_metres / m;
    }
}

/// Read the mouthpiece beta factor from a raw instrument.
pub fn get_beta(raw: &InstrumentRaw) -> Option<f64> {
    raw.mouthpiece.beta
}

/// Set the mouthpiece beta factor on a raw instrument.
pub fn set_beta(raw: &mut InstrumentRaw, value: f64) {
    raw.mouthpiece.beta = Some(value);
}

/// Read the reed alpha factor from a raw instrument.
///
/// Matches Java `ReedCalibratorObjectiveFunction.getGeometryPoint()`:
/// checks single_reed, then double_reed, then lip_reed.
pub fn get_alpha(raw: &InstrumentRaw) -> Option<f64> {
    let mp = &raw.mouthpiece;
    mp.single_reed.as_ref().map(|r| r.alpha)
        .or_else(|| mp.double_reed.as_ref().map(|r| r.alpha))
        .or_else(|| mp.lip_reed.as_ref().map(|r| r.alpha))
}

/// Set the reed alpha factor on a raw instrument.
///
/// Matches Java `ReedCalibratorObjectiveFunction.setGeometryPoint()`:
/// sets on single_reed, else double_reed, else lip_reed.
pub fn set_alpha(raw: &mut InstrumentRaw, value: f64) {
    let mp = &mut raw.mouthpiece;
    if let Some(ref mut r) = mp.single_reed { r.alpha = value; }
    else if let Some(ref mut r) = mp.double_reed { r.alpha = value; }
    else if let Some(ref mut r) = mp.lip_reed { r.alpha = value; }
}

/// Extract hole diameters sorted by bore position ascending (in metres).
///
/// Returns N diameters for N holes, matching Java `HoleSizeObjectiveFunction.getGeometryPoint()`.
pub fn get_hole_diameters(raw: &InstrumentRaw) -> Vec<f64> {
    let m = raw.length_type.to_metres();
    let mut sorted: Vec<(f64, f64)> = raw
        .holes
        .iter()
        .map(|h| (h.bore_position * m, h.diameter * m))
        .collect();
    sorted.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    sorted.iter().map(|&(_, d)| d).collect()
}

/// Set hole diameters from a vector sorted by bore position ascending (in metres).
pub fn set_hole_diameters(raw: &mut InstrumentRaw, diameters: &[f64]) {
    let m = raw.length_type.to_metres();
    let n_holes = raw.holes.len();
    let mut hole_order: Vec<usize> = (0..n_holes).collect();
    hole_order.sort_by(|&a, &b| {
        raw.holes[a]
            .bore_position
            .partial_cmp(&raw.holes[b].bore_position)
            .unwrap()
    });
    for (i, &idx) in hole_order.iter().enumerate() {
        if i < diameters.len() {
            raw.holes[idx].diameter = diameters[i] / m;
        }
    }
}

/// Extract HolePosition geometry vector (in metres).
///
/// Matches Java `HolePositionObjectiveFunction.getGeometryPoint()`:
/// ```text
/// geometry[0]   = end of bore position
/// geometry[1]   = spacing: last_hole → bore_end
/// geometry[2]   = spacing: second_to_last → last_hole
/// ...
/// geometry[N]   = spacing: first_hole → second_hole
/// ```
///
/// Holes are sorted by bore position ascending. The loop iterates from
/// last hole (farthest from mouthpiece) backward.
pub fn get_hole_geometry_position(raw: &InstrumentRaw) -> Vec<f64> {
    let m = raw.length_type.to_metres();
    let n_holes = raw.holes.len();

    // Sort holes by bore position ascending
    let mut sorted_positions: Vec<f64> = raw
        .holes
        .iter()
        .map(|h| h.bore_position * m)
        .collect();
    sorted_positions.sort_by(|a, b| a.partial_cmp(b).unwrap());

    // Bore end = max bore point position
    let bore_end = raw
        .bore_points
        .iter()
        .map(|bp| bp.bore_position * m)
        .fold(f64::NEG_INFINITY, f64::max);

    let mut geometry = vec![0.0; n_holes + 1];

    // [0] bore end position
    geometry[0] = bore_end;

    // [1..N] spacings indexed by hole (matching Java HolePositionObjectiveFunction):
    // geometry[i+1] = prior_position - sorted_holes[i].position
    // Loop from bottom (i=n-1) to top (i=0), building spacings between
    // consecutive holes, ending with top-to-second-from-top.
    //
    // Result: geometry[1] = top-to-second spacing, ..., geometry[N] = bottom-to-bore-end
    let mut prior_pos = bore_end;
    for i in (0..n_holes).rev() {
        geometry[i + 1] = prior_pos - sorted_positions[i];
        prior_pos = sorted_positions[i];
    }

    geometry
}

/// Apply a HolePosition geometry vector (in metres) to a raw instrument.
///
/// Reverse of [`get_hole_geometry_position`]. Also adjusts the bore end
/// position with PRESERVE_TAPER semantics (interpolating/extrapolating
/// the bore diameter at the new end position).
pub fn set_hole_geometry_position(raw: &mut InstrumentRaw, geometry: &[f64]) {
    let m = raw.length_type.to_metres();
    let n_holes = raw.holes.len();

    // Sort hole indices by bore position ascending
    let mut hole_order: Vec<usize> = (0..n_holes).collect();
    hole_order.sort_by(|&a, &b| {
        raw.holes[a]
            .bore_position
            .partial_cmp(&raw.holes[b].bore_position)
            .unwrap()
    });

    // [0] bore end — update last bore point with PRESERVE_TAPER
    let new_bore_end = geometry[0];
    let new_dia = interpolate_bore_diameter(&raw.bore_points, new_bore_end, m);
    if let Some(last_bp) = raw
        .bore_points
        .iter_mut()
        .max_by(|a, b| a.bore_position.partial_cmp(&b.bore_position).unwrap())
    {
        last_bp.bore_position = new_bore_end / m;
        if let Some(dia) = new_dia {
            last_bp.bore_diameter = dia / m;
        }
    }

    // Reconstruct hole positions from spacings (working bottom-up)
    // geometry[1] = bore_end - last_hole
    // geometry[2] = last_hole - second_to_last
    // ...
    let mut prior_pos = new_bore_end;
    for i in (0..n_holes).rev() {
        let hole_pos = prior_pos - geometry[i + 1];
        raw.holes[hole_order[i]].bore_position = hole_pos / m;
        prior_pos = hole_pos;
    }
}

/// Extract the HoleFromTop geometry vector from a raw instrument (in metres).
///
/// Returns `[bore_end, top_hole_fraction, spacing_1..N-1, diameter_0..N-1]`
/// where:
/// - `bore_end` = last bore point position (metres)
/// - `top_hole_fraction` = (top hole - mouthpiece) / (bore_end - mouthpiece)
/// - `spacing_i` = distance between consecutive holes (metres)
/// - `diameter_i` = hole diameters sorted by position ascending (metres)
///
/// Holes are sorted by bore position ascending (top to bottom).
pub fn get_hole_geometry_from_top(raw: &InstrumentRaw) -> Vec<f64> {
    let m = raw.length_type.to_metres();
    let n_holes = raw.holes.len();

    // Sort holes by bore position ascending
    let mut sorted_holes: Vec<(f64, f64)> = raw
        .holes
        .iter()
        .map(|h| (h.bore_position * m, h.diameter * m))
        .collect();
    sorted_holes.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    // Bore end = max bore point position
    let bore_end = raw
        .bore_points
        .iter()
        .map(|bp| bp.bore_position * m)
        .fold(f64::NEG_INFINITY, f64::max);

    let mouthpiece_pos = raw.mouthpiece.position * m;

    let mut geometry = Vec::with_capacity(1 + 2 * n_holes);

    // [0] bore end position
    geometry.push(bore_end);

    // [1] top hole as fraction of bore length from mouthpiece
    if n_holes > 0 {
        let top_hole_pos = sorted_holes[0].0;
        geometry.push((top_hole_pos - mouthpiece_pos) / (bore_end - mouthpiece_pos));
    }

    // [2..N] spacings between consecutive holes
    for i in 1..n_holes {
        geometry.push(sorted_holes[i].0 - sorted_holes[i - 1].0);
    }

    // [N+1..2N] hole diameters (position order)
    for i in 0..n_holes {
        geometry.push(sorted_holes[i].1);
    }

    geometry
}

/// Apply a HoleFromTop geometry vector (in metres) to a raw instrument.
///
/// The geometry format matches [`get_hole_geometry_from_top`]:
/// `[bore_end, top_hole_fraction, spacing_1..N-1, diameter_0..N-1]`.
///
/// After calling this, you must re-compile the instrument before evaluation.
pub fn set_hole_geometry_from_top(raw: &mut InstrumentRaw, geometry: &[f64]) {
    let m = raw.length_type.to_metres();
    let n_holes = raw.holes.len();
    let mouthpiece_pos = raw.mouthpiece.position * m;

    // Sort holes by bore position to establish index mapping
    let mut hole_order: Vec<usize> = (0..n_holes).collect();
    hole_order.sort_by(|&a, &b| {
        raw.holes[a]
            .bore_position
            .partial_cmp(&raw.holes[b].bore_position)
            .unwrap()
    });

    // [0] bore end — update last bore point position
    let new_bore_end = geometry[0];
    // For PRESERVE_BORE: interpolate/extrapolate diameter at new position.
    let new_dia = interpolate_bore_diameter(&raw.bore_points, new_bore_end, m);
    // Find the bore point with max position and update it
    if let Some(last_bp) = raw
        .bore_points
        .iter_mut()
        .max_by(|a, b| a.bore_position.partial_cmp(&b.bore_position).unwrap())
    {
        last_bp.bore_position = new_bore_end / m;
        if let Some(dia) = new_dia {
            last_bp.bore_diameter = dia / m;
        }
    }

    // Compute hole positions from fraction + spacings
    if n_holes > 0 {
        let bore_length_from_edge = new_bore_end - mouthpiece_pos;
        let top_hole_pos = geometry[1] * bore_length_from_edge + mouthpiece_pos;

        // Set top hole position
        raw.holes[hole_order[0]].bore_position = top_hole_pos / m;

        // Set remaining hole positions from spacings
        let mut prior_pos = top_hole_pos;
        for i in 1..n_holes {
            let hole_pos = prior_pos + geometry[i + 1];
            raw.holes[hole_order[i]].bore_position = hole_pos / m;
            prior_pos = hole_pos;
        }

        // Set hole diameters
        for i in 0..n_holes {
            raw.holes[hole_order[i]].diameter = geometry[n_holes + 1 + i] / m;
        }
    }
}

/// Interpolate bore diameter at a position (in metres), using bore points.
fn interpolate_bore_diameter(
    bore_points: &[wid_types::BorePointRaw],
    position_m: f64,
    m: f64,
) -> Option<f64> {
    if bore_points.len() < 2 {
        return bore_points.first().map(|bp| bp.bore_diameter * m);
    }

    let mut points: Vec<(f64, f64)> = bore_points
        .iter()
        .map(|bp| (bp.bore_position * m, bp.bore_diameter * m))
        .collect();
    points.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    // Find bounding segment
    if position_m <= points[0].0 {
        // Extrapolate from first segment
        let (x0, y0) = points[0];
        let (x1, y1) = points[1];
        if (x1 - x0).abs() < f64::EPSILON {
            return Some(y0);
        }
        return Some(y0 + (position_m - x0) * (y1 - y0) / (x1 - x0));
    }
    if position_m >= points.last().unwrap().0 {
        // Extrapolate from last segment
        let n = points.len();
        let (x0, y0) = points[n - 2];
        let (x1, y1) = points[n - 1];
        if (x1 - x0).abs() < f64::EPSILON {
            return Some(y1);
        }
        return Some(y0 + (position_m - x0) * (y1 - y0) / (x1 - x0));
    }

    // Interpolate
    for w in points.windows(2) {
        if position_m >= w[0].0 && position_m <= w[1].0 {
            let (x0, y0) = w[0];
            let (x1, y1) = w[1];
            if (x1 - x0).abs() < f64::EPSILON {
                return Some(y0);
            }
            return Some(y0 + (position_m - x0) * (y1 - y0) / (x1 - x0));
        }
    }

    None
}

// ── Internal types and helpers ──────────────────────────────────

#[derive(Debug, Clone)]
struct BorePointM {
    position: f64,
    diameter: f64,
}

#[derive(Debug, Clone)]
struct HoleM {
    name: Option<String>,
    position: f64,
    diameter: f64,
    height: f64,
    inner_curvature_radius: Option<f64>,
    bore_diameter: f64,
}

/// Create bore sections from bore_points up to (not including) `right_position`.
///
/// Consumes bore points that are fully traversed (removes them from the list).
fn make_sections(
    bore_points: &mut Vec<BorePointM>,
    right_position: f64,
    components: &mut Vec<Component>,
) {
    // Find how many points are strictly before right_position
    let head_count = bore_points
        .iter()
        .take_while(|p| p.position < right_position)
        .count();

    if head_count > 1 {
        // Create sections between consecutive points in the head.
        // Between existing bore points, zero-length sections cannot occur,
        // so we don't need the mutable bore_points reference in add_section.
        for i in 0..head_count - 1 {
            let length = bore_points[i + 1].position - bore_points[i].position;
            components.push(Component::Bore(BoreSection {
                length: length.max(MINIMUM_CONE_LENGTH),
                left_radius: bore_points[i].diameter / 2.0,
                right_radius: bore_points[i + 1].diameter / 2.0,
                right_bore_position: bore_points[i + 1].position,
            }));
        }
        // Remove consumed points (all but the last in the head)
        bore_points.drain(0..head_count - 1);
    }
}

/// Interpolate bore diameter at `position`, create a bore section, and
/// update bore_points. Returns the interpolated bore diameter.
fn process_position(
    bore_points: &mut Vec<BorePointM>,
    position: f64,
    components: &mut Vec<Component>,
) -> f64 {
    if bore_points.len() < 2 {
        // Edge case: only one point left, use its diameter
        return bore_points.first().map(|p| p.diameter).unwrap_or(0.0);
    }

    let left_pos = bore_points[0].position;
    let left_dia = bore_points[0].diameter;
    let right_pos = bore_points[1].position;
    let right_dia = bore_points[1].diameter;

    // Interpolate bore diameter at this position
    let bore_diameter = if (right_dia - left_dia).abs() < f64::EPSILON || (position - left_pos).abs() < f64::EPSILON {
        // Cylindrical bore or position at left boundary
        left_dia
    } else if right_pos > left_pos {
        // Linear interpolation
        left_dia + (position - left_pos) * (right_dia - left_dia) / (right_pos - left_pos)
    } else {
        // Degenerate: average
        0.5 * (left_dia + right_dia)
    };

    // If position falls between bore points, insert a new bore point
    if right_pos > position {
        // Create section from left to the new point.
        // Matches Java Instrument.addSection(): when length is zero,
        // bump both the section's right_bore_position and the new bore
        // point's position by MINIMUM_CONE_LENGTH.
        let left = bore_points[0].clone();
        let raw_length = position - left.position;
        let (length, new_pos) = if raw_length <= 0.0 {
            (MINIMUM_CONE_LENGTH, position + MINIMUM_CONE_LENGTH)
        } else {
            (raw_length, position)
        };
        let section = BoreSection {
            length,
            left_radius: left.diameter / 2.0,
            right_radius: bore_diameter / 2.0,
            right_bore_position: new_pos,
        };
        components.push(Component::Bore(section));
        // Remove the left point and insert the new point at the front
        let new_point = BorePointM {
            position: new_pos,
            diameter: bore_diameter,
        };
        bore_points.remove(0);
        bore_points.insert(0, new_point);
    } else {
        // Position is at or past right point - create section normally
        let left = bore_points[0].clone();
        let right = bore_points[1].clone();
        let mut length = right.position - left.position;
        let mut right_position = right.position;
        if length <= 0.0 {
            length = MINIMUM_CONE_LENGTH;
            right_position = left.position + MINIMUM_CONE_LENGTH;
        }
        components.push(Component::Bore(BoreSection {
            length,
            left_radius: left.diameter / 2.0,
            right_radius: right.diameter / 2.0,
            right_bore_position: right_position,
        }));
        bore_points.remove(0);
    }

    bore_diameter
}

fn build_mouthpiece_type(
    mp: &wid_types::MouthpieceRaw,
    m: f64,
) -> MouthpieceType {
    if let Some(ref f) = mp.fipple {
        MouthpieceType::Fipple {
            window_length: f.window_length * m,
            window_width: f.window_width * m,
            fipple_factor: f.fipple_factor,
            window_height: f.window_height.map(|v| v * m),
            windway_length: f.windway_length.map(|v| v * m),
            windway_height: f.windway_height.map(|v| v * m),
        }
    } else if let Some(ref e) = mp.embouchure_hole {
        MouthpieceType::EmbouchureHole {
            length: e.length * m,
            width: e.width * m,
            height: e.height * m,
            airstream_length: e.airstream_length * m,
            airstream_height: e.airstream_height * m,
        }
    } else if let Some(ref sr) = mp.single_reed {
        MouthpieceType::SimpleReed { alpha: sr.alpha, is_lip_reed: false }
    } else if let Some(ref dr) = mp.double_reed {
        MouthpieceType::SimpleReed { alpha: dr.alpha, is_lip_reed: false }
    } else if let Some(ref lr) = mp.lip_reed {
        MouthpieceType::SimpleReed { alpha: lr.alpha, is_lip_reed: true }
    } else {
        panic!("Unsupported mouthpiece type");
    }
}

/// Compute the loop gain factor from mouthpiece parameters (Auvray, 2012).
///
/// For fipple instruments: `G0 = 8 * h * sqrt(2h/wl) * exp(beta * wl / h) / (wl * ww)`
/// where h = windwayHeight, wl = windowLength, ww = windowWidth.
/// Returns None if beta or windwayHeight is absent.
fn compute_gain_factor(
    mp: &wid_types::MouthpieceRaw,
    compiled_type: &MouthpieceType,
) -> Option<f64> {
    let nominal_beta = mp.beta.unwrap_or(0.35);

    match compiled_type {
        MouthpieceType::Fipple {
            window_length,
            window_width,
            windway_height: Some(wh),
            ..
        } => {
            // Java: 8 * windwayHeight * sqrt(2 * windwayHeight / windowLength)
            //       * exp(beta * windowLength / windwayHeight)
            //       / (windowLength * windowWidth)
            Some(
                8.0 * wh
                    * (2.0 * wh / window_length).sqrt()
                    * (nominal_beta * window_length / wh).exp()
                    / (window_length * window_width),
            )
        }
        MouthpieceType::EmbouchureHole {
            length,
            airstream_length,
            airstream_height,
            ..
        } => {
            Some(
                8.0 * airstream_height
                    * (2.0 * airstream_height / airstream_length).sqrt()
                    * (nominal_beta * airstream_length / airstream_height).exp()
                    / (length * airstream_length),
            )
        }
        // SimpleReed: no gain model (pressure-node boundary condition)
        _ => None,
    }
}

/// Validate raw instrument geometry. Returns a list of error messages.
fn validate(raw: &InstrumentRaw) -> Vec<String> {
    let mut errors = Vec::new();

    if raw.name.is_empty() {
        errors.push("Instrument must have a name.".to_string());
    }
    if raw.bore_points.len() < 2 {
        errors.push("Instrument must have at least two bore points.".to_string());
    }
    if raw.bore_points.len() >= 2 {
        let min_pos = raw
            .bore_points
            .iter()
            .map(|p| p.bore_position)
            .fold(f64::INFINITY, f64::min);
        let max_pos = raw
            .bore_points
            .iter()
            .map(|p| p.bore_position)
            .fold(f64::NEG_INFINITY, f64::max);
        if max_pos <= min_pos {
            errors.push("Bore length must be positive.".to_string());
        }
        // Validate mouthpiece is within bore range
        if raw.mouthpiece.position < min_pos || raw.mouthpiece.position > max_pos {
            errors.push("Mouthpiece position must be within bore range.".to_string());
        }
        // Validate holes are within bore range
        for hole in &raw.holes {
            if hole.bore_position < raw.mouthpiece.position || hole.bore_position > max_pos {
                errors.push(format!(
                    "Hole '{}' is outside the valid bore range.",
                    hole.name.as_deref().unwrap_or("unnamed")
                ));
            }
        }
    }
    if raw.mouthpiece.fipple.is_none()
        && raw.mouthpiece.embouchure_hole.is_none()
        && raw.mouthpiece.single_reed.is_none()
        && raw.mouthpiece.double_reed.is_none()
        && raw.mouthpiece.lip_reed.is_none()
    {
        errors.push("Mouthpiece must have a type (fipple, embouchure hole, etc.).".to_string());
    }

    errors
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;
    use wid_types::parse_instrument_xml;

    const INCHES_TO_METRES: f64 = 0.0254;

    const NAF_6HOLE_XML: &str = include_str!(
        "../../../../oracle/v2.6.0/NafStudy/instruments/0.75-bore_6-hole_NAF_starter.xml"
    );
    const NAF_0HOLE_XML: &str =
        include_str!("../../../../golden/scenarios/support/NAF-FF-02_instrument_0hole.xml");

    fn compile_naf_6hole() -> InstrumentCompiled {
        let raw = parse_instrument_xml(NAF_6HOLE_XML).unwrap();
        compile(&raw).unwrap()
    }

    fn compile_naf_0hole() -> InstrumentCompiled {
        let raw = parse_instrument_xml(NAF_0HOLE_XML).unwrap();
        compile(&raw).unwrap()
    }

    // ── Component count matches Java golden internals ───────────

    #[test]
    fn six_hole_has_13_components() {
        let inst = compile_naf_6hole();
        assert_eq!(inst.components.len(), 13); // 7 bore sections + 6 holes
    }

    #[test]
    fn six_hole_has_1_headspace_section() {
        let inst = compile_naf_6hole();
        assert_eq!(inst.mouthpiece.headspace.len(), 1);
    }

    #[test]
    fn zero_hole_has_1_component() {
        let inst = compile_naf_0hole();
        // 0 holes, mouthpiece splits bore into headspace + 1 main section
        assert_eq!(inst.components.len(), 1);
        assert_eq!(inst.mouthpiece.headspace.len(), 1);
    }

    // ── Bore diameter and position validation ───────────────────

    #[test]
    fn mouthpiece_position_in_metres() {
        let inst = compile_naf_6hole();
        assert_abs_diff_eq!(
            inst.mouthpiece.position,
            0.18000068040110218 * INCHES_TO_METRES,
            epsilon = 1e-15
        );
    }

    #[test]
    fn bore_radius_matches_golden() {
        // Golden internals: boreRadius_m = 0.00952501543050833
        let inst = compile_naf_6hole();
        let bore_radius = inst.mouthpiece.bore_diameter / 2.0;
        assert_abs_diff_eq!(bore_radius, 0.00952501543050833, epsilon = 1e-10);
    }

    #[test]
    fn termination_position_matches() {
        let inst = compile_naf_6hole();
        assert_abs_diff_eq!(
            inst.termination.bore_position,
            12.790953423936331 * INCHES_TO_METRES,
            epsilon = 1e-10
        );
    }

    #[test]
    fn termination_flange_diameter_in_metres() {
        let inst = compile_naf_6hole();
        assert_abs_diff_eq!(
            inst.termination.flange_diameter,
            1.1250018225009841 * INCHES_TO_METRES,
            epsilon = 1e-10
        );
    }

    // ── Component ordering ──────────────────────────────────────

    #[test]
    fn components_alternate_bore_hole() {
        let inst = compile_naf_6hole();
        // Pattern: Bore, Hole, Bore, Hole, ..., Bore
        for (i, c) in inst.components.iter().enumerate() {
            match c {
                Component::Bore(_) => assert!(
                    i % 2 == 0,
                    "BoreSection at odd index {i}"
                ),
                Component::Hole(_) => assert!(
                    i % 2 == 1,
                    "Hole at even index {i}"
                ),
            }
        }
    }

    #[test]
    fn holes_are_in_ascending_position_order() {
        let inst = compile_naf_6hole();
        let hole_positions: Vec<f64> = inst
            .components
            .iter()
            .filter_map(|c| {
                if let Component::Hole(h) = c {
                    Some(h.position)
                } else {
                    None
                }
            })
            .collect();

        for w in hole_positions.windows(2) {
            assert!(w[0] < w[1], "Holes not in ascending order");
        }
    }

    #[test]
    fn bore_sections_have_positive_length() {
        let inst = compile_naf_6hole();
        for c in &inst.components {
            if let Component::Bore(bs) = c {
                assert!(bs.length > 0.0, "Zero-length bore section");
            }
        }
    }

    // ── Headspace validation ────────────────────────────────────

    #[test]
    fn headspace_section_ends_at_mouthpiece() {
        let inst = compile_naf_6hole();
        let hs = &inst.mouthpiece.headspace[0];
        assert_abs_diff_eq!(
            hs.right_bore_position,
            inst.mouthpiece.position,
            epsilon = 1e-15
        );
    }

    #[test]
    fn headspace_starts_at_bore_origin() {
        let inst = compile_naf_6hole();
        let hs = &inst.mouthpiece.headspace[0];
        // Left end is at position 0 (bore start), so length = mouthpiece position
        assert_abs_diff_eq!(
            hs.length,
            inst.mouthpiece.position,
            epsilon = 1e-12
        );
    }

    // ── Fipple factor passthrough ───────────────────────────────

    #[test]
    fn fipple_factor_preserved() {
        let inst = compile_naf_6hole();
        match &inst.mouthpiece.mouthpiece_type {
            MouthpieceType::Fipple { fipple_factor, .. } => {
                assert_abs_diff_eq!(fipple_factor.unwrap(), 0.75, epsilon = 1e-15);
            }
            _ => panic!("Expected fipple mouthpiece"),
        }
    }

    #[test]
    fn windway_height_converted_to_metres() {
        let inst = compile_naf_6hole();
        match &inst.mouthpiece.mouthpiece_type {
            MouthpieceType::Fipple { windway_height, .. } => {
                assert_abs_diff_eq!(
                    windway_height.unwrap(),
                    0.03200012096019596 * INCHES_TO_METRES,
                    epsilon = 1e-15
                );
            }
            _ => panic!("Expected fipple mouthpiece"),
        }
    }

    // ── Validation tests ────────────────────────────────────────

    #[test]
    fn reject_empty_name() {
        let mut raw = parse_instrument_xml(NAF_6HOLE_XML).unwrap();
        raw.name = String::new();
        let result = compile(&raw);
        assert!(result.is_err());
    }

    #[test]
    fn reject_single_bore_point() {
        let mut raw = parse_instrument_xml(NAF_6HOLE_XML).unwrap();
        raw.bore_points.truncate(1);
        let result = compile(&raw);
        assert!(result.is_err());
    }

    // ── Hole bore diameter interpolation ────────────────────────

    #[test]
    fn hole_bore_diameters_match_cylindrical_bore() {
        let inst = compile_naf_6hole();
        let expected_bore_dia = 0.750001215000656 * INCHES_TO_METRES;
        // For a cylindrical bore, all holes should have the same bore diameter
        for c in &inst.components {
            if let Component::Hole(h) = c {
                assert_abs_diff_eq!(h.bore_diameter, expected_bore_dia, epsilon = 1e-10);
            }
        }
    }

    // ── Fipple factor mutation ───────────────────────────────────

    #[test]
    fn get_fipple_factor_returns_value() {
        let raw = parse_instrument_xml(NAF_6HOLE_XML).unwrap();
        assert_abs_diff_eq!(get_fipple_factor(&raw).unwrap(), 0.75, epsilon = 1e-15);
    }

    #[test]
    fn set_fipple_factor_updates_value() {
        let mut raw = parse_instrument_xml(NAF_6HOLE_XML).unwrap();
        set_fipple_factor(&mut raw, 0.5);
        assert_abs_diff_eq!(get_fipple_factor(&raw).unwrap(), 0.5, epsilon = 1e-15);
    }

    // ── Hole geometry extraction ─────────────────────────────────

    // Golden initialGeometry from NAF-OPT-01
    const GOLDEN_INITIAL_GEOMETRY: [f64; 13] = [
        0.3248902169679828,
        0.26393387003800606,
        0.02084975171698325,
        0.020849751716983278,
        0.04085938293871649,
        0.02865934261586897,
        0.028659342615868943,
        0.0057100938065062215,
        0.006327228446346466,
        0.006056222214560144,
        0.007836036154750887,
        0.007616195298537355,
        0.007846589456097008,
    ];

    #[test]
    fn get_hole_geometry_matches_golden() {
        let raw = parse_instrument_xml(NAF_6HOLE_XML).unwrap();
        let geom = get_hole_geometry_from_top(&raw);
        assert_eq!(geom.len(), 13);
        for (i, (&actual, &expected)) in
            geom.iter().zip(GOLDEN_INITIAL_GEOMETRY.iter()).enumerate()
        {
            assert!(
                (actual - expected).abs() < 1e-10,
                "geometry[{i}]: expected {expected}, got {actual}, diff {}",
                (actual - expected).abs()
            );
        }
    }

    #[test]
    fn set_hole_geometry_roundtrips() {
        let mut raw = parse_instrument_xml(NAF_6HOLE_XML).unwrap();
        let original = get_hole_geometry_from_top(&raw);
        // Apply the same geometry back
        set_hole_geometry_from_top(&mut raw, &original);
        let roundtripped = get_hole_geometry_from_top(&raw);
        assert_eq!(original.len(), roundtripped.len());
        for (i, (a, b)) in original.iter().zip(roundtripped.iter()).enumerate() {
            assert!(
                (a - b).abs() < 1e-12,
                "roundtrip mismatch at [{i}]: {a} vs {b}"
            );
        }
    }

    #[test]
    fn set_hole_geometry_changes_instrument() {
        let mut raw = parse_instrument_xml(NAF_6HOLE_XML).unwrap();
        let mut geom = get_hole_geometry_from_top(&raw);
        // Double all diameters
        let n_holes = raw.holes.len();
        for i in 0..n_holes {
            geom[n_holes + 1 + i] *= 2.0;
        }
        set_hole_geometry_from_top(&mut raw, &geom);
        let new_geom = get_hole_geometry_from_top(&raw);
        for i in 0..n_holes {
            assert_abs_diff_eq!(
                new_geom[n_holes + 1 + i],
                GOLDEN_INITIAL_GEOMETRY[n_holes + 1 + i] * 2.0,
                epsilon = 1e-12
            );
        }
    }
}
