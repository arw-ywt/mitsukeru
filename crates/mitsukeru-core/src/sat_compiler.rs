// sat_compiler.rs

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::{self, Write};

use crate::model::{BusinessRule, Item};

/// Errors that can occur during 3-SAT reduction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SatCompileError {
    /// A business rule references an item index outside the valid range.
    InvalidRuleIndex { id: usize, item_count: usize },
    /// The number of CNF variables exceeded the system addressable range.
    VariableOverflow,
    /// Internal invariant was violated during circuit synthesis.
    InvariantViolation,
}

impl fmt::Display for SatCompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRuleIndex { id, item_count } => {
                write!(f, "Invalid rule index {id} (total items: {item_count})")
            }
            Self::VariableOverflow => {
                write!(f, "Total SAT variable count overflowed memory limits")
            }
            Self::InvariantViolation => {
                write!(
                    f,
                    "mitsukeru (sat_compiler): internal circuit invariant violated"
                )
            }
        }
    }
}

impl core::error::Error for SatCompileError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Literal(pub i32);
pub type Clause = Vec<Literal>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitExpansionMode {
    Auto,
    ForceUnroll,
    Parameterized,
}

#[derive(Debug, Clone)]
pub struct CompilerConfig {
    pub mode: CircuitExpansionMode,
    pub expansion_threshold: u32,
}

impl Default for CompilerConfig {
    fn default() -> Self {
        const DEFAULT_EXPANSION_THRESHOLD: u32 = 64;
        Self {
            mode: CircuitExpansionMode::Auto,
            expansion_threshold: DEFAULT_EXPANSION_THRESHOLD,
        }
    }
}

pub struct CnfBuilder {
    pub next_var: i32,
    pub clauses: Vec<Clause>,
    pub var_descriptions: Vec<String>,
    pub true_lit: Literal,
    pub false_lit: Literal,
}

impl Default for CnfBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl CnfBuilder {
    #[must_use]
    pub fn new() -> Self {
        let mut builder = Self {
            next_var: 1,
            clauses: Vec::new(),
            var_descriptions: Vec::new(),
            true_lit: Literal(0),
            false_lit: Literal(0),
        };

        let t = builder.alloc_var("CONST_TRUE");
        builder.clauses.push(vec![t]);
        builder.true_lit = t;
        builder.false_lit = Literal(-t.0);

        builder
    }

    pub fn alloc_var(&mut self, desc: &str) -> Literal {
        let v = self.next_var;
        self.next_var += 1;
        self.var_descriptions.push(format!("v{v}: {desc}"));
        Literal(v)
    }

    pub fn add_clause(&mut self, lits: Vec<Literal>) {
        self.clauses.push(lits);
    }

    pub fn add_xor_gate(&mut self, out: Literal, a: Literal, b: Literal) {
        self.add_clause(vec![Literal(-a.0), Literal(-b.0), Literal(-out.0)]);
        self.add_clause(vec![a, b, Literal(-out.0)]);
        self.add_clause(vec![Literal(-a.0), b, out]);
        self.add_clause(vec![a, Literal(-b.0), out]);
    }

    pub fn add_and_gate(&mut self, out: Literal, a: Literal, b: Literal) {
        self.add_clause(vec![Literal(-a.0), Literal(-b.0), out]);
        self.add_clause(vec![a, Literal(-out.0)]);
        self.add_clause(vec![b, Literal(-out.0)]);
    }

    pub fn add_or_gate(&mut self, out: Literal, a: Literal, b: Literal) {
        self.add_clause(vec![a, b, Literal(-out.0)]);
        self.add_clause(vec![Literal(-a.0), out]);
        self.add_clause(vec![Literal(-b.0), out]);
    }

    pub fn add_full_adder(&mut self, a: Literal, b: Literal, cin: Literal) -> (Literal, Literal) {
        let sum = self.alloc_var("fa_sum");
        let cout = self.alloc_var("fa_cout");

        let xor1 = self.alloc_var("fa_xor1");
        self.add_xor_gate(xor1, a, b);
        self.add_xor_gate(sum, xor1, cin);

        let and1 = self.alloc_var("fa_and1");
        let and2 = self.alloc_var("fa_and2");
        self.add_and_gate(and1, a, b);
        self.add_and_gate(and2, xor1, cin);
        self.add_or_gate(cout, and1, and2);

        (sum, cout)
    }

