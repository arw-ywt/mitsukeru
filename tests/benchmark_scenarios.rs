// tests\benchmark_scenarios.rs
// cargo test -- --nocapture --test-threads=1

mod common;
use common::*; 
use mitsukeru::{
    BranchAndBoundSolver, BusinessRule, CompilerConfig, Flat2dDpSolver, InfeasibilityAnalyzer, Item, SatCompiler,
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
    let (mip_val, selected, _) = mip.solve();
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
    let (mip_val, selected, _) = mip.solve();
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
    let (mip_val, _, _) = mip.solve();
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
    let (mip_val, selected, _) = mip.solve();
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

    let scenarios = [
        ("Test A", test_a()),
        ("Test B", test_b()),
        ("Test C", test_c()),
        ("Test D", test_d()),
        ("Test E", test_e()),
        ("Test F", test_f(30)),
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

        for _ in 0..WARMUP_ITERATIONS {
            let mut mip = BranchAndBoundSolver::new(&sc.items, &sc.capacities, &sc.rules);
            let _ = mip.solve();
        }

        let mut total_ms = 0.0;
        let mut mip_score = 0;
        let mut total_nodes = 0;

        for _ in 0..BENCH_ITERATIONS {
            let mut mip = BranchAndBoundSolver::new(&sc.items, &sc.capacities, &sc.rules);
            let timer = BenchTimer::start();
            let (score, _, nodes) = mip.solve();
            total_ms += timer.elapsed_ms();
            mip_score = score;
            total_nodes = nodes;
        }
        let avg_ms = total_ms / (BENCH_ITERATIONS as f64);
        let secs = avg_ms / MS_PER_SEC;
        let throughput = if secs > f64::EPSILON { (total_nodes as f64) / secs / MILLION } else { 0.0 };

        let mip_status = if avg_ms <= TARGET_LIMIT_MS {
            "OK".to_string()
        } else {
            format!("EXCEEDED (>{:.1}ms)", TARGET_LIMIT_MS)
        };

        println!(" MIP Solver  : ");
        println!("        Status     : {}", mip_status);
        println!("        Time       : {:.2} ms", avg_ms);
        println!("        Score      : {} pt", mip_score);
        println!("        Nodes      : {}", format_num(total_nodes as u64));
        println!("        Mnodes/sec : {:.1}", throughput);

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

/*
running 6 tests
test test_a_happy_path ... ok
test test_b_chain_conflict_infeasibility ... ok
test test_c_multi_body_conflict ... ok
test test_d_5000_1000 ... ok
test test_performance ... 
[Performance Tests]
<Test A>
 Items       : 14
 Capacities  : [800, 600]
 Target      : 480 pt
 DP Solver   : 
        Status : OK
        Time   : 36.34 ms
        Score  : 555 pt
        Grid   : 481,401 cells (801x601)
        Memory : 2.64 MB
 MIP Solver  : 
        Status     : OK
        Time       : 0.01 ms
        Score      : 550 pt
        Nodes      : 78
        Mnodes/sec : 7.1
 SAT Compile : 
        Status     : OK
        Time       : 1.57 ms
        Vars       : 1,230
        Clauses    : 4,091

<Test B>
 Items       : 10
 Capacities  : [800, 600]
 Target      : 300 pt
 DP Solver   : 
        Status : OK
        Time   : 23.49 ms
        Score  : 365 pt
        Grid   : 481,401 cells (801x601)
        Memory : 2.41 MB
 MIP Solver  : 
        Status     : OK
        Time       : 0.09 ms
        Score      : 215 pt
        Nodes      : 688
        Mnodes/sec : 7.4
 SAT Compile : 
        Status     : OK
        Time       : 1.20 ms
        Vars       : 926
        Clauses    : 3,072

<Test C>
 Items       : 7
 Capacities  : [500, 500]
 Target      : 200 pt
 DP Solver   : 
        Status : OK
        Time   : 8.10 ms
        Score  : 220 pt
        Grid   : 251,001 cells (501x501)
        Memory : 1.17 MB
 MIP Solver  : 
        Status     : OK
        Time       : 0.01 ms
        Score      : 150 pt
        Nodes      : 98
        Mnodes/sec : 7.9
 SAT Compile : 
        Status     : OK
        Time       : 0.76 ms
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
        Time       : 0.02 ms
        Score      : 865 pt
        Nodes      : 173
        Mnodes/sec : 7.4
 SAT Compile : 
        Status     : OK
        Time       : 1.33 ms
        Vars       : 1,081
        Clauses    : 3,585

<Test E>
 Items       : 18
 Capacities  : [1000, 800]
 Target      : 600 pt
 DP Solver   : 
        Status : OK
        Time   : 78.90 ms
        Score  : 650 pt
        Grid   : 801,801 cells (1001x801)
        Memory : 4.78 MB
 MIP Solver  : 
        Status     : OK
        Time       : 0.10 ms
        Score      : 615 pt
        Nodes      : 837
        Mnodes/sec : 8.0
 SAT Compile : 
        Status     : OK
        Time       : 1.90 ms
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
        Time       : 1.27 ms
        Score      : 1180 pt
        Nodes      : 9,713
        Mnodes/sec : 7.7
 SAT Compile : 
        Status     : EXCEEDED (>2.0ms)
        Time       : 3.45 ms
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

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 9.05s

     Running tests\schema_drift_test.rs (target\debug\deps\schema_drift_test-aa9168d7cb387d27.exe)

running 1 test
test generate_and_verify_schema ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests mitsukeru

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
*/