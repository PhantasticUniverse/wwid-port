// Geometry ordering logic ported from Java — index-based loops preserve clarity.
#![allow(clippy::needless_range_loop)]
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

// ── Grouped hole geometry (HoleGroupFromTop parameterization) ────

/// Mapping from each hole index to its geometry dimension, plus the averaging
/// factor. Used by grouped hole optimizers.
///
/// Matches Java `HoleGroupPositionObjectiveFunction.computeDimensionByHole()`.
pub struct HoleGroupMapping {
    /// For each hole, the geometry dimension that encodes the spacing after
    /// this hole. In the FromTop variant, geometry indices are offset by +1
    /// (dimension 0 = bore end, dimension 1 = top ratio).
    pub dimension_by_hole: Vec<usize>,
    /// For each hole, the number of holes sharing that dimension (used to
    /// average spacings in getGeometry / replicate in setGeometry).
    pub group_size: Vec<f64>,
    /// Number of geometry dimensions for the position part (excluding the
    /// bore-end-to-last-hole gap that's implicit in FromTop).
    pub n_position_dims: usize,
}

/// Compute the dimension-by-hole mapping for a set of hole groups.
///
/// `hole_groups` is e.g. `[[0,1,2],[3,4,5]]` for a 6-hole NAF.
/// Returns the mapping used by both get and set geometry functions.
///
/// Matches Java `HoleGroupPositionObjectiveFunction.validateHoleGroups()`
/// + `computeDimensionByHole()`.
pub fn compute_hole_group_mapping(
    n_holes: usize,
    hole_groups: &[Vec<u32>],
) -> HoleGroupMapping {
    if n_holes == 0 {
        return HoleGroupMapping {
            dimension_by_hole: Vec::new(),
            group_size: Vec::new(),
            n_position_dims: 1, // just bore end
        };
    }

    let mut dimension_by_hole = vec![0usize; n_holes];
    let mut group_size = vec![0.0f64; n_holes];

    // Count number of hole spaces (matching Java validateHoleGroups).
    // Each multi-hole group contributes 1 for within-group spacing.
    // Each gap between groups contributes 1.
    // Plus 1 for last-hole-to-bore-end.
    let mut n_hole_spaces: usize = 0;
    let mut current_idx: i32 = -1;
    for group in hole_groups {
        if group.len() > 1 {
            n_hole_spaces += 1; // within-group spacing
        }
        for (j, &hole_idx) in group.iter().enumerate() {
            if j == 0 && current_idx >= 0 && hole_idx as i32 != current_idx {
                n_hole_spaces += 1; // inter-group gap
            }
            current_idx = hole_idx as i32;
        }
    }
    n_hole_spaces += 1; // last hole to bore end

    // Compute dimension assignments (matching Java computeDimensionByHole).
    let mut dimension = 1; // dimension 0 is bore_end
    for group in hole_groups {
        if group.len() > 1 {
            // All holes except the last in the group share the within-group
            // spacing dimension.
            for &hole_idx in &group[..group.len() - 1] {
                dimension_by_hole[hole_idx as usize] = dimension;
                group_size[hole_idx as usize] = (group.len() - 1) as f64;
            }
            dimension += 1;
        }
        if !group.is_empty() {
            // Last hole gets the inter-group (or last-to-bore-end) dimension.
            let last_hole = *group.last().unwrap() as usize;
            dimension_by_hole[last_hole] = dimension;
            group_size[last_hole] = 1.0;
            dimension += 1;
        }
    }

    // For the FromTop variant: total position dims = 1 (bore_end) + n_hole_spaces.
    // The FromTop variant adds a top-hole ratio replacing what would be
    // the first-hole-to-? dimension.
    let n_position_dims = 1 + n_hole_spaces;

    HoleGroupMapping {
        dimension_by_hole,
        group_size,
        n_position_dims,
    }
}

/// Extract grouped hole geometry from the "FromTop" parameterization.
///
/// Returns `[bore_end, top_ratio, group_spacings..., diameters...]`.
///
/// Matches Java `HoleGroupPositionFromTopObjectiveFunction.getGeometryPoint()`
/// merged with `HoleSizeObjectiveFunction.getGeometryPoint()`.
pub fn get_hole_group_geometry_from_top(
    raw: &InstrumentRaw,
    hole_groups: &[Vec<u32>],
) -> Vec<f64> {
    let m = raw.length_type.to_metres();
    let n_holes = raw.holes.len();
    let mapping = compute_hole_group_mapping(n_holes, hole_groups);

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

    let mouthpiece_pos = raw.mouthpiece.position * m;

    let mut geometry = vec![0.0; mapping.n_position_dims];

    // [0] bore end
    geometry[0] = bore_end;

    if n_holes > 0 {
        // [1] top hole ratio (from mouthpiece to bore end)
        geometry[1] = (sorted_positions[0] - mouthpiece_pos)
            / (bore_end - mouthpiece_pos);

        // Accumulate grouped spacings from top to bottom.
        // Holes 1..n use dimensionByHole[0..n-1] to index into geometry.
        let mut prior_pos = sorted_positions[0];
        for i in 1..n_holes {
            let dim = mapping.dimension_by_hole[i - 1]; // Java: dimensionByHole[i-2] with i starting at 2
            geometry[dim + 1] += (sorted_positions[i] - prior_pos)
                / mapping.group_size[i - 1];
            prior_pos = sorted_positions[i];
        }
    }

    // Append hole diameters (sorted by position ascending)
    let diameters = get_hole_diameters(raw);
    geometry.extend_from_slice(&diameters);

    geometry
}

/// Apply grouped hole geometry from the "FromTop" parameterization.
///
/// The geometry format matches [`get_hole_group_geometry_from_top`]:
/// `[bore_end, top_ratio, group_spacings..., diameters...]`.
///
/// After calling this, you must re-compile the instrument before evaluation.
pub fn set_hole_group_geometry_from_top(
    raw: &mut InstrumentRaw,
    geometry: &[f64],
    hole_groups: &[Vec<u32>],
) {
    let m = raw.length_type.to_metres();
    let n_holes = raw.holes.len();
    let mapping = compute_hole_group_mapping(n_holes, hole_groups);

    // Sort hole indices by bore position ascending
    let mut hole_order: Vec<usize> = (0..n_holes).collect();
    hole_order.sort_by(|&a, &b| {
        raw.holes[a]
            .bore_position
            .partial_cmp(&raw.holes[b].bore_position)
            .unwrap()
    });

    // [0] bore end — update last bore point with PRESERVE_BORE (interpolate)
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

    if n_holes > 0 {
        // [1] top hole position from ratio
        let mouthpiece_pos = raw.mouthpiece.position * m;
        let bore_length_from_edge = new_bore_end - mouthpiece_pos;
        let top_hole_pos = geometry[1] * bore_length_from_edge + mouthpiece_pos;
        raw.holes[hole_order[0]].bore_position = top_hole_pos / m;

        // Reconstruct remaining hole positions from grouped spacings
        let mut prior_pos = top_hole_pos;
        for i in 1..n_holes {
            let dim = mapping.dimension_by_hole[i - 1];
            let hole_pos = prior_pos + geometry[dim + 1];
            raw.holes[hole_order[i]].bore_position = hole_pos / m;
            prior_pos = hole_pos;
        }

        // Set hole diameters from the tail of the geometry vector
        let diameter_offset = mapping.n_position_dims;
        for i in 0..n_holes {
            if diameter_offset + i < geometry.len() {
                raw.holes[hole_order[i]].diameter = geometry[diameter_offset + i] / m;
            }
        }
    }
}

// ── Taper geometry (SingleTaperSimpleRatio parameterization) ────

/// Extract taper geometry from the bore profile.
///
/// Returns `[diameter_ratio, taper_start_fraction, taper_length_fraction]`:
/// - `diameter_ratio` = head diameter / foot diameter
/// - `taper_start_fraction` = distance from top to taper start / bore length
/// - `taper_length_fraction` = taper length / (bore length below taper start)
///
/// Matches Java `SingleTaperSimpleRatioObjectiveFunction.getGeometryPoint()`.
pub fn get_taper_geometry(raw: &InstrumentRaw) -> [f64; 3] {
    let m = raw.length_type.to_metres();

    // Sort bore points by position
    let mut sorted: Vec<(f64, f64)> = raw
        .bore_points
        .iter()
        .map(|bp| (bp.bore_position * m, bp.bore_diameter * m))
        .collect();
    sorted.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    let n = sorted.len();
    let (top_pos, top_dia) = sorted[0];
    let (next_pos, next_dia) = sorted[1];
    let (pen_pos, pen_dia) = sorted[n - 2];
    let (bot_pos, bot_dia) = sorted[n - 1];
    let bore_length = bot_pos - top_pos;

    let ratio = top_dia / bot_dia;

    let (taper_start, taper_end);
    if (top_dia - bot_dia).abs() < 0.0001 {
        // Bore doesn't really taper
        taper_start = top_pos;
        taper_end = bot_pos;
    } else {
        taper_start = if (top_dia - next_dia).abs() < 0.0001 {
            next_pos // taper starts on second point
        } else {
            top_pos // taper starts on first point
        };
        taper_end = if (bot_dia - pen_dia).abs() < 0.0001 {
            pen_pos // taper ends on second-last point
        } else {
            bot_pos // taper ends on bottom point
        };
    }

    let start_frac = (taper_start - top_pos) / bore_length;
    let length_frac = (taper_end - taper_start)
        / (bore_length - taper_start + top_pos);

    [ratio, start_frac, length_frac]
}

