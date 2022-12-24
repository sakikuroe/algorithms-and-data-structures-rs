/// verified by
/// - AtCoder | [AtCoder Beginner Contest 281 E - Least Elements](https://atcoder.jp/contests/abc281/tasks/abc281_e), ([submittion](https://atcoder.jp/contests/abc281/submissions/37286128))
/// - AtCoder | [Chokudai SpeedRun 001 J - 転倒数](https://atcoder.jp/contests/chokudai_S001/tasks/chokudai_S001_j), ([submittion](https://atcoder.jp/contests/chokudai_S001/submissions/37286099))
/// - AtCoder | [AtCoder Beginner Contest 174 F - Range Set Query](https://atcoder.jp/contests/abc174/tasks/abc174_f), ([submittion](https://atcoder.jp/contests/abc174/submissions/37286021))
/// - AtCoder | [AtCoder Beginner Contest 241（Sponsored by Panasonic） D - Sequence Query](https://atcoder.jp/contests/abc241/tasks/abc241_d), ([submittion](https://atcoder.jp/contests/abc241/submissions/37308280))
/// - AtCoder | [AtCoder Beginner Contest 234 D - Prefix K-th Max](https://atcoder.jp/contests/abc234/tasks/abc234_d), ([submittion](https://atcoder.jp/contests/abc234/submissions/37313950))
/// - Library Checker | [Range Kth Smallest](https://judge.yosupo.jp/problem/range_kth_smallest), ([submittion](https://judge.yosupo.jp/submission/116350))
/// - Aizu Online Judge | [Hard Beans](https://judge.u-aizu.ac.jp/onlinejudge/description.jsp?id=1549), ([submittion](https://judge.u-aizu.ac.jp/onlinejudge/review.jsp?rid=7183138#1))

// ceil(log2(cardinality(X)))
// ex) when v : &[u32] -> 32,
//     when v : &[u64] -> 64
const WAVELET_MATRIX_HEIGHT: usize = 64;

#[derive(Clone)]
pub struct WaveletMatrix {
    bit_table: Vec<WaveletMatrixRow>,
    accum: Vec<WaveletMatrixRow>,
}

impl WaveletMatrix {
    pub fn new(v: &[usize]) -> Self {
        let mut v = v.to_vec();
        let mut bit_table = vec![];
        let mut accum = vec![];
        for i in (0..WAVELET_MATRIX_HEIGHT).rev() {
            bit_table.push(WaveletMatrixRow::new(
                v.iter().map(|&x| (x >> i) & 1).collect(),
            ));
            accum.push(WaveletMatrixRow::new(
                v.iter()
                    .map(|&x| if (x >> i) & 1 == 0 { x } else { 0 })
                    .collect::<Vec<_>>(),
            ));

            // slow code
            // ```
            // v.sort_by_key(|&x| (x >> i) & 1);
            // ```
            v = v
                .iter()
                .filter(|&x| (x >> i) & 1 == 0)
                .chain(v.iter().filter(|&x| (x >> i) & 1 == 1))
                .copied()
                .collect::<Vec<_>>();
        }

        WaveletMatrix { bit_table, accum }
    }

    /// Returns:
    ///     v[i]
    pub fn access(&self, mut i: usize) -> Option<usize> {
        if i >= self.bit_table[0].len() {
            return None;
        }

        let mut res = 0;

        for row in &self.bit_table {
            res <<= 1;
            res |= row.access(i);
            i = match row.access(i) {
                0 => i - row.rank(i),
                1 => row.len() - row.rank(row.len()) + row.rank(i),
                _ => {
                    unreachable!()
                }
            };
        }

        Some(res)
    }

    /// Returns:
    ///     v[l..r].into_iter().filter(|y| **y == x).count()
    pub fn rank(&self, mut l: usize, mut r: usize, x: usize) -> usize {
        for (i, row) in self.bit_table.iter().rev().enumerate().rev() {
            if (x >> i) & 1 == 0 {
                l = l - row.rank(l);
                r = r - row.rank(r);
            } else {
                l = row.rank(l) + (row.len() - row.rank(row.len()));
                r = row.rank(r) + (row.len() - row.rank(row.len()));
            }
        }

        r - l
    }

