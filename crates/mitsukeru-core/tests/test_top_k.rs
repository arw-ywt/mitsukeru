// tests/test_top_k.rs
// cargo test --test test_top_k -- --nocapture

mod common;
use common::test_a;
use mitsukeru::{Optimizer, ProblemInput};

#[test]
fn test_top_3_candidates_generation() {
    let prob = test_a();

    let input = ProblemInput {
        items: prob.items.clone(),
        capacities: prob.capacities.clone(),
        rules: prob.rules.clone(),
        target_value: prob.target_val,
    };

    let opt = Optimizer::new(&input).with_top_k(3);
    let solution = opt.solve().expect("Should solve successfully");

    println!("\n=== Top-3 Candidates Test Result ===");
    println!("Best Optimal Value: {} pt", solution.best_value);
    println!(
        "Optimality Gap    : {:.2}%",
        solution.optimality_gap * 100.0
    );
    println!("Upper Bound       : {:.1} pt", solution.upper_bound);

    assert_eq!(solution.best_value, 550);
    assert!(
        solution.optimality_gap.abs() <= f64::EPSILON,
        "Optimal solution must have 0% gap"
    );

    let candidates = solution
        .top_candidates
        .expect("Top candidates must be present");
    assert!(
        candidates.len() >= 2,
        "Should return at least top alternative candidates, found {}",
        candidates.len()
    );

    println!("\n[Alternative Options]");
    for (rank, cand) in candidates.iter().enumerate() {
        println!(
            " Rank {}: {} pt | Items: {:?}",
            rank + 1,
            cand.value,
            cand.selected_item_ids
        );

        let mut w0 = 0;
        let mut w1 = 0;
        for &id in &cand.selected_item_ids {
            w0 += prob.items[id].weights[0];
            w1 += prob.items[id].weights[1];
        }
        assert!(
            w0 <= prob.capacities[0],
            "Candidate must respect capacity 0"
        );
        assert!(
            w1 <= prob.capacities[1],
            "Candidate must respect capacity 1"
        );
    }

    for i in 0..candidates.len() - 1 {
        assert!(
            candidates[i].value >= candidates[i + 1].value,
            "Candidates must be sorted in descending order of value"
        );
    }

    assert_eq!(
        candidates[0].value, solution.best_value,
        "Rank 1 candidate must match the best value"
    );
}
