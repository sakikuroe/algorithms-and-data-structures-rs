use std::ops::RangeBounds;

use super::{wavelet_matrix::WaveletMatrix, wavelet_matrix_range};

/// 累積和を保持し、値の和に関するクエリをサポートするベクタである。
#[derive(Clone)]
struct AccumulateVector {
    accum_table: Vec<usize>,
}

impl AccumulateVector {
    /// 値のスライスから累積和ベクタを作成する。
    fn new(values: &[usize]) -> Self {
        let mut accum_table = vec![0; values.len() + 1];
        for (i, &value) in values.iter().enumerate() {
            accum_table[i + 1] = accum_table[i] + value;
        }
        Self { accum_table }
    }

    /// 先頭から `index` 個の値の和を返す。
    fn rank(&self, index: usize) -> usize {
        self.accum_table[index]
    }
}

/// 範囲内の個数・順位・値の和に関するクエリをサポートする Wavelet Matrix である。
///
/// `WaveletMatrix` より多くの累積情報を保持するため、和を必要としない場合は
/// `WaveletMatrix` を使用する。
#[derive(Clone)]
pub struct WaveletMatrixWithSum {
    wavelet_matrix: WaveletMatrix,
    accum_table: Vec<AccumulateVector>,
    accum_v: AccumulateVector,
}

