// src/dp_branchless.rs

use alloc::format;
use alloc::vec::Vec;
use core::mem;

use crate::model::{Item, OptimizeError};
use crate::validation::DEFAULT_MAX_DP_MEMORY_BYTES;

pub struct Flat2dDpSolver {
    cap1: usize,
    cap2: usize,
    stride: usize,
    area: usize,
    max_bytes: usize,
    table: Vec<u32>,
    decision_history: Vec<u64>,
}

impl Flat2dDpSolver {
    /// Creates a new 2D DP solver bounded by [`DEFAULT_MAX_DP_MEMORY_BYTES`].
    ///
    /// # Errors
    ///
    /// Returns [`OptimizeError::InvalidInput`] if capacity size calculations overflow `usize`
    /// or if the DP table alone exceeds [`DEFAULT_MAX_DP_MEMORY_BYTES`].
    pub fn new(cap1: u32, cap2: u32) -> Result<Self, OptimizeError> {
        Self::new_with_budget(cap1, cap2, DEFAULT_MAX_DP_MEMORY_BYTES)
    }

    /// Creates a new 2D DP solver with an explicit memory budget in bytes.
    ///
    /// # Errors
    ///
    /// Returns [`OptimizeError::InvalidInput`] if capacity size calculations overflow `usize`
    /// or if the DP table exceeds `max_bytes`.
    pub fn new_with_budget(cap1: u32, cap2: u32, max_bytes: usize) -> Result<Self, OptimizeError> {
        let c1 = usize::try_from(cap1).map_err(|_| OptimizeError::InvalidInput {
            message: format!("Capacity 1 ({cap1}) exceeds addressable usize range."),
        })?;
        let c2 = usize::try_from(cap2).map_err(|_| OptimizeError::InvalidInput {
            message: format!("Capacity 2 ({cap2}) exceeds addressable usize range."),
        })?;

        let c1_plus_1 = c1
            .checked_add(1)
            .ok_or_else(|| OptimizeError::InvalidInput {
                message: format!("Capacity 1 ({cap1}) + 1 overflows usize."),
            })?;
        let stride = c2
            .checked_add(1)
            .ok_or_else(|| OptimizeError::InvalidInput {
                message: format!("Capacity 2 ({cap2}) + 1 overflows usize."),
            })?;
        let area = c1_plus_1
            .checked_mul(stride)
            .ok_or_else(|| OptimizeError::InvalidInput {
                message: format!("2D DP grid area ({c1_plus_1} * {stride}) overflows usize."),
            })?;

        let table_bytes =
            area.checked_mul(mem::size_of::<u32>())
                .ok_or_else(|| OptimizeError::InvalidInput {
                    message: alloc::string::String::from("2D DP table byte size overflows usize."),
                })?;

        if table_bytes > max_bytes {
            return Err(OptimizeError::InvalidInput {
                message: format!(
                    "2D DP table requires {table_bytes} bytes, exceeding memory budget ({max_bytes} bytes)."
                ),
            });
        }

        Ok(Self {
            cap1: c1,
            cap2: c2,
            stride,
            area,
            max_bytes,
            table: vec![0; area],
            decision_history: Vec::new(),
        })
    }

    fn get_index(&self, w1: usize, w2: usize) -> usize {
        w1 * self.stride + w2
    }

    fn prepare_history(&mut self, item_count: usize) -> Result<(), OptimizeError> {
        let total_bits =
            item_count
                .checked_mul(self.area)
                .ok_or_else(|| OptimizeError::InvalidInput {
                    message: alloc::string::String::from(
                        "2D DP decision history bit count overflows usize.",
                    ),
                })?;
        let num_u64s = total_bits.div_ceil(64);
        let history_bytes = num_u64s.checked_mul(mem::size_of::<u64>()).ok_or_else(|| {
            OptimizeError::InvalidInput {
                message: alloc::string::String::from(
                    "2D DP decision history byte size overflows usize.",
                ),
            }
        })?;
        let table_bytes = self.table.len() * mem::size_of::<u32>();
        let total_required_bytes =
            table_bytes
                .checked_add(history_bytes)
                .ok_or_else(|| OptimizeError::InvalidInput {
                    message: alloc::string::String::from(
                        "2D DP total memory requirement overflows usize.",
                    ),
                })?;

        if total_required_bytes > self.max_bytes {
            return Err(OptimizeError::InvalidInput {
                message: format!(
                    "2D DP requires {total_required_bytes} bytes, exceeding memory budget ({} bytes).",
                    self.max_bytes
                ),
            });
        }

        self.table.fill(0);
        self.decision_history.resize(num_u64s, 0);
        self.decision_history.fill(0);
        Ok(())
    }

    fn reconstruct_selected(&self, items: &[Item]) -> Vec<usize> {
        let mut selected_ids = Vec::new();
        let mut cur_w1 = self.cap1;
        let mut cur_w2 = self.cap2;

        for item_idx in (0..items.len()).rev() {
            let item = &items[item_idx];
            if item.weights.len() < 2 {
                continue;
            }

            let Ok(iw1) = usize::try_from(item.weights[0]) else {
                continue;
            };
            let Ok(iw2) = usize::try_from(item.weights[1]) else {
                continue;
            };

            if cur_w1 >= iw1 && cur_w2 >= iw2 {
                let curr_idx = self.get_index(cur_w1, cur_w2);
                let bit_idx = item_idx * self.area + curr_idx;
                let is_taken =
                    (self.decision_history[bit_idx / 64] & (1u64 << (bit_idx % 64))) != 0;

                if is_taken {
                    selected_ids.push(item.id);
                    cur_w1 -= iw1;
                    cur_w2 -= iw2;
                }
            }
        }
        selected_ids.reverse();
        selected_ids
    }

