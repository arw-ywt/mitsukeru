// src/validation.rs

use alloc::format;

use crate::model::{BusinessRule, Item, OptimizeError, OptimizerConfig};

pub const MAX_SUPPORTED_ITEMS: usize = 2048;
pub const MAX_CAPACITY_DIMS: usize = 8;
pub const MAX_SUPPORTED_RULES: usize = 32_768;
pub const MAX_TOP_K: usize = 256;
pub const DEFAULT_MAX_DP_MEMORY_BYTES: usize = 64 * 1024 * 1024;
pub const DEFAULT_MAX_SAT_VARS: usize = 250_000;
pub const DEFAULT_MAX_SAT_CLAUSES: usize = 1_000_000;

/// Validates core problem slices against architectural and safety ceilings.
///
/// # Errors
///
/// Returns [`OptimizeError::InvalidInput`] if:
/// - `items.len()` exceeds [`MAX_SUPPORTED_ITEMS`],
/// - `capacities.len()` exceeds [`MAX_CAPACITY_DIMS`],
/// - `rules.len()` exceeds [`MAX_SUPPORTED_RULES`], or
/// - the sum of all item values overflows `u32::MAX`.
pub fn validate_problem_input(
    items: &[Item],
    capacities: &[u32],
    rules: &[BusinessRule],
) -> Result<(), OptimizeError> {
    let item_count = items.len();
    if item_count > MAX_SUPPORTED_ITEMS {
        return Err(OptimizeError::InvalidInput {
            message: format!(
                "Item count ({item_count}) exceeds the maximum supported limit of {MAX_SUPPORTED_ITEMS} items. Please pre-filter candidates."
            ),
        });
    }

    let dim_count = capacities.len();
    if dim_count > MAX_CAPACITY_DIMS {
        return Err(OptimizeError::InvalidInput {
            message: format!(
                "Capacity dimension count ({dim_count}) exceeds the maximum supported limit of {MAX_CAPACITY_DIMS} dimensions."
            ),
        });
    }

    let rule_count = rules.len();
    if rule_count > MAX_SUPPORTED_RULES {
        return Err(OptimizeError::InvalidInput {
            message: format!(
                "Rule count ({rule_count}) exceeds the maximum supported limit of {MAX_SUPPORTED_RULES} rules."
            ),
        });
    }

    let mut total_value: u32 = 0;
    for item in items {
        total_value =
            total_value
                .checked_add(item.value)
                .ok_or_else(|| OptimizeError::InvalidInput {
                    message: format!(
                        "Sum of item values exceeds the maximum supported u32 limit ({}).",
                        u32::MAX
                    ),
                })?;
    }

    Ok(())
}

/// Validates optimizer configuration bounds.
///
/// # Errors
///
/// Returns [`OptimizeError::InvalidInput`] if `top_k` exceeds [`MAX_TOP_K`]
/// or if `time_limit_ms` is negative or non-finite (`NaN` / `Inf`).
pub fn validate_optimizer_config(config: &OptimizerConfig) -> Result<(), OptimizeError> {
    if let Some(k) = config.top_k
        && k > MAX_TOP_K
    {
        return Err(OptimizeError::InvalidInput {
            message: format!("top_k ({k}) exceeds the maximum supported limit of {MAX_TOP_K}."),
        });
    }

    if let Some(limit_ms) = config.time_limit_ms
        && (!limit_ms.is_finite() || limit_ms < 0.0)
    {
        return Err(OptimizeError::InvalidInput {
            message: format!("time_limit_ms ({limit_ms}) must be a finite, non-negative number."),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;
    use alloc::vec::Vec;

    fn dummy_item(id: usize, value: u32) -> Item {
        Item {
            id,
            name: format!("item_{id}"),
            weights: vec![1],
            value,
        }
    }

    #[test]
    fn test_validate_problem_input_accepts_boundaries() {
        let items: Vec<Item> = (0..MAX_SUPPORTED_ITEMS).map(|i| dummy_item(i, 1)).collect();
        let capacities = vec![100; MAX_CAPACITY_DIMS];
        let rules: Vec<BusinessRule> = (0..MAX_SUPPORTED_RULES)
            .map(|i| BusinessRule::Conflicts {
                a_id: i % MAX_SUPPORTED_ITEMS,
                b_id: (i + 1) % MAX_SUPPORTED_ITEMS,
            })
            .collect();

        assert!(validate_problem_input(&items, &capacities, &rules).is_ok());
    }

    #[test]
    fn test_validate_problem_input_rejects_excess_items() {
        let items: Vec<Item> = (0..=MAX_SUPPORTED_ITEMS)
            .map(|i| dummy_item(i, 1))
            .collect();
        let res = validate_problem_input(&items, &[10], &[]);
        assert!(matches!(res, Err(OptimizeError::InvalidInput { .. })));
    }

    #[test]
    fn test_validate_problem_input_rejects_excess_dims() {
        let items = vec![dummy_item(0, 10)];
        let capacities = vec![10; MAX_CAPACITY_DIMS + 1];
        let res = validate_problem_input(&items, &capacities, &[]);
        assert!(matches!(res, Err(OptimizeError::InvalidInput { .. })));
    }

    #[test]
    fn test_validate_problem_input_rejects_excess_rules() {
        let items = vec![dummy_item(0, 10), dummy_item(1, 10)];
        let rules = vec![BusinessRule::Conflicts { a_id: 0, b_id: 1 }; MAX_SUPPORTED_RULES + 1];
        let res = validate_problem_input(&items, &[10], &rules);
        assert!(matches!(res, Err(OptimizeError::InvalidInput { .. })));
    }

    #[test]
    fn test_validate_problem_input_rejects_value_sum_overflow() {
        let items = vec![dummy_item(0, u32::MAX), dummy_item(1, 1)];
        let res = validate_problem_input(&items, &[10], &[]);
        assert!(matches!(res, Err(OptimizeError::InvalidInput { .. })));
    }

    #[test]
    fn test_validate_optimizer_config_limits() {
        let mut cfg = OptimizerConfig {
            top_k: Some(MAX_TOP_K),
            ..OptimizerConfig::default()
        };
        assert!(validate_optimizer_config(&cfg).is_ok());

        cfg.top_k = Some(MAX_TOP_K + 1);
        assert!(matches!(
            validate_optimizer_config(&cfg),
            Err(OptimizeError::InvalidInput { .. })
        ));

        cfg.top_k = None;
        for bad in [-1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            cfg.time_limit_ms = Some(bad);
            assert!(matches!(
                validate_optimizer_config(&cfg),
                Err(OptimizeError::InvalidInput { .. })
            ));
        }
    }
}
