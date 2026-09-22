use std::ops::RangeBounds;

use super::{bit_vector, wavelet_matrix_range};

/// 値のシーケンスを表現し、範囲内の値の順位クエリをサポートするデータ構造である。
#[derive(Clone)]
pub struct WaveletMatrix {
    pub(super) height: usize,
    pub(super) bit_table: Vec<bit_vector::BitVector>,
    pub(super) zero_counts: Vec<usize>,
    pub(super) sorted_v: Vec<usize>,
    pub(super) len: usize,
}

impl WaveletMatrix {
    /// `usize` のスライスから新しい `WaveletMatrix` を作成する。
    ///
    /// # Args
    /// - `v`: 格納する値のスライス。
    ///
    /// # Returns
    /// 値を座標圧縮して構築した `WaveletMatrix`。
    pub fn new(v: &[usize]) -> Self {
        let mut sorted_v = v.to_vec();
        sorted_v.sort_unstable();
        sorted_v.dedup();

        let mut compress = v
            .iter()
            .map(|&x| sorted_v.partition_point(|&y| y < x))
            .collect::<Vec<_>>();
        let height = if sorted_v.is_empty() {
            1
        } else {
            usize::BITS as usize - sorted_v.len().leading_zeros() as usize
        };
        let mut bit_table = Vec::with_capacity(height);
        let mut zero_counts = Vec::with_capacity(height);
        // Two reusable buffers for the stable partition at each level.
        let mut next = vec![0_usize; compress.len()];

        for i in (0..height).rev() {
            let num_zeros = compress.iter().filter(|&&x| ((x >> i) & 1) == 0).count();
            let mut words = vec![0_u64; compress.len() / u64::BITS as usize + 1];
            let mut zero_pos = 0_usize;
            let mut one_pos = num_zeros;
            let mut word = 0_u64;
            let mut word_index = 0_usize;
            for (pos, &x) in compress.iter().enumerate() {
                if ((x >> i) & 1) == 0 {
                    next[zero_pos] = x;
                    zero_pos += 1;
                } else {
                    next[one_pos] = x;
                    one_pos += 1;
                    word |= 1_u64 << (pos & 63);
                }
                if (pos & 63) == 63 {
                    words[word_index] = word;
                    word_index += 1;
                    word = 0;
                }
            }
            if (compress.len() & 63) != 0 {
                words[word_index] = word;
            }
            zero_counts.push(num_zeros);
            bit_table.push(bit_vector::BitVector::from_words(words, compress.len()));
            std::mem::swap(&mut compress, &mut next);
        }

        Self {
            height,
            bit_table,
            zero_counts,
            sorted_v,
            len: v.len(),
        }
    }

    /// 元のシーケンスの要素数を返す。
    pub fn len(&self) -> usize {
        self.len
    }

    /// 元のシーケンスが空であるかを返す。
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// 元のシーケンスの `index` 番目の値を返す。
    ///
    /// 範囲外のインデックスに対しては `None` を返す。
    pub fn get(&self, index: usize) -> Option<usize> {
        if index >= self.len {
            return None;
        }

        let mut position = index;
        let mut compressed_value = 0;
        for (level, (i, bit)) in (0..self.height)
            .rev()
            .zip(self.bit_table.iter())
            .enumerate()
        {
            let (rank, next_rank) = bit.rank_pair(position, position + 1);
            let is_one = next_rank != rank;
            let zeros = self.zero_counts[level];
            if is_one {
                compressed_value |= 1_usize << i;
                position = rank + zeros;
            } else {
                position -= rank;
            }
        }

        Some(self.sorted_v[compressed_value])
    }

    /// 値の圧縮インデックスが `upper` 未満となる要素の個数を返す。
    fn count_less_than_compressed(&self, mut l: usize, mut r: usize, upper: usize) -> usize {
        if r <= l || upper == 0 {
            return 0;
        }
        // 圧縮値はつねに `sorted_v.len()` 未満なので、上限が種類数以上なら
        // 区間内の全要素が条件を満たす。
        if upper >= self.sorted_v.len() {
            return r - l;
        }

        let mut result = 0;
        for (level, (i, bit)) in (0..self.height)
            .rev()
            .zip(self.bit_table.iter())
            .enumerate()
        {
            let (rank_l, rank_r) = bit.rank_pair(l, r);
            if (upper >> i) & 1 == 0 {
                l -= rank_l;
                r -= rank_r;
            } else {
                result += (r - l) - (rank_r - rank_l);
                let zeros = self.zero_counts[level];
                l = rank_l + zeros;
                r = rank_r + zeros;
            }
        }
        result
    }

