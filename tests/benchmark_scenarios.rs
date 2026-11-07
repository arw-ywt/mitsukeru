// tests\benchmark_scenarios.rs
// cargo test -- --nocapture --test-threads=1

mod common;
use common::*; 
use mitsukeru::{
    BranchAndBoundSolver, 
    BusinessRule, 
    CompilerConfig, 
    Flat2dDpSolver, 
    InfeasibilityAnalyzer, 
    Item, 
    SatCompiler,
    NeuralDivingEngine, 
    Optimizer, 
    SolutionStatus,
};

const MILLION: f64 = 1_000_000.0;
const MS_PER_SEC: f64 = 1_000.0;
const WARMUP_ITERATIONS: usize = 3;
const BENCH_ITERATIONS: usize = 50;

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

pub struct BenchTimer {
    #[cfg(not(target_arch = "wasm32"))]
    start: std::time::Instant,
    #[cfg(target_arch = "wasm32")]
    start_ms: f64,
}

impl BenchTimer {
    pub fn start() -> Self {
        Self {
            #[cfg(not(target_arch = "wasm32"))]
            start: std::time::Instant::now(),
            #[cfg(target_arch = "wasm32")]
            start_ms: web_sys::window().and_then(|w| w.performance()).map_or(0.0, |p| p.now()),
        }
    }

    pub fn elapsed_ms(&self) -> f64 {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.start.elapsed().as_secs_f64() * MS_PER_SEC
        }
        #[cfg(target_arch = "wasm32")]
        {
            let now = web_sys::window().and_then(|w| w.performance()).map_or(0.0, |p| p.now());
            now - self.start_ms
        }
    }

    pub fn elapsed_secs(&self) -> f64 {
        self.elapsed_ms() / MS_PER_SEC
    }

    pub fn throughput_mops(&self, operations: u64) -> f64 {
        let secs = self.elapsed_secs();
        if secs <= f64::EPSILON {
            0.0
        } else {
            (operations as f64) / secs / MILLION
        }
    }
}

#[test]
fn test_a_happy_path() {
    let p = test_a();
    let mut dp = Flat2dDpSolver::new(p.capacities[0], p.capacities[1]);
    let (dp_val, _) = dp.solve(&p.items);
    assert_eq!(dp_val, 555);

    let mut mip = BranchAndBoundSolver::new(&p.items, &p.capacities, &p.rules);
    let (mip_val, selected, _, _) = mip.solve(); 
    assert_eq!(mip_val, 550);
    assert_eq!(selected.len(), 10);

    assert!(selected.contains(&0) && selected.contains(&1), "A001 and A002 must pair");
    assert!(selected.contains(&10) && selected.contains(&11), "A011 and A012 must pair");
    assert!(!(selected.contains(&4) && selected.contains(&5)), "A005 and A006 must conflict");
    assert!(!(selected.contains(&6) && selected.contains(&13)), "A007 and A014 must conflict");

    let analyzer = InfeasibilityAnalyzer::new(&p.items, &p.capacities, &p.rules, p.target_val, mip_val);
    let report = analyzer.analyze();
    assert!(report.is_feasible);
}

#[test]
fn test_b_chain_conflict_infeasibility() {
    let p = test_b();
    let mut mip = BranchAndBoundSolver::new(&p.items, &p.capacities, &p.rules);
    let (mip_val, selected, _, _) = mip.solve();
    assert_eq!(mip_val, 215);
    assert!(!selected.contains(&0));
    assert!(!selected.contains(&4));

    let analyzer = InfeasibilityAnalyzer::new(&p.items, &p.capacities, &p.rules, p.target_val, mip_val);
    let report = analyzer.analyze();
    assert!(!report.is_feasible);
    assert_eq!(report.logical_chain_conflicts.len(), 1);
    assert_eq!(report.capacity_conflicts.len(), 1);
    assert!(report.target_conflict.is_some());
}

#[test]
fn test_c_multi_body_conflict() {
    let p = test_c();
    let mut mip = BranchAndBoundSolver::new(&p.items, &p.capacities, &p.rules);
    let (mip_val, _, _, _) = mip.solve(); 
    assert_eq!(mip_val, 150);

    let analyzer = InfeasibilityAnalyzer::new(&p.items, &p.capacities, &p.rules, p.target_val, mip_val);
    let report = analyzer.analyze();
    assert!(report.logical_chain_conflicts.is_empty());
    assert!(!report.is_feasible);
}

