// tests/test_f_scaling.rs
// cargo test --test test_f_scaling --release -- --nocapture

mod common;
use common::test_f;
use mitsukeru::{BranchAndBoundSolver, Optimizer, ProblemInput, SolutionStatus, SolverStrategy};
use std::time::Instant;

fn format_num(val: u64) -> String {
    let s = val.to_string();
    let mut result = String::new();
    let offset = s.len() % 3;
    for (i, ch) in s.chars().enumerate() {
        if i > 0 && (i % 3 == offset || (offset == 0 && i % 3 == 0)) {
            result.push(',');
        }
        result.push(ch);
    }
    result
}

#[test]
fn test_f_scale_sweep() {
    println!("\n==========================================================================================");
    println!("                           Test F Scaling & Stress Sweep Benchmark                        ");
    println!("==========================================================================================");
    println!(
        "{:<6} | {:<9} | {:<10} | {:<10} | {:<12} | {:<10} | {:<8}",
        "Items", "Strategy", "Status", "Score", "Nodes", "Time (ms)", "Mnodes/s"
    );
    println!("------------------------------------------------------------------------------------------");

    let n_list = [16, 32, 64, 128, 256, 512, 1024];

    for &n in &n_list {
        let prob = test_f(n);

        for &strategy in &[SolverStrategy::Pure, SolverStrategy::Info, SolverStrategy::Mixed] {
            let strat_label = match strategy {
                SolverStrategy::Pure => "Pure",
                SolverStrategy::Info => "Info",
                SolverStrategy::Mixed => "Mixed",
            };

            let input = ProblemInput {
                items: prob.items.clone(),
                capacities: prob.capacities.clone(),
                rules: prob.rules.clone(),
                target_value: prob.target_val,
                strategy,
            };

            let start = Instant::now();
            let opt = Optimizer::new(&input);
            let solution = opt.solve().expect("Should solve successfully");
            let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;

            let secs = elapsed_ms / 1000.0;
            let mnodes_per_sec = if secs > f64::EPSILON {
                (solution.explored_nodes as f64) / secs / 1_000_000.0
            } else {
                0.0
            };

            let status_str = match solution.status {
                SolutionStatus::Optimal => "Optimal",
                SolutionStatus::Approximate => "Approx",
            };

            println!(
                "{:<6} | {:<9} | {:<10} | {:<10} | {:<12} | {:<10.2} | {:<8.1}",
                n,
                strat_label,
                status_str,
                format!("{} pt", solution.best_value),
                format_num(solution.explored_nodes),
                elapsed_ms,
                mnodes_per_sec
            );

            assert!(
                solution.best_value > 0,
                "Failed to find positive solution for n={}",
                n
            );
        }
        println!("------------------------------------------------------------------------------------------");
    }
}