/// Apply taper geometry to a raw instrument, replacing all bore points.
///
/// `taper` = `[diameter_ratio, taper_start_fraction, taper_length_fraction]`.
///
/// Creates 2–4 bore points defining a single taper profile. The foot
/// (bottom) diameter is invariant — the head diameter is `foot * ratio`.
///
/// Matches Java `SingleTaperSimpleRatioObjectiveFunction.setGeometryPoint()`.
pub fn set_taper_geometry(raw: &mut InstrumentRaw, taper: &[f64; 3]) {
    let m = raw.length_type.to_metres();

    // Get current top and bottom positions
    let mut sorted: Vec<(f64, f64)> = raw
        .bore_points
        .iter()
        .map(|bp| (bp.bore_position * m, bp.bore_diameter * m))
        .collect();
    sorted.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    let top_pos = sorted[0].0;
    let bot_pos = sorted.last().unwrap().0;
    let foot_diameter = sorted.last().unwrap().1;
    let head_diameter = foot_diameter * taper[0];
    let bore_length = bot_pos - top_pos;
    let taper_start = taper[1] * bore_length;
    let taper_length = (taper[2] * (bore_length - taper_start)).max(MINIMUM_CONE_LENGTH);

    let mut new_points = Vec::with_capacity(4);

    // Head point
    new_points.push(wid_types::BorePointRaw {
        name: None,
        bore_position: top_pos / m,
        bore_diameter: head_diameter / m,
    });

    // Optional: taper starts below head
    if taper_start > 0.0 {
        let start_pos = (top_pos + taper_start).min(bot_pos);
        new_points.push(wid_types::BorePointRaw {
            name: None,
            bore_position: start_pos / m,
            bore_diameter: head_diameter / m,
        });
    }

    // Taper end point
    let taper_end = (taper_start + taper_length).min(bore_length);
    new_points.push(wid_types::BorePointRaw {
        name: None,
        bore_position: (top_pos + taper_end) / m,
        bore_diameter: foot_diameter / m,
    });

    // Optional: taper ends above foot
    if taper_start + taper_length < bore_length {
        new_points.push(wid_types::BorePointRaw {
            name: None,
            bore_position: bot_pos / m,
            bore_diameter: foot_diameter / m,
        });
    }

    raw.bore_points = new_points;
}

/// Set the bore end position with MOVE_BOTTOM semantics.
///
/// Unlike PRESERVE_TAPER (which interpolates the diameter at the new position),
/// MOVE_BOTTOM simply moves the last bore point position without changing its
/// diameter. Used by taper optimizers where the bore profile is handled
/// separately by `set_taper_geometry`.
pub fn set_bore_end_move_bottom(raw: &mut InstrumentRaw, new_bore_end: f64) {
    let m = raw.length_type.to_metres();
    if let Some(last_bp) = raw
        .bore_points
        .iter_mut()
        .max_by(|a, b| a.bore_position.partial_cmp(&b.bore_position).unwrap())
    {
        last_bp.bore_position = new_bore_end / m;
    }
}

// ── Bore length adjustment ──────────────────────────────────────

/// How the bore end position is adjusted when hole positions change bore length.
///
/// Matches Java `BoreLengthAdjustmentInterface.BoreLengthAdjustmentType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoreLengthAdjust {
    /// Interpolate/extrapolate bore diameter at new end position to maintain taper.
    PreserveTaper,
    /// Move bottom bore point position without changing its diameter.
    MoveBottom,
    /// Shift all bore points from the bell index downward by the net change.
    /// Diameters are preserved (bell shape maintained).
    PreserveBell,
}

/// Minimum spacing between bore points to prevent overlap (metres).
pub const MINIMUM_BORE_POINT_SPACING: f64 = 0.00001;

/// Set the bore end position using the specified adjustment mode.
///
/// - `PreserveTaper`: interpolate bore diameter at new position
/// - `MoveBottom`: just move last bore point, keep diameter
/// - `PreserveBell`: shift all bore points from bell index downward
pub fn set_bore_end_adjusted(
    raw: &mut InstrumentRaw,
    new_bore_end: f64,
    adjust: BoreLengthAdjust,
) {
    match adjust {
        BoreLengthAdjust::PreserveTaper => {
            let m = raw.length_type.to_metres();
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
        }
        BoreLengthAdjust::MoveBottom => {
            set_bore_end_move_bottom(raw, new_bore_end);
        }
        BoreLengthAdjust::PreserveBell => {
            let m = raw.length_type.to_metres();
            let mut sorted_indices: Vec<usize> = (0..raw.bore_points.len()).collect();
            sorted_indices.sort_by(|&a, &b| {
                raw.bore_points[a]
                    .bore_position
                    .partial_cmp(&raw.bore_points[b].bore_position)
                    .unwrap()
            });

            let last_sorted = *sorted_indices.last().unwrap();
            let old_bore_end = raw.bore_points[last_sorted].bore_position * m;
            let net_change = new_bore_end - old_bore_end;

            let bell_index = find_bell_sorted(raw, &sorted_indices);

            // Shift all bore points from bell index downward (in reverse to
            // avoid cascading overlap issues).
            for si in (bell_index..sorted_indices.len()).rev() {
                let idx = sorted_indices[si];
                let mut new_pos = raw.bore_points[idx].bore_position * m + net_change;
                // Prevent overlap with point above
                if si > 0 {
                    let above_idx = sorted_indices[si - 1];
                    let above_pos = raw.bore_points[above_idx].bore_position * m;
                    if new_pos < above_pos + MINIMUM_BORE_POINT_SPACING {
                        new_pos = above_pos + MINIMUM_BORE_POINT_SPACING;
                    }
                }
                raw.bore_points[idx].bore_position = new_pos / m;
            }
        }
    }
}

/// Set hole positions from a HolePosition geometry vector with a specified
/// bore length adjustment mode.
///
/// Like [`set_hole_geometry_position`] but uses the given [`BoreLengthAdjust`]
/// mode instead of always using PRESERVE_TAPER.
pub fn set_hole_positions_adjusted(
    raw: &mut InstrumentRaw,
    geometry: &[f64],
    adjust: BoreLengthAdjust,
) {
    let m = raw.length_type.to_metres();
    let n_holes = raw.holes.len();

    let mut hole_order: Vec<usize> = (0..n_holes).collect();
    hole_order.sort_by(|&a, &b| {
        raw.holes[a]
            .bore_position
            .partial_cmp(&raw.holes[b].bore_position)
            .unwrap()
    });

    let new_bore_end = geometry[0];
    set_bore_end_adjusted(raw, new_bore_end, adjust);

    // Reconstruct hole positions from spacings
    let mut prior_pos = new_bore_end;
    for i in (0..n_holes).rev() {
        let hole_pos = prior_pos - geometry[i + 1];
        raw.holes[hole_order[i]].bore_position = hole_pos / m;
        prior_pos = hole_pos;
    }
}

// ── Bore point identification utilities ─────────────────────────

/// Find the bell start index: the bore point after the longest segment.
///
/// Matches Java `BoreLengthAdjusterPreserveBell.findBell()`. The algorithm
/// tracks the longest bore segment and returns the index of the bore point
/// at the bottom of that segment.
///
/// Returns an index in the position-sorted bore point array.
pub fn find_bell(raw: &InstrumentRaw) -> usize {
    let mut sorted_indices: Vec<usize> = (0..raw.bore_points.len()).collect();
    sorted_indices.sort_by(|&a, &b| {
        raw.bore_points[a]
            .bore_position
            .partial_cmp(&raw.bore_points[b].bore_position)
            .unwrap()
    });
    find_bell_sorted(raw, &sorted_indices)
}

/// Internal: find bell index given pre-sorted indices.
fn find_bell_sorted(raw: &InstrumentRaw, sorted_indices: &[usize]) -> usize {
    let m = raw.length_type.to_metres();
    let n = sorted_indices.len();
    if n <= 1 {
        return 0;
    }

    let mut longest_segment = 0.0;
    let mut last_position = raw.bore_points[sorted_indices[0]].bore_position * m;
    let mut bell_index = n - 1;

    for si in 1..n {
        let pos = raw.bore_points[sorted_indices[si]].bore_position * m;
        if pos - last_position >= longest_segment {
            bell_index = si;
            // Match Java: stores absolute position instead of segment length.
            // This is a known Java quirk that we replicate for parity.
            longest_segment = pos;
        }
        last_position = pos;
    }

    bell_index
}

