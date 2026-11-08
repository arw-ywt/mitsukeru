// dp_branchless.rs

use crate::model::Item;

pub struct Flat2dDpSolver {
    cap1: usize,
    cap2: usize,
    table: Vec<u32>,
    decision_history: Vec<u64>,
}

impl Flat2dDpSolver {
    #[must_use] 
    pub fn new(cap1: u32, cap2: u32) -> Self {
        let c1 = cap1 as usize;
        let c2 = cap2 as usize;
        let size = (c1 + 1) * (c2 + 1);
        Self {
            cap1: c1,
            cap2: c2,
            table: vec![0; size],
            decision_history: Vec::new(),
        }
    }

    
    fn get_index(&self, w1: usize, w2: usize) -> usize {
        w1 * (self.cap2 + 1) + w2
    }

    pub fn solve(&mut self, items: &[Item]) -> (u32, Vec<usize>) {
        self.table.fill(0);
        let stride = self.cap2 + 1;
        let area = (self.cap1 + 1) * stride;
        
        let total_bits = items.len() * area;
        let num_u64s = total_bits.div_ceil(64);
        self.decision_history.resize(num_u64s, 0);
        self.decision_history.fill(0);

        for (item_idx, item) in items.iter().enumerate() {
            if item.weights.len() < 2 {
                continue;
            }
            let iw1 = item.weights[0] as usize;
            let iw2 = item.weights[1] as usize;
            let val = item.value;
            let history_offset = item_idx * area;

            for w1 in (iw1..=self.cap1).rev() {
                let row_offset = w1 * stride;
                let prev_row_offset = (w1 - iw1) * stride;
                let count = self.cap2 - iw2 + 1;

                unsafe {
                    let mut curr_ptr = self.table.as_mut_ptr().add(row_offset + self.cap2);
                    let mut prev_ptr = self.table.as_ptr().add(prev_row_offset + self.cap2 - iw2);
                    let mut bit_idx = history_offset + row_offset + self.cap2;
                    let hist_ptr = self.decision_history.as_mut_ptr();

                    for _ in 0..count {
                        let current_val = *curr_ptr;
                        let candidate_val = *prev_ptr + val;

                        if candidate_val > current_val {
                            *curr_ptr = candidate_val;
                            *hist_ptr.add(bit_idx >> 6) |= 1u64 << (bit_idx & 63);
                        }

                        curr_ptr = curr_ptr.sub(1);
                        prev_ptr = prev_ptr.sub(1);
                        bit_idx -= 1;
                    }
                }
            }
        }

        let best_value = self.table[self.get_index(self.cap1, self.cap2)];
        
        let mut selected_ids = Vec::new();
        let mut cur_w1 = self.cap1;
        let mut cur_w2 = self.cap2;

        for item_idx in (0..items.len()).rev() {
            let item = &items[item_idx];
            if item.weights.len() < 2 {
                continue;
            }
            let iw1 = item.weights[0] as usize;
            let iw2 = item.weights[1] as usize;

            if cur_w1 >= iw1 && cur_w2 >= iw2 {
                let curr_idx = self.get_index(cur_w1, cur_w2);
                let bit_idx = item_idx * area + curr_idx;

                let is_taken = unsafe {
                    (*self.decision_history.get_unchecked(bit_idx / 64) & (1u64 << (bit_idx % 64))) != 0
                };

                if is_taken {
                    selected_ids.push(item.id);
                    cur_w1 -= iw1;
                    cur_w2 -= iw2;
                }
            }
        }
        selected_ids.reverse();

        (best_value, selected_ids)
    }
    
    #[must_use] 
    pub fn allocated_bytes(&self) -> (usize, usize, usize) {
        let table_bytes = self.table.capacity() * std::mem::size_of::<u32>();
        let history_bytes = self.decision_history.capacity() * std::mem::size_of::<u64>();
        let total = table_bytes + history_bytes + std::mem::size_of::<Self>();
        (table_bytes, history_bytes, total)
    }

}

