pub struct FenwickTree {
    size: usize,
    data: Vec<usize>,
}

/// *CAUTION* 1-based numbering
impl FenwickTree {
    pub fn new(size: usize) -> FenwickTree {
        FenwickTree {
            size,
            data: vec![0; size + 1],
        }
    }

    pub fn add(&mut self, mut idx: usize, x: usize) {
        while idx <= self.size {
            self.data[idx] += x;
            idx += idx & (!idx + 1);
        }
    }

    // return data[1] + ... + data[idx];
    pub fn sum(&self, mut idx: usize) -> usize {
        let mut res = 0;
        while idx > 0 {
            res += self.data[idx];
            idx -= idx & (!idx + 1);
        }

        res
    }
}
