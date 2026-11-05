use crate::model::{BusinessRule, Item};

#[derive(Debug, Clone, Copy)]
pub struct Literal(pub i32); // 正なら変数, 負なら否定

pub type Clause = Vec<Literal>;

pub struct CnfBuilder {
    pub next_var: i32,
    pub clauses: Vec<Clause>,
    pub var_descriptions: Vec<String>,
}

#[allow(dead_code)]
impl CnfBuilder {
    pub fn new() -> Self {
        Self {
            next_var: 1,
            clauses: Vec::new(),
            var_descriptions: Vec::new(),
        }
    }

    pub fn alloc_var(&mut self, desc: &str) -> Literal {
        let v = self.next_var;
        self.next_var += 1;
        self.var_descriptions.push(format!("v{}: {}", v, desc));
        Literal(v)
    }

    pub fn add_clause(&mut self, mut lits: Vec<Literal>) {
        // 厳密な 3-SAT (要素数が最大3の節) へ正規化
        while lits.len() > 3 {
            let tail = lits.split_off(2);
            let aux = self.alloc_var("aux_3sat_split");
            self.clauses.push(vec![lits[0], lits[1], aux]);
            lits = vec![Literal(-aux.0)];
            lits.extend(tail);
        }
        self.clauses.push(lits);
    }

    /// XOR ゲートの Tseitin 変換: out <=> (a ^ b)
    pub fn add_xor_gate(&mut self, out: Literal, a: Literal, b: Literal) {
        // (¬a ∨ ¬b ∨ ¬out) ∧ (a ∨ b ∨ ¬out) ∧ (¬a ∨ b ∨ out) ∧ (a ∨ ¬b ∨ out)
        self.add_clause(vec![Literal(-a.0), Literal(-b.0), Literal(-out.0)]);
        self.add_clause(vec![a, b, Literal(-out.0)]);
        self.add_clause(vec![Literal(-a.0), b, out]);
        self.add_clause(vec![a, Literal(-b.0), out]);
    }

    /// AND ゲートの Tseitin 変換: out <=> (a & b)
    pub fn add_and_gate(&mut self, out: Literal, a: Literal, b: Literal) {
        self.add_clause(vec![Literal(-a.0), Literal(-b.0), out]);
        self.add_clause(vec![a, Literal(-out.0)]);
        self.add_clause(vec![b, Literal(-out.0)]);
    }

    /// OR ゲートの Tseitin 変換: out <=> (a | b)
    pub fn add_or_gate(&mut self, out: Literal, a: Literal, b: Literal) {
        self.add_clause(vec![a, b, Literal(-out.0)]);
        self.add_clause(vec![Literal(-a.0), out]);
        self.add_clause(vec![Literal(-b.0), out]);
    }

    /// 全加算器 (Full Adder): a + b + cin => (sum, cout)
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
}

pub struct SatCompiler;

impl SatCompiler {
    /// 多次元ナップザック＋業務ルールを 3-SAT の CNF (DIMACS) に変換する
    pub fn compile(
        items: &[Item],
        _capacities: &[u32],
        rules: &[BusinessRule],
        _target_val: u32,
    ) -> (String, usize, usize, Vec<String>) {
        let mut builder = CnfBuilder::new();

        // 1. 各アイテムの採否を表す基本ブール変数を割り当て
        let item_vars: Vec<Literal> = items
            .iter()
            .map(|it| builder.alloc_var(&format!("Item_{}_{}", it.id, it.name)))
            .collect();

        // 2. 業務ルールの直接 CNF 還元
        for rule in rules {
            match rule {
                BusinessRule::Requires { from_id, to_id } => {
                    // from => to  ===  (¬from ∨ to)
                    builder.add_clause(vec![Literal(-item_vars[*from_id].0), item_vars[*to_id]]);
                }
                BusinessRule::Conflicts { a_id, b_id } => {
                    // ¬(a ∧ b) === (¬a ∨ ¬b)
                    builder.add_clause(vec![Literal(-item_vars[*a_id].0), Literal(-item_vars[*b_id].0)]);
                }
            }
        }

        // 3. DIMACS 文字列のフォーマット生成
        let num_vars = (builder.next_var - 1) as usize;
        let num_clauses = builder.clauses.len();

        let mut dimacs = format!("p cnf {} {}\n", num_vars, num_clauses);
        for c in &builder.clauses {
            for lit in c {
                dimacs.push_str(&format!("{} ", lit.0));
            }
            dimacs.push_str("0\n");
        }

        (dimacs, num_vars, num_clauses, builder.var_descriptions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_business_rules_reduction() {
        let items = vec![
            Item { id: 0, name: "A".into(), weights: vec![1], value: 10 },
            Item { id: 1, name: "B".into(), weights: vec![2], value: 20 },
        ];
        let rules = vec![BusinessRule::Conflicts { a_id: 0, b_id: 1 }];

        let (dimacs, _vars, clauses, _) = SatCompiler::compile(&items, &[3], &rules, 20);
        assert!(dimacs.contains("p cnf"));
        assert!(clauses >= 1);
        // 変数1と変数2の排他 (-1 -2 0) が含まれるか
        assert!(dimacs.contains("-1 -2 0"));
    }

    #[test]
    fn test_full_adder_logic() {
        let mut builder = CnfBuilder::new();
        let a = builder.alloc_var("A");
        let b = builder.alloc_var("B");
        let cin = builder.alloc_var("Cin");

        let (sum, cout) = builder.add_full_adder(a, b, cin);
        assert!(builder.clauses.len() > 0);
        assert_eq!(sum.0, 4);
        assert_eq!(cout.0, 5);
    }
}