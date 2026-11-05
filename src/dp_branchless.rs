use crate::model::Item;

pub struct Flat2dDpSolver {
    cap1: usize,
    cap2: usize,
    table: Vec<u32>,
    // どのステップで更新が起きたかを記録するフラットバッファ
    decision_history: Vec<bool>,
}

impl Flat2dDpSolver {
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

    #[inline(always)]
    fn get_index(&self, w1: usize, w2: usize) -> usize {
        w1 * (self.cap2 + 1) + w2
    }

    /// 最適解のスコアと、採択されたアイテムIDのリストを返す
    pub fn solve(&mut self, items: &[Item]) -> (u32, Vec<usize>) {
        self.table.fill(0);
        let stride = self.cap2 + 1;
        let area = (self.cap1 + 1) * stride;
        
        // 履歴バッファの確保 (アイテム数 × グリッドサイズ)
        self.decision_history.resize(items.len() * area, false);
        self.decision_history.fill(false);

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

                for w2 in (0..=self.cap2).rev() {
                    let can_fit = (w2 >= iw2) as u32;
                    let prev_w2 = if w2 >= iw2 { w2 - iw2 } else { 0 };

                    unsafe {
                        let curr_idx = row_offset + w2;
                        let prev_idx = prev_row_offset + prev_w2;

                        let current_val = *self.table.get_unchecked(curr_idx);
                        let candidate_val = *self.table.get_unchecked(prev_idx) + val;

                        let next_candidate = (candidate_val * can_fit) | (current_val * (1 - can_fit));
                        if next_candidate > current_val {
                            *self.table.get_unchecked_mut(curr_idx) = next_candidate;
                            // このアイテムを採用したことを記録
                            *self.decision_history.get_unchecked_mut(history_offset + curr_idx) = true;
                        }
                    }
                }
            }
        }

        let best_value = self.table[self.get_index(self.cap1, self.cap2)];

        // --- バックトラック（逆順探索で採択アイテムを特定） ---
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
                let history_idx = item_idx * area + curr_idx;

                if self.decision_history[history_idx] {
                    selected_ids.push(item.id);
                    cur_w1 -= iw1;
                    cur_w2 -= iw2;
                }
            }
        }
        selected_ids.reverse();

        (best_value, selected_ids)
    }
}