    /// Counts compressed values in `[lower, upper)` while sharing the common prefix traversal.
    fn count_in_value_range_compressed(
        &self,
        mut l: usize,
        mut r: usize,
        lower: usize,
        upper: usize,
    ) -> usize {
        if upper <= lower || r <= l {
            return 0;
        }

        let mut lower_l = l;
        let mut lower_r = r;
        let mut upper_l = l;
        let mut upper_r = r;
        let mut lower_result = 0;
        let mut upper_result = 0;
        let mut diverged = false;

        for (level, (i, bit)) in (0..self.height)
            .rev()
            .zip(self.bit_table.iter())
            .enumerate()
        {
            if !diverged && ((lower >> i) & 1) == ((upper >> i) & 1) {
                let (rank_l, rank_r) = bit.rank_pair(l, r);
                if (lower >> i) & 1 == 0 {
                    l -= rank_l;
                    r -= rank_r;
                } else {
                    let zeros = (r - l) - (rank_r - rank_l);
                    lower_result += zeros;
                    upper_result += zeros;
                    let zeros_total = self.zero_counts[level];
                    l = rank_l + zeros_total;
                    r = rank_r + zeros_total;
                }
                continue;
            }

            if !diverged {
                diverged = true;
                lower_l = l;
                lower_r = r;
                upper_l = l;
                upper_r = r;
            }

            let (lower_rank_l, lower_rank_r) = bit.rank_pair(lower_l, lower_r);
            if (lower >> i) & 1 == 0 {
                lower_l -= lower_rank_l;
                lower_r -= lower_rank_r;
            } else {
                lower_result += (lower_r - lower_l) - (lower_rank_r - lower_rank_l);
                let zeros_total = self.zero_counts[level];
                lower_l = lower_rank_l + zeros_total;
                lower_r = lower_rank_r + zeros_total;
            }

            let (upper_rank_l, upper_rank_r) = bit.rank_pair(upper_l, upper_r);
            if (upper >> i) & 1 == 0 {
                upper_l -= upper_rank_l;
                upper_r -= upper_rank_r;
            } else {
                upper_result += (upper_r - upper_l) - (upper_rank_r - upper_rank_l);
                let zeros_total = self.zero_counts[level];
                upper_l = upper_rank_l + zeros_total;
                upper_r = upper_rank_r + zeros_total;
            }
        }

        upper_result - lower_result
    }

    /// `index_range` 内の `upper` 未満の要素数を返す。
    pub fn count_less_than<I>(&self, index_range: I, upper: usize) -> usize
    where
        I: RangeBounds<usize>,
    {
        let (l, r) = wavelet_matrix_range::normalize_index_range(index_range, self.len);
        let upper = self.sorted_v.partition_point(|&x| x < upper);
        self.count_less_than_compressed(l, r, upper)
    }

    /// `index_range` 内の `lower` 以上の要素数を返す。
    pub fn count_more_than<I>(&self, index_range: I, lower: usize) -> usize
    where
        I: RangeBounds<usize>,
    {
        let (l, r) = wavelet_matrix_range::normalize_index_range(index_range, self.len);
        let lower = self.sorted_v.partition_point(|&x| x < lower);
        (r - l) - self.count_less_than_compressed(l, r, lower)
    }

    /// `index_range` と `value_range` の両方に含まれる要素数を返す。
    pub fn count<I, V>(&self, index_range: I, value_range: V) -> usize
    where
        I: RangeBounds<usize>,
        V: RangeBounds<usize>,
    {
        let (l, r) = wavelet_matrix_range::normalize_index_range(index_range, self.len);
        let (lower, upper) =
            wavelet_matrix_range::normalize_value_range(value_range, &self.sorted_v);
        if upper <= lower {
            return 0;
        }
        self.count_in_value_range_compressed(l, r, lower, upper)
    }

