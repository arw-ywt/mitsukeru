use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub id: usize,
    pub name: String,
    pub weights: Vec<u32>,
    pub value: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum BusinessRule {
    Requires { from_id: usize, to_id: usize },
    Conflicts { a_id: usize, b_id: usize },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnapsackInput {
    pub items: Vec<Item>,
    pub capacities: Vec<u32>,
    pub rules: Vec<BusinessRule>,
    pub target_value: Option<u32>,
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictReport {
    pub is_feasible: bool,
    pub logical_chain_conflicts: Vec<String>,
    pub capacity_conflicts: Vec<String>,
    pub target_conflict: Option<String>,
}