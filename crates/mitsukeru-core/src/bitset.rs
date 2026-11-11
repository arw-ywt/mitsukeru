// src/bitset.rs

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct BitSet<const W: usize> {
    pub words: [u64; W],
}

impl<const W: usize> Default for BitSet<W> {
    fn default() -> Self {
        Self { words: [0; W] }
    }
}

impl<const W: usize> BitSet<W> {
    pub const CAPACITY: usize = W * 64;

    #[must_use]
    pub const fn new() -> Self {
        Self { words: [0; W] }
    }

    pub fn set(&mut self, idx: usize) {
        if idx < Self::CAPACITY {
            self.words[idx / 64] |= 1u64 << (idx % 64);
        }
    }

    #[must_use]
    pub fn has(&self, idx: usize) -> bool {
        if idx < Self::CAPACITY {
            (self.words[idx / 64] & (1u64 << (idx % 64))) != 0
        } else {
            false
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        let mut acc = 0u64;
        let mut i = 0;
        while i < W {
            acc |= self.words[i];
            i += 1;
        }
        acc == 0
    }

    #[must_use]
    pub fn is_disjoint(&self, other: &Self) -> bool {
        let mut acc = 0u64;
        let mut i = 0;
        while i < W {
            acc |= self.words[i] & other.words[i];
            i += 1;
        }
        acc == 0
    }

    pub fn union_with(&mut self, other: &Self) {
        let mut i = 0;
        while i < W {
            self.words[i] |= other.words[i];
            i += 1;
        }
    }

    #[must_use]
    pub fn intersect(&self, other: &Self) -> Self {
        let mut res = Self::new();
        let mut i = 0;
        while i < W {
            res.words[i] = self.words[i] & other.words[i];
            i += 1;
        }
        res
    }

    pub fn for_each_one<F: FnMut(usize)>(&self, mut f: F) {
        let mut i = 0;
        while i < W {
            let mut w = self.words[i];
            while w != 0 {
                let bit_idx = w.trailing_zeros() as usize;
                f(i * 64 + bit_idx);
                w &= w - 1;
            }
            i += 1;
        }
    }

    #[must_use]
    pub fn count_ones(&self) -> usize {
        let mut count = 0;
        let mut i = 0;
        while i < W {
            count += self.words[i].count_ones() as usize;
            i += 1;
        }
        count
    }
}
