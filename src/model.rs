// model.rs

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// # Knapsack Input Configuration
/// The root problem specification defining candidate items, resource capacities,
/// dependency/exclusion rules, and an optional target score.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KnapsackInput {
    /// List of candidate items available for selection.
    pub items: Vec<Item>,
    /// Capacity limits for each resource dimension (e.g., [Budget, Weight, Power]).
    pub capacities: Vec<u32>,
    /// Logical business constraints governing item selections.
    pub rules: Vec<BusinessRule>,
    /// Optional target objective value to satisfy.
    pub target_value: Option<u32>,
}

/// # Selectable Item
/// Represents a decision candidate with multi-dimensional resource consumption
/// and an associated objective value.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Item {
    /// Unique identifier of the item (zero-based index).
    pub id: usize,
    /// Display name or human-readable label of the item.
    pub name: String,
    /// Consumption weights for each corresponding capacity dimension.
    pub weights: Vec<u32>,
    /// Value, profit, or score gained if this item is selected.
    pub value: u32,
}

/// # Business Rule
/// Logical constraints specifying dependencies or mutual exclusions between items.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type")]
pub enum BusinessRule {
    /// # Requirement Constraint
    /// Selecting `from_id` strictly requires selecting `to_id`.
    Requires {
        /// The item requiring the prerequisite.
        from_id: usize,
        /// The item that must be included as a prerequisite.
        to_id: usize,
    },
    /// # Conflict Constraint
    /// Items `a_id` and `b_id` are mutually exclusive and cannot be selected together.
    Conflicts {
        /// The first conflicting item.
        a_id: usize,
        /// The second conflicting item.
        b_id: usize,
    },
}

/// # Optimal Solution
/// Contains the optimization result and execution statistics.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Solution {
    /// Maximum achievable score/value under given constraints.
    pub best_value: u32,
    /// IDs of the selected items.
    pub selected_item_ids: Vec<usize>,
    /// Execution time in milliseconds.
    pub execution_time_ms: f64,
    /// Number of search nodes explored during optimization.
    pub explored_nodes: u64,
}

/// # Optimization Error
/// Represents an infeasible configuration or validation failure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizeError {
    /// The problem cannot be satisfied due to conflicting constraints.
    Infeasible(ConflictReport),
    /// The input parameters are malformed or invalid.
    InvalidInput(String),
}

impl std::fmt::Display for OptimizeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OptimizeError::Infeasible(report) => 
                write!(
                    f,
                    "Infeasible problem: {} chain conflict(s), {} capacity conflict(s)",
                    report.logical_chain_conflicts.len(),
                    report.capacity_conflicts.len()
                ),
            OptimizeError::InvalidInput(msg) => 
                write!(
                    f, 
                    "Invalid input: {}", 
                    msg
                ),
        }
    }
}

impl std::error::Error for OptimizeError {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DpResult {
    pub best_value: u32,
    pub selected_item_ids: Vec<usize>,
    pub execution_time_ms: f64,
    pub memory_mb: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SatReductionResult {
    pub num_variables: usize,
    pub num_clauses: usize,
    pub dimacs_cnf: String,
    pub variable_map: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MipResult {
    pub best_value: u32,
    pub selected_item_ids: Vec<usize>,
    pub execution_time_ms: f64,
    pub explored_nodes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ConflictReport {
    pub is_feasible: bool,
    pub logical_chain_conflicts: Vec<String>,
    pub capacity_conflicts: Vec<String>,
    pub target_conflict: Option<String>,
}