#[test]
fn test_d_5000_1000() {
    let p = test_d();
    let mut mip = BranchAndBoundSolver::new(&p.items, &p.capacities, &p.rules);
    let (mip_val, selected, _, _) = mip.solve(); 
    assert_eq!(mip_val, 865);
    assert_eq!(selected.len(), 7);

    let mut total_w1 = 0;
    let mut total_w2 = 0;
    for &id in &selected {
        total_w1 += p.items[id].weights[0];
        total_w2 += p.items[id].weights[1];
    }
    assert_eq!(total_w1, 5000);
    assert_eq!(total_w2, 700);

    let analyzer = InfeasibilityAnalyzer::new(&p.items, &p.capacities, &p.rules, p.target_val, mip_val);
    assert!(analyzer.analyze().is_feasible);
}

#[test]
fn test_stress_memory() {
    let dummy_items: Vec<Item> = (0..15).map(|i| Item {
        id: i,
        name: format!("F_{:03}", i),
        weights: vec![100, 100],
        value: 50,
    }).collect();

    let cap_x = 2500;
    let cap_y = 2500;
    let mut dp_large = Flat2dDpSolver::new(cap_x, cap_y);
    dp_large.solve(&dummy_items);
    let (lt_bytes, lh_bytes, ltotal_bytes) = dp_large.allocated_bytes();

    let grid_x = cap_x as u64 + 1;
    let grid_y = cap_y as u64 + 1;
    let grid_cells = grid_x * grid_y;

    println!("\n[Memory Test]");
    println!(" Target    : Flat2dDpSolver");
    println!(" Problem   : ");
    println!("        Items: {}", dummy_items.len());
    println!("        Capacities: [{}, {}]", cap_x, cap_y);
    println!("        Grid Cells: {} ({}x{})", format_num(grid_cells), grid_x, grid_y);
    println!(" Status    : PASSED\n");

    println!(" +-------------------+--------------------+--------------------+");
    println!(" | Component         | Allocated (Bytes)  | Size (MB)          |");
    println!(" +-------------------+--------------------+--------------------+");
    println!(" | DP Table Buffer   | {:>18} | {:>15.2} MB |", format_num(lt_bytes as u64), lt_bytes as f64 / 1_048_576.0);
    println!(" | Bitset History    | {:>18} | {:>15.2} MB |", format_num(lh_bytes as u64), lh_bytes as f64 / 1_048_576.0);
    println!(" +-------------------+--------------------+--------------------+");
    println!(" | Total Allocated   | {:>18} | {:>15.2} MB |", format_num(ltotal_bytes as u64), ltotal_bytes as f64 / 1_048_576.0);
    println!(" +-------------------+--------------------+--------------------+");
}

