// tests/common/mod.rs
#![allow(dead_code)]

use mitsukeru::{BusinessRule, Item};
use std::collections::BTreeSet;

#[derive(Clone)]
pub struct ProblemInstance {
    pub name: &'static str,
    pub items: Vec<Item>,
    pub capacities: Vec<u32>,
    pub rules: Vec<BusinessRule>,
    pub target_val: Option<u32>,
}

pub fn test_a() -> ProblemInstance {
    ProblemInstance {
        name: "Test A",
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

pub fn test_b() -> ProblemInstance {
    ProblemInstance {
        name: "Test B",
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

pub fn test_c() -> ProblemInstance {
    ProblemInstance {
        name: "Test C",
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

pub fn test_d() -> ProblemInstance {
    ProblemInstance {
        name: "Test D",
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

pub fn test_e() -> ProblemInstance {
    ProblemInstance {
        name: "Test E",
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

pub fn test_f(n: usize) -> ProblemInstance {
    let items: Vec<Item> = (0..n).map(|i| {
        let base_w1 = 120 + u32::try_from((i * 37) % 70).unwrap();
        let base_w2 = 100 + u32::try_from((i * 41) % 60).unwrap();
        let val = 60 + u32::try_from((i * 29) % 35).unwrap();
        Item {
            id: i,
            name: format!("F_{i:03}"),
            weights: vec![base_w1, base_w2],
            value: val,
        }
    }).collect();

    let n_u32 = u32::try_from(n).unwrap();
    let cap_w1 = n_u32 * 70;
    let cap_w2 = n_u32 * 58;
    let target_val = n_u32 * 33;

    let mut rules = Vec::new();
    for i in (0..n.saturating_sub(2)).step_by(4) {
        if i % 8 == 0 {
            rules.push(BusinessRule::Requires { from_id: i, to_id: i + 2 });
        } else {
            rules.push(BusinessRule::Conflicts { a_id: i, b_id: i + 2 });
        }
    }

    ProblemInstance {
        name: "Test F",
        items,
        capacities: vec![cap_w1, cap_w2],
        rules,
        target_val: Some(target_val),
    }
}

pub fn test_g() -> ProblemInstance {
    let mut items = Vec::new();

    for i in 0..2 {
        items.push(Item {
            id: i,
            name: format!("TRAP_{i:02}"),
            weights: vec![20, 20],
            value: 100,
        });
    }

    for i in 2..4 {
        items.push(Item {
            id: i,
            name: format!("TRAP_BURDEN_{i:02}"),
            weights: vec![450, 400],
            value: 10,
        });
    }

    for i in 4..12 {
        items.push(Item {
            id: i,
            name: format!("SYNERGY_{i:02}"),
            weights: vec![80, 70],
            value: 120,
        });
    }

    for i in 12..28 {
        let w1 = 70 + u32::try_from((i * 19) % 40).unwrap();
        let w2 = 60 + u32::try_from((i * 23) % 40).unwrap();
        let val = 60 + u32::try_from((i * 13) % 35).unwrap();
        items.push(Item {
            id: i,
            name: format!("GEN_{i:02}"),
            weights: vec![w1, w2],
            value: val,
        });
    }

    let rules = vec![
        BusinessRule::Requires { from_id: 0, to_id: 2 },
        BusinessRule::Requires { from_id: 1, to_id: 3 },
        BusinessRule::Requires { from_id: 4, to_id: 5 },
        BusinessRule::Requires { from_id: 6, to_id: 7 },
        BusinessRule::Requires { from_id: 8, to_id: 9 },
        BusinessRule::Conflicts { a_id: 0, b_id: 13 },
        BusinessRule::Conflicts { a_id: 1, b_id: 17 },
    ];

    ProblemInstance {
        name: "Test G (Honey Trap & Synergy, 28 items)",
        items,
        capacities: vec![1200, 1000],
        rules,
        target_val: Some(1100),
    }
}


pub fn test_h() -> ProblemInstance {
    let mut items = Vec::new();

    for i in 0..4 {
        items.push(Item {
            id: i,
            name: format!("TRAP_{i:02}"),
            weights: vec![30, 25],
            value: 120,
        });
    }

    for i in 4..8 {
        items.push(Item {
            id: i,
            name: format!("TRAP_BURDEN_{i:02}"),
            weights: vec![380, 320],
            value: 5,
        });
    }

    for i in 8..20 {
        items.push(Item {
            id: i,
            name: format!("SYNERGY_{i:02}"),
            weights: vec![90, 75],
            value: 110,
        });
    }

    for i in 20..40 {
        let w1 = 70 + u32::try_from((i * 17) % 50).unwrap();
        let w2 = 60 + u32::try_from((i * 23) % 40).unwrap();
        let val = 50 + u32::try_from((i * 13) % 40).unwrap();
        items.push(Item {
            id: i,
            name: format!("GEN_{i:02}"),
            weights: vec![w1, w2],
            value: val,
        });
    }

    let mut rules = Vec::new();

    for i in 0..4 {
        rules.push(BusinessRule::Requires { from_id: i, to_id: i + 4 });
    }

    rules.push(BusinessRule::Requires { from_id: 8, to_id: 9 });
    rules.push(BusinessRule::Requires { from_id: 10, to_id: 11 });
    rules.push(BusinessRule::Requires { from_id: 12, to_id: 13 });
    rules.push(BusinessRule::Conflicts { a_id: 0, b_id: 21 });
    rules.push(BusinessRule::Conflicts { a_id: 1, b_id: 25 });
    rules.push(BusinessRule::Conflicts { a_id: 2, b_id: 30 });

    ProblemInstance {
        name: "Test G (Honey Trap & Synergy, 40 items)",
        items,
        capacities: vec![1500, 1200],
        rules,
        target_val: Some(1300),
    }
}

pub fn test_i() -> ProblemInstance {
    let mut items = Vec::new();
    let n = 54;

    for i in 0..6 {
        items.push(Item {
            id: i,
            name: format!("TRAP_{i:02}"),
            weights: vec![25, 20],
            value: 130, 
        });
    }

    for i in 6..12 {
        items.push(Item {
            id: i,
            name: format!("TRAP_BURDEN_{i:02}"),
            weights: vec![300, 250],
            value: 5,
        });
    }

    for i in 12..28 {
        items.push(Item {
            id: i,
            name: format!("SYNERGY_{i:02}"),
            weights: vec![70, 60],
            value: 95,
        });
    }

    for i in 28..n {
        let w1 = 50 + u32::try_from((i * 17) % 40).unwrap();
        let w2 = 40 + u32::try_from((i * 23) % 35).unwrap();
        let val = 40 + u32::try_from((i * 13) % 30).unwrap();
        items.push(Item {
            id: i,
            name: format!("GEN_{i:02}"),
            weights: vec![w1, w2],
            value: val,
        });
    }

    let mut rules = Vec::new();

    for i in 0..6 {
        rules.push(BusinessRule::Requires { from_id: i, to_id: i + 6 });
    }
    for i in (12..24).step_by(2) {
        rules.push(BusinessRule::Requires { from_id: i, to_id: i + 1 });
    }

    for i in 0..6 {
        rules.push(BusinessRule::Conflicts { a_id: i, b_id: 28 + i * 3 });
    }

    ProblemInstance {
        name: "Test I (Approx vs Optimal Showcase, 54 items)",
        items,
        capacities: vec![1800, 1500],
        rules,
        target_val: Some(1500),
    }
}

pub fn test_j() -> ProblemInstance {
    let mut items = Vec::new();
    let n = 40;
    for i in 0..n {
        let w1 = 50 + u32::try_from((i * 31) % 100).unwrap();
        let w2 = 50 + u32::try_from((i * 37) % 100).unwrap();
        let val = 40 + u32::try_from((i * 19) % 80).unwrap();
        items.push(Item {
            id: i,
            name: format!("PURE_{i:02}"),
            weights: vec![w1, w2],
            value: val,
        });
    }

    ProblemInstance {
        name: "Test J (Pure Knapsack, No Rules)",
        items,
        capacities: vec![1500, 1500],
        rules: vec![],
        target_val: Some(800),
    }
}

pub fn test_k() -> ProblemInstance {
    let mut items = Vec::new();
    let n = 36;
    for i in 0..n {
        items.push(Item {
            id: i,
            name: format!("SYM_{i:02}"),
            weights: vec![50, 50],
            value: 100,
        });
    }

    let mut rules = Vec::new();
    for i in (0..n).step_by(4) {
        rules.push(BusinessRule::Requires { from_id: i, to_id: i + 1 });
        rules.push(BusinessRule::Conflicts { a_id: i, b_id: i + 2 });
    }

    ProblemInstance {
        name: "Test K (High Symmetry Stress)",
        items,
        capacities: vec![600, 600],
        rules,
        target_val: Some(600),
    }
}

pub fn test_l() -> ProblemInstance {
    let mut items = Vec::new();
    let n = 100;
    
    for i in 0..10 {
        items.push(Item {
            id: i,
            name: format!("MEGA_TRAP_{i:02}"),
            weights: vec![20, 20],
            value: 200,
        });
    }
    for i in 10..20 {
        items.push(Item {
            id: i,
            name: format!("MEGA_BURDEN_{i:02}"),
            weights: vec![500, 500],
            value: 1,
        });
    }
    for i in 20..n {
        let w1 = 40 + u32::try_from((i * 13) % 60).unwrap();
        let w2 = 40 + u32::try_from((i * 17) % 60).unwrap();
        let val = 30 + u32::try_from((i * 11) % 50).unwrap();
        items.push(Item {
            id: i,
            name: format!("CHAOS_{i:02}"),
            weights: vec![w1, w2],
            value: val,
        });
    }

    let mut rules = Vec::new();
    for i in 0..10 {
        rules.push(BusinessRule::Requires { from_id: i, to_id: i + 10 });
    }
    for i in (20..90).step_by(5) {
        rules.push(BusinessRule::Requires { from_id: i, to_id: i + 1 });
        rules.push(BusinessRule::Conflicts { a_id: i, b_id: i + 3 });
    }

    ProblemInstance {
        name: "Test L (100 Items Chaos Stress Test)",
        items,
        capacities: vec![2500, 2500],
        rules,
        target_val: Some(1800),
    }
}

pub fn test_m() -> ProblemInstance {
    let n = 50;
    let mut items = Vec::with_capacity(n);
    let optimal_set: [usize; 10] = [2, 8, 11, 17, 24, 26, 33, 39, 40, 48];
    let is_opt = |id: usize| optimal_set.contains(&id);
    let opt_w1 = [44, 45, 43, 46, 44, 45, 43, 45, 44, 45];
    let opt_w2 = [45, 43, 46, 44, 44, 45, 44, 43, 46, 44];

    let mut opt_idx = 0;
    for i in 0..n {
        let (w1, w2, val) = if is_opt(i) {
            let res = (opt_w1[opt_idx], opt_w2[opt_idx], 50);
            opt_idx += 1;
            res
        } else {
            let w1 = 45 + u32::try_from((i * 7) % 20).unwrap();
            let w2 = 45 + u32::try_from((i * 11) % 20).unwrap();
            let val = 25 + u32::try_from((i * 3) % 15).unwrap();
            (w1, w2, val)
        };

        items.push(Item {
            id: i,
            name: format!("MESH_{i:02}"),
            weights: vec![w1, w2],
            value: val,
        });
    }

    let mut conflict_pairs = BTreeSet::new();

    for b in 0..10 {
        let start = b * 5;
        for i in start..(start + 5) {
            for j in (i + 1)..(start + 5) {
                conflict_pairs.insert((i, j));
            }
        }
    }

    let mesh_offsets = [1, 2, 3, 6, 7, 11, 13, 17];
    for i in 0..n {
        for &offset in &mesh_offsets {
            let j = (i + offset) % n;
            let (min_id, max_id) = if i < j { (i, j) } else { (j, i) };
            if min_id != max_id 
                && !(is_opt(min_id) && is_opt(max_id)) {
                    conflict_pairs.insert((min_id, max_id));
                }
        }
    }

    let rules = conflict_pairs
        .into_iter()
        .map(|(a_id, b_id)| BusinessRule::Conflicts { a_id, b_id })
        .collect();

    ProblemInstance {
        name: "Test M (Web of Conflicts, 50 items)",
        items,
        capacities: vec![450, 450],
        rules,
        target_val: Some(500),
    }
}