/// Find a bore point by name (case-insensitive substring match).
///
/// Returns the index in position-sorted order.
/// If `find_last` is true, returns the highest-position match; otherwise lowest.
fn find_bore_point_by_name(
    raw: &InstrumentRaw,
    name: &str,
    find_last: bool,
) -> Option<usize> {
    let m = raw.length_type.to_metres();
    let mut indexed: Vec<(usize, f64, &Option<String>)> = raw
        .bore_points
        .iter()
        .enumerate()
        .map(|(i, bp)| (i, bp.bore_position * m, &bp.name))
        .collect();
    indexed.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

    let name_lower = name.to_lowercase();
    let matches: Vec<usize> = indexed
        .iter()
        .enumerate()
        .filter(|(_, (_, _, bp_name))| {
            bp_name
                .as_ref()
                .map(|n| n.to_lowercase().contains(&name_lower))
                .unwrap_or(false)
        })
        .map(|(sorted_idx, _)| sorted_idx)
        .collect();

    if matches.is_empty() {
        None
    } else if find_last {
        Some(*matches.last().unwrap())
    } else {
        Some(matches[0])
    }
}

/// Find the lowest head bore point — the boundary between headjoint and body.
///
/// Used by `BoreDiameterFromTop` to determine how many bore points from the
/// top are optimized (`n_changed`).
///
/// Search strategy (matches Java `BoreDiameterFromTopObjectiveFunction.getLowestPoint()`):
/// 1. Find the highest bore point whose name contains `point_name` (default "Head")
/// 2. Fallback: find the lowest bore point above the top tonehole
/// 3. Final fallback: return 0
pub fn find_head_point(raw: &InstrumentRaw, point_name: &str) -> usize {
    // Strategy 1: find by name (last/highest match)
    if let Some(idx) = find_bore_point_by_name(raw, point_name, true) {
        return idx;
    }

    // Strategy 2: fallback heuristic based on hole positions
    let m = raw.length_type.to_metres();
    let mut sorted_pos: Vec<f64> = raw
        .bore_points
        .iter()
        .map(|bp| bp.bore_position * m)
        .collect();
    sorted_pos.sort_by(|a, b| a.partial_cmp(b).unwrap());

    if sorted_pos.len() <= 2 {
        return 0;
    }

    let top_hole_pos = if !raw.holes.is_empty() {
        raw.holes
            .iter()
            .map(|h| h.bore_position * m)
            .fold(f64::INFINITY, f64::min)
    } else {
        // No holes: use midpoint of bore
        0.5 * (sorted_pos[0] + sorted_pos.last().unwrap())
    };

    // Find lowest bore point above top hole (scan from bottom up, skip endpoints)
    for idx in (1..sorted_pos.len() - 1).rev() {
        if sorted_pos[idx] < top_hole_pos {
            return idx;
        }
    }

    0
}

/// Find the top of the body section — first body bore point index.
///
/// Used by `BoreDiameterFromBottom` to determine how many bore points from
/// the top are unchanged (`n_unchanged`).
///
/// Search strategy (matches Java `BoreDiameterFromBottomObjectiveFunction.getTopOfBody()`):
/// 1. Find lowest bore point named "Body"
/// 2. Find highest bore point named "Head"
/// 3. Fallback: lowest bore point above top tonehole
/// 4. Final fallback: return 0
pub fn find_body_top(raw: &InstrumentRaw) -> usize {
    let m = raw.length_type.to_metres();

    if raw.bore_points.len() <= 2 {
        return 0;
    }

    // Strategy 1: find lowest "Body" point
    if let Some(idx) = find_bore_point_by_name(raw, "Body", false) {
        return idx;
    }

    // Strategy 2: find highest "Head" point
    if let Some(idx) = find_bore_point_by_name(raw, "Head", true) {
        return idx;
    }

    // Strategy 3: fallback heuristic based on hole positions
    let mut sorted_pos: Vec<f64> = raw
        .bore_points
        .iter()
        .map(|bp| bp.bore_position * m)
        .collect();
    sorted_pos.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let top_hole_pos = if !raw.holes.is_empty() {
        raw.holes
            .iter()
            .map(|h| h.bore_position * m)
            .fold(f64::INFINITY, f64::min)
    } else {
        0.5 * (sorted_pos[0] + sorted_pos.last().unwrap())
    };

    for idx in (1..sorted_pos.len() - 1).rev() {
        if sorted_pos[idx] < top_hole_pos {
            return idx;
        }
    }

    0
}

// ── Bore diameter from top ──────────────────────────────────────

/// Extract bore diameter ratios from the top of the bore.
///
/// Returns `n_changed` ratios where `ratio[i] = diameter[i] / diameter[i+1]`,
/// working from the topmost bore point downward. The reference point (first
/// unchanged point) is at sorted index `n_changed`.
///
/// Matches Java `BoreDiameterFromTopObjectiveFunction.getGeometryPoint()`.
pub fn get_bore_diameter_from_top(raw: &InstrumentRaw, n_changed: usize) -> Vec<f64> {
    let m = raw.length_type.to_metres();
    let mut sorted: Vec<(f64, f64)> = raw
        .bore_points
        .iter()
        .map(|bp| (bp.bore_position * m, bp.bore_diameter * m))
        .collect();
    sorted.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    let mut geometry = vec![0.0; n_changed];
    for i in (0..n_changed).rev() {
        let next_dia = sorted[i + 1].1.max(0.000001);
        geometry[i] = sorted[i].1 / next_dia;
    }
    geometry
}

/// Apply bore diameter ratios from the top.
///
/// Each `ratios[i]` is multiplied by the next bore point's diameter to set
/// bore point `i`'s diameter. The reference is the first unchanged point
/// at sorted index `n_changed`.
///
/// Matches Java `BoreDiameterFromTopObjectiveFunction.setGeometryPoint()`.
pub fn set_bore_diameter_from_top(
    raw: &mut InstrumentRaw,
    ratios: &[f64],
    n_changed: usize,
) {
    let m = raw.length_type.to_metres();
    let mut sorted_indices: Vec<usize> = (0..raw.bore_points.len()).collect();
    sorted_indices.sort_by(|&a, &b| {
        raw.bore_points[a]
            .bore_position
            .partial_cmp(&raw.bore_points[b].bore_position)
            .unwrap()
    });

    // Reference diameter from the first unchanged point
    let ref_idx = sorted_indices[n_changed];
    let mut next_dia = raw.bore_points[ref_idx].bore_diameter * m;

    for i in (0..n_changed).rev() {
        let idx = sorted_indices[i];
        let new_dia = ratios[i] * next_dia;
        raw.bore_points[idx].bore_diameter = new_dia / m;
        next_dia = new_dia;
    }
}

// ── Bore diameter from bottom ───────────────────────────────────

/// Extract bore diameter ratios from the bottom of the bore.
///
/// Returns `n_dims` ratios (where `n_dims = total_points - n_unchanged`)
/// where `ratio[i] = diameter[unchanged+i] / diameter[unchanged+i-1]`,
/// working upward from the first changing point.
///
/// Matches Java `BoreDiameterFromBottomObjectiveFunction.getGeometryPoint()`.
pub fn get_bore_diameter_from_bottom(raw: &InstrumentRaw, n_unchanged: usize) -> Vec<f64> {
    let m = raw.length_type.to_metres();
    let mut sorted: Vec<(f64, f64)> = raw
        .bore_points
        .iter()
        .map(|bp| (bp.bore_position * m, bp.bore_diameter * m))
        .collect();
    sorted.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    let n_dims = sorted.len() - n_unchanged;
    let mut geometry = vec![0.0; n_dims];
    let mut prior_dia = sorted[n_unchanged - 1].1;

    for i in 0..n_dims {
        let dia = sorted[n_unchanged + i].1;
        geometry[i] = dia / prior_dia.max(0.000001);
        prior_dia = dia;
    }
    geometry
}