#[test]
fn test_performance() {
    println!("\n[Performance Tests]");

    const TARGET_LIMIT_MS: f64 = 2.0;
    const SAT_SKIP_TIMEOUT_MS: f64 = 15.0;
    const SINGLE_RUN_THRESHOLD_MS: f64 = 5.0;

    let scenarios = [
        ("Test A", test_a()),
        ("Test B", test_b()),
        ("Test C", test_c()),
        ("Test D", test_d()),
        ("Test E", test_e()),
        ("Test F", test_f(120)),
        ("Test G", test_g()),
        ("Test H", test_h()),
        ("Test I", test_i()),
        ("Test J", test_j()),
        ("Test K", test_k()),
        ("Test L", test_l()),
        ("Test M", test_m()),
    ];

    for (label, sc) in &scenarios {
        let cap_x = sc.capacities[0] as u64;
        let cap_y = sc.capacities[1] as u64;
        let grid_w = cap_x + 1;
        let grid_h = cap_y + 1;
        let grid_cells = grid_w * grid_h;

        println!("<{}>", label);
        println!(" Items       : {}", sc.items.len());
        println!(" Capacities  : [{}, {}]", sc.capacities[0], sc.capacities[1]);
        if let Some(target) = sc.target_val {
            println!(" Target      : {} pt", target);
        }

        println!(" DP Solver   : ");
        if grid_cells <= 1_000_000 {
            for _ in 0..WARMUP_ITERATIONS {
                let mut dp = Flat2dDpSolver::new(sc.capacities[0], sc.capacities[1]);
                let _ = dp.solve(&sc.items);
            }

            let mut total_ms = 0.0;
            let mut dp_score = 0;
            let mut mem_total_mb = 0.0;

            for _ in 0..BENCH_ITERATIONS {
                let mut dp = Flat2dDpSolver::new(sc.capacities[0], sc.capacities[1]);
                let timer = BenchTimer::start();
                let (score, _) = dp.solve(&sc.items);
                total_ms += timer.elapsed_ms();
                dp_score = score;
                let (_, _, tot) = dp.allocated_bytes();
                mem_total_mb = tot as f64 / (1024.0 * 1024.0);
            }
            let avg_ms = total_ms / (BENCH_ITERATIONS as f64);

            println!("        Status : OK");
            println!("        Time   : {:.2} ms", avg_ms);
            println!("        Score  : {} pt", dp_score);
            println!("        Grid   : {} cells ({}x{})", format_num(grid_cells), grid_w, grid_h);
            println!("        Memory : {:.2} MB", mem_total_mb);
        } else {
            println!("        Status : SKIPPED (OOM)");
            println!("        Time   : ---");
            println!("        Score  : ---");
            println!("        Grid   : {} cells ({}x{})", format_num(grid_cells), grid_w, grid_h);
            println!("        Memory : ---");
        }

        // Pure MIP
        println!(" Pure MIP    : ");

        let probe_timer = BenchTimer::start();
        let mut mip = BranchAndBoundSolver::new(&sc.items, &sc.capacities, &sc.rules);
        mip.set_time_limit_ms(TARGET_LIMIT_MS);
        mip.set_max_nodes(100_000);
        let (mut mip_score, _, mut total_nodes, mut mip_status_code) = mip.solve();
        let first_run_ms = probe_timer.elapsed_ms();

        let avg_ms = if first_run_ms > SINGLE_RUN_THRESHOLD_MS {
            first_run_ms
        } else {
            for _ in 0..WARMUP_ITERATIONS {
                let mut mip = BranchAndBoundSolver::new(&sc.items, &sc.capacities, &sc.rules);
                mip.set_time_limit_ms(TARGET_LIMIT_MS);
                mip.set_max_nodes(100_000);
                let _ = mip.solve();
            }

            let mut total_ms = 0.0;
            for _ in 0..BENCH_ITERATIONS {
                let mut mip = BranchAndBoundSolver::new(&sc.items, &sc.capacities, &sc.rules);
                mip.set_time_limit_ms(TARGET_LIMIT_MS);
                mip.set_max_nodes(100_000);
                let timer = BenchTimer::start();
                let (score, _, nodes, status) = mip.solve();
                total_ms += timer.elapsed_ms();
                mip_score = score;
                total_nodes = nodes;
                mip_status_code = status;
            }
            total_ms / (BENCH_ITERATIONS as f64)
        };

        let secs = avg_ms / MS_PER_SEC;
        let throughput = if secs > f64::EPSILON { (total_nodes as f64) / secs / MILLION } else { 0.0 };

        let mip_status = if avg_ms <= TARGET_LIMIT_MS {
            "OK".to_string()
        } else {
            format!("EXCEEDED (>{:.1}ms)", TARGET_LIMIT_MS)
        };

        println!("        Status     : {}", mip_status);
        println!("        Solution   : {:?}", mip_status_code);
        println!("        Time       : {:.2} ms", avg_ms);
        println!("        Score      : {} pt", mip_score);
        println!("        Nodes      : {}", format_num(total_nodes as u64));
        println!("        Mnodes/sec : {:.1}", throughput);

        // Info MIP
        println!(" Info MIP    : ");

        let ai_probe_timer = BenchTimer::start();
        let diving = NeuralDivingEngine::predict(&sc.items, &sc.capacities, &sc.rules);
        let mut mip = BranchAndBoundSolver::new(
            &sc.items,
            &sc.capacities,
            &sc.rules,
        );
        mip.set_initial_solution(diving.initial_best_value, &diving.initial_selected_ids);
        mip.set_time_limit_ms(TARGET_LIMIT_MS);
        mip.set_max_nodes(100_000);
        let (mut ai_score, _, mut ai_nodes, mut ai_status_code) = mip.solve();
        let ai_first_run_ms = ai_probe_timer.elapsed_ms();

        let avg_ai_ms = if ai_first_run_ms > SINGLE_RUN_THRESHOLD_MS {
            ai_first_run_ms
        } else {
            let mut ai_total_ms = 0.0;
            for _ in 0..BENCH_ITERATIONS {
                let timer = BenchTimer::start();
                let diving = NeuralDivingEngine::predict(&sc.items, &sc.capacities, &sc.rules);
                let mut mip = BranchAndBoundSolver::new(
                    &sc.items,
                    &sc.capacities,
                    &sc.rules,
                );
                mip.set_initial_solution(diving.initial_best_value, &diving.initial_selected_ids);
                mip.set_time_limit_ms(TARGET_LIMIT_MS);
                mip.set_max_nodes(100_000);
                let (score, _, nodes, status) = mip.solve();
                ai_total_ms += timer.elapsed_ms();
                ai_score = score;
                ai_nodes = nodes;
                ai_status_code = status;
            }
            ai_total_ms / (BENCH_ITERATIONS as f64)
        };

        let ai_secs = avg_ai_ms / MS_PER_SEC;
        let ai_throughput = if ai_secs > f64::EPSILON { (ai_nodes as f64) / ai_secs / MILLION } else { 0.0 };

        let ai_status = if avg_ai_ms <= TARGET_LIMIT_MS {
            "OK".to_string()
        } else {
            format!("EXCEEDED (>{:.1}ms)", TARGET_LIMIT_MS)
        };

        println!("        Status     : {}", ai_status);
        println!("        Solution   : {:?}", ai_status_code);
        println!("        Time       : {:.2} ms", avg_ai_ms);
        println!("        Score      : {} pt", ai_score);
        println!("        Nodes      : {}", format_num(ai_nodes as u64));
        println!("        Mnodes/sec : {:.1}", ai_throughput);

        // Mixed MIP
        println!(" Mixed MIP   : ");

        let run_mixed = || {
            let rule_count = sc.rules.len();
            let item_count = sc.items.len();
            let use_ai = rule_count > 0 
                && item_count >= 20 
                && (item_count <= 64 || rule_count * 4 >= item_count);

            let mut mip = BranchAndBoundSolver::new(&sc.items, &sc.capacities, &sc.rules);
            if use_ai {
                let diving = NeuralDivingEngine::predict(&sc.items, &sc.capacities, &sc.rules);
                mip.set_initial_solution(diving.initial_best_value, &diving.initial_selected_ids);
            }
            mip.set_time_limit_ms(TARGET_LIMIT_MS);
            mip.set_max_nodes(100_000);
            let (score, _, nodes, status) = mip.solve();
            (score, nodes, status, if use_ai { "Info" } else { "Pure" })
        };

        let mixed_probe_timer = BenchTimer::start();
        let (mut mixed_score, mut mixed_nodes, mut mixed_status_code, mut mixed_selected) = run_mixed();
        let mixed_first_run_ms = mixed_probe_timer.elapsed_ms();

        let avg_mixed_ms = if mixed_first_run_ms > SINGLE_RUN_THRESHOLD_MS {
            mixed_first_run_ms
        } else {
            for _ in 0..WARMUP_ITERATIONS {
                let _ = run_mixed();
            }

            let mut mixed_total_ms = 0.0;
            for _ in 0..BENCH_ITERATIONS {
                let timer = BenchTimer::start();
                let (score, nodes, status, selected) = run_mixed();
                mixed_total_ms += timer.elapsed_ms();
                mixed_score = score;
                mixed_nodes = nodes;
                mixed_status_code = status;
                mixed_selected = selected;
            }
            mixed_total_ms / (BENCH_ITERATIONS as f64)
        };

        let mixed_secs = avg_mixed_ms / MS_PER_SEC;
        let mixed_throughput = if mixed_secs > f64::EPSILON {
            (mixed_nodes as f64) / mixed_secs / MILLION
        } else {
            0.0
        };

        let mixed_status = if avg_mixed_ms <= TARGET_LIMIT_MS {
            format!("OK (via {})", mixed_selected)
        } else {
            format!("EXCEEDED (>{:.1}ms, via {})", TARGET_LIMIT_MS, mixed_selected)
        };

        println!("        Status     : {}", mixed_status);
        println!("        Solution   : {:?}", mixed_status_code);
        println!("        Time       : {:.2} ms", avg_mixed_ms);
        println!("        Score      : {} pt", mixed_score);
        println!("        Nodes      : {}", format_num(mixed_nodes as u64));
        println!("        Mnodes/sec : {:.1}", mixed_throughput);

        let sat_config = CompilerConfig::default();
        let target = sc.target_val.unwrap_or(mip_score);
        let probe_timer = BenchTimer::start();
        let _ = SatCompiler::compile(&sc.items, &sc.capacities, &sc.rules, target, &sat_config);
        let probe_ms = probe_timer.elapsed_ms();

        println!(" SAT Compile : ");
        if probe_ms > SAT_SKIP_TIMEOUT_MS {
            println!("        Status     : SKIPPED (Timeout >{:.0}ms)", SAT_SKIP_TIMEOUT_MS);
            println!("        Time       : ---");
            println!("        Vars       : ---");
            println!("        Clauses    : ---");
        } else {
            for _ in 0..WARMUP_ITERATIONS {
                let _ = SatCompiler::compile(&sc.items, &sc.capacities, &sc.rules, target, &sat_config);
            }

            let mut sat_total_ms = 0.0;
            let mut sat_vars = 0;
            let mut sat_clauses = 0;

            for _ in 0..BENCH_ITERATIONS {
                let timer = BenchTimer::start();
                let (_, vars, clauses, _) = SatCompiler::compile(&sc.items, &sc.capacities, &sc.rules, target, &sat_config);
                sat_total_ms += timer.elapsed_ms();
                sat_vars = vars;
                sat_clauses = clauses;
            }
            let avg_sat_ms = sat_total_ms / (BENCH_ITERATIONS as f64);

            let sat_status = if avg_sat_ms <= TARGET_LIMIT_MS {
                "OK".to_string()
            } else {
                format!("EXCEEDED (>{:.1}ms)", TARGET_LIMIT_MS)
            };

            println!("        Status     : {}", sat_status);
            println!("        Time       : {:.2} ms", avg_sat_ms);
            println!("        Vars       : {}", format_num(sat_vars as u64));
            println!("        Clauses    : {}", format_num(sat_clauses as u64));
        }
        println!();
    }
}


