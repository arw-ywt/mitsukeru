// tests/integration_tests.rs
// cargo test --release -- --nocapture --test-threads=1

mod common;
use common::*; 
use mitsukeru::{
    BranchAndBoundSolver,
    CompilerConfig, 
    Flat2dDpSolver, 
    InfeasibilityAnalyzer, 
    Item, 
    SatCompiler,
    GraphHeuristics, 
    Optimizer, 
    SolutionStatus,
};

const MILLION: f64 = 1_000_000.0;
const MS_PER_SEC: f64 = 1_000.0;
const WARMUP_ITERATIONS: usize = 3;
const BENCH_ITERATIONS_U32: u32 = 50;
const BENCH_ITERATIONS: usize = BENCH_ITERATIONS_U32 as usize;
const BENCH_ITERATIONS_F64: f64 = BENCH_ITERATIONS_U32 as f64;
const BYTES_PER_MB_F64: f64 = 1024.0 * 1024.0;

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

fn bytes_to_mb(bytes: usize) -> f64 {
    const BYTES_PER_MB: usize = 1024 * 1024;
    let mb_whole = u32::try_from(bytes / BYTES_PER_MB).unwrap_or(u32::MAX);
    let mb_rem = u32::try_from(bytes % BYTES_PER_MB).unwrap_or(0);
    f64::from(mb_whole) + (f64::from(mb_rem) / BYTES_PER_MB_F64)
}

fn calc_throughput(nodes: u64, secs: f64) -> f64 {
    if secs <= f64::EPSILON {
        return 0.0;
    }
    let mnodes = u32::try_from(nodes / 1_000_000).unwrap_or(u32::MAX);
    let rem_nodes = u32::try_from(nodes % 1_000_000).unwrap_or(0);
    (f64::from(mnodes) + (f64::from(rem_nodes) / MILLION)) / secs
}

pub struct BenchTimer {
    #[cfg(not(target_arch = "wasm32"))]
    start: std::time::Instant,
    #[cfg(target_arch = "wasm32")]
    start_ms: f64,
}

impl BenchTimer {
    #[must_use] 
    pub fn start() -> Self {
        Self {
            #[cfg(not(target_arch = "wasm32"))]
            start: std::time::Instant::now(),
            #[cfg(target_arch = "wasm32")]
            start_ms: web_sys::window().and_then(|w| w.performance()).map_or(0.0, |p| p.now()),
        }
    }

    #[must_use] 
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

    #[must_use] 
    pub fn elapsed_secs(&self) -> f64 {
        self.elapsed_ms() / MS_PER_SEC
    }

    #[must_use] 
    pub fn throughput_mops(&self, operations: u64) -> f64 {
        calc_throughput(operations, self.elapsed_secs())
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
        name: format!("F_{i:03}"),
        weights: vec![100, 100],
        value: 50,
    }).collect();

    let cap_x = 2500;
    let cap_y = 2500;
    let mut dp_large = Flat2dDpSolver::new(cap_x, cap_y);
    dp_large.solve(&dummy_items);
    let (table_bytes, history_bytes, total_alloc_bytes) = dp_large.allocated_bytes();

    let grid_x = u64::from(cap_x) + 1;
    let grid_y = u64::from(cap_y) + 1;
    let grid_cells = grid_x * grid_y;

    println!("\n[Memory Test]");
    println!(" Target    : Flat2dDpSolver");
    println!(" Problem   : ");
    println!("        Items: {}", dummy_items.len());
    println!("        Capacities: [{cap_x}, {cap_y}]");
    println!("        Grid Cells: {} ({}x{})", format_num(grid_cells), grid_x, grid_y);
    println!(" Status    : PASSED\n");

    println!(" +-------------------+--------------------+--------------------+");
    println!(" | Component         | Allocated (Bytes)  | Size (MB)          |");
    println!(" +-------------------+--------------------+--------------------+");
    println!(" | DP Table Buffer   | {:>18} | {:>15.2} MB |", format_num(table_bytes as u64), bytes_to_mb(table_bytes));
    println!(" | Bitset History    | {:>18} | {:>15.2} MB |", format_num(history_bytes as u64), bytes_to_mb(history_bytes));
    println!(" +-------------------+--------------------+--------------------+");
    println!(" | Total Allocated   | {:>18} | {:>15.2} MB |", format_num(total_alloc_bytes as u64), bytes_to_mb(total_alloc_bytes));
    println!(" +-------------------+--------------------+--------------------+");
}

