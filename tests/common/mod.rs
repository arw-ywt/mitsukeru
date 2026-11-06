// tests/common/mod.rs
use mitsukeru::{BusinessRule, Item};

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

    let cap_w1 = (n as u32) * 70;
    let cap_w2 = (n as u32) * 58;
    let target_val = (n as u32) * 33;

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