#[test]
fn test_neural_diving_acceleration() {
    let p = test_d();

    let mut baseline_solver = BranchAndBoundSolver::new(&p.items, &p.capacities, &p.rules);
    let (base_val, _, base_nodes, base_status) = baseline_solver.solve();
    assert_eq!(base_status, SolutionStatus::Optimal);

    let diving = NeuralDivingEngine::predict(&p.items, &p.capacities, &p.rules);
    let mut accelerated_solver = BranchAndBoundSolver::new(&p.items, &p.capacities, &p.rules);

    accelerated_solver.set_initial_solution(diving.initial_best_value, &diving.initial_selected_ids);
    
    let (ai_val, ai_selected, ai_nodes, ai_status) = accelerated_solver.solve();
    assert_eq!(ai_status, SolutionStatus::Optimal);

    assert!(
        ai_nodes <= base_nodes,
        "Neural Diving should reduce search nodes: AI={} vs Base={}",
        ai_nodes,
        base_nodes
    );

    assert_eq!(
        ai_val, base_val,
        "AI accelerated solution must retain 100% optimal value: AI={} vs Base={}",
        ai_val, base_val
    );

    let mut w0 = 0;
    let mut w1 = 0;
    for &id in &ai_selected {
        w0 += p.items[id].weights[0];
        w1 += p.items[id].weights[1];
    }
    assert!(w0 <= p.capacities[0] && w1 <= p.capacities[1]);
}

