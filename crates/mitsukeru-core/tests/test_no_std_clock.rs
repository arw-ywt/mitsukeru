use core::sync::atomic::{AtomicU64, Ordering};
use mitsukeru::Optimizer;
use mitsukeru::clock::Clock;
use mitsukeru::model::{Item, ProblemInput, SolutionStatus};

struct MockHardwareClock {
    micros: AtomicU64,
}

impl MockHardwareClock {
    // Helper to advance simulated hardware time e.g. timeout testing
    // DC with a reason
    #[allow(dead_code)]
    fn new(start_micros: u64) -> Self {
        Self {
            micros: AtomicU64::new(start_micros),
        }
    }

    #[allow(dead_code)]
    fn advance_ms(&self, ms: u64) {
        self.micros.fetch_add(ms * 1000, Ordering::Relaxed);
    }
}

impl Clock for MockHardwareClock {
    fn now_micros(&self) -> u64 {
        self.micros.load(Ordering::Relaxed)
    }
}

fn sample_problem() -> ProblemInput {
    ProblemInput {
        items: vec![
            Item {
                id: 0,
                name: "ItemA".into(),
                weights: vec![10],
                value: 100,
            },
            Item {
                id: 1,
                name: "ItemB".into(),
                weights: vec![15],
                value: 200,
            },
        ],
        capacities: vec![20],
        rules: vec![],
        target_value: None,
    }
}

#[test]
#[cfg(not(feature = "std"))]
fn test_no_std_with_external_clock_success() {
    let input = sample_problem();
    let hw_clock = MockHardwareClock::new(1_000_000);

    let solution = Optimizer::new(&input)
        .with_clock(&hw_clock)
        .with_time_limit_ms(50.0)
        .solve()
        .expect(
            "Should solve successfully when an external clock is provided in zero-dependency mode",
        );

    assert_eq!(solution.best_value, 200);
    assert_eq!(solution.selected_item_ids, vec![1]);
    assert_eq!(solution.status, SolutionStatus::Optimal);
}

#[test]
#[cfg(not(feature = "std"))]
fn test_no_std_without_clock_returns_invalid_input() {
    let input = sample_problem();

    let result = Optimizer::new(&input).with_time_limit_ms(10.0).solve();

    assert!(
        matches!(
            result,
            Err(mitsukeru::model::OptimizeError::InvalidInput { .. })
        ),
        "Should return InvalidInput error when time limit is set but no clock is available"
    );
}

#[test]
fn test_no_std_node_limit_only_without_clock() {
    let input = sample_problem();

    let solution = Optimizer::new(&input)
        .preset_exact()
        .with_max_nodes(1_000)
        .solve()
        .expect("Should solve successfully without a clock when time limits are disabled");

    assert_eq!(solution.best_value, 200);
    assert_eq!(solution.selected_item_ids, vec![1]);
    assert_eq!(solution.status, SolutionStatus::Optimal);
}