fn bench_dp(sc: &ProblemInstance, grid_cells: u64, grid_w: u64, grid_h: u64) {
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
            mem_total_mb = bytes_to_mb(tot);
        }
        let avg_ms = total_ms / BENCH_ITERATIONS_F64;

        println!("        Status : OK");
        println!("        Time   : {avg_ms:.2} ms");
        println!("        Score  : {dp_score} pt");
        println!("        Grid   : {} cells ({}x{})", format_num(grid_cells), grid_w, grid_h);
        println!("        Memory : {mem_total_mb:.2} MB");
    } else {
        println!("        Status : SKIPPED (OOM)");
        println!("        Time   : ---");
        println!("        Score  : ---");
        println!("        Grid   : {} cells ({}x{})", format_num(grid_cells), grid_w, grid_h);
        println!("        Memory : ---");
    }
}

fn bench_pure_bnb(sc: &ProblemInstance, target_limit: f64, single_run_thresh: f64) -> u32 {
    println!(" Pure BnB    : ");

    let probe_timer = BenchTimer::start();
    let mut mip = BranchAndBoundSolver::new(&sc.items, &sc.capacities, &sc.rules);
    mip.set_time_limit_ms(target_limit);
    mip.set_max_nodes(100_000);
    let (mut mip_score, _, mut total_nodes, mut mip_status_code) = mip.solve();
    let first_run_ms = probe_timer.elapsed_ms();

    let avg_ms = if first_run_ms > single_run_thresh {
        first_run_ms
    } else {
        for _ in 0..WARMUP_ITERATIONS {
            let mut mip = BranchAndBoundSolver::new(&sc.items, &sc.capacities, &sc.rules);
            mip.set_time_limit_ms(target_limit);
            mip.set_max_nodes(100_000);
            let _ = mip.solve();
        }

        let mut total_ms = 0.0;
        for _ in 0..BENCH_ITERATIONS {
            let mut mip = BranchAndBoundSolver::new(&sc.items, &sc.capacities, &sc.rules);
            mip.set_time_limit_ms(target_limit);
            mip.set_max_nodes(100_000);
            let timer = BenchTimer::start();
            let (score, _, nodes, status) = mip.solve();
            total_ms += timer.elapsed_ms();
            mip_score = score;
            total_nodes = nodes;
            mip_status_code = status;
        }
        total_ms / BENCH_ITERATIONS_F64
    };

    let throughput = calc_throughput(total_nodes, avg_ms / MS_PER_SEC);
    let mip_status = if avg_ms <= target_limit {
        "OK".to_string()
    } else {
        format!("EXCEEDED (>{target_limit:.1}ms)")
    };

    println!("        Status     : {mip_status}");
    println!("        Solution   : {mip_status_code:?}");
    println!("        Time       : {avg_ms:.2} ms");
    println!("        Score      : {mip_score} pt");
    println!("        Nodes      : {}", format_num(total_nodes));
    println!("        Mnodes/sec : {throughput:.1}");

    mip_score
}

fn bench_heuristic_bnb(sc: &ProblemInstance, target_limit: f64, single_run_thresh: f64) {
    println!(" Heuristic BnB    : ");

    let heur_probe_timer = BenchTimer::start();
    let prediction = GraphHeuristics::predict(&sc.items, &sc.capacities, &sc.rules);
    let mut mip = BranchAndBoundSolver::new(&sc.items, &sc.capacities, &sc.rules);
    mip.set_initial_solution(prediction.initial_best_value, &prediction.initial_selected_ids);
    mip.set_time_limit_ms(target_limit);
    mip.set_max_nodes(100_000);
    let (mut heur_score, _, mut heur_nodes, mut heur_status_code) = mip.solve();
    let heur_first_run_ms = heur_probe_timer.elapsed_ms();

    let avg_heur_ms = if heur_first_run_ms > single_run_thresh {
        heur_first_run_ms
    } else {
        let mut heur_total_ms = 0.0;
        for _ in 0..BENCH_ITERATIONS {
            let timer = BenchTimer::start();
            let pred = GraphHeuristics::predict(&sc.items, &sc.capacities, &sc.rules);
            let mut mip = BranchAndBoundSolver::new(&sc.items, &sc.capacities, &sc.rules);
            mip.set_initial_solution(pred.initial_best_value, &pred.initial_selected_ids);
            mip.set_time_limit_ms(target_limit);
            mip.set_max_nodes(100_000);
            let (score, _, nodes, status) = mip.solve();
            heur_total_ms += timer.elapsed_ms();
            heur_score = score;
            heur_nodes = nodes;
            heur_status_code = status;
        }
        heur_total_ms / BENCH_ITERATIONS_F64
    };

    let heur_throughput = calc_throughput(heur_nodes, avg_heur_ms / MS_PER_SEC);
    let heur_status = if avg_heur_ms <= target_limit {
        "OK".to_string()
    } else {
        format!("EXCEEDED (>{target_limit:.1}ms)")
    };

    println!("        Status     : {heur_status}");
    println!("        Solution   : {heur_status_code:?}");
    println!("        Time       : {avg_heur_ms:.2} ms");
    println!("        Score      : {heur_score} pt");
    println!("        Nodes      : {}", format_num(heur_nodes));
    println!("        Mnodes/sec : {heur_throughput:.1}");
}

