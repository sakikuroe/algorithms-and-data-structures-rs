use std::ops::RangeBounds;

use super::{bit_vector, wavelet_matrix_range};

/// 値のシーケンスを表現し、範囲内の値の順位クエリをサポートするデータ構造である。
#[derive(Clone)]
pub struct WaveletMatrix {
    pub(super) height: usize,
    pub(super) bit_table: Vec<bit_vector::BitVector>,
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

        for i in (0..height).rev() {
            bit_table.push(bit_vector::BitVector::new(
                &compress
                    .iter()
                    .map(|&x| ((x >> i) & 1) as u8)
                    .collect::<Vec<_>>(),
            ));
            compress = compress
                .iter()
                .filter(|&x| ((x >> i) & 1) == 0)
                .chain(compress.iter().filter(|&x| ((x >> i) & 1) == 1))
                .cloned()
                .collect::<Vec<_>>();
        }

        Self {
            height,
            bit_table,
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
        for (i, bit) in (0..self.height).rev().zip(self.bit_table.iter()) {
            let rank = bit.rank(position);
            let is_one = bit.rank(position + 1) != rank;
            let zeros = bit.len() - bit.rank(bit.len());
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
        if r <= l {
            return 0;
        }

        let mut result = 0;
        for (i, bit) in (0..self.height).rev().zip(self.bit_table.iter()) {
            let rank_l = bit.rank(l);
            let rank_r = bit.rank(r);
            if (upper >> i) & 1 == 0 {
                l -= rank_l;
                r -= rank_r;
            } else {
                result += (r - l) - (rank_r - rank_l);
                let zeros = bit.len() - bit.rank(bit.len());
                l = rank_l + zeros;
                r = rank_r + zeros;
            }
        }
        result
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
        self.count_less_than_compressed(l, r, upper) - self.count_less_than_compressed(l, r, lower)
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
        for (i, bit) in (0..self.height).rev().zip(self.bit_table.iter()) {
            let rank_l = bit.rank(l);
            let rank_r = bit.rank(r);
            let zeros = (r - l) - (rank_r - rank_l);
            if k < zeros {
                l -= rank_l;
                r -= rank_r;
            } else {
                let zeros_total = bit.len() - bit.rank(bit.len());
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
