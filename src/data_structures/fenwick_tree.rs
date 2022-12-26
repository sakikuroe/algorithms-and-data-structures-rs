pub struct FenwickTree {
    size: usize,
    data: Vec<usize>,
}

impl FenwickTree {
    pub fn new(size: usize) -> FenwickTree {
        FenwickTree {
            size,
            data: vec![0; size + 1],
        }
    }

    pub fn add(&mut self, mut idx: usize, x: usize) {
        idx += 1;
        while idx <= self.size {
            self.data[idx] += x;
            idx += idx & (!idx + 1);
        }
    }

    // return sum of data[l..r];
    pub fn sum(&self, l: usize, r: usize) -> usize {
        // return sum of data[0..r];
        let accum = |mut idx: usize| {
            let mut res = 0;
            while idx > 0 {
                res += self.data[idx];
                idx -= idx & (!idx + 1);
            }
            res
        };

        accum(r) - accum(l)
    }
}