    /// `index_range` と `value_range` に含まれる要素を昇順に並べたときの `k` 番目の値を返す。
    ///
    /// `k` は 0 始まりであり、対象要素が存在しない場合は `None` を返す。
    pub fn get_kth_smallest<I, V>(&self, index_range: I, value_range: V, k: usize) -> Option<usize>
    where
        I: RangeBounds<usize>,
        V: RangeBounds<usize>,
    {
        let (l, r) = wavelet_matrix_range::normalize_index_range(index_range, self.len);
        let (lower, upper) =
            wavelet_matrix_range::normalize_value_range(value_range, &self.sorted_v);
        let first = self.count_less_than_compressed(l, r, lower);
        let end = self.count_less_than_compressed(l, r, upper);
        if upper <= lower || k >= end - first {
            return None;
        }
        self.get_kth_smallest_compressed(l, r, first + k)
    }

    /// `index_range` と `value_range` に含まれる要素を降順に並べたときの `k` 番目の値を返す。
    ///
    /// `k` は 0 始まりであり、対象要素が存在しない場合は `None` を返す。
    pub fn get_kth_largest<I, V>(&self, index_range: I, value_range: V, k: usize) -> Option<usize>
    where
        I: RangeBounds<usize>,
        V: RangeBounds<usize>,
    {
        let (l, r) = wavelet_matrix_range::normalize_index_range(index_range, self.len);
        let (lower, upper) =
            wavelet_matrix_range::normalize_value_range(value_range, &self.sorted_v);
        let first = self.count_less_than_compressed(l, r, lower);
        let end = self.count_less_than_compressed(l, r, upper);
        if upper <= lower || k >= end - first {
            return None;
        }
        self.get_kth_smallest_compressed(l, r, end - 1 - k)
    }

