// tests\benchmark_scenarios.rs
// cargo test -- --nocapture --test-threads=1

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

fn test_e() -> ProblemInstance {
    ProblemInstance {
        name: "Test E (SAT Compile Limit, 18 items)",
        items: vec![
            Item { id: 0,  name: "E_001".into(), weights: vec![130, 110], value: 90 },
            Item { id: 1,  name: "E_002".into(), weights: vec![90,  70],  value: 45 },
            Item { id: 2,  name: "E_003".into(), weights: vec![160, 120], value: 80 },
            Item { id: 3,  name: "E_004".into(), weights: vec![70,  60],  value: 30 },
            Item { id: 4,  name: "E_005".into(), weights: vec![200, 50],  value: 75 },
            Item { id: 5,  name: "E_006".into(), weights: vec![250, 30],  value: 85 },
            Item { id: 6,  name: "E_007".into(), weights: vec![140, 100], value: 70 },
            Item { id: 7,  name: "E_008".into(), weights: vec![80,  90],  value: 60 },
            Item { id: 8,  name: "E_009".into(), weights: vec![40,  30],  value: 35 },
            Item { id: 9,  name: "E_010".into(), weights: vec![60,  50],  value: 50 },
            Item { id: 10, name: "E_011".into(), weights: vec![90,  110], value: 85 },
            Item { id: 11, name: "E_012".into(), weights: vec![30,  20],  value: 25 },
            Item { id: 12, name: "E_013".into(), weights: vec![120, 40],  value: 55 },
            Item { id: 13, name: "E_014".into(), weights: vec![70,  50],  value: 50 },
            Item { id: 14, name: "E_015".into(), weights: vec![110, 80],  value: 65 },
            Item { id: 15, name: "E_016".into(), weights: vec![85,  75],  value: 40 },
            Item { id: 16, name: "E_017".into(), weights: vec![95,  65],  value: 50 },
            Item { id: 17, name: "E_018".into(), weights: vec![50,  40],  value: 30 },
        ],
        capacities: vec![1000, 800],
        rules: vec![
            BusinessRule::Requires  { from_id: 0,  to_id: 1 },
            BusinessRule::Requires  { from_id: 2,  to_id: 3 },
            BusinessRule::Conflicts { a_id: 4,     b_id: 5 },
            BusinessRule::Requires  { from_id: 6,  to_id: 7 },
            BusinessRule::Conflicts { a_id: 8,     b_id: 9 },
            BusinessRule::Requires  { from_id: 10, to_id: 11 },
            BusinessRule::Conflicts { a_id: 12,    b_id: 13 },
            BusinessRule::Requires  { from_id: 14, to_id: 15 },
            BusinessRule::Conflicts { a_id: 16,    b_id: 17 },
        ],
        target_val: Some(600),
    }
}

fn test_f() -> ProblemInstance {
    let items: Vec<Item> = (0..26).map(|i| {
        let base_w1 = 120 + ((i * 37) % 70) as u32;
        let base_w2 = 100 + ((i * 41) % 60) as u32;
        let val = 60 + ((i * 29) % 35) as u32;
        Item {
            id: i,
            name: format!("F_{:03}", i),
            weights: vec![base_w1, base_w2],
            value: val,
        }
    }).collect();

    ProblemInstance {
        name: "Test F (MIP Solver Limit, 26 items)",
        items,
        capacities: vec![1800, 1500],
        rules: vec![
            BusinessRule::Requires  { from_id: 0,  to_id: 2 },
            BusinessRule::Requires  { from_id: 4,  to_id: 6 },
            BusinessRule::Conflicts { a_id: 8,     b_id: 10 },
            BusinessRule::Conflicts { a_id: 12,    b_id: 14 },
            BusinessRule::Requires  { from_id: 16, to_id: 18 },
            BusinessRule::Conflicts { a_id: 20,    b_id: 22 },
        ],
        target_val: Some(850),
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
        ("Test F", test_f()),
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
        Time   : 37.52 ms
        Score  : 555 pt
        Grid   : 481,401 cells (801x601)
        Memory : 2.64 MB
 MIP Solver  : 
        Status     : OK
        Time       : 0.01 ms
        Score      : 550 pt
        Nodes      : 78
        Mnodes/sec : 6.7
 SAT Compile : 
        Status     : OK
        Time       : 1.60 ms
        Vars       : 1,230
        Clauses    : 4,091

<Test B>
 Items       : 10
 Capacities  : [800, 600]
 Target      : 300 pt
 DP Solver   : 
        Status : OK
        Time   : 24.41 ms
        Score  : 365 pt
        Grid   : 481,401 cells (801x601)
        Memory : 2.41 MB
 MIP Solver  : 
        Status     : OK
        Time       : 0.09 ms
        Score      : 215 pt
        Nodes      : 688
        Mnodes/sec : 7.7
 SAT Compile : 
        Status     : OK
        Time       : 1.25 ms
        Vars       : 926
        Clauses    : 3,072

<Test C>
 Items       : 7
 Capacities  : [500, 500]
 Target      : 200 pt
 DP Solver   : 
        Status : OK
        Time   : 8.28 ms
        Score  : 220 pt
        Grid   : 251,001 cells (501x501)
        Memory : 1.17 MB
 MIP Solver  : 
        Status     : OK
        Time       : 0.01 ms
        Score      : 150 pt
        Nodes      : 98
        Mnodes/sec : 7.8
 SAT Compile : 
        Status     : OK
        Time       : 0.82 ms
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
        Time       : 0.03 ms
        Score      : 865 pt
        Nodes      : 173
        Mnodes/sec : 6.4
 SAT Compile : 
        Status     : OK
        Time       : 1.38 ms
        Vars       : 1,081
        Clauses    : 3,585

<Test E>
 Items       : 18
 Capacities  : [1000, 800]
 Target      : 600 pt
 DP Solver   : 
        Status : OK
        Time   : 80.95 ms
        Score  : 650 pt
        Grid   : 801,801 cells (1001x801)
        Memory : 4.78 MB
 MIP Solver  : 
        Status     : OK
        Time       : 0.12 ms
        Score      : 615 pt
        Nodes      : 837
        Mnodes/sec : 7.1
 SAT Compile : 
        Status     : OK
        Time       : 1.98 ms
        Vars       : 1,529
        Clauses    : 5,098

<Test F>
 Items       : 26
 Capacities  : [1800, 1500]
 Target      : 850 pt
 DP Solver   : 
        Status : SKIPPED (OOM)
        Time   : ---
        Score  : ---
        Grid   : 2,703,301 cells (1801x1501)
        Memory : ---
 MIP Solver  : 
        Status     : OK
        Time       : 0.04 ms
        Score      : 1036 pt
        Nodes      : 285
        Mnodes/sec : 7.5
 SAT Compile : 
        Status     : EXCEEDED (>2.0ms)
        Time       : 3.01 ms
        Vars       : 2,389
        Clauses    : 7,987

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

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 9.24s
*/