/// Apply bore diameter ratios from the bottom.
///
/// Applies ratios to bore points from `n_unchanged` downward, using the
/// last unchanged point as reference. Also adjusts the termination flange
/// diameter to preserve flange overhang.
///
/// Matches Java `BoreDiameterFromBottomObjectiveFunction.setGeometryPoint()`.
pub fn set_bore_diameter_from_bottom(
    raw: &mut InstrumentRaw,
    ratios: &[f64],
    n_unchanged: usize,
) {
    let m = raw.length_type.to_metres();
    let mut sorted_indices: Vec<usize> = (0..raw.bore_points.len()).collect();
    sorted_indices.sort_by(|&a, &b| {
        raw.bore_points[a]
            .bore_position
            .partial_cmp(&raw.bore_points[b].bore_position)
            .unwrap()
    });

    // Save original termination (bottom) diameter before changes
    let last_idx = *sorted_indices.last().unwrap();
    let old_term_dia = raw.bore_points[last_idx].bore_diameter * m;

    // Reference diameter from last unchanged point
    let ref_idx = sorted_indices[n_unchanged - 1];
    let mut prior_dia = raw.bore_points[ref_idx].bore_diameter * m;

    for i in 0..ratios.len() {
        let idx = sorted_indices[n_unchanged + i];
        let new_dia = ratios[i] * prior_dia;
        raw.bore_points[idx].bore_diameter = new_dia / m;
        prior_dia = new_dia;
    }

    // Adjust flange diameter to preserve overhang width.
    // flange_overhang = flange_dia - bore_dia  (constant)
    // new_flange = old_flange + (new_bore_dia - old_bore_dia)
    let new_term_dia = raw.bore_points[last_idx].bore_diameter * m;
    let delta = new_term_dia - old_term_dia;
    raw.termination.flange_diameter += delta / m;
}

// ── Bore spacing from top ───────────────────────────────────────

/// Extract absolute bore point spacings from the top.
///
/// Returns `n_changed` spacings where `spacing[i]` is the distance between
/// bore points at sorted indices `i` and `i+1`.
///
/// Matches Java `BoreSpacingFromTopObjectiveFunction.getGeometryPoint()`.
pub fn get_bore_spacing_from_top(raw: &InstrumentRaw, n_changed: usize) -> Vec<f64> {
    let m = raw.length_type.to_metres();
    let mut sorted: Vec<f64> = raw
        .bore_points
        .iter()
        .map(|bp| bp.bore_position * m)
        .collect();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let mut geometry = vec![0.0; n_changed];
    for i in 0..n_changed {
        geometry[i] = sorted[i + 1] - sorted[i];
    }
    geometry
}

/// Apply absolute bore point spacings from the top.
///
/// Sets positions of bore points at sorted indices `1..=n_changed` based
/// on cumulative spacings from the top bore point.
///
/// Matches Java `BoreSpacingFromTopObjectiveFunction.setGeometryPoint()`.
pub fn set_bore_spacing_from_top(
    raw: &mut InstrumentRaw,
    spacings: &[f64],
    n_changed: usize,
) {
    let m = raw.length_type.to_metres();
    let mut sorted_indices: Vec<usize> = (0..raw.bore_points.len()).collect();
    sorted_indices.sort_by(|&a, &b| {
        raw.bore_points[a]
            .bore_position
            .partial_cmp(&raw.bore_points[b].bore_position)
            .unwrap()
    });

    let top_pos = raw.bore_points[sorted_indices[0]].bore_position * m;
    let mut prior_pos = top_pos;

    for i in 0..n_changed {
        let idx = sorted_indices[i + 1];
        let new_pos = prior_pos + spacings[i];
        raw.bore_points[idx].bore_position = new_pos / m;
        prior_pos = new_pos;
    }
}

/// Clamp bore spacing upper bounds to prevent bore point reordering.
///
/// If the sum of upper bounds would push bore points past the available space
/// (between top bore point and the first bore point after the optimized range),
/// scales all upper bounds proportionally.
///
/// Matches Java `BoreSpacingFromTopObjectiveFunction.setUpperBounds()`.
pub fn clamp_bore_spacing_upper_bounds(
    raw: &InstrumentRaw,
    n_changed: usize,
    upper_bounds: &mut [f64],
) {
    let m = raw.length_type.to_metres();
    let mut sorted_pos: Vec<f64> = raw
        .bore_points
        .iter()
        .map(|bp| bp.bore_position * m)
        .collect();
    sorted_pos.sort_by(|a, b| a.partial_cmp(b).unwrap());

    // Only clamp if there are bore points below the optimized range
    if n_changed + 1 >= sorted_pos.len() {
        return;
    }

    let available = sorted_pos[n_changed + 1] - sorted_pos[0];
    let total_upper: f64 = upper_bounds.iter().sum();

    // Java uses epsilon offset: triggers when sum + 0.0001 > available,
    // and scales by available / (sum + 0.0001) to ensure strict feasibility.
    if total_upper + 0.0001 > available {
        let scale = available / (total_upper + 0.0001);
        for ub in upper_bounds.iter_mut() {
            *ub *= scale;
        }
    }
}

// ── Bore position ───────────────────────────────────────────────

/// Extract bore position geometry as fractional positions.
///
/// If `bottom_fixed` is false, the first dimension is the absolute bottom
/// bore point position. Remaining dimensions are fractional positions of
/// interior bore points relative to the available space.
///
/// `n_unchanged` = number of bore points from the top that are fixed.
///
/// Matches Java `BorePositionObjectiveFunction.getGeometryPoint()`.
pub fn get_bore_position(
    raw: &InstrumentRaw,
    n_unchanged: usize,
    bottom_fixed: bool,
) -> Vec<f64> {
    let m = raw.length_type.to_metres();
    let mut sorted: Vec<f64> = raw
        .bore_points
        .iter()
        .map(|bp| bp.bore_position * m)
        .collect();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let n = sorted.len();
    let unchanged_bottom: usize = if bottom_fixed { 1 } else { 0 };
    let n_dims = n - n_unchanged - unchanged_bottom;

    let mut geometry = Vec::with_capacity(n_dims);
    let last_pos = *sorted.last().unwrap();

    let mut dim = 0;

    if !bottom_fixed {
        // First dimension: absolute bottom position
        geometry.push(last_pos);
        dim = 1;
    }

    // Remaining dimensions: fractional positions
    let mut prior_pos = sorted[n_unchanged - 1];
    for d in dim..n_dims {
        let bp_idx = if bottom_fixed {
            n_unchanged + d
        } else {
            n_unchanged + (d - 1)
        };
        if bp_idx < n {
            let frac = (sorted[bp_idx] - prior_pos) / (last_pos - prior_pos);
            geometry.push(frac);
            prior_pos = sorted[bp_idx];
        }
    }

    geometry
}

/// Apply bore position geometry (fractional positions).
///
/// Inverse of [`get_bore_position`].
///
/// Matches Java `BorePositionObjectiveFunction.setGeometryPoint()`.
pub fn set_bore_position(
    raw: &mut InstrumentRaw,
    positions: &[f64],
    n_unchanged: usize,
    bottom_fixed: bool,
) {
    let m = raw.length_type.to_metres();
    let mut sorted_indices: Vec<usize> = (0..raw.bore_points.len()).collect();
    sorted_indices.sort_by(|&a, &b| {
        raw.bore_points[a]
            .bore_position
            .partial_cmp(&raw.bore_points[b].bore_position)
            .unwrap()
    });

    let last_sorted_idx = *sorted_indices.last().unwrap();
    let mut last_pos = raw.bore_points[last_sorted_idx].bore_position * m;

    let mut dim = 0;

    if !bottom_fixed {
        // First dimension: absolute bottom position
        last_pos = positions[0];
        raw.bore_points[last_sorted_idx].bore_position = last_pos / m;
        dim = 1;
    }

    // Remaining dimensions: fractional positions
    let ref_idx = sorted_indices[n_unchanged - 1];
    let mut prior_pos = raw.bore_points[ref_idx].bore_position * m;

    let n_fracs = positions.len() - dim;
    for d in 0..n_fracs {
        let bp_sorted_idx = n_unchanged + d;
        if bp_sorted_idx < sorted_indices.len() {
            let idx = sorted_indices[bp_sorted_idx];
            let new_pos = prior_pos + positions[dim + d] * (last_pos - prior_pos);
            raw.bore_points[idx].bore_position = new_pos / m;
            prior_pos = new_pos;
        }
    }
}

// ── Basic taper (2D) ────────────────────────────────────────────

/// Extract basic taper geometry: `[head_length_fraction, foot_diameter_ratio]`.
///
/// A simple 2-section taper model with 3 bore points:
/// - Top point (fixed position and diameter)
/// - Middle point (position varies, diameter fixed)
/// - Bottom point (position fixed, diameter varies)
///
/// `head_length_fraction` = (middle position - top position) / bore length
/// `foot_diameter_ratio` = bottom diameter / middle diameter
///
/// Matches Java `BasicTaperObjectiveFunction.getGeometryPoint()`.
pub fn get_basic_taper(raw: &InstrumentRaw) -> [f64; 2] {
    let m = raw.length_type.to_metres();
    let mut sorted: Vec<(f64, f64)> = raw
        .bore_points
        .iter()
        .map(|bp| (bp.bore_position * m, bp.bore_diameter * m))
        .collect();
    sorted.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    let top = sorted[0];
    let middle = sorted[1];
    let bottom = sorted.last().unwrap();
    let bore_length = bottom.0 - top.0;

    let head_frac = if bore_length > 0.0 {
        (middle.0 - top.0) / bore_length
    } else {
        0.0
    };
    let foot_ratio = if middle.1 > 0.0 {
        bottom.1 / middle.1
    } else {
        1.0
    };

    [head_frac, foot_ratio]
}