    /// Solves the 2D knapsack problem exactly without business rules.
    ///
    /// # Errors
    ///
    /// Returns [`OptimizeError::InvalidInput`] if history allocation overflows `usize`,
    /// exceeds the configured memory budget, or if DP values overflow `u32::MAX`.
    pub fn solve(&mut self, items: &[Item]) -> Result<(u32, Vec<usize>), OptimizeError> {
        self.prepare_history(items.len())?;

        for (item_idx, item) in items.iter().enumerate() {
            if item.weights.len() < 2 {
                continue;
            }
            let Ok(iw1) = usize::try_from(item.weights[0]) else {
                continue;
            };
            let Ok(iw2) = usize::try_from(item.weights[1]) else {
                continue;
            };
            if iw1 > self.cap1 || iw2 > self.cap2 {
                continue;
            }

            let val = item.value;
            let history_offset = item_idx * self.area;
            let prev_len = self.stride - iw2;

            for w1 in (iw1..=self.cap1).rev() {
                let row_offset = w1 * self.stride;

                if iw1 > 0 {
                    let prev_row_offset = (w1 - iw1) * self.stride;
                    let (head, tail) = self.table.split_at_mut(row_offset);
                    let prev_slice = &head[prev_row_offset..prev_row_offset + prev_len];
                    let curr_slice = &mut tail[iw2..self.stride];

                    for (k, (curr_val, &prev_val)) in
                        curr_slice.iter_mut().zip(prev_slice.iter()).enumerate()
                    {
                        let candidate_val = prev_val.checked_add(val).ok_or_else(|| {
                            OptimizeError::InvalidInput {
                                message: format!(
                                    "2D DP accumulated value exceeded u32::MAX ({}).",
                                    u32::MAX
                                ),
                            }
                        })?;

                        if candidate_val > *curr_val {
                            *curr_val = candidate_val;
                            let bit_idx = history_offset + row_offset + iw2 + k;
                            self.decision_history[bit_idx >> 6] |= 1u64 << (bit_idx & 63);
                        }
                    }
                } else {
                    let row = &mut self.table[row_offset..row_offset + self.stride];
                    for w2 in (iw2..=self.cap2).rev() {
                        let prev_val = row[w2 - iw2];
                        let candidate_val = prev_val.checked_add(val).ok_or_else(|| {
                            OptimizeError::InvalidInput {
                                message: format!(
                                    "2D DP accumulated value exceeded u32::MAX ({}).",
                                    u32::MAX
                                ),
                            }
                        })?;

                        if candidate_val > row[w2] {
                            row[w2] = candidate_val;
                            let bit_idx = history_offset + row_offset + w2;
                            self.decision_history[bit_idx >> 6] |= 1u64 << (bit_idx & 63);
                        }
                    }
                }
            }
        }

        let best_value = self.table[self.get_index(self.cap1, self.cap2)];
        let selected_ids = self.reconstruct_selected(items);

        Ok((best_value, selected_ids))
    }

    #[must_use]
    pub fn allocated_bytes(&self) -> (usize, usize, usize) {
        let table_bytes = self.table.capacity() * mem::size_of::<u32>();
        let history_bytes = self.decision_history.capacity() * mem::size_of::<u64>();
        let total = table_bytes + history_bytes + mem::size_of::<Self>();
        (table_bytes, history_bytes, total)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn test_dp_rejects_capacity_overflow_and_budget() {
        let err = Flat2dDpSolver::new(u32::MAX, u32::MAX);
        assert!(matches!(err, Err(OptimizeError::InvalidInput { .. })));

        let err_65535 = Flat2dDpSolver::new(65_535, 65_535);
        assert!(matches!(err_65535, Err(OptimizeError::InvalidInput { .. })));

        let err_budget = Flat2dDpSolver::new_with_budget(100, 100, 1024);
        assert!(matches!(
            err_budget,
            Err(OptimizeError::InvalidInput { .. })
        ));
    }

    #[test]
    fn test_dp_rejects_history_budget_overflow() {
        let mut solver =
            Flat2dDpSolver::new_with_budget(100, 100, 100 * 1024).expect("valid budget");
        let items: Vec<Item> = (0..200)
            .map(|id| Item {
                id,
                name: format!("item_{id}"),
                weights: vec![1, 1],
                value: 10,
            })
            .collect();

        let res = solver.solve(&items);
        assert!(matches!(res, Err(OptimizeError::InvalidInput { .. })));
    }

    #[test]
    fn test_dp_zero_weight_and_asymmetric_overweight_safe() {
        let mut solver = Flat2dDpSolver::new(5, 5).expect("valid solver");
        let items = vec![
            Item {
                id: 0,
                name: "free".into(),
                weights: vec![0, 0],
                value: 50,
            },
            Item {
                id: 1,
                name: "over_w2".into(),
                weights: vec![3, 10],
                value: 999,
            },
            Item {
                id: 2,
                name: "fit".into(),
                weights: vec![5, 5],
                value: 100,
            },
        ];

        let (best, selected) = solver.solve(&items).expect("solve succeeds");
        assert_eq!(best, 150);
        assert_eq!(selected, vec![0, 2]);
    }

    #[test]
    fn test_dp_rejects_u32_value_overflow() {
        let mut solver = Flat2dDpSolver::new(10, 10).expect("valid solver");
        let items = vec![
            Item {
                id: 0,
                name: "a".into(),
                weights: vec![1, 1],
                value: u32::MAX,
            },
            Item {
                id: 1,
                name: "b".into(),
                weights: vec![1, 1],
                value: 1,
            },
        ];

        let res = solver.solve(&items);
        assert!(matches!(res, Err(OptimizeError::InvalidInput { .. })));
    }
}
