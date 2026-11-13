// src/model.rs

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

#[cfg(feature = "schemars")]
use schemars::JsonSchema;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// ## Solver Optimization Strategy
/// Execution mode controlling the search and optimization algorithm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum SolverStrategy {
    /// Automatically selects the faster strategy (`Pure` vs. `Heuristic`) based on problem size and constraint density (default).
    #[default]
    Mixed,
    /// Exact branch-and-bound search without graph-based structural heuristics.
    Pure,
    /// Branch-and-bound search guided by structural heuristics, leveraging constraint graph embeddings and a greedy strategy for an initial warm start.
    Heuristic,
}

/// ## Problem Input Configuration
/// The root problem specification defining candidate items, resource capacities,
/// dependency/exclusion rules, and an optional target score.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct ProblemInput {
    /// List of candidate items available for selection.
    pub items: Vec<Item>,
    /// Capacity limits for each resource dimension.
    pub capacities: Vec<u32>,
    /// Logical business constraints governing item selections.
    pub rules: Vec<BusinessRule>,
    /// Optional target objective value to satisfy.
    pub target_value: Option<u32>,
}

impl ProblemInput {
    #[must_use]
    pub fn new(
        items: Vec<Item>,
        capacities: Vec<u32>,
        rules: Vec<BusinessRule>,
        target_value: Option<u32>,
    ) -> Self {
        Self {
            items,
            capacities,
            rules,
            target_value,
        }
    }
}

/// ## Optimizer Configuration
/// Parameters governing solver behavior, constraints, and search strategy.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct OptimizerConfig {
    /// The search strategy (e.g., Pure, Heuristic, Mixed).
    pub strategy: SolverStrategy,
    /// The execution time limit in milliseconds.
    pub time_limit_ms: Option<f64>,
    /// The maximum number of search nodes to explore.
    pub max_nodes: Option<u64>,
    /// The number of top candidate solutions to retain.
    pub top_k: Option<usize>,
}

impl Default for OptimizerConfig {
    fn default() -> Self {
        Self::preset_fast()
    }
}

impl OptimizerConfig {
    /// [Preset] Ultra-fast mode optimized for synchronous Web / WASM environments (default, 2.0 ms timeout).
    #[must_use]
    pub fn preset_fast() -> Self {
        Self {
            strategy: SolverStrategy::Mixed,
            time_limit_ms: Some(if cfg!(debug_assertions) { 50.0 } else { 2.0 }),
            max_nodes: Some(100_000),
            top_k: None,
        }
    }

    /// [Preset] High-quality batch processing mode (100 ms timeout, up to 1M nodes).
    #[must_use]
    pub fn preset_accurate() -> Self {
        Self {
            strategy: SolverStrategy::Mixed,
            time_limit_ms: Some(100.0),
            max_nodes: Some(1_000_000),
            top_k: None,
        }
    }

    /// [Preset] Exact solution mode with unbounded search time and complete optimality proof.
    #[must_use]
    pub fn preset_exact() -> Self {
        Self {
            strategy: SolverStrategy::Mixed,
            time_limit_ms: None,
            max_nodes: None,
            top_k: None,
        }
    }
}

/// ## Optimization Request (Wasm / JSON)
/// A unified request payload encapsulating problem data and solver configuration parameters.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct OptimizeRequest {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub problem: ProblemInput,
    #[cfg_attr(feature = "serde", serde(default))]
    pub config: Option<OptimizerConfig>,
}

/// ## Selectable Item
/// Represents a decision candidate with multi-dimensional resource consumption
/// and an associated objective value.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
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

/// ## Business Rule
/// Logical constraints specifying dependencies or mutual exclusions between items.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(tag = "type", rename_all = "snake_case"))]
pub enum BusinessRule {
    /// ## Requirement Constraint
    /// Selecting `from_id` strictly requires selecting `to_id`.
    Requires {
        /// The item requiring the prerequisite.
        from_id: usize,
        /// The item that must be included as a prerequisite.
        to_id: usize,
    },
    /// ## Conflict Constraint
    /// Items `a_id` and `b_id` are mutually exclusive and cannot be selected together.
    Conflicts {
        /// The first conflicting item.
        a_id: usize,
        /// The second conflicting item.
        b_id: usize,
    },
}

/// ## Solution Optimality Status
/// Indicates whether the returned solution is proven optimal, an approximation,
/// or bounded due to time/resource limits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum SolutionStatus {
    /// Mathematically proven optimal solution (all branches explored and pruned).
    Optimal,
    /// Feasible approximation or incumbent solution obtained before full convergence
    /// (e.g., reached timeout limit, node limit, or generated by primal heuristics).
    Approximate,
}

impl fmt::Display for SolutionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SolutionStatus::Optimal => write!(f, "Optimal"),
            SolutionStatus::Approximate => write!(f, "Approximate"),
        }
    }
}