/// Apply basic taper geometry, replacing all bore points with 3 new points.
///
/// `taper[0]` = head length fraction (middle point position)
/// `taper[1]` = foot diameter ratio (bottom diameter / middle diameter)
///
/// Top point: original position, original diameter (from sorted[0])
/// Middle point: new position, original diameter (from sorted[1])
/// Bottom point: original position, new diameter = middle_dia * taper[1]
///
/// Matches Java `BasicTaperObjectiveFunction.setGeometryPoint()`.
pub fn set_basic_taper(raw: &mut InstrumentRaw, taper: &[f64; 2]) {
    let m = raw.length_type.to_metres();
    let mut sorted: Vec<(f64, f64)> = raw
        .bore_points
        .iter()
        .map(|bp| (bp.bore_position * m, bp.bore_diameter * m))
        .collect();
    sorted.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    let top_pos = sorted[0].0;
    let top_dia = sorted[0].1;
    let middle_dia = sorted[1].1;
    let bottom_pos = sorted.last().unwrap().0;
    let bore_length = bottom_pos - top_pos;

    let middle_pos = top_pos + bore_length * taper[0];
    let bottom_dia = middle_dia * taper[1];

    raw.bore_points = vec![
        wid_types::BorePointRaw {
            name: None,
            bore_position: top_pos / m,
            bore_diameter: top_dia / m,
        },
        wid_types::BorePointRaw {
            name: None,
            bore_position: middle_pos / m,
            bore_diameter: middle_dia / m,
        },
        wid_types::BorePointRaw {
            name: None,
            bore_position: bottom_pos / m,
            bore_diameter: bottom_dia / m,
        },
    ];
}

// ── Stopper position (Flute) ────────────────────────────────────

/// Extract stopper position: distance from top bore point to embouchure hole edge.
///
/// For embouchure hole instruments, the distance is measured from the top bore
/// point to the upper edge of the embouchure hole (center - half length).
///
/// Matches Java `StopperPositionObjectiveFunction.getGeometryPoint()`.
pub fn get_stopper_position(raw: &InstrumentRaw) -> f64 {
    let m = raw.length_type.to_metres();
    let top_pos = raw
        .bore_points
        .iter()
        .map(|bp| bp.bore_position * m)
        .fold(f64::INFINITY, f64::min);
    let mp_pos = raw.mouthpiece.position * m;

    let mut distance = mp_pos - top_pos;
    if let Some(ref emb) = raw.mouthpiece.embouchure_hole {
        distance -= 0.5 * emb.length * m;
    }
    distance
}