    /// Returns:
    ///     {
    ///         use itertools::Itertools;
    ///         v[l..r].into_iter().sorted().nth(k)
    ///     }
    pub fn get_kth_smallest(&self, mut l: usize, mut r: usize, mut k: usize) -> Option<usize> {
        if l >= r || r - l <= k {
            return None;
        }

        let mut res = 0;

        for (i, row) in self.bit_table.iter().rev().enumerate().rev() {
            let j = (r - l) - (row.rank(r) - row.rank(l));
            if j > k {
                l = l - row.rank(l);
                r = r - row.rank(r);
            } else {
                l = row.rank(l) + (row.len() - row.rank(row.len()));
                r = row.rank(r) + (row.len() - row.rank(row.len()));
                res |= 1 << i;
                k -= j;
            }
        }

        Some(res)
    }

    /// Returns:
    ///     {
    ///         use itertools::Itertools;
    ///         v[l..r].into_iter().sorted().rev().nth(k)
    ///     }
    pub fn get_kth_largest(&self, l: usize, r: usize, k: usize) -> Option<usize> {
        if !(r >= l + 1 + k) {
            None
        } else {
            self.get_kth_smallest(l, r, r - l - 1 - k)
        }
    }

    /// Returns:
    ///     {
    ///         use itertools::Itertools;
    ///         v[l..r].into_iter().sorted().take(k).sum::<usize>()
    ///     }
    pub fn get_lower_sum(&self, mut l: usize, mut r: usize, mut k: usize) -> usize {
        let mut res = 0;
        let mut kth = 0;

        for (i, (row, sum)) in (self.bit_table.iter().zip(self.accum.iter()))
            .rev()
            .enumerate()
            .rev()
        {
            let j = (r - l) - (row.rank(r) - row.rank(l));
            if j > k {
                l = l - row.rank(l);
                r = r - row.rank(r);
            } else {
                res += sum.rank(r);
                res -= sum.rank(l);
                l = row.rank(l) + (row.len() - row.rank(row.len()));
                r = row.rank(r) + (row.len() - row.rank(row.len()));
                kth |= 1 << i;
                k -= j;
            }
        }

        res += k * kth;

        res
    }

    /// Returns:
    ///     v[l..r].into_iter().filter(|y| lower <= **y && **y < upper).count()
    pub fn count(&self, l: usize, r: usize, lower: usize, upper: usize) -> usize {
        let range_freq_sub = |mut l: usize, mut r: usize, upper: usize| -> usize {
            let mut res = 0;

            for (i, row) in self.bit_table.iter().rev().enumerate().rev() {
                if (upper >> i) & 1 == 0 {
                    l = l - row.rank(l);
                    r = r - row.rank(r);
                } else {
                    res += (r - l) - (row.rank(r) - row.rank(l));
                    l = row.rank(l) + (row.len() - row.rank(row.len()));
                    r = row.rank(r) + (row.len() - row.rank(row.len()));
                }
            }

            res
        };

        range_freq_sub(l, r, upper) - range_freq_sub(l, r, lower)
    }

    /// Returns:
    ///     {
    ///         use itertools::Itertools;
    ///         v[l..r].into_iter().filter(|y| lower <= **y).sorted().nth(k)
    ///     }
    pub fn get_above_kth_smallest(
        &self,
        l: usize,
        r: usize,
        lower: usize,
        k: usize,
    ) -> Option<usize> {
        let cnt = self.count(l, r, 0, lower);
        if cnt + k >= r - l {
            None
        } else {
            Some(self.get_kth_smallest(l, r, cnt + k).unwrap())
        }
    }

    /// Returns:
    ///     v[l..r].into_iter().filter(|y| **y < upper).sorted().rev().nth(k)
    pub fn get_below_kth_largest(
        &self,
        l: usize,
        r: usize,
        upper: usize,
        k: usize,
    ) -> Option<usize> {
        let cnt = self.count(l, r, 0, upper);
        if cnt <= k {
            None
        } else {
            Some(self.get_kth_smallest(l, r, cnt - 1 - k).unwrap())
        }
    }
}

#[derive(Clone)]
pub struct WaveletMatrixRow {
    accum: Vec<usize>,
}

impl WaveletMatrixRow {
    pub fn new(bit: Vec<usize>) -> Self {
        let mut accum = vec![0; bit.len() + 1];
        for i in 0..bit.len() {
            accum[i + 1] = accum[i] + bit[i];
        }
        WaveletMatrixRow { accum }
    }

    pub fn len(&self) -> usize {
        self.accum.len() - 1
    }

    pub fn access(&self, i: usize) -> usize {
        self.accum[i + 1] - self.accum[i]
    }

    pub fn rank(&self, i: usize) -> usize {
        self.accum[i]
    }
}
