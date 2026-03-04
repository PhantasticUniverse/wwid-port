//! Constraints XML model for optimization bounds.
//!
//! Constraints define the lower and upper bounds for optimization parameters.
//! The bounds arrays extracted from constraints preserve insertion order:
//! categories in order of first appearance, then constraints within each category.
//! This ordering is ABI — the optimizer's parameter vector indices must match.

use serde::{Deserialize, Serialize};

/// A set of optimization constraints loaded from WIDesigner XML.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename = "constraints")]
pub struct Constraints {
    #[serde(rename = "constraintsName")]
    pub name: String,
    #[serde(rename = "objectiveDisplayName")]
    pub objective_display_name: String,
    #[serde(rename = "objectiveFunctionName")]
    pub objective_function_name: String,
    #[serde(rename = "numberOfHoles")]
    pub number_of_holes: u32,
    #[serde(rename = "constraint", default)]
    pub constraint_list: Vec<Constraint>,
    /// Hole groups for grouped-hole optimizers (e.g. `[[0,1,2],[3,4,5]]`).
    ///
    /// Stored in constraints XML so that the optimizer can reconstruct
    /// group structure when re-loading saved constraints.
    #[serde(rename = "holeGroups", default, skip_serializing_if = "Option::is_none")]
    pub hole_groups: Option<HoleGroupsXml>,
}

/// XML wrapper for hole groups: `<holeGroups><holeGroup><holeIdx>0</holeIdx>...</holeGroup></holeGroups>`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HoleGroupsXml {
    #[serde(rename = "holeGroup", default)]
    pub groups: Vec<HoleGroupXml>,
}

/// A single hole group containing hole indices.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HoleGroupXml {
    #[serde(rename = "holeIdx", default)]
    pub indices: Vec<u32>,
}

impl Constraints {
    /// Get hole groups as a `Vec<Vec<u32>>` if present.
    pub fn hole_groups_array(&self) -> Option<Vec<Vec<u32>>> {
        self.hole_groups.as_ref().map(|hg| {
            hg.groups.iter().map(|g| g.indices.clone()).collect()
        })
    }

    /// Set hole groups from a `Vec<Vec<u32>>`.
    pub fn set_hole_groups(&mut self, groups: Vec<Vec<u32>>) {
        self.hole_groups = Some(HoleGroupsXml {
            groups: groups.into_iter().map(|indices| HoleGroupXml { indices }).collect(),
        });
    }
}

/// A single optimization constraint with bounds.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Constraint {
    #[serde(rename = "displayName")]
    pub display_name: String,
    pub category: String,
    #[serde(rename = "type")]
    pub constraint_type: ConstraintType,
    #[serde(rename = "lowerBound", default, skip_serializing_if = "Option::is_none")]
    pub lower_bound: Option<f64>,
    #[serde(rename = "upperBound", default, skip_serializing_if = "Option::is_none")]
    pub upper_bound: Option<f64>,
}

/// Type of constraint, determining how bounds are interpreted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum ConstraintType {
    /// Bound values have physical dimensions (length in metres).
    DIMENSIONAL,
    /// Bound values are dimensionless (ratios, fractions).
    DIMENSIONLESS,
    /// Integer-valued bounds.
    INTEGER,
    /// Boolean (0/1) bounds.
    BOOLEAN,
}

impl Constraints {
    /// Extract lower bounds array in category-then-constraint order.
    ///
    /// Categories are ordered by first appearance. Within each category,
    /// constraints appear in their XML order. Null bounds default to 0.0.
    pub fn lower_bounds(&self) -> Vec<f64> {
        self.extract_bounds(true)
    }

    /// Extract upper bounds array in category-then-constraint order.
    pub fn upper_bounds(&self) -> Vec<f64> {
        self.extract_bounds(false)
    }

    /// Total number of constraints (= dimension of the optimization problem).
    pub fn total_constraints(&self) -> usize {
        self.constraint_list.len()
    }

    fn extract_bounds(&self, is_lower: bool) -> Vec<f64> {
        // Collect unique categories in insertion order
        let mut categories: Vec<&str> = Vec::new();
        for c in &self.constraint_list {
            if !categories.contains(&c.category.as_str()) {
                categories.push(&c.category);
            }
        }

        let mut bounds = Vec::with_capacity(self.constraint_list.len());
        for category in &categories {
            for c in &self.constraint_list {
                if c.category == *category {
                    let value = if is_lower {
                        c.lower_bound.unwrap_or(0.0)
                    } else {
                        c.upper_bound.unwrap_or(0.0)
                    };
                    bounds.push(value);
                }
            }
        }
        bounds
    }
}
