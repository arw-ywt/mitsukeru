use mitsukeru::{
    BranchAndBoundSolver, BusinessRule, CompilerConfig, Flat2dDpSolver, InfeasibilityAnalyzer, Item, SatCompiler,
};

const MILLION: f64 = 1_000_000.0;
const MS_PER_SEC: f64 = 1_000.0;
const WARMUP_ITERATIONS: usize = 3;
const BENCH_ITERATIONS: usize = 50;

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

#[derive(Clone)]
pub struct ProblemInstance {
    pub name: &'static str,
    pub items: Vec<Item>,
    pub capacities: Vec<u32>,
    pub rules: Vec<BusinessRule>,
    pub target_val: Option<u32>,
}

fn test_a() -> ProblemInstance {
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

fn test_b() -> ProblemInstance {
    ProblemInstance {
        name: "Test B (Chain Conflict, 10 items)",
        items: vec![
            Item { id: 0, name: "B_001".into(), weights: vec![140, 110], value: 95 },
            Item { id: 1, name: "B_002".into(), weights: vec![90,  80],  value: 40 },
            Item { id: 2, name: "B_003".into(), weights: vec![180, 50],  value: 30 },
            Item { id: 3, name: "B_004".into(), weights: vec![70,  40],  value: 25 },
            Item { id: 4, name: "B_005".into(), weights: vec![180, 120], value: 90 },
            Item { id: 5, name: "B_006".into(), weights: vec![320, 210], value: 35 },
            Item { id: 6, name: "B_007".into(), weights: vec![350, 60],  value: 50 },
            Item { id: 7, name: "B_008".into(), weights: vec![80,  160], value: 45 },
            Item { id: 8, name: "B_009".into(), weights: vec![60,  20],  value: 30 },
            Item { id: 9, name: "B_010".into(), weights: vec![40,  30],  value: 35 },
        ],
        capacities: vec![800, 600],
        rules: vec![
            BusinessRule::Requires  { from_id: 0, to_id: 1 },
            BusinessRule::Requires  { from_id: 1, to_id: 2 },
            BusinessRule::Requires  { from_id: 2, to_id: 3 },
            BusinessRule::Conflicts { a_id: 0,    b_id: 3 },
            BusinessRule::Requires  { from_id: 4, to_id: 5 },
            BusinessRule::Requires  { from_id: 4, to_id: 6 },
        ],
        target_val: Some(300),
    }
}

fn test_c() -> ProblemInstance {
    ProblemInstance {
        name: "Test C (Multi Body Conflict, 7 items)",
        items: vec![
            Item { id: 0, name: "C_001".into(), weights: vec![200, 160], value: 90 },
            Item { id: 1, name: "C_002".into(), weights: vec![180, 170], value: 85 },
            Item { id: 2, name: "C_003".into(), weights: vec![210, 150], value: 95 },
            Item { id: 3, name: "C_004".into(), weights: vec![70,  60],  value: 20 },
            Item { id: 4, name: "C_005".into(), weights: vec![60,  50],  value: 20 },
            Item { id: 5, name: "C_006".into(), weights: vec![50,  60],  value: 20 },
            Item { id: 6, name: "C_007".into(), weights: vec![30,  20],  value: 15 },
        ],
        capacities: vec![500, 500],
        rules: vec![
            BusinessRule::Requires  { from_id: 0, to_id: 3 },
            BusinessRule::Requires  { from_id: 1, to_id: 4 },
            BusinessRule::Requires  { from_id: 2, to_id: 5 },
            BusinessRule::Conflicts { a_id: 3,    b_id: 4 },
            BusinessRule::Conflicts { a_id: 4,    b_id: 5 },
        ],
        target_val: Some(200),
    }
}

fn test_d() -> ProblemInstance {
    ProblemInstance {
        name: "Test D (12 items, 5000x1000)",
        items: vec![
            Item { id: 0,  name: "D_001".into(), weights: vec![1200, 120], value: 150 },
            Item { id: 1,  name: "D_002".into(), weights: vec![1200, 0],   value: 110 },
            Item { id: 2,  name: "D_003".into(), weights: vec![200,  0],   value: 10 },
            Item { id: 3,  name: "D_004".into(), weights: vec![600,  60],  value: 85 },
            Item { id: 4,  name: "D_005".into(), weights: vec![800,  80],  value: 100 },
            Item { id: 5,  name: "D_006".into(), weights: vec![800,  0],   value: 75 },
            Item { id: 6,  name: "D_007".into(), weights: vec![1500, 150], value: 180 },
            Item { id: 7,  name: "D_008".into(), weights: vec![900,  180], value: 200 },
            Item { id: 8,  name: "D_009".into(), weights: vec![300,  60],  value: 80 },
            Item { id: 9,  name: "D_010".into(), weights: vec![1000, 200], value: 240 },
            Item { id: 10, name: "D_011".into(), weights: vec![700,  70],  value: 70 },
            Item { id: 11, name: "D_012".into(), weights: vec![400,  0],   value: 20 },
        ],
        capacities: vec![5000, 1000],
        rules: vec![
            BusinessRule::Conflicts { a_id: 0, b_id: 1 },
            BusinessRule::Requires  { from_id: 0, to_id: 2 },
            BusinessRule::Conflicts { a_id: 4, b_id: 5 },
            BusinessRule::Requires  { from_id: 8, to_id: 7 },
            BusinessRule::Conflicts { a_id: 4, b_id: 6 },
            BusinessRule::Requires  { from_id: 9, to_id: 8 },
        ],
        target_val: Some(650),
    }
}

//-----------------------------------------------------------------------

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
fn test_e_performance() {

    let scenarios = [test_a(), test_d()];

    for sc in &scenarios {
        println!("\n{}", sc.name);

        let grid_cells = (sc.capacities[0] as u64 + 1) * (sc.capacities[1] as u64 + 1);
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
            println!("  [DP Solver]   : {:>7.2} ms (avg of {}) | Score: {:>4} pt | Memory: {:.2} MB",
                     avg_ms, BENCH_ITERATIONS, dp_score, mem_total_mb);
        } else {
            println!("  [DP Solver]   : [Skipped] Grid too large ({} cells, prevents OOM)", grid_cells);
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

        println!("  [MIP Solver]  : {:>7.2} ms (avg of {}) | Score: {:>4} pt | Nodes: {:>5} ({:.1} Mnodes/sec)",
                 avg_ms, BENCH_ITERATIONS, mip_score, total_nodes, throughput);

        let sat_config = CompilerConfig::default();
        let target = sc.target_val.unwrap_or(mip_score);

        for _ in 0..WARMUP_ITERATIONS {
            let _ = SatCompiler::compile(&sc.items, &sc.capacities, &sc.rules, target, &sat_config);
        }

        let mut total_ms = 0.0;
        let mut sat_vars = 0;
        let mut sat_clauses = 0;

        for _ in 0..BENCH_ITERATIONS {
            let timer = BenchTimer::start();
            let (_, vars, clauses, _) = SatCompiler::compile(&sc.items, &sc.capacities, &sc.rules, target, &sat_config);
            total_ms += timer.elapsed_ms();
            sat_vars = vars;
            sat_clauses = clauses;
        }
        let avg_ms = total_ms / (BENCH_ITERATIONS as f64);

        println!("  [SAT Compile] : {:>7.2} ms (avg of {}) | Vars: {:>5} | Clauses: {:>5}",
                 avg_ms, BENCH_ITERATIONS, sat_vars, sat_clauses);
    }
}

#[test]
fn test_f_stress_memory() {

    let dummy_items: Vec<Item> = (0..15).map(|i| Item {
        id: i,
        name: format!("F_{:03}", i),
        weights: vec![100, 100],
        value: 50,
    }).collect();

    let mut dp_large = Flat2dDpSolver::new(2500, 2500);
    dp_large.solve(&dummy_items);
    let (lt_bytes, lh_bytes, ltotal_bytes) = dp_large.allocated_bytes();

    println!("[Large Stress Instance (15 items, Grid: 2501x2501)]");
    println!("  - DP Table Buffer : {:>10} bytes ({:.2} MB)", lt_bytes, lt_bytes as f64 / 1_048_576.0);
    println!("  - Bitset History  : {:>10} bytes ({:.2} MB)", lh_bytes, lh_bytes as f64 / 1_048_576.0);
    println!("  - Total Allocated : {:>10} bytes ({:.2} MB)", ltotal_bytes, ltotal_bytes as f64 / 1_048_576.0);
}

/*
Test A (14 items, 800x600)
test test_c_multi_body_conflict ... ok
test test_d_5000_1000 ... ok
test test_b_chain_conflict_infeasibility ... ok
test test_a_happy_path ... ok
[Large Stress Instance (15 items, Grid: 2501x2501)]
  - DP Table Buffer :   25020004 bytes (23.86 MB)
  - Bitset History  :   11728128 bytes (11.18 MB)
  - Total Allocated :   36748196 bytes (35.05 MB)
test test_f_stress_memory ... ok
  [DP Solver]   :   38.01 ms (avg of 50) | Score:  555 pt | Memory: 2.64 MB
  [MIP Solver]  :    0.19 ms (avg of 50) | Score:  550 pt | Nodes:  2092 (11.3 Mnodes/sec)
  [SAT Compile] :    2.94 ms (avg of 50) | Vars:  2135 | Clauses:  7168

Test D (12 items, 5000x1000)
  [DP Solver]   : [Skipped] Grid too large (5006001 cells, prevents OOM)
  [MIP Solver]  :    0.03 ms (avg of 50) | Score:  865 pt | Nodes:   338 (10.9 Mnodes/sec)
  [SAT Compile] :    2.07 ms (avg of 50) | Vars:  1636 | Clauses:  5472
test test_e_performance ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.30s
*/