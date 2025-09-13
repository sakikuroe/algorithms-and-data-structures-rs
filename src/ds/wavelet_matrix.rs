use super::bit_vector;

/// A data structure that represents a sequence of values and supports fast ranking queries.
/// 値のシーケンスを表現し, 高速な rank クエリをサポートするデータ構造である.
#[derive(Clone)]
pub struct WaveletMatrix {
    height: usize,
    bit_table: Vec<bit_vector::BitVector>,
    sorted_v: Vec<usize>,
}

impl WaveletMatrix {
    /// Creates a new WaveletMatrix from a slice of `usize`.
    /// `usize` のスライスから新しい WaveletMatrix を作成する.
    ///
    /// # Args
    /// - `v`: A slice of `usize` to be stored in the WaveletMatrix.
    ///        WaveletMatrix に格納する `usize` のスライス.
    ///
    /// # Returns
    /// `WaveletMatrix`: A new instance of WaveletMatrix.
    ///                  WaveletMatrix の新しいインスタンス.
    ///
    /// # Complexity
    /// - Time complexity: O(N log N), where N is the length of `v`.
    ///                          ここで N は `v` の長さである.
    /// - Space complexity: O(N log U), where N is the length of `v` and U is the number of distinct values in `v`.
    ///                           ここで N は `v` の長さ, U は `v` のユニークな値の数である.
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

    /// Returns the count of elements `y` such that `y < upper` in the range `v[l..r]`.
    /// `v[l..r]` の範囲にある要素 `y` のうち, `y < upper` となるものの個数を返す.
    ///
    /// # Args
    /// - `l`: The start index of the range (inclusive).
    ///        範囲の開始インデックス (inclusive).
    /// - `r`: The end index of the range (exclusive).
    ///        範囲の終了インデックス (exclusive).
    /// - `upper`: The upper bound value (exclusive).
    ///            上限値 (exclusive).
    ///
    /// # Returns
    /// `usize`: The number of elements less than `upper` in the specified range.
    ///          指定された範囲内の `upper` 未満の要素数.
    ///
    /// # Complexity
    /// - Time complexity: O(log U), where U is the number of distinct values in the original sequence.
    ///                          ここで U は元のシーケンスにおけるユニークな値の数である.
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

    /// Returns the count of elements `y` such that `y >= lower` in the range `v[l..r]`.
    /// `v[l..r]` の範囲にある要素 `y` のうち, `y >= lower` となるものの個数を返す.
    ///
    /// # Args
    /// - `l`: The start index of the range (inclusive).
    ///        範囲の開始インデックス (inclusive).
    /// - `r`: The end index of the range (exclusive).
    ///        範囲の終了インデックス (exclusive).
    /// - `lower`: The lower bound value (inclusive).
    ///            下限値 (inclusive).
    ///
    /// # Returns
    /// `usize`: The number of elements greater than or equal to `lower` in the specified range.
    ///          指定された範囲内の `lower` 以上の要素数.
    ///
    /// # Complexity
    /// - Time complexity: O(log U), where U is the number of distinct values in the original sequence.
    ///                               ここで U は元のシーケンスにおけるユニークな値の数である.
    pub fn count_more_than(&self, l: usize, r: usize, lower: usize) -> usize {
        if r <= l {
            return 0;
        }

        r - l - self.count_less_than(l, r, lower)
    }

    /// Returns the count of elements `y` such that `lower <= y < upper` in the range `v[l..r]`.
    /// `v[l..r]` の範囲にある要素 `y` のうち, `lower <= y < upper` となるものの個数を返す.
    ///
    /// # Args
    /// - `l`: The start index of the range (inclusive).
    ///        範囲の開始インデックス (inclusive).
    /// - `r`: The end index of the range (exclusive).
    ///        範囲の終了インデックス (exclusive).
    /// - `lower`: The lower bound value (inclusive).
    ///            下限値 (inclusive).
    /// - `upper`: The upper bound value (exclusive).
    ///            上限値 (exclusive).
    ///
    /// # Returns
    /// `usize`: The number of elements in the range [`lower`, `upper`) in the specified range of `v`.
    ///          `v` の指定された範囲内にある, [`lower`, `upper`) の範囲の要素数.
    ///
    /// # Complexity
    /// - Time complexity: O(log U), where U is the number of distinct values in the original sequence.
    ///                          ここで U は元のシーケンスにおけるユニークな値の数である.
    pub fn count(&self, l: usize, r: usize, lower: usize, upper: usize) -> usize {
        if r <= l {
            return 0;
        }

        // Return 0 immediately to prevent subtraction overflow.
        // 減算によるオーバーフローを防ぐため, 直ちに 0 を返す.
        if lower >= upper {
            return 0;
        }

        self.count_less_than(l, r, upper) - self.count_less_than(l, r, lower)
    }
}