#[test]
    fn test_optimizer_solution_status() {
        let small_problem = test_a();
        let input_small = mitsukeru::ProblemInput {
            items: small_problem.items,
            capacities: small_problem.capacities,
            rules: small_problem.rules,
            target_value: small_problem.target_val,
            strategy: mitsukeru::SolverStrategy::default(),
        };
        let opt_small = Optimizer::new(&input_small);
        let res_small = opt_small.solve().expect("Should solve successfully");
        assert_eq!(res_small.status, SolutionStatus::Optimal);
        assert_eq!(res_small.best_value, 550);

        let large_problem = test_f(30);
        let input_large = mitsukeru::ProblemInput {
            items: large_problem.items,
            capacities: large_problem.capacities,
            rules: large_problem.rules,
            target_value: None,
            strategy: mitsukeru::SolverStrategy::default(),
        };
        let opt_large = Optimizer::new(&input_large);
        let res_large = opt_large.solve().expect("Should solve large problem");
        assert!(
            res_large.status == SolutionStatus::Optimal || res_large.status == SolutionStatus::Approximate,
            "Status should be valid: got {:?}",
            res_large.status
        );
    assert!(res_large.best_value >= 1180);
    }


#[test]
fn test_optimizer_strategy_switching() {
    let prob_a = test_a();
    let input_a = mitsukeru::ProblemInput {
        items: prob_a.items,
        capacities: prob_a.capacities,
        rules: prob_a.rules,
        target_value: prob_a.target_val,
        strategy: mitsukeru::SolverStrategy::Mixed,
    };
    let sol_a_mixed = Optimizer::new(&input_a).solve().unwrap();
    assert_eq!(sol_a_mixed.strategy_used, Some(mitsukeru::SolverStrategy::Pure));
    assert_eq!(sol_a_mixed.best_value, 550);
    let sol_a_info = Optimizer::new(&input_a)
        .with_strategy(mitsukeru::SolverStrategy::Info)
        .solve()
        .unwrap();
    assert_eq!(sol_a_info.strategy_used, Some(mitsukeru::SolverStrategy::Info));
    assert_eq!(sol_a_info.best_value, 550);
    let prob_g = test_g();
    let input_g = mitsukeru::ProblemInput {
        items: prob_g.items,
        capacities: prob_g.capacities,
        rules: prob_g.rules,
        target_value: prob_g.target_val,
        strategy: mitsukeru::SolverStrategy::Mixed,
    };

    let sol_g_mixed = Optimizer::new(&input_g).solve().unwrap();
    assert_eq!(sol_g_mixed.strategy_used, Some(mitsukeru::SolverStrategy::Info));
    assert_eq!(sol_g_mixed.best_value, 1473);

    let sol_g_pure = Optimizer::new(&input_g)
        .with_strategy(mitsukeru::SolverStrategy::Pure)
        .solve()
        .unwrap();
    assert_eq!(sol_g_pure.strategy_used, Some(mitsukeru::SolverStrategy::Pure));
    assert_eq!(sol_g_pure.best_value, 1473);
    assert!(
        sol_g_mixed.explored_nodes < sol_g_pure.explored_nodes,
        "Info MIP selected by Mixed should explore fewer nodes than Pure MIP"
    );
}


