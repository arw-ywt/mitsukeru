// tests/test_quickstart.rs
// cargo test --test test_quickstart -- --nocapture

use mitsukeru::{BusinessRule, Item, Optimizer, ProblemInput, SolverStrategy};

#[test]
fn test_readme_quickstart_example() {
    let input = ProblemInput::new(
        vec![
            Item {
                id: 0,
                name: "Pro Camera".into(),
                weights: vec![10, 5],
                value: 500,
            },
            Item {
                id: 1,
                name: "Carbon Tripod".into(),
                weights: vec![3, 2],
                value: 120,
            },
            Item {
                id: 2,
                name: "LED Light A".into(),
                weights: vec![5, 3],
                value: 200,
            },
            Item {
                id: 3,
                name: "LED Light B".into(),
                weights: vec![5, 3],
                value: 210,
            },
        ],
        vec![20, 10],
        vec![
            BusinessRule::Requires {
                from_id: 0,
                to_id: 1,
            },
            BusinessRule::Conflicts { a_id: 2, b_id: 3 },
        ],
        None,
    );

    let solution = Optimizer::new(&input)
        .with_strategy(SolverStrategy::Mixed)
        .solve()
        .expect("Quickstart execution failed");

    println!("\n=== QuickStart Test Output ===");
    println!("Best Value: {} pt", solution.best_value);
    println!("Selected Items: {:?}", solution.selected_item_ids);
    println!("Optimality Gap: {:.2}%", solution.optimality_gap * 100.0);

    assert!(solution.best_value > 0);

    let has_conflict =
        solution.selected_item_ids.contains(&2) && solution.selected_item_ids.contains(&3);
    assert!(!has_conflict, "Conflict rule violated!");

    if solution.selected_item_ids.contains(&0) {
        assert!(
            solution.selected_item_ids.contains(&1),
            "Required rule violated!"
        );
    }
}