/// Apply stopper position: set the distance from top bore point to embouchure hole.
///
/// Moves the top bore point (and potentially adjacent bore points that collide).
/// If `preserve_taper` is true, bore diameter is interpolated at the new position.
///
/// Matches Java `StopperPositionObjectiveFunction.setGeometryPoint()`.
pub fn set_stopper_position(
    raw: &mut InstrumentRaw,
    distance: f64,
    preserve_taper: bool,
) {
    let m = raw.length_type.to_metres();
    let mp_pos = raw.mouthpiece.position * m;

    let mut new_top = mp_pos - distance;
    if let Some(ref emb) = raw.mouthpiece.embouchure_hole {
        new_top -= 0.5 * emb.length * m;
    }

    // Sort bore points by position to find top
    let mut sorted_indices: Vec<usize> = (0..raw.bore_points.len()).collect();
    sorted_indices.sort_by(|&a, &b| {
        raw.bore_points[a]
            .bore_position
            .partial_cmp(&raw.bore_points[b].bore_position)
            .unwrap()
    });

    // Interpolate diameter at new position if preserve_taper
    if preserve_taper {
        if let Some(new_dia) = interpolate_bore_diameter(&raw.bore_points, new_top, m) {
            let top_idx = sorted_indices[0];
            raw.bore_points[top_idx].bore_diameter = new_dia / m;
        }
    }

    // Set top bore point position
    let top_idx = sorted_indices[0];
    raw.bore_points[top_idx].bore_position = new_top / m;

    // Prevent collision with bore points below. Walk downward and push
    // any points that are now at or above the new top position.
    let mut current_top = new_top;
    for si in 1..sorted_indices.len() - 1 {
        let idx = sorted_indices[si];
        let pos = raw.bore_points[idx].bore_position * m;
        if pos <= current_top {
            current_top += MINIMUM_BORE_POINT_SPACING;
            if preserve_taper {
                if let Some(new_dia) =
                    interpolate_bore_diameter(&raw.bore_points, current_top, m)
                {
                    raw.bore_points[idx].bore_diameter = new_dia / m;
                }
            }
            raw.bore_points[idx].bore_position = current_top / m;
        } else {
            break;
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
    // Reject non-finite bore point values (guards partial_cmp().unwrap() in sorting)
    for (i, bp) in raw.bore_points.iter().enumerate() {
        if !bp.bore_position.is_finite() || !bp.bore_diameter.is_finite() {
            errors.push(format!(
                "Bore point {} has non-finite position or diameter.",
                i + 1
            ));
        }
        if bp.bore_diameter <= 0.0 {
            errors.push(format!(
                "Bore point {} diameter must be positive.",
                i + 1
            ));
        }
    }
    // Reject non-finite hole values
    for hole in &raw.holes {
        let name = hole.name.as_deref().unwrap_or("unnamed");
        if !hole.bore_position.is_finite() || !hole.diameter.is_finite() || !hole.height.is_finite()
        {
            errors.push(format!("Hole '{}' has non-finite geometry.", name));
        }
    }
    // Reject non-finite mouthpiece position
    if !raw.mouthpiece.position.is_finite() {
        errors.push("Mouthpiece position is non-finite.".to_string());
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

    // ── Grouped hole geometry tests ──────────────────────────────

    #[test]
    fn grouped_mapping_6hole_2groups() {
        // 6-hole NAF with groups {0,1,2} and {3,4,5}
        let groups = vec![vec![0, 1, 2], vec![3, 4, 5]];
        let mapping = compute_hole_group_mapping(6, &groups);

        // dimensionByHole: holes 0,1 share dim 1 (group0 spacing),
        // hole 2 gets dim 2 (inter-group gap), holes 3,4 share dim 3
        // (group1 spacing), hole 5 gets dim 4 (last-to-bore-end)
        assert_eq!(mapping.dimension_by_hole, vec![1, 1, 2, 3, 3, 4]);
        assert_eq!(mapping.group_size, vec![2.0, 2.0, 1.0, 2.0, 2.0, 1.0]);
        // n_position_dims = 1 (bore_end) + 4 (hole spaces) = 5
        assert_eq!(mapping.n_position_dims, 5);
    }

    #[test]
    fn grouped_mapping_7hole_3groups() {
        // 7-hole NAF with groups {0,1,2}, {3,4,5}, {6}
        let groups = vec![vec![0, 1, 2], vec![3, 4, 5], vec![6]];
        let mapping = compute_hole_group_mapping(7, &groups);

        assert_eq!(mapping.dimension_by_hole, vec![1, 1, 2, 3, 3, 4, 5]);
        assert_eq!(mapping.group_size, vec![2.0, 2.0, 1.0, 2.0, 2.0, 1.0, 1.0]);
        assert_eq!(mapping.n_position_dims, 6);
    }

    #[test]
    fn grouped_geometry_length_6hole_2groups() {
        let raw = parse_instrument_xml(NAF_6HOLE_XML).unwrap();
        let groups = vec![vec![0, 1, 2], vec![3, 4, 5]];
        let geom = get_hole_group_geometry_from_top(&raw, &groups);

        // position dims (5) + diameter dims (6) = 11
        assert_eq!(geom.len(), 11);

        // [0] = bore end (same as HoleFromTop)
        assert_abs_diff_eq!(geom[0], GOLDEN_INITIAL_GEOMETRY[0], epsilon = 1e-10);

        // [1] = top hole ratio (same as HoleFromTop)
        assert_abs_diff_eq!(geom[1], GOLDEN_INITIAL_GEOMETRY[1], epsilon = 1e-10);

        // Diameters at [5..11] should match HoleFromTop diameters at [7..13]
        for i in 0..6 {
            assert_abs_diff_eq!(
                geom[5 + i],
                GOLDEN_INITIAL_GEOMETRY[7 + i],
                epsilon = 1e-10,
            );
        }
    }

    #[test]
    fn grouped_geometry_roundtrips_6hole() {
        let mut raw = parse_instrument_xml(NAF_6HOLE_XML).unwrap();
        let groups = vec![vec![0, 1, 2], vec![3, 4, 5]];

        let original = get_hole_group_geometry_from_top(&raw, &groups);
        set_hole_group_geometry_from_top(&mut raw, &original, &groups);
        let roundtripped = get_hole_group_geometry_from_top(&raw, &groups);

        assert_eq!(original.len(), roundtripped.len());
        for (i, (a, b)) in original.iter().zip(roundtripped.iter()).enumerate() {
            assert!(
                (a - b).abs() < 1e-12,
                "grouped roundtrip mismatch at [{i}]: {a} vs {b}"
            );
        }
    }

    #[test]
    fn grouped_set_changes_diameters() {
        let mut raw = parse_instrument_xml(NAF_6HOLE_XML).unwrap();
        let groups = vec![vec![0, 1, 2], vec![3, 4, 5]];

        let mut geom = get_hole_group_geometry_from_top(&raw, &groups);
        let n_pos = 5; // position dims
        // Double all diameters
        for i in 0..6 {
            geom[n_pos + i] *= 2.0;
        }
        set_hole_group_geometry_from_top(&mut raw, &geom, &groups);
        let new_geom = get_hole_group_geometry_from_top(&raw, &groups);

        for i in 0..6 {
            let original_dia = GOLDEN_INITIAL_GEOMETRY[7 + i]; // from HoleFromTop format
            assert_abs_diff_eq!(
                new_geom[n_pos + i],
                original_dia * 2.0,
                epsilon = 1e-12
            );
        }
    }

    // ── Taper geometry tests ──────────────────────────────────────

    #[test]
    fn taper_geometry_roundtrips() {
        let mut raw = parse_instrument_xml(NAF_6HOLE_XML).unwrap();
        let original = get_taper_geometry(&raw);

        // ratio ~1.0 for cylindrical bore
        assert_abs_diff_eq!(original[0], 1.0, epsilon = 0.01);

        // Set a known taper and round-trip
        let taper = [1.2, 0.1, 0.8];
        set_taper_geometry(&mut raw, &taper);
        let recovered = get_taper_geometry(&raw);

        assert_abs_diff_eq!(recovered[0], taper[0], epsilon = 1e-10);
        assert_abs_diff_eq!(recovered[1], taper[1], epsilon = 1e-6);
        assert_abs_diff_eq!(recovered[2], taper[2], epsilon = 1e-6);
    }

    #[test]
    fn taper_set_creates_4_bore_points_for_mid_taper() {
        let mut raw = parse_instrument_xml(NAF_6HOLE_XML).unwrap();
        // Taper starts at 20% bore, extends 50% of remaining = 40% of bore
        // Should create 4 points: head, taper_start, taper_end, foot
        set_taper_geometry(&mut raw, &[1.3, 0.2, 0.5]);
        assert_eq!(raw.bore_points.len(), 4);
    }

    #[test]
    fn taper_set_creates_2_bore_points_for_full_taper() {
        let mut raw = parse_instrument_xml(NAF_6HOLE_XML).unwrap();
        // Taper starts at top (0.0), extends full bore length (1.0)
        // Should create 2 points: head and foot
        set_taper_geometry(&mut raw, &[1.3, 0.0, 1.0]);
        assert_eq!(raw.bore_points.len(), 2);
    }

    #[test]
    fn taper_set_preserves_bore_positions() {
        let mut raw = parse_instrument_xml(NAF_6HOLE_XML).unwrap();
        let m = raw.length_type.to_metres();

        // Get original top and bottom positions
        let mut sorted_pos: Vec<f64> = raw.bore_points.iter().map(|bp| bp.bore_position * m).collect();
        sorted_pos.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let top_pos = sorted_pos[0];
        let bot_pos = *sorted_pos.last().unwrap();

        set_taper_geometry(&mut raw, &[1.2, 0.1, 0.8]);

        // Top and bottom positions should be preserved
        let new_sorted: Vec<f64> = raw.bore_points.iter().map(|bp| bp.bore_position * m).collect();
        let new_min = new_sorted.iter().cloned().fold(f64::INFINITY, f64::min);
        let new_max = new_sorted.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        assert_abs_diff_eq!(new_min, top_pos, epsilon = 1e-12);
        assert_abs_diff_eq!(new_max, bot_pos, epsilon = 1e-12);
    }

    #[test]
    fn taper_set_head_diameter_is_foot_times_ratio() {
        let mut raw = parse_instrument_xml(NAF_6HOLE_XML).unwrap();
        let m = raw.length_type.to_metres();

        let ratio = 1.25;
        set_taper_geometry(&mut raw, &[ratio, 0.1, 0.8]);

        let mut sorted: Vec<(f64, f64)> = raw.bore_points.iter()
            .map(|bp| (bp.bore_position * m, bp.bore_diameter * m))
            .collect();
        sorted.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        let head_dia = sorted[0].1;
        let foot_dia = sorted.last().unwrap().1;
        assert_abs_diff_eq!(head_dia, foot_dia * ratio, epsilon = 1e-12);
    }

    #[test]
    fn bore_end_move_bottom() {
        let mut raw = parse_instrument_xml(NAF_6HOLE_XML).unwrap();
        let m = raw.length_type.to_metres();

        // Get original bottom bore diameter
        let mut sorted: Vec<(f64, f64)> = raw.bore_points.iter()
            .map(|bp| (bp.bore_position * m, bp.bore_diameter * m))
            .collect();
        sorted.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        let original_bot_dia = sorted.last().unwrap().1;

        let new_end = 0.5; // 0.5 metres
        set_bore_end_move_bottom(&mut raw, new_end);

        // Find the new bottom point
        let mut new_sorted: Vec<(f64, f64)> = raw.bore_points.iter()
            .map(|bp| (bp.bore_position * m, bp.bore_diameter * m))
            .collect();
        new_sorted.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        let new_bot = new_sorted.last().unwrap();

        // Position changed, diameter preserved
        assert_abs_diff_eq!(new_bot.0, new_end, epsilon = 1e-10);
        assert_abs_diff_eq!(new_bot.1, original_bot_dia, epsilon = 1e-12);
    }

    // ── Bore diameter from top/bottom round-trip tests ───────────

    const WHISTLE_XML: &str =
        include_str!("../../../../oracle/v2.6.0/WhistleStudy/instruments/SamplePVC-Whistle.xml");
    const FLUTE_XML: &str =
        include_str!("../../../../oracle/v2.6.0/FluteStudy/instruments/SamplePVC-Flute.xml");
    const CHANTER_XML: &str =
        include_str!("../../../../oracle/v2.6.0/ReedStudy/instruments/SampleChanter.xml");

    #[test]
    fn bore_diameter_from_top_roundtrip() {
        let mut raw = parse_instrument_xml(WHISTLE_XML).unwrap();
        let m = raw.length_type.to_metres();
        let n_changed = 1; // 3 bore points, 1 from top changes
        let original: Vec<f64> = raw.bore_points.iter()
            .map(|bp| bp.bore_diameter * m).collect();

        let ratios = get_bore_diameter_from_top(&raw, n_changed);
        assert_eq!(ratios.len(), n_changed);

        set_bore_diameter_from_top(&mut raw, &ratios, n_changed);
        let after: Vec<f64> = raw.bore_points.iter()
            .map(|bp| bp.bore_diameter * m).collect();

        for (a, b) in original.iter().zip(after.iter()) {
            assert_abs_diff_eq!(a, b, epsilon = 1e-12);
        }
    }

    #[test]
    fn bore_diameter_from_top_mutation() {
        let mut raw = parse_instrument_xml(WHISTLE_XML).unwrap();
        let m = raw.length_type.to_metres();
        let n_changed = 1;

        // Increasing ratio by 10% should widen the top bore diameter
        let mut ratios = get_bore_diameter_from_top(&raw, n_changed);
        let original_top_dia = {
            let mut s: Vec<(f64, f64)> = raw.bore_points.iter()
                .map(|bp| (bp.bore_position * m, bp.bore_diameter * m)).collect();
            s.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
            s[0].1
        };
        ratios[0] *= 1.1;
        set_bore_diameter_from_top(&mut raw, &ratios, n_changed);

        let new_top_dia = {
            let mut s: Vec<(f64, f64)> = raw.bore_points.iter()
                .map(|bp| (bp.bore_position * m, bp.bore_diameter * m)).collect();
            s.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
            s[0].1
        };
        assert_abs_diff_eq!(new_top_dia, original_top_dia * 1.1, epsilon = 1e-10);
    }

    #[test]
    fn bore_diameter_from_bottom_roundtrip() {
        let mut raw = parse_instrument_xml(CHANTER_XML).unwrap();
        let m = raw.length_type.to_metres();
        // Chanter has 5 bore points; keep 2 unchanged from top
        let n_unchanged = 2;
        let original: Vec<f64> = raw.bore_points.iter()
            .map(|bp| bp.bore_diameter * m).collect();
        let original_flange = raw.termination.flange_diameter;

        let ratios = get_bore_diameter_from_bottom(&raw, n_unchanged);
        assert_eq!(ratios.len(), raw.bore_points.len() - n_unchanged);

        set_bore_diameter_from_bottom(&mut raw, &ratios, n_unchanged);
        let after: Vec<f64> = raw.bore_points.iter()
            .map(|bp| bp.bore_diameter * m).collect();

        for (a, b) in original.iter().zip(after.iter()) {
            assert_abs_diff_eq!(a, b, epsilon = 1e-12);
        }
        assert_abs_diff_eq!(raw.termination.flange_diameter, original_flange, epsilon = 1e-12);
    }

    #[test]
    fn bore_diameter_from_bottom_adjusts_flange() {
        let mut raw = parse_instrument_xml(CHANTER_XML).unwrap();
        let m = raw.length_type.to_metres();
        let n_unchanged = 2;
        let original_flange = raw.termination.flange_diameter * m;

        let mut ratios = get_bore_diameter_from_bottom(&raw, n_unchanged);
        // Double the last ratio — this changes the bottom diameter
        let last = ratios.len() - 1;
        ratios[last] *= 2.0;
        set_bore_diameter_from_bottom(&mut raw, &ratios, n_unchanged);

        // Flange should have shifted by the change in bottom diameter
        let new_flange = raw.termination.flange_diameter * m;
        assert!(
            (new_flange - original_flange).abs() > 1e-6,
            "flange should change when bottom diameter changes"
        );
    }

    // ── Bore spacing from top ─────────────────────────────────────

    #[test]
    fn bore_spacing_from_top_roundtrip() {
        let mut raw = parse_instrument_xml(WHISTLE_XML).unwrap();
        let m = raw.length_type.to_metres();
        let n_changed = 2; // 3 bore points → 2 spacings
        let original_pos: Vec<f64> = {
            let mut s: Vec<f64> = raw.bore_points.iter()
                .map(|bp| bp.bore_position * m).collect();
            s.sort_by(|a, b| a.partial_cmp(b).unwrap());
            s
        };

        let spacings = get_bore_spacing_from_top(&raw, n_changed);
        assert_eq!(spacings.len(), n_changed);

        set_bore_spacing_from_top(&mut raw, &spacings, n_changed);
        let after_pos: Vec<f64> = {
            let mut s: Vec<f64> = raw.bore_points.iter()
                .map(|bp| bp.bore_position * m).collect();
            s.sort_by(|a, b| a.partial_cmp(b).unwrap());
            s
        };

        for (a, b) in original_pos.iter().zip(after_pos.iter()) {
            assert_abs_diff_eq!(a, b, epsilon = 1e-12);
        }
    }

    #[test]
    fn bore_spacing_mutation() {
        let mut raw = parse_instrument_xml(WHISTLE_XML).unwrap();
        let _m = raw.length_type.to_metres();
        let n_changed = 2;

        let mut spacings = get_bore_spacing_from_top(&raw, n_changed);
        // Doubling the first spacing doubles the gap between bore points 0 and 1
        let original_gap = spacings[0];
        spacings[0] *= 2.0;
        set_bore_spacing_from_top(&mut raw, &spacings, n_changed);

        let new_spacings = get_bore_spacing_from_top(&raw, n_changed);
        assert_abs_diff_eq!(new_spacings[0], original_gap * 2.0, epsilon = 1e-12);
    }

    // ── Bore position round-trip ──────────────────────────────────

    #[test]
    fn bore_position_roundtrip_bottom_free() {
        let mut raw = parse_instrument_xml(CHANTER_XML).unwrap();
        let m = raw.length_type.to_metres();
        // 5 bore points, 2 unchanged, bottom not fixed → 3 dims
        let n_unchanged = 2;
        let original_pos: Vec<f64> = {
            let mut s: Vec<f64> = raw.bore_points.iter()
                .map(|bp| bp.bore_position * m).collect();
            s.sort_by(|a, b| a.partial_cmp(b).unwrap());
            s
        };

        let positions = get_bore_position(&raw, n_unchanged, false);
        assert_eq!(positions.len(), 3); // 5 - 2 - 0 = 3

        set_bore_position(&mut raw, &positions, n_unchanged, false);
        let after_pos: Vec<f64> = {
            let mut s: Vec<f64> = raw.bore_points.iter()
                .map(|bp| bp.bore_position * m).collect();
            s.sort_by(|a, b| a.partial_cmp(b).unwrap());
            s
        };

        for (a, b) in original_pos.iter().zip(after_pos.iter()) {
            assert_abs_diff_eq!(a, b, epsilon = 1e-10);
        }
    }

    #[test]
    fn bore_position_roundtrip_bottom_fixed() {
        let mut raw = parse_instrument_xml(CHANTER_XML).unwrap();
        let m = raw.length_type.to_metres();
        // 5 bore points, 2 unchanged, bottom fixed → 2 dims (fractional only)
        let n_unchanged = 2;
        let original_pos: Vec<f64> = {
            let mut s: Vec<f64> = raw.bore_points.iter()
                .map(|bp| bp.bore_position * m).collect();
            s.sort_by(|a, b| a.partial_cmp(b).unwrap());
            s
        };

        let positions = get_bore_position(&raw, n_unchanged, true);
        assert_eq!(positions.len(), 2); // 5 - 2 - 1 = 2

        set_bore_position(&mut raw, &positions, n_unchanged, true);
        let after_pos: Vec<f64> = {
            let mut s: Vec<f64> = raw.bore_points.iter()
                .map(|bp| bp.bore_position * m).collect();
            s.sort_by(|a, b| a.partial_cmp(b).unwrap());
            s
        };

        for (a, b) in original_pos.iter().zip(after_pos.iter()) {
            assert_abs_diff_eq!(a, b, epsilon = 1e-10);
        }
    }

    #[test]
    fn bore_position_midpoint_fractions() {
        let mut raw = parse_instrument_xml(CHANTER_XML).unwrap();
        let n_unchanged = 2;

        // Setting all fractions to 0.5 should place interior bore points at midpoints
        let n_dims = raw.bore_points.len() - n_unchanged - 1; // bottom fixed
        let positions: Vec<f64> = vec![0.5; n_dims];
        set_bore_position(&mut raw, &positions, n_unchanged, true);

        let result = get_bore_position(&raw, n_unchanged, true);
        for &frac in &result {
            assert_abs_diff_eq!(frac, 0.5, epsilon = 1e-10);
        }
    }

    // ── Basic taper ──────────────────────────────────────────────

    #[test]
    fn basic_taper_roundtrip() {
        let mut raw = parse_instrument_xml(WHISTLE_XML).unwrap();
        let m = raw.length_type.to_metres();
        let original_pos: Vec<f64> = {
            let mut s: Vec<(f64, f64)> = raw.bore_points.iter()
                .map(|bp| (bp.bore_position * m, bp.bore_diameter * m)).collect();
            s.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
            s.iter().map(|x| x.0).collect()
        };

        let taper = get_basic_taper(&raw);
        set_basic_taper(&mut raw, &taper);

        // After set, bore has exactly 3 points
        assert_eq!(raw.bore_points.len(), 3);

        // Top and bottom positions preserved
        let mut new_sorted: Vec<f64> = raw.bore_points.iter()
            .map(|bp| bp.bore_position * m).collect();
        new_sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        assert_abs_diff_eq!(new_sorted[0], original_pos[0], epsilon = 1e-10);
        assert_abs_diff_eq!(*new_sorted.last().unwrap(), *original_pos.last().unwrap(), epsilon = 1e-10);
    }

    #[test]
    fn basic_taper_cylindrical() {
        let mut raw = parse_instrument_xml(WHISTLE_XML).unwrap();
        let m = raw.length_type.to_metres();

        // Setting taper_ratio=1.0 means bottom_dia = middle_dia
        set_basic_taper(&mut raw, &[0.3, 1.0]);
        let mut sorted: Vec<(f64, f64)> = raw.bore_points.iter()
            .map(|bp| (bp.bore_position * m, bp.bore_diameter * m)).collect();
        sorted.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        // Middle and bottom should have same diameter
        assert_abs_diff_eq!(sorted[1].1, sorted[2].1, epsilon = 1e-12);
    }

    // ── Stopper position (Flute) ─────────────────────────────────

    #[test]
    fn stopper_position_roundtrip() {
        let mut raw = parse_instrument_xml(FLUTE_XML).unwrap();
        let m = raw.length_type.to_metres();
        let original_top = {
            let mut s: Vec<f64> = raw.bore_points.iter()
                .map(|bp| bp.bore_position * m).collect();
            s.sort_by(|a, b| a.partial_cmp(b).unwrap());
            s[0]
        };

        let distance = get_stopper_position(&raw);
        set_stopper_position(&mut raw, distance, false);

        let new_top = {
            let mut s: Vec<f64> = raw.bore_points.iter()
                .map(|bp| bp.bore_position * m).collect();
            s.sort_by(|a, b| a.partial_cmp(b).unwrap());
            s[0]
        };

        assert_abs_diff_eq!(new_top, original_top, epsilon = 1e-10);
    }

    #[test]
    fn stopper_position_increasing_distance_moves_top() {
        let mut raw = parse_instrument_xml(FLUTE_XML).unwrap();
        let m = raw.length_type.to_metres();

        let original_distance = get_stopper_position(&raw);
        let original_top = {
            let mut s: Vec<f64> = raw.bore_points.iter()
                .map(|bp| bp.bore_position * m).collect();
            s.sort_by(|a, b| a.partial_cmp(b).unwrap());
            s[0]
        };

        // Increase distance → top bore point moves further from mouthpiece (lower position)
        set_stopper_position(&mut raw, original_distance + 0.01, false);
        let new_top = {
            let mut s: Vec<f64> = raw.bore_points.iter()
                .map(|bp| bp.bore_position * m).collect();
            s.sort_by(|a, b| a.partial_cmp(b).unwrap());
            s[0]
        };

        assert!(new_top < original_top, "increasing distance should move top point down");
    }

    // ── BoreLengthAdjust tests ───────────────────────────────────

    #[test]
    fn preserve_bell_shifts_bell_points() {
        let mut raw = parse_instrument_xml(CHANTER_XML).unwrap();
        let m = raw.length_type.to_metres();

        // Get original bore positions
        let mut orig_pos: Vec<f64> = raw.bore_points.iter()
            .map(|bp| bp.bore_position * m).collect();
        orig_pos.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let original_end = *orig_pos.last().unwrap();
        let orig_diameters: Vec<f64> = {
            let mut s: Vec<(f64, f64)> = raw.bore_points.iter()
                .map(|bp| (bp.bore_position * m, bp.bore_diameter * m)).collect();
            s.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
            s.iter().map(|x| x.1).collect()
        };

        // Extend bore by 10mm
        let delta = 0.01;
        set_bore_end_adjusted(&mut raw, original_end + delta, BoreLengthAdjust::PreserveBell);

        let mut new_pos: Vec<f64> = raw.bore_points.iter()
            .map(|bp| bp.bore_position * m).collect();
        new_pos.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let new_diameters: Vec<f64> = {
            let mut s: Vec<(f64, f64)> = raw.bore_points.iter()
                .map(|bp| (bp.bore_position * m, bp.bore_diameter * m)).collect();
            s.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
            s.iter().map(|x| x.1).collect()
        };

        // Diameters should be unchanged
        for (a, b) in orig_diameters.iter().zip(new_diameters.iter()) {
            assert_abs_diff_eq!(a, b, epsilon = 1e-12);
        }

        // End position should have changed
        assert_abs_diff_eq!(*new_pos.last().unwrap(), original_end + delta, epsilon = 1e-10);
    }

    #[test]
    fn move_bottom_preserves_diameter() {
        let mut raw = parse_instrument_xml(WHISTLE_XML).unwrap();
        let m = raw.length_type.to_metres();
        let original_bot_dia = {
            let mut s: Vec<(f64, f64)> = raw.bore_points.iter()
                .map(|bp| (bp.bore_position * m, bp.bore_diameter * m)).collect();
            s.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
            s.last().unwrap().1
        };

        set_bore_end_adjusted(&mut raw, 0.3, BoreLengthAdjust::MoveBottom);
        let new_bot_dia = {
            let mut s: Vec<(f64, f64)> = raw.bore_points.iter()
                .map(|bp| (bp.bore_position * m, bp.bore_diameter * m)).collect();
            s.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
            s.last().unwrap().1
        };

        assert_abs_diff_eq!(new_bot_dia, original_bot_dia, epsilon = 1e-12);
    }

    #[test]
    fn preserve_taper_interpolates_diameter() {
        let mut raw = parse_instrument_xml(WHISTLE_XML).unwrap();
        let m = raw.length_type.to_metres();
        let original_bot_dia = {
            let mut s: Vec<(f64, f64)> = raw.bore_points.iter()
                .map(|bp| (bp.bore_position * m, bp.bore_diameter * m)).collect();
            s.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
            s.last().unwrap().1
        };

        // Whistle bore is tapered between points 0 and 1 (10mm→11.9mm at -5.4→19.6),
        // then cylindrical from 19.6→267.5. Moving end to 200mm should still give
        // 11.9mm diameter (cylindrical section).
        set_bore_end_adjusted(&mut raw, 0.2, BoreLengthAdjust::PreserveTaper);
        let new_bot = {
            let mut s: Vec<(f64, f64)> = raw.bore_points.iter()
                .map(|bp| (bp.bore_position * m, bp.bore_diameter * m)).collect();
            s.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
            *s.last().unwrap()
        };

        // Position moved
        assert_abs_diff_eq!(new_bot.0, 0.2, epsilon = 1e-10);
        // Diameter interpolated (cylindrical section → same as original cylinder dia)
        assert_abs_diff_eq!(new_bot.1, original_bot_dia, epsilon = 1e-6);
    }

    // ── find_head_point / find_body_top fallback tests ────────────

    #[test]
    fn find_head_point_fallback_whistle() {
        // PVC Whistle has no named bore points, 3 bore points, holes starting at 117mm
        let raw = parse_instrument_xml(WHISTLE_XML).unwrap();
        let idx = find_head_point(&raw, "Head");
        // With 3 bore points and no names, fallback should find a bore point
        // above the top hole. Bore points at -5.4, 19.6, 267.5mm; top hole at 117mm.
        // Scanning from bottom: 267.5 > 117 (skip), 19.6 < 117 → return index 1
        assert_eq!(idx, 1);
    }

    #[test]
    fn find_body_top_fallback_chanter() {
        // SampleChanter has no named bore points, 5 bore points, holes starting at 91.9mm
        let raw = parse_instrument_xml(CHANTER_XML).unwrap();
        let idx = find_body_top(&raw);
        // Bore points at -40, -32, 8, 9, 322.3mm; top hole at 91.9mm
        // Scanning from bottom: 322.3 > 91.9 (skip), 9 < 91.9 → return index 3
        assert_eq!(idx, 3);
    }

    #[test]
    fn find_bell_returns_reasonable_index() {
        let raw = parse_instrument_xml(CHANTER_XML).unwrap();
        let idx = find_bell(&raw);
        // 5 bore points: -40, -32, 8, 9, 322.3mm
        // Segments: 8mm, 40mm, 1mm, 313.3mm
        // Longest is the last segment (313.3mm), so bell starts at index 4
        assert_eq!(idx, 4);
    }

    #[test]
    fn clamp_bore_spacing_upper_bounds_scales() {
        let raw = parse_instrument_xml(WHISTLE_XML).unwrap();
        let n_changed = 1; // only 1 spacing (between points 0 and 1)
        let mut upper = vec![1.0]; // Way too big

        clamp_bore_spacing_upper_bounds(&raw, n_changed, &mut upper);
        // Should be scaled down if the spacing exceeds available space
        // Available = point[2] - point[0] (but n_changed=1 means we need point at index 2)
        assert!(upper[0] < 1.0, "upper bound should be clamped");
    }

    // ── Defensive: non-finite validation ────────────────────────

    #[test]
    fn nan_bore_position_rejected() {
        use wid_types::*;
        let raw = InstrumentRaw {
            name: "test".to_string(),
            description: None,
            length_type: LengthType::Metres,
            mouthpiece: MouthpieceRaw {
                position: 0.0,
                beta: None,
                fipple: Some(FippleRaw {
                    window_length: 0.01,
                    window_width: 0.01,
                    fipple_factor: Some(0.75),
                    window_height: None,
                    windway_length: None,
                    windway_height: None,
                }),
                embouchure_hole: None,
                single_reed: None,
                double_reed: None,
                lip_reed: None,
            },
            bore_points: vec![
                BorePointRaw { name: None, bore_position: 0.0, bore_diameter: 0.02 },
                BorePointRaw { name: None, bore_position: f64::NAN, bore_diameter: 0.02 },
            ],
            holes: vec![],
            termination: TerminationRaw { flange_diameter: 0.0 },
        };
        let result = compile(&raw);
        assert!(result.is_err(), "compile should reject NaN bore position");
        let msgs = result.unwrap_err().messages;
        assert!(msgs.iter().any(|m| m.contains("non-finite")), "error should mention non-finite: {:?}", msgs);
    }
}
