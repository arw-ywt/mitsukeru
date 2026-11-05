// benches/solver_bench.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mitsukeru::{BranchAndBoundSolver, BusinessRule, Item, Optimizer, KnapsackInput};

#[derive(Clone)]
pub struct ProblemInstance {
    pub name: &'static str,
    pub items: Vec<Item>,
    pub capacities: Vec<u32>,
    pub rules: Vec<BusinessRule>,
    pub target_val: Option<u32>,
}

fn problem_test_a() -> ProblemInstance {
    ProblemInstance {
        name: "Test A (14 items, 800x600)",
        items: vec![
            Item { id: 0,  name: "A001".into(), weights: vec![120, 95],  value: 85 },
            Item { id: 1,  name: "A002".into(), weights: vec![45,  30],  value: 20 },
            Item { id: 2,  name: "A003".into(), weights: vec![180, 140], value: 90 },
            Item { id: 3,  name: "A004".into(), weights: vec![60,  75],  value: 35 },
            Item { id: 4,  name: "A005".into(), weights: vec![210, 40],  value: 70 },
            Item { id: 5,  name: "A006".into(), weights: vec![280, 10],  value: 95 },
            Item { id: 6,  name: "A007".into(), weights: vec![150, 110], value: 80 },
            Item { id: 7,  name: "A008".into(), weights: vec![90,  85],  value: 65 },
            Item { id: 8,  name: "A009".into(), weights: vec![35,  20],  value: 40 },
            Item { id: 9,  name: "A010".into(), weights: vec![50,  45],  value: 55 },
            Item { id: 10, name: "A011".into(), weights: vec![85,  120], value: 90 },
            Item { id: 11, name: "A012".into(), weights: vec![25,  15],  value: 30 },
            Item { id: 12, name: "A013".into(), weights: vec![140, 25],  value: 50 },
            Item { id: 13, name: "A014".into(), weights: vec![75,  60],  value: 60 },
        ],
        capacities: vec![800, 600],
        rules: vec![
            BusinessRule::Requires  { from_id: 0,  to_id: 1 },
            BusinessRule::Requires  { from_id: 2,  to_id: 3 },
            BusinessRule::Conflicts { a_id: 4,     b_id: 5 },
            BusinessRule::Requires  { from_id: 10, to_id: 11 },
            BusinessRule::Conflicts { a_id: 6,     b_id: 13 },
        ],
        target_val: Some(480),
    }
}

fn bench_solver_scenarios(c: &mut Criterion) {
    let problem = problem_test_a();
    c.bench_function("bnb_solve_test_a (14 items)", |b| {
        b.iter(|| {
            let mut solver = BranchAndBoundSolver::new(
                black_box(&problem.items),
                black_box(&problem.capacities),
                black_box(&problem.rules),
            );
            solver.solve()
        })
    });
    
    let input = KnapsackInput {
        items: problem.items,
        capacities: problem.capacities,
        rules: problem.rules,
        target_value: problem.target_val,
    };

    c.bench_function("optimizer_full_solve_test_a", |b| {
        b.iter(|| {
            let optimizer = Optimizer::new(black_box(&input));
            optimizer.solve()
        })
    });
}

criterion_group!(benches, bench_solver_scenarios);
criterion_main!(benches);