    pub fn add_ripple_carry_adder(
        &mut self,
        a_bits: &[Literal],
        b_bits: &[Literal],
    ) -> Vec<Literal> {
        let max_len = a_bits.len().max(b_bits.len());
        let mut sum_bits = Vec::with_capacity(max_len + 1);
        let mut carry = self.false_lit;

        for i in 0..max_len {
            let a = a_bits.get(i).copied().unwrap_or(self.false_lit);
            let b = b_bits.get(i).copied().unwrap_or(self.false_lit);
            let (s, c) = self.add_full_adder(a, b, carry);
            sum_bits.push(s);
            carry = c;
        }
        sum_bits.push(carry);
        sum_bits
    }

    pub fn add_less_than_or_equal(&mut self, a_bits: &[Literal], b_bits: &[Literal]) {
        let max_len = a_bits.len().max(b_bits.len());
        let mut carry = self.true_lit;
        for i in 0..max_len {
            let a = a_bits.get(i).copied().unwrap_or(self.false_lit);
            let b = b_bits.get(i).copied().unwrap_or(self.false_lit);
            let not_a = Literal(-a.0);
            let (_s, cout) = self.add_full_adder(not_a, b, carry);
            carry = cout;
        }
        self.add_clause(vec![carry]);
    }
}

pub struct SatCompiler;

impl SatCompiler {
    /// Compiles a combinatorial constraint optimization problem into a 3-SAT (DIMACS CNF) reduction.
    ///
    /// Encodes item selections, mutual exclusions/prerequisites, capacity upper-bounds,
    /// and target value lower-bounds into satisfiability clauses.
    ///
    /// # Errors
    ///
    /// Returns [`SatCompileError::InvalidRuleIndex`] if any rule references an item ID >= `items.len()`.
    /// Returns [`SatCompileError::VariableOverflow`] if variable allocation exceeds `usize::MAX`.
    pub fn compile(
        items: &[Item],
        capacities: &[u32],
        rules: &[BusinessRule],
        target_val: u32,
        config: &CompilerConfig,
    ) -> Result<(String, usize, usize, Vec<String>), SatCompileError> {
        let n = items.len();

        for rule in rules {
            match *rule {
                BusinessRule::Requires { from_id, to_id } => {
                    if from_id >= n {
                        return Err(SatCompileError::InvalidRuleIndex {
                            id: from_id,
                            item_count: n,
                        });
                    }
                    if to_id >= n {
                        return Err(SatCompileError::InvalidRuleIndex {
                            id: to_id,
                            item_count: n,
                        });
                    }
                }
                BusinessRule::Conflicts { a_id, b_id } => {
                    if a_id >= n {
                        return Err(SatCompileError::InvalidRuleIndex {
                            id: a_id,
                            item_count: n,
                        });
                    }
                    if b_id >= n {
                        return Err(SatCompileError::InvalidRuleIndex {
                            id: b_id,
                            item_count: n,
                        });
                    }
                }
            }
        }

        let mut builder = CnfBuilder::new();

        let item_vars: Vec<Literal> = items
            .iter()
            .map(|it| builder.alloc_var(&format!("Item_{}_{}", it.id, it.name)))
            .collect();

        for rule in rules {
            match *rule {
                BusinessRule::Requires { from_id, to_id } => {
                    builder.add_clause(vec![Literal(-item_vars[from_id].0), item_vars[to_id]]);
                }
                BusinessRule::Conflicts { a_id, b_id } => {
                    builder.add_clause(vec![
                        Literal(-item_vars[a_id].0),
                        Literal(-item_vars[b_id].0),
                    ]);
                }
            }
        }

        let max_param_val = capacities
            .iter()
            .copied()
            .chain(core::iter::once(target_val))
            .max()
            .unwrap_or(0);

        let use_parameterized = match config.mode {
            CircuitExpansionMode::ForceUnroll => false,
            CircuitExpansionMode::Parameterized => true,
            CircuitExpansionMode::Auto => max_param_val > config.expansion_threshold,
        };

        for (dim, &cap) in capacities.iter().enumerate() {
            let dim_weights: Vec<(Literal, u32)> = items
                .iter()
                .enumerate()
                .map(|(idx, it)| (item_vars[idx], it.weights.get(dim).copied().unwrap_or(0)))
                .collect();

            let sum_bits =
                Self::build_linear_sum_tree(&mut builder, &dim_weights, &format!("cap_{dim}"));

            if use_parameterized {
                let cap_bits =
                    Self::alloc_parameter_bus(&mut builder, cap, &format!("param_cap_{dim}"));
                builder.add_less_than_or_equal(&sum_bits, &cap_bits);
            } else {
                let cap_bits = Self::to_constant_bits(&builder, cap);
                builder.add_less_than_or_equal(&sum_bits, &cap_bits);
            }
        }

        if target_val > 0 {
            let val_terms: Vec<(Literal, u32)> = items
                .iter()
                .enumerate()
                .map(|(idx, it)| (item_vars[idx], it.value))
                .collect();

            let val_sum_bits = Self::build_linear_sum_tree(&mut builder, &val_terms, "val_sum");

            let target_bits = if use_parameterized {
                Self::alloc_parameter_bus(&mut builder, target_val, "param_target_val")
            } else {
                Self::to_constant_bits(&builder, target_val)
            };

            builder.add_less_than_or_equal(&target_bits, &val_sum_bits);
        }

        if builder.next_var < 1 {
            return Err(SatCompileError::InvariantViolation);
        }

        let raw_num_vars = builder.next_var - 1;

        let num_vars =
            usize::try_from(raw_num_vars).map_err(|_| SatCompileError::VariableOverflow)?;

        let num_clauses = builder.clauses.len();
        let mut dimacs = String::new();
        writeln!(dimacs, "p cnf {num_vars} {num_clauses}")
            .map_err(|_| SatCompileError::VariableOverflow)?;

        for c in &builder.clauses {
            for lit in c {
                write!(dimacs, "{} ", lit.0).map_err(|_| SatCompileError::VariableOverflow)?;
            }
            dimacs.push_str("0\n");
        }

        Ok((dimacs, num_vars, num_clauses, builder.var_descriptions))
    }