fn bench_mixed_auto(sc: &ProblemInstance, target_limit: f64, single_run_thresh: f64) {
    println!(" Mixed Auto   : ");

    let run_mixed = || {
        let rule_count = sc.rules.len();
        let item_count = sc.items.len();
        let use_heuristic = rule_count > 0 
            && item_count >= 20 
            && (item_count <= 64 || rule_count * 4 >= item_count);

        let mut mip = BranchAndBoundSolver::new(&sc.items, &sc.capacities, &sc.rules);
        if use_heuristic {
            let prediction = GraphHeuristics::predict(&sc.items, &sc.capacities, &sc.rules);
            mip.set_initial_solution(prediction.initial_best_value, &prediction.initial_selected_ids);
        }
        mip.set_time_limit_ms(target_limit);
        mip.set_max_nodes(100_000);
        let (score, _, nodes, status) = mip.solve();
        (score, nodes, status, if use_heuristic { "Heuristic" } else { "Pure" })
    };

    let mixed_probe_timer = BenchTimer::start();
    let (mut mixed_score, mut mixed_nodes, mut mixed_status_code, mut mixed_selected) = run_mixed();
    let mixed_first_run_ms = mixed_probe_timer.elapsed_ms();

    let avg_mixed_ms = if mixed_first_run_ms > single_run_thresh {
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
        mixed_total_ms / BENCH_ITERATIONS_F64
    };

    let mixed_throughput = calc_throughput(mixed_nodes, avg_mixed_ms / MS_PER_SEC);
    let mixed_status = if avg_mixed_ms <= target_limit {
        format!("OK (via {mixed_selected})")
    } else {
        format!("EXCEEDED (>{target_limit:.1}ms, via {mixed_selected})")
    };

    println!("        Status     : {mixed_status}");
    println!("        Solution   : {mixed_status_code:?}");
    println!("        Time       : {avg_mixed_ms:.2} ms");
    println!("        Score      : {mixed_score} pt");
    println!("        Nodes      : {}", format_num(mixed_nodes));
    println!("        Mnodes/sec : {mixed_throughput:.1}");
}

fn bench_sat_compile(sc: &ProblemInstance, mip_score: u32, target_limit: f64, timeout_ms: f64) {
    let sat_config = CompilerConfig::default();
    let target = sc.target_val.unwrap_or(mip_score);
    let probe_timer = BenchTimer::start();
    let _ = SatCompiler::compile(&sc.items, &sc.capacities, &sc.rules, target, &sat_config);
    let probe_ms = probe_timer.elapsed_ms();

    println!(" SAT Compile : ");
    if probe_ms > timeout_ms {
        println!("        Status     : SKIPPED (Timeout >{timeout_ms:.0}ms)");
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
        let avg_sat_ms = sat_total_ms / BENCH_ITERATIONS_F64;

        let sat_status = if avg_sat_ms <= target_limit {
            "OK".to_string()
        } else {
            format!("EXCEEDED (>{target_limit:.1}ms)")
        };

        println!("        Status     : {sat_status}");
        println!("        Time       : {avg_sat_ms:.2} ms");
        println!("        Vars       : {}", format_num(sat_vars as u64));
        println!("        Clauses    : {}", format_num(sat_clauses as u64));
    }
}

