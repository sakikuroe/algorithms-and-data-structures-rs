use super::bit_vector;

#[derive(Clone)]
pub struct WaveletMatrix {
    height: usize,
    bit_table: Vec<bit_vector::BitVector>,
    sorted_v: Vec<usize>,
}

impl WaveletMatrix {
    pub fn new(v: &[usize]) -> Self {
        let mut sorted_v = v.to_vec();
        sorted_v.sort_unstable();
        sorted_v.dedup();
        let mut compress = v
            .iter()
            .map(|&x| sorted_v.partition_point(|&y| y < x))
            .collect::<Vec<_>>();
        let height = (1..).find(|i| 1 << i >= compress.len() + 1).unwrap() + 1;
        let mut bit_table: Vec<bit_vector::BitVector> = vec![];
        for i in (0..height).rev() {
            bit_table.push(bit_vector::BitVector::new(
                &compress
                    .iter()
                    .map(|&x| ((x >> i) & 1) as u8)
                    .collect::<Vec<_>>(),
            ));
            compress = compress
                .iter()
                .filter(|&x| (x >> i) & 1 == 0)
                .chain(compress.iter().filter(|&x| (x >> i) & 1 == 1))
                .cloned()
                .collect::<Vec<_>>();
        }

        WaveletMatrix {
            height,
            bit_table,
            sorted_v,
        }
    }

    /// Returns:
    ///     v[l..r].into_iter().filter(|y| y < upper).count()
    pub fn count_less_than(&self, mut l: usize, mut r: usize, upper: usize) -> usize {
        if r <= l {
            return 0;
        }

        let upper_idx = self.sorted_v.partition_point(|&x| x < upper);
        let mut res = 0;

        for (i, bit) in (0..self.height).rev().zip(self.bit_table.iter()) {
            let rank_l = bit.rank(l);
            let rank_r = bit.rank(r);

            if (upper_idx >> i) & 1 == 0 {
                l = l - rank_l;
                r = r - rank_r;
            } else {
                res += (r - l) - (rank_r - rank_l);
                let len_minus_rank_len = bit.len() - bit.rank(bit.len());
                l = rank_l + len_minus_rank_len;
                r = rank_r + len_minus_rank_len;
            }
        }

        res
    }

    /// Returns:
    ///     v[l..r].into_iter().filter(|y| lower <= y ).count()
    pub fn count_more_than(&self, l: usize, r: usize, lower: usize) -> usize {
        if r <= l {
            return 0;
        }

        r - l - self.count_less_than(l, r, lower)
    }

    /// Returns:
    ///     v[l..r].into_iter().filter(|y| lower <= y < upper).count()
    pub fn count(&self, l: usize, r: usize, lower: usize, upper: usize) -> usize {
        if r <= l {
            return 0;
        }

        self.count_less_than(l, r, upper) - self.count_less_than(l, r, lower)
    }
}
