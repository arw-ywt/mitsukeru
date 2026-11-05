// benches/solver_bench.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mitsukeru::{BranchAndBoundSolver, BusinessRule, Item};

fn bench_branch_and_bound(c: &mut Criterion) {
    let items = vec![
        Item { id: 0, name: "A".into(), weights: vec![10, 10], value: 50 },
        Item { id: 1, name: "B".into(), weights: vec![10, 10], value: 60 },
    ];
    let capacities = vec![20, 20];
    let rules = vec![BusinessRule::Conflicts { a_id: 0, b_id: 1 }];

    c.bench_function("bnb_solve_small", |b| {
        b.iter(|| {
            let mut solver = BranchAndBoundSolver::new(
                black_box(&items),
                black_box(&capacities),
                black_box(&rules),
            );
            solver.solve()
        })
    });
}

criterion_group!(benches, bench_branch_and_bound);
criterion_main!(benches);