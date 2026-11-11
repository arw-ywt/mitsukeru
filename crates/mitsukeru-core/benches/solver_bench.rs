// benches/solver_bench.rs

// cargo bench --bench solver_bench -- --save-baseline initial

// cargo bench --bench solver_bench -- --baseline initial

// target/criterion/report/index.html

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use mitsukeru::{BranchAndBoundSolver, Optimizer, ProblemInput};

#[path = "../tests/common/mod.rs"]
mod common;
use common::{test_a, test_b, test_c, test_d, test_e, test_f, test_g};

fn bench_solver_instances(c: &mut Criterion) {
    let mut group = c.benchmark_group("solvers_suite");

    let instances = vec![
        test_a(),
        test_b(),
        test_c(),
        test_d(),
        test_e(),
        test_f(26),
        test_g(),
    ];

    for (idx, problem) in instances.into_iter().enumerate() {
        let prefix = if problem.name.starts_with("Test") {
            problem
                .name
                .split('(')
                .next()
                .unwrap_or(problem.name)
                .trim()
                .replace(' ', "_")
        } else {
            let letter = (b'A' + u8::try_from(idx).unwrap()) as char;
            format!("Test_{letter}")
        };
        let id = format!("{}_{}items", prefix, problem.items.len());

        group.bench_with_input(BenchmarkId::new("BnB_Core", &id), &problem, |b, p| {
            b.iter(|| {
                let mut solver = BranchAndBoundSolver::new(
                    black_box(&p.items),
                    black_box(&p.capacities),
                    black_box(&p.rules),
                );
                solver.solve()
            });
        });

        let input = ProblemInput {
            items: problem.items,
            capacities: problem.capacities,
            rules: problem.rules,
            target_value: problem.target_val,
        };

        group.bench_with_input(BenchmarkId::new("Opt_Mixed", &id), &input, |b, inp| {
            b.iter(|| {
                let optimizer = Optimizer::new(black_box(inp));
                optimizer.solve()
            });
        });

        group.bench_with_input(BenchmarkId::new("Opt_Pure", &id), &input, |b, inp| {
            b.iter(|| {
                let optimizer =
                    Optimizer::new(black_box(inp)).with_strategy(mitsukeru::SolverStrategy::Pure);
                optimizer.solve()
            });
        });

        group.bench_with_input(BenchmarkId::new("Opt_Heuristic", &id), &input, |b, inp| {
            b.iter(|| {
                let optimizer = Optimizer::new(black_box(inp))
                    .with_strategy(mitsukeru::SolverStrategy::Heuristic);
                optimizer.solve()
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench_solver_instances);
criterion_main!(benches);