    fn build_linear_sum_tree(
        builder: &mut CnfBuilder,
        terms: &[(Literal, u32)],
        prefix: &str,
    ) -> Vec<Literal> {
        let mut vectors: Vec<Vec<Literal>> = Vec::new();

        for &(x_i, weight) in terms {
            if weight == 0 {
                continue;
            }
            let mut weighted_bits = Vec::new();
            let mut w = weight;
            while w > 0 {
                let bit = w & 1;
                if bit == 1 {
                    weighted_bits.push(x_i);
                } else {
                    weighted_bits.push(builder.false_lit);
                }
                w >>= 1;
            }
            vectors.push(weighted_bits);
        }

        if vectors.is_empty() {
            return vec![builder.false_lit];
        }

        let mut step = 0;
        while vectors.len() > 1 {
            let mut next_vectors = Vec::new();
            for chunk in vectors.chunks(2) {
                if chunk.len() == 2 {
                    let sum_res = builder.add_ripple_carry_adder(&chunk[0], &chunk[1]);
                    builder
                        .var_descriptions
                        .push(format!("{prefix}_tree_step_{step}_merge"));
                    next_vectors.push(sum_res);
                } else {
                    next_vectors.push(chunk[0].clone());
                }
            }
            vectors = next_vectors;
            step += 1;
        }

        vectors
            .into_iter()
            .next()
            .unwrap_or_else(|| vec![builder.false_lit])
    }

    fn to_constant_bits(builder: &CnfBuilder, mut val: u32) -> Vec<Literal> {
        if val == 0 {
            return vec![builder.false_lit];
        }
        let mut bits = Vec::new();
        while val > 0 {
            if val & 1 == 1 {
                bits.push(builder.true_lit);
            } else {
                bits.push(builder.false_lit);
            }
            val >>= 1;
        }
        bits
    }

    fn alloc_parameter_bus(
        builder: &mut CnfBuilder,
        mut val: u32,
        desc_prefix: &str,
    ) -> Vec<Literal> {
        let mut bits = Vec::new();
        let mut bit_idx = 0;
        let original_val = val;

        while val > 0 || bits.is_empty() {
            let is_one = (val & 1) == 1;
            let param_var = builder.alloc_var(&format!(
                "{}_bit{}_{}",
                desc_prefix,
                bit_idx,
                i32::from(is_one)
            ));

            if is_one {
                builder.add_clause(vec![param_var]);
            } else {
                builder.add_clause(vec![Literal(-param_var.0)]);
            }

            bits.push(param_var);
            val >>= 1;
            bit_idx += 1;
        }

        builder
            .var_descriptions
            .push(format!("ParamBus: {desc_prefix} = {original_val}"));
        bits
    }
}
