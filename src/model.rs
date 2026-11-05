use serde::{Deserialize, Serialize};

/// アイテム定義
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub id: usize,
    pub name: String,
    pub weights: Vec<u32>, // 多次元リソース消費（例: [重量, 予算]）
    pub value: u32,        // 満足度・価値
}

/// 論理ビジネスルール
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum BusinessRule {
    /// 依存: if A then B (¬A ∨ B)
    Requires { from_id: usize, to_id: usize },
    /// 排他: at most one of A or B (¬A ∨ ¬B)
    Conflicts { a_id: usize, b_id: usize },
}

/// ソルバー入力
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnapsackInput {
    pub items: Vec<Item>,
    pub capacities: Vec<u32>, // 各次元の上限
    pub rules: Vec<BusinessRule>,
    pub target_value: Option<u32>, // SAT還元時の目標満足度（下限値）
}

/// DP最適化の出力結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DpResult {
    pub best_value: u32,
    pub selected_item_ids: Vec<usize>,
    pub execution_time_ms: f64,
    pub memory_mb: f64,
}

/// 3-SAT 還元の出力結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SatReductionResult {
    pub num_variables: usize,
    pub num_clauses: usize,
    pub dimacs_cnf: String,
    pub variable_map: Vec<String>, // 各変数IDが回路の何に対応するか
}

/// MIP / 分枝限定法の出力結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MipResult {
    pub best_value: u32,
    pub selected_item_ids: Vec<usize>,
    pub execution_time_ms: f64,
    pub explored_nodes: u64, // 探索したノード数（枝刈りの証拠）
}

/// 矛盾診断レポート
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictReport {
    pub is_feasible: bool,
    pub logical_chain_conflicts: Vec<String>,
    pub capacity_conflicts: Vec<String>,
    pub target_conflict: Option<String>,
}