    /// 値の圧縮インデックスが `k` 番目となる要素を返す。
    fn get_kth_smallest_compressed(
        &self,
        mut l: usize,
        mut r: usize,
        mut k: usize,
    ) -> Option<usize> {
        if r <= l || r - l <= k {
            return None;
        }

        let mut compressed_value = 0;
        for (level, (i, bit)) in (0..self.height)
            .rev()
            .zip(self.bit_table.iter())
            .enumerate()
        {
            let (rank_l, rank_r) = bit.rank_pair(l, r);
            let zeros = (r - l) - (rank_r - rank_l);
            if k < zeros {
                l -= rank_l;
                r -= rank_r;
            } else {
                let zeros_total = self.zero_counts[level];
                l = rank_l + zeros_total;
                r = rank_r + zeros_total;
                compressed_value |= 1_usize << i;
                k -= zeros;
            }
        }
        Some(self.sorted_v[compressed_value])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Background: 重複値を含むシーケンスから構築した `WaveletMatrix`。
    fn create_wavelet_matrix() -> WaveletMatrix {
        WaveletMatrix::new(&[5, 4, 8, 6, 0, 7, 2, 5])
    }

    /// Background: `usize::MAX` を含むシーケンスから構築した `WaveletMatrix`。
    fn create_wavelet_matrix_with_max_value() -> WaveletMatrix {
        WaveletMatrix::new(&[0, usize::MAX, 1, 100, usize::MAX])
    }

    // get のテスト: 戻り値を検証する。
    mod get {
        use super::*;

        /// Scenario: 元の位置に対応する値を返す。
        /// - Given: 値を格納した `WaveletMatrix` がある。
        /// - When: 有効な位置と範囲外の位置から値を取得する。
        /// - Then: 有効な位置では値を、範囲外では `None` を返す。
        #[test]
        fn returns_value_at_index() {
            // Given
            let sut = create_wavelet_matrix();
            // When, Then
            assert_eq!(5, sut.get(0).unwrap());
            assert_eq!(5, sut.get(7).unwrap());
            assert_eq!(None, sut.get(8));
        }
    }

    // 大規模入力のテスト: 明らかな二次計算量を検出する。
    mod large_input {
        use super::*;

        /// Scenario: 要素数が十分に大きい入力を構築してクエリを実行する。
        /// - Given: 100,000 個の異なる値を持つシーケンスがある。
        /// - When: `WaveletMatrix` を構築し、代表的なクエリを実行する。
        /// - Then: 現実的な時間で完了し、結果が正しい。
        #[test]
        fn builds_and_queries_large_input() {
            // Given
            let data = (0..100_000).collect::<Vec<_>>();
            // When
            let sut = WaveletMatrix::new(&data);
            // Then
            assert_eq!(100_000, sut.len());
            assert_eq!(Some(99_999), sut.get(99_999));
            assert_eq!(25_000, sut.count(.., 50_000..75_000));
            assert_eq!(Some(50_000), sut.get_kth_smallest(.., .., 50_000));
            assert_eq!(Some(99_999), sut.get_kth_largest(.., .., 0));
        }
    }

    // count のテスト: 戻り値を検証する。
    mod count {
        use super::*;

        /// Scenario: インデックス範囲と値範囲に含まれる要素数を返す。
        /// - Given: 値を格納した `WaveletMatrix` がある。
        /// - When: 包含・排他境界を含む複数の範囲で個数を求める。
        /// - Then: 各範囲に含まれる要素数を返す。
        #[test]
        fn counts_values_in_ranges() {
            // Given
            let sut = create_wavelet_matrix();
            // When, Then
            assert_eq!(4, sut.count(0..8, 4..7));
            assert_eq!(3, sut.count(2..7, 5..=8));
            assert_eq!(5, sut.count(.., 5..));
            assert_eq!(2, sut.count(.., ..=2));
            let start = 8;
            let end = 3;
            assert_eq!(0, sut.count(start..end, ..));
        }

        /// Scenario: `usize::MAX` を含む値範囲を正しく扱う。
        /// - Given: `usize::MAX` を含む `WaveletMatrix` がある。
        /// - When: 上限なしと最大値を含む値範囲の個数を求める。
        /// - Then: オーバーフローせず正しい個数を返す。
        #[test]
        fn handles_usize_max_value() {
            // Given
            let sut = create_wavelet_matrix_with_max_value();
            // When, Then
            assert_eq!(5, sut.count(.., ..));
            assert_eq!(2, sut.count(.., usize::MAX..=usize::MAX));
        }
    }

    // count_less_than と count_more_than のテスト: 戻り値を検証する。
    mod threshold_count {
        use super::*;

        /// Scenario: 閾値に対する個数を返す。
        /// - Given: 値を格納した `WaveletMatrix` がある。
        /// - When: 閾値未満と閾値以上の個数を求める。
        /// - Then: 条件を満たす要素数を返す。
        #[test]
        fn counts_values_on_each_side_of_threshold() {
            // Given
            let sut = create_wavelet_matrix();
            // When, Then
            assert_eq!(3, sut.count_less_than(0..8, 5));
            assert_eq!(5, sut.count_more_than(0..8, 5));
            let start = 8;
            let end = 3;
            assert_eq!(0, sut.count_more_than(start..end, 5));
        }
    }

    // get_kth_smallest と get_kth_largest のテスト: 戻り値を検証する。
    mod kth {
        use super::*;

        /// Scenario: 値範囲内の順位要素を返す。
        /// - Given: 重複値を含む `WaveletMatrix` がある。
        /// - When: 小さい順と大きい順の順位要素を求める。
        /// - Then: 指定順位の値を返す。
        #[test]
        fn returns_kth_values() {
            // Given
            let sut = create_wavelet_matrix();
            // When, Then
            assert_eq!(Some(5), sut.get_kth_smallest(.., 5.., 0));
            assert_eq!(Some(8), sut.get_kth_smallest(2..7, 5.., 2));
            assert_eq!(Some(7), sut.get_kth_largest(.., ..8, 0));
            assert_eq!(Some(0), sut.get_kth_largest(.., .., 7));
        }

        /// Scenario: 対象要素がない、または順位が範囲外である。
        /// - Given: 値を格納した `WaveletMatrix` がある。
        /// - When: 空の値範囲または範囲外の順位を指定する。
        /// - Then: `None` を返す。
        #[test]
        fn returns_none_for_missing_rank() {
            // Given
            let sut = create_wavelet_matrix();
            // When, Then
            assert_eq!(None, sut.get_kth_smallest(.., 8..8, 0));
            assert_eq!(None, sut.get_kth_largest(.., .., 8));
        }
    }
}