#[test]
fn test_performance() {
    const TARGET_LIMIT_MS: f64 = 2.0;
    const SAT_SKIP_TIMEOUT_MS: f64 = 15.0;
    const SINGLE_RUN_THRESHOLD_MS: f64 = 5.0;

    println!("\n[Performance Tests]");

    let test_cases = [
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

    for (label, sc) in &test_cases {
        let cap_x = u64::from(sc.capacities[0]);
        let cap_y = u64::from(sc.capacities[1]);
        let grid_w = cap_x + 1;
        let grid_h = cap_y + 1;
        let grid_cells = grid_w * grid_h;

        println!("<{label}>");
        println!(" Items       : {}", sc.items.len());
        println!(" Capacities  : [{}, {}]", sc.capacities[0], sc.capacities[1]);
        if let Some(target) = sc.target_val {
            println!(" Target      : {target} pt");
        }

        bench_dp(sc, grid_cells, grid_w, grid_h);
        let mip_score = bench_pure_bnb(sc, TARGET_LIMIT_MS, SINGLE_RUN_THRESHOLD_MS);
        bench_heuristic_bnb(sc, TARGET_LIMIT_MS, SINGLE_RUN_THRESHOLD_MS);
        bench_mixed_auto(sc, TARGET_LIMIT_MS, SINGLE_RUN_THRESHOLD_MS);
        bench_sat_compile(sc, mip_score, TARGET_LIMIT_MS, SAT_SKIP_TIMEOUT_MS);
        println!();
    }
}

#[test]
fn test_heuristic_acceleration() {
    let p = test_d();

    let mut baseline_solver = BranchAndBoundSolver::new(&p.items, &p.capacities, &p.rules);
    let (base_val, _, base_nodes, base_status) = baseline_solver.solve();
    assert_eq!(base_status, SolutionStatus::Optimal);

    let prediction = GraphHeuristics::predict(&p.items, &p.capacities, &p.rules);
    let mut accelerated_solver = BranchAndBoundSolver::new(&p.items, &p.capacities, &p.rules);

    accelerated_solver.set_initial_solution(prediction.initial_best_value, &prediction.initial_selected_ids);
    
    let (heur_val, heur_selected, heur_nodes, heur_status) = accelerated_solver.solve();
    assert_eq!(heur_status, SolutionStatus::Optimal);

    assert!(
        heur_nodes <= base_nodes,
        "graph-based heuristics should reduce search nodes: AI={heur_nodes} vs Base={base_nodes}"
    );

    assert_eq!(
        heur_val, base_val,
        "AI accelerated solution must retain 100% optimal value: AI={heur_val} vs Base={base_val}"
    );

    let mut w0 = 0;
    let mut w1 = 0;
    for &id in &heur_selected {
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
    };
    let opt_large = Optimizer::new(&input_large);
    let res_large = opt_large.solve().expect("Should solve large problem");
    assert!(
        res_large.status == SolutionStatus::Optimal || res_large.status == SolutionStatus::Approximate,
        "Status should be valid: got {:?}",
        res_large.status
    );
    assert!(res_large.best_value >= 1000);
}

#[test]
fn test_optimizer_strategy_switching() {
    let prob_a = test_a();
    let input_a = mitsukeru::ProblemInput {
        items: prob_a.items,
        capacities: prob_a.capacities,
        rules: prob_a.rules,
        target_value: prob_a.target_val,
    };
    let a_mixed_sol = Optimizer::new(&input_a).solve().unwrap();
    assert_eq!(a_mixed_sol.strategy_used, Some(mitsukeru::SolverStrategy::Pure));
    assert_eq!(a_mixed_sol.best_value, 550);

    let sol_a_info = Optimizer::new(&input_a)
        .with_strategy(mitsukeru::SolverStrategy::Heuristic)
        .solve()
        .unwrap();
    assert_eq!(sol_a_info.strategy_used, Some(mitsukeru::SolverStrategy::Heuristic));
    assert_eq!(sol_a_info.best_value, 550);

    let prob_g = test_g();
    let input_g = mitsukeru::ProblemInput {
        items: prob_g.items,
        capacities: prob_g.capacities,
        rules: prob_g.rules,
        target_value: prob_g.target_val,
    };

    let g_mixed_sol = Optimizer::new(&input_g).solve().unwrap();
    assert_eq!(g_mixed_sol.strategy_used, Some(mitsukeru::SolverStrategy::Heuristic));
    assert_eq!(g_mixed_sol.best_value, 1473);

    let sol_g_pure = Optimizer::new(&input_g)
        .with_strategy(mitsukeru::SolverStrategy::Pure)
        .solve()
        .unwrap();
    assert_eq!(sol_g_pure.strategy_used, Some(mitsukeru::SolverStrategy::Pure));
    assert_eq!(sol_g_pure.best_value, 1473);
    assert!(
        g_mixed_sol.explored_nodes < sol_g_pure.explored_nodes,
        "Heuristic BnB selected by Mixed should explore fewer nodes than Pure BnB"
    );
}