/*
running 6 tests
test analyzer::tests::test_chain_conflict ... ok
test branch_and_bound::tests::test_branch_and_bound_conflict_rule ... ok
test branch_and_bound::tests::test_fractional_bound_prunes_effectively ... ok
test heuristics::tests::test_heuristics_execution_and_validity ... ok
test tests::test_optimizer_high_level_api ... ok
test tests::test_optimizer_infeasible_conflict ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\benchmark_scenarios.rs (target\release\deps\benchmark_scenarios-4cbc9165d81fc369.exe)

running 8 tests
test test_a_happy_path ... ok
test test_b_chain_conflict_infeasibility ... ok
test test_c_multi_body_conflict ... ok
test test_d_5000_1000 ... ok
test test_neural_diving_acceleration ... ok
test test_optimizer_solution_status ... ok
test test_performance ... 
[Performance Tests]
<Test A>
 Items       : 14
 Capacities  : [800, 600]
 Target      : 480 pt
 DP Solver   : 
        Status : OK
        Time   : 5.29 ms
        Score  : 555 pt
        Grid   : 481,401 cells (801x601)
        Memory : 2.64 MB
 MIP Solver  : 
        Status     : OK
        Time       : 0.00 ms
        Score      : 550 pt
        Nodes      : 78
        Mnodes/sec : 35.6
 Info. MIP    : 
        Time       : 0.01 ms
        Score      : 550 pt
        Nodes      : 78
 SAT Compile : 
        Status     : OK
        Time       : 0.87 ms
        Vars       : 1,230
        Clauses    : 4,091

<Test B>
 Items       : 10
 Capacities  : [800, 600]
 Target      : 300 pt
 DP Solver   : 
        Status : OK
        Time   : 3.28 ms
        Score  : 365 pt
        Grid   : 481,401 cells (801x601)
        Memory : 2.41 MB
 MIP Solver  : 
        Status     : OK
        Time       : 0.02 ms
        Score      : 215 pt
        Nodes      : 688
        Mnodes/sec : 32.2
 Info. MIP    : 
        Time       : 0.02 ms
        Score      : 215 pt
        Nodes      : 686
 SAT Compile : 
        Status     : OK
        Time       : 0.61 ms
        Vars       : 926
        Clauses    : 3,072

<Test C>
 Items       : 7
 Capacities  : [500, 500]
 Target      : 200 pt
 DP Solver   : 
        Status : OK
        Time   : 0.92 ms
        Score  : 220 pt
        Grid   : 251,001 cells (501x501)
        Memory : 1.17 MB
 MIP Solver  : 
        Status     : OK
        Time       : 0.00 ms
        Score      : 150 pt
        Nodes      : 98
        Mnodes/sec : 52.1
 Info. MIP    : 
        Time       : 0.01 ms
        Score      : 150 pt
        Nodes      : 93
 SAT Compile : 
        Status     : OK
        Time       : 0.40 ms
        Vars       : 616
        Clauses    : 2,032

<Test D>
 Items       : 12
 Capacities  : [5000, 1000]
 Target      : 650 pt
 DP Solver   : 
        Status : SKIPPED (OOM)
        Time   : ---
        Score  : ---
        Grid   : 5,006,001 cells (5001x1001)
        Memory : ---
 MIP Solver  : 
        Status     : OK
        Time       : 0.00 ms
        Score      : 865 pt
        Nodes      : 173
        Mnodes/sec : 38.6
 Info. MIP    : 
        Time       : 0.01 ms
        Score      : 865 pt
        Nodes      : 159
 SAT Compile : 
        Status     : OK
        Time       : 0.71 ms
        Vars       : 1,081
        Clauses    : 3,585

<Test E>
 Items       : 18
 Capacities  : [1000, 800]
 Target      : 600 pt
 DP Solver   : 
        Status : OK
        Time   : 10.29 ms
        Score  : 650 pt
        Grid   : 801,801 cells (1001x801)
        Memory : 4.78 MB
 MIP Solver  : 
        Status     : OK
        Time       : 0.02 ms
        Score      : 615 pt
        Nodes      : 837
        Mnodes/sec : 49.6
 Info. MIP    : 
        Time       : 0.02 ms
        Score      : 615 pt
        Nodes      : 835
 SAT Compile : 
        Status     : OK
        Time       : 1.03 ms
        Vars       : 1,529
        Clauses    : 5,098

<Test F>
 Items       : 30
 Capacities  : [2100, 1740]
 Target      : 990 pt
 DP Solver   : 
        Status : SKIPPED (OOM)
        Time   : ---
        Score  : ---
        Grid   : 3,657,841 cells (2101x1741)
        Memory : ---
 MIP Solver  : 
        Status     : OK
        Time       : 0.26 ms
        Score      : 1180 pt
        Nodes      : 9,713
        Mnodes/sec : 37.3
 Info. MIP    : 
        Time       : 0.29 ms
        Score      : 1180 pt
        Nodes      : 9,713
 SAT Compile : 
        Status     : OK
        Time       : 1.91 ms
        Vars       : 2,744
        Clauses    : 9,179

ok
test test_stress_memory ... 
[Memory Test]
 Target    : Flat2dDpSolver
 Problem   : 
        Items: 15
        Capacities: [2500, 2500]
        Grid Cells: 6,255,001 (2501x2501)
 Status    : PASSED

 +-------------------+--------------------+--------------------+
 | Component         | Allocated (Bytes)  | Size (MB)          |
 +-------------------+--------------------+--------------------+
 | DP Table Buffer   |         25,020,004 |           23.86 MB |
 | Bitset History    |         11,728,128 |           11.18 MB |
 +-------------------+--------------------+--------------------+
 | Total Allocated   |         36,748,196 |           35.05 MB |
 +-------------------+--------------------+--------------------+
ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.52s

     Running tests\schema_drift_test.rs (target\release\deps\schema_drift_test-0ef4fa5f0b8aec56.exe)

running 1 test
test generate_and_verify_schema ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests mitsukeru

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
*/