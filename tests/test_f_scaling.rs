// tests/test_f_scaling.rs
// cargo test --test test_f_scaling --release -- --nocapture

mod common;
use common::test_f;
use mitsukeru::{Optimizer, ProblemInput, SolutionStatus, SolverStrategy};
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
    const STACK_SIZE: usize = 8 * 1024 * 1024;

    let handler = std::thread::Builder::new()
        .name("f_scale_sweep".to_string())
        .stack_size(STACK_SIZE)
        .spawn(run_f_scale_sweep_inner)
        .expect("Failed to spawn sweep thread");

    handler.join().expect("Test thread panicked or overflowed");
}

fn run_f_scale_sweep_inner() {
    println!(
        "\n====================================================================================================="
    );
    println!(
        "                               Test F Scaling & Stress Sweep Benchmark                               "
    );
    println!(
        "====================================================================================================="
    );
    println!(
        "{:<6} | {:<9} | {:<10} | {:<10} | {:<9} | {:<12} | {:<10} | {:<8}",
        "Items", "Strategy", "Status", "Score", "Gap (%)", "Nodes", "Time (ms)", "Mnodes/s"
    );
    println!(
        "-----------------------------------------------------------------------------------------------------"
    );

    let n_list = if cfg!(debug_assertions) {
        vec![16, 32, 64, 128, 256]
    } else {
        vec![16, 32, 64, 128, 256, 512, 1024, 2048]
    };

    for &n in &n_list {
        let prob = test_f(n);

        for &strategy in &[
            SolverStrategy::Pure,
            SolverStrategy::Heuristic,
            SolverStrategy::Mixed,
        ] {
            let strat_label = match strategy {
                SolverStrategy::Pure => "Pure",
                SolverStrategy::Heuristic => "Heuristic",
                SolverStrategy::Mixed => "Mixed",
            };

            let input = ProblemInput {
                items: prob.items.clone(),
                capacities: prob.capacities.clone(),
                rules: prob.rules.clone(),
                target_value: prob.target_val,
            };

            let start = Instant::now();
            let opt = Optimizer::new(&input).with_strategy(strategy);
            let solution = opt.solve().expect("Should solve successfully");
            let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;

            let secs = elapsed_ms / 1000.0;
            let mnodes_per_sec = if secs > f64::EPSILON {
                let mnodes = u32::try_from(solution.explored_nodes / 1_000_000).unwrap_or(u32::MAX);
                let rem = u32::try_from(solution.explored_nodes % 1_000_000).unwrap_or(0);
                (f64::from(mnodes) + (f64::from(rem) / 1_000_000.0)) / secs
            } else {
                0.0
            };

            let status_str = match solution.status {
                SolutionStatus::Optimal => "Optimal",
                SolutionStatus::Approximate => "Approx",
            };

            let gap_str = format!("{:.2}%", solution.optimality_gap * 100.0);

            println!(
                "{:<6} | {:<9} | {:<10} | {:<10} | {:<9} | {:<12} | {:<10.2} | {:<8.1}",
                n,
                strat_label,
                status_str,
                format!("{} pt", solution.best_value),
                gap_str,
                format_num(solution.explored_nodes),
                elapsed_ms,
                mnodes_per_sec
            );

            assert!(
                solution.best_value > 0,
                "Failed to find positive solution for n={n}"
            );
        }
        println!(
            "-----------------------------------------------------------------------------------------------------"
        );
    }
}