/// ## Candidate Solution
/// Represents an alternative feasible configuration among the top candidates.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct CandidateSolution {
    /// Achieved score/value for this candidate.
    pub value: u32,
    /// IDs of the selected items.
    pub selected_item_ids: Vec<usize>,
}

/// ## Optimal Solution
/// Contains the optimization result and execution statistics.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct Solution {
    /// Maximum achievable score/value under given constraints.
    pub best_value: u32,
    /// IDs of the selected items.
    pub selected_item_ids: Vec<usize>,
    /// Execution time in milliseconds.
    pub execution_time_ms: f64,
    /// Number of search nodes explored during optimization.
    pub explored_nodes: u64,
    /// Indicates whether the solution is mathematically optimal or an approximation.
    pub status: SolutionStatus,
    /// The actual search strategy employed during optimization (records whether `Pure` or `Heuristic` was chosen under `Mixed` mode).
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub strategy_used: Option<SolverStrategy>,
    /// Optional list of top-ranked alternative solutions (Ranked 1st to K-th).
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub top_candidates: Option<Vec<CandidateSolution>>,
    /// Theoretical upper bound on the objective value (Dual bound).
    pub upper_bound: f64,
    /// Relative optimality gap ((Upper Bound - Best Value) / Best Value). 0.0 if proven optimal.
    pub optimality_gap: f64,
}

/// ## Conflict Analysis Report
/// Structured diagnostic report explaining why a problem instance is infeasible.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct ConflictReport {
    /// True if the configuration satisfies all constraints.
    pub is_feasible: bool,
    /// Detailed list of prerequisite and mutual exclusion chain conflicts.
    pub logical_chain_conflicts: Vec<ChainConflict>,
    /// Detailed list of resource capacity overflows.
    pub capacity_conflicts: Vec<CapacityConflict>,
    /// Target value unreachability conflict, if a target was specified.
    pub target_conflict: Option<TargetConflict>,
}

/// ## Logical Chain Conflict Details
/// Identifies a conflict where a dependency chain leads to a mutually exclusive item.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct ChainConflict {
    /// The root item being evaluated.
    pub root_id: usize,
    /// The prerequisite item that conflicts with the root.
    pub conflicting_id: usize,
    /// Ordered list of item IDs forming the requirement path (e.g., [A, B, C]).
    pub path_ids: Vec<usize>,
    /// Human-readable explanation of the chain conflict.
    pub message: String,
}

/// ## Capacity Overflow Conflict Details
/// Identifies which resource limit is exceeded and by which items.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct CapacityConflict {
    /// The root item triggering this capacity check.
    pub root_id: usize,
    /// The resource dimension index (0-based) that overflowed.
    pub dimension: usize,
    /// Maximum allowed capacity limit.
    pub capacity_limit: u32,
    /// Actual resource amount consumed.
    pub consumed_weight: u64,
    /// List of item IDs included in this bundle.
    pub bundle_ids: Vec<usize>,
    /// Human-readable explanation of the overflow.
    pub message: String,
}

/// ## Target Value Conflict Details
/// Identifies when a target objective score cannot be achieved.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct TargetConflict {
    /// The requested target value.
    pub target_value: u32,
    /// The maximum achievable score under constraints.
    pub achievable_value: u32,
    /// Human-readable explanation.
    pub message: String,
}

/// ## Optimization Error
/// Represents an infeasible configuration or validation failure.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(tag = "type"))]
pub enum OptimizeError {
    /// The problem cannot be satisfied due to conflicting constraints.
    Infeasible { report: ConflictReport },
    /// The input parameters are malformed or invalid.
    InvalidInput { message: String },
}

impl fmt::Display for OptimizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OptimizeError::Infeasible { report } => write!(
                f,
                "Infeasible problem: {} chain conflict(s), {} capacity conflict(s)",
                report.logical_chain_conflicts.len(),
                report.capacity_conflicts.len()
            ),
            OptimizeError::InvalidInput { message } => write!(f, "Invalid input: {message}"),
        }
    }
}

impl core::error::Error for OptimizeError {}

/// ## 2D Dynamic Programming Result
/// Output returned by the branchless 2D DP solver.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct DpResult {
    pub best_value: u32,
    pub selected_item_ids: Vec<usize>,
    pub execution_time_ms: f64,
    pub memory_mb: f64,
}

/// ## 3-SAT Reduction Result
/// Result of compiling the combinatorial optimization problem into a DIMACS CNF representation.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct SatReductionResult {
    pub num_variables: usize,
    pub num_clauses: usize,
    pub dimacs_cnf: String,
    pub variable_map: Vec<String>,
}

/// ## Branch and Bound Result
/// Direct output returned by the standalone Branch-and-Bound solver.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct BnbResult {
    pub best_value: u32,
    pub selected_item_ids: Vec<usize>,
    pub execution_time_ms: f64,
    pub explored_nodes: u64,
    pub status: SolutionStatus,
}