impl WaveletMatrixWithSum {
    /// `usize` のスライスから新しい `WaveletMatrixWithSum` を作成する。
    pub fn new(values: &[usize]) -> Self {
        let wavelet_matrix = WaveletMatrix::new(values);
        let mut compress = values
            .iter()
            .map(|&x| wavelet_matrix.sorted_v.partition_point(|&y| y < x))
            .collect::<Vec<_>>();
        let mut accum_table = Vec::with_capacity(wavelet_matrix.height);

        for i in (0..wavelet_matrix.height).rev() {
            accum_table.push(AccumulateVector::new(
                &compress
                    .iter()
                    .map(|&x| {
                        if (x >> i) & 1 == 0 {
                            wavelet_matrix.sorted_v[x]
                        } else {
                            0
                        }
                    })
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
            wavelet_matrix,
            accum_table,
            accum_v: AccumulateVector::new(values),
        }
    }

    /// 元のシーケンスの要素数を返す。
    pub fn len(&self) -> usize {
        self.wavelet_matrix.len()
    }

    /// 元のシーケンスが空であるかを返す。
    pub fn is_empty(&self) -> bool {
        self.wavelet_matrix.is_empty()
    }

    /// 元のシーケンスの `index` 番目の値を返す。
    pub fn get(&self, index: usize) -> Option<usize> {
        self.wavelet_matrix.get(index)
    }

    /// `index_range` と `value_range` の両方に含まれる要素数を返す。
    pub fn count<I, V>(&self, index_range: I, value_range: V) -> usize
    where
        I: RangeBounds<usize>,
        V: RangeBounds<usize>,
    {
        self.wavelet_matrix.count(index_range, value_range)
    }

    /// `index_range` と `value_range` に含まれる要素を昇順に並べたときの `k` 番目の値を返す。
    pub fn get_kth_smallest<I, V>(&self, index_range: I, value_range: V, k: usize) -> Option<usize>
    where
        I: RangeBounds<usize>,
        V: RangeBounds<usize>,
    {
        self.wavelet_matrix
            .get_kth_smallest(index_range, value_range, k)
    }

    /// `index_range` と `value_range` に含まれる要素を降順に並べたときの `k` 番目の値を返す。
    pub fn get_kth_largest<I, V>(&self, index_range: I, value_range: V, k: usize) -> Option<usize>
    where
        I: RangeBounds<usize>,
        V: RangeBounds<usize>,
    {
        self.wavelet_matrix
            .get_kth_largest(index_range, value_range, k)
    }

    /// `index_range` 内の `upper` 未満の要素の和を返す。
    pub fn get_sum_less_than<I>(&self, index_range: I, upper: usize) -> usize
    where
        I: RangeBounds<usize>,
    {
        self.get_sum(index_range, ..upper)
    }

    /// `index_range` 内の `lower` 以上の要素の和を返す。
    pub fn get_sum_more_than<I>(&self, index_range: I, lower: usize) -> usize
    where
        I: RangeBounds<usize>,
    {
        self.get_sum(index_range, lower..)
    }

    /// `index_range` と `value_range` の両方に含まれる要素の和を返す。
    pub fn get_sum<I, V>(&self, index_range: I, value_range: V) -> usize
    where
        I: RangeBounds<usize>,
        V: RangeBounds<usize>,
    {
        let (l, r) = wavelet_matrix_range::normalize_index_range(index_range, self.len());
        let (lower, upper) =
            wavelet_matrix_range::normalize_value_range(value_range, &self.wavelet_matrix.sorted_v);
        if upper <= lower {
            return 0;
        }
        self.get_sum_less_than_compressed(l, r, upper)
            - self.get_sum_less_than_compressed(l, r, lower)
    }

    /// `index_range` 内の `k` 個の最小要素の和を返す。
    pub fn get_sum_k_smallest<I>(&self, index_range: I, k: usize) -> Option<usize>
    where
        I: RangeBounds<usize>,
    {
        let (mut l, mut r) = wavelet_matrix_range::normalize_index_range(index_range, self.len());
        if r - l < k {
            return None;
        }
        if k == 0 {
            return Some(0);
        }

        let mut result = 0;
        let mut value = 0;
        let mut remaining = k;
        for (i, (bit, sum)) in (0..self.wavelet_matrix.height).rev().zip(
            self.wavelet_matrix
                .bit_table
                .iter()
                .zip(self.accum_table.iter()),
        ) {
            let (rank_l, rank_r) = bit.rank_pair(l, r);
            let zeros = (r - l) - (rank_r - rank_l);
            if remaining < zeros {
                l -= rank_l;
                r -= rank_r;
            } else {
                result += sum.rank(r) - sum.rank(l);
                let zeros_total = bit.len() - bit.rank(bit.len());
                l = rank_l + zeros_total;
                r = rank_r + zeros_total;
                value |= 1_usize << i;
                remaining -= zeros;
            }
        }
        result += remaining * self.wavelet_matrix.sorted_v[value];
        Some(result)
    }

    /// `index_range` 内の `k` 個の最大要素の和を返す。
    pub fn get_sum_k_largest<I>(&self, index_range: I, k: usize) -> Option<usize>
    where
        I: RangeBounds<usize> + Clone,
    {
        let (l, r) = wavelet_matrix_range::normalize_index_range(index_range.clone(), self.len());
        if r - l < k {
            return None;
        }
        Some(self.total_sum(l, r) - self.get_sum_k_smallest(index_range, r - l - k)?)
    }

    /// `index_range` 内の各値 `y` に対する `min(y, x)` の和を返す。
    pub fn get_sum_min<I>(&self, index_range: I, x: usize) -> usize
    where
        I: RangeBounds<usize> + Clone,
    {
        self.get_sum_less_than(index_range.clone(), x)
            + x * self.wavelet_matrix.count_more_than(index_range, x)
    }

    /// `index_range` 内の各値 `y` に対する `max(y, x)` の和を返す。
    pub fn get_sum_max<I>(&self, index_range: I, x: usize) -> usize
    where
        I: RangeBounds<usize> + Clone,
    {
        x * self.wavelet_matrix.count_less_than(index_range.clone(), x)
            + self.get_sum_more_than(index_range, x)
    }

    /// `index_range` 内の各値 `y` と `x` の絶対差の和を返す。
    pub fn get_sum_abs_diff<I>(&self, index_range: I, x: usize) -> usize
    where
        I: RangeBounds<usize> + Clone,
    {
        let less_count = self.wavelet_matrix.count_less_than(index_range.clone(), x);
        let more_count = self.wavelet_matrix.count_more_than(index_range.clone(), x);
        let less_sum = self.get_sum_less_than(index_range.clone(), x);
        let more_sum = self.get_sum_more_than(index_range, x);
        (x * less_count - less_sum) + (more_sum - x * more_count)
    }

    /// `index_range` 内の各値を `value_range` 内へ収めるための距離の和を返す。
    ///
    /// `value_range` が空の場合は `0` を返す。
    pub fn get_sum_distance_to_range<I, V>(&self, index_range: I, value_range: V) -> usize
    where
        I: RangeBounds<usize> + Clone,
        V: RangeBounds<usize> + Clone,
    {
        let lower = match value_range.start_bound() {
            std::ops::Bound::Included(&value) => value,
            std::ops::Bound::Excluded(&value) => value.saturating_add(1),
            std::ops::Bound::Unbounded => 0,
        };
        let upper = match value_range.end_bound() {
            std::ops::Bound::Included(&value) => value,
            std::ops::Bound::Excluded(&value) => value.saturating_sub(1),
            std::ops::Bound::Unbounded => usize::MAX,
        };
        if lower > upper
            || matches!(
                value_range.start_bound(),
                std::ops::Bound::Excluded(&usize::MAX)
            )
            || matches!(value_range.end_bound(), std::ops::Bound::Excluded(&0))
        {
            return 0;
        }

        let (l, r) = wavelet_matrix_range::normalize_index_range(index_range.clone(), self.len());
        let below_count = self.wavelet_matrix.count_less_than(l..r, lower);
        let below_sum = self.get_sum_less_than(l..r, lower);
        let total_sum = self.total_sum(l, r);
        let within_sum = self.get_sum(l..r, ..=upper);
        let above_count = r - l - self.wavelet_matrix.count(l..r, ..=upper);
        let above_sum = total_sum - within_sum;
        lower * below_count - below_sum + above_sum - upper * above_count
    }

    /// `index_range` の全要素の和を返す。
    fn total_sum(&self, l: usize, r: usize) -> usize {
        self.accum_v.rank(r) - self.accum_v.rank(l)
    }

    /// 圧縮インデックスが `upper` 未満となる要素の和を返す。
    fn get_sum_less_than_compressed(&self, mut l: usize, mut r: usize, upper: usize) -> usize {
        if r <= l {
            return 0;
        }

        let mut result = 0;
        for (i, (bit, sum)) in (0..self.wavelet_matrix.height).rev().zip(
            self.wavelet_matrix
                .bit_table
                .iter()
                .zip(self.accum_table.iter()),
        ) {
            let (rank_l, rank_r) = bit.rank_pair(l, r);
            if (upper >> i) & 1 == 0 {
                l -= rank_l;
                r -= rank_r;
            } else {
                result += sum.rank(r) - sum.rank(l);
                let zeros = bit.len() - bit.rank(bit.len());
                l = rank_l + zeros;
                r = rank_r + zeros;
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 背景: 重複値を含むシーケンスから構築した `WaveletMatrixWithSum`。
    fn create_wavelet_matrix() -> WaveletMatrixWithSum {
        WaveletMatrixWithSum::new(&[5, 4, 8, 6, 0, 7, 2, 5])
    }

    // 和クエリのテスト: 戻り値を検証する。
    mod sum {
        use super::*;

        /// 状況: 空のシーケンスに対する和クエリを処理する。
        /// - 前提: 空の `WaveletMatrixWithSum` がある。
        /// - 操作: 空の範囲と 0 個の順位和を求める。
        /// - 結果: 空の和として `0` または `Some(0)` を返す。
        #[test]
        fn handles_empty_sequence() {
            // 前提
            let sut = WaveletMatrixWithSum::new(&[]);
            // 操作と結果
            assert_eq!(0, sut.get_sum(.., ..));
            assert_eq!(Some(0), sut.get_sum_k_smallest(.., 0));
            assert_eq!(Some(0), sut.get_sum_k_largest(.., 0));
            let lower = 1;
            let upper = 0;
            assert_eq!(0, sut.get_sum_distance_to_range(.., lower..=upper));
            assert_eq!(
                0,
                sut.get_sum_distance_to_range(
                    ..,
                    (
                        std::ops::Bound::Excluded(usize::MAX),
                        std::ops::Bound::Unbounded,
                    ),
                )
            );
            assert_eq!(0, sut.get_sum_distance_to_range(.., ..0));
        }

        /// 状況: 値範囲、順位、距離に関する和を返す。
        /// - 前提: 重複値を含む `WaveletMatrixWithSum` がある。
        /// - 操作: 各種の和クエリを求める。
        /// - 結果: それぞれの定義に従った和を返す。
        #[test]
        fn returns_sums_for_value_queries() {
            // 前提
            let sut = create_wavelet_matrix();
            // 操作と結果
            assert_eq!(20, sut.get_sum(.., 4..7));
            assert_eq!(6, sut.get_sum_k_smallest(.., 3).unwrap());
            assert_eq!(21, sut.get_sum_k_largest(.., 3).unwrap());
            assert_eq!(34, sut.get_sum_min(.., 6));
            assert_eq!(51, sut.get_sum_max(.., 6));
            assert_eq!(17, sut.get_sum_abs_diff(.., 6));
            assert_eq!(9, sut.get_sum_distance_to_range(.., 4..=6));
            assert_eq!(
                0,
                sut.get_sum_distance_to_range(
                    ..,
                    (
                        std::ops::Bound::Excluded(usize::MAX),
                        std::ops::Bound::Unbounded,
                    ),
                )
            );
            assert_eq!(0, sut.get_sum_distance_to_range(.., ..0));
        }

        /// 状況: 要素数を超える個数の順位和を求める。
        /// - 前提: 要素数 8 の `WaveletMatrixWithSum` がある。
        /// - 操作: 9 個の順位和を求める。
        /// - 結果: `None` を返す。
        #[test]
        fn returns_none_when_k_exceeds_length() {
            // 前提
            let sut = create_wavelet_matrix();
            // 操作と結果
            assert_eq!(None, sut.get_sum_k_smallest(.., 9));
            assert_eq!(None, sut.get_sum_k_largest(.., 9));
        }

        /// 状況: 要素数が十分に大きい入力を構築して和クエリを実行する。
        /// - 前提: 100,000 個の異なる値を持つシーケンスがある。
        /// - 操作: `WaveletMatrixWithSum` を構築し、全体和と順位和を求める。
        /// - 結果: 現実的な時間で完了し、結果が正しい。
        #[test]
        fn builds_and_sums_large_input() {
            // 前提
            let data = (0..100_000).collect::<Vec<_>>();
            let expected_total = 100_000_usize * 99_999 / 2;
            // 操作
            let sut = WaveletMatrixWithSum::new(&data);
            // 結果
            assert_eq!(expected_total, sut.get_sum(.., ..));
            assert_eq!(10 * 11 / 2, sut.get_sum_k_smallest(.., 11).unwrap());
            assert_eq!(99_989 * 11 + 55, sut.get_sum_k_largest(.., 11).unwrap());
        }
    }
}
