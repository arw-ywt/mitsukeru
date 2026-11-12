use mitsukeru::dp_branchless::Flat2dDpSolver;
use mitsukeru::model::Item;

fn make_item(id: usize, name: &str, weights: Vec<u32>, value: u32) -> Item {
    Item {
        id,
        name: name.into(),
        weights,
        value,
    }
}

#[test]
fn test_dp_asymmetric_overweight_item_does_not_panic() {
    let mut solver = Flat2dDpSolver::new(10, 5);

    let items = vec![
        make_item(0, "asymmetric_overweight", vec![2, 10], 100),
        make_item(1, "valid_item", vec![3, 2], 50),
    ];

    let (best_value, selected_ids) = solver.solve(&items);

    assert_eq!(best_value, 50);
    assert_eq!(selected_ids, vec![1]);
}

#[test]
fn test_dp_boundary_extreme_weights() {
    let mut solver = Flat2dDpSolver::new(5, 5);

    let items = vec![
        make_item(0, "both_over", vec![100, 100], 999),
        make_item(1, "dim0_over", vec![10, 1], 500),
        make_item(2, "dim1_over", vec![1, 10], 500),
        make_item(3, "exact_fit", vec![5, 5], 10),
    ];

    let (best_value, selected_ids) = solver.solve(&items);

    assert_eq!(best_value, 10);
    assert_eq!(selected_ids, vec![3]);
}

#[test]
fn test_dp_insufficient_weights() {
    let mut solver = Flat2dDpSolver::new(10, 10);

    let items = vec![
        make_item(0, "one_weight", vec![5], 100),
        make_item(1, "zero_weight", vec![], 100),
        make_item(2, "valid_weights", vec![2, 2], 20),
    ];

    let (best_value, selected_ids) = solver.solve(&items);

    assert_eq!(best_value, 20);
    assert_eq!(selected_ids, vec![2]);
}
