use std::ops::RangeBounds;

use super::{wavelet_matrix::WaveletMatrix, wavelet_matrix_range};

/// 累積和を保持し、値の和に関するクエリをサポートするベクタである。
#[derive(Clone)]
struct AccumulateVector {
    /// 先頭から各要素数までの累積和。`accum_table[0]` は 0 である。
    accum_table: Vec<usize>,
}

impl AccumulateVector {
    /// 値のスライスから累積和ベクタを作成する。
    ///
    /// # Args
    /// - `values`: 累積和を作る非負整数列。
    ///
    /// # Returns
    /// `values` の先頭に 0 を置いた長さ `values.len() + 1` の累積和ベクタを返す。
    ///
    /// # Panics
    /// オーバーフローチェックが有効な場合、累積和が `usize` の範囲を超えるとパニックする。
    /// チェックが無効な場合、オーバーフローした加算はラップする。
    ///
    /// # Complexity
    /// - 時間計算量: O(N) である。$N$ は `values` の長さである。
    /// - 空間計算量: O(N) である。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::ds::wavelet_matrix_with_sum::WaveletMatrixWithSum;
    ///
    /// let matrix = WaveletMatrixWithSum::new(&[2, 3, 5]);
    /// assert_eq!(5, matrix.get_sum(0..2, ..));
    /// ```
    fn new(values: &[usize]) -> Self {
        // 区間和を `prefix[r] - prefix[l]` で求められるよう、先頭に 0 を置く。
        let mut accum_table = vec![0; values.len() + 1];
        for (i, &value) in values.iter().enumerate() {
            // i 番目までの和に現在の値を加え、i + 1 個までの累積和を保存する。
            accum_table[i + 1] = accum_table[i] + value;
        }
        Self { accum_table }
    }

    /// 先頭から `index` 個の値の和を返す。
    ///
    /// # Args
    /// - `index`: 累積和を求める要素数。`index` は元の列の長さ以下でなければならない。
    ///
    /// # Returns
    /// 先頭 `index` 個の値の和を返す。
    ///
    /// # Panics
    /// `index` が列の長さを超える場合にパニックする。
    ///
    /// # Complexity
    /// - 時間計算量: O(1) である。
    /// - 空間計算量: O(1) である。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::ds::wavelet_matrix_with_sum::WaveletMatrixWithSum;
    ///
    /// let matrix = WaveletMatrixWithSum::new(&[2, 3, 5]);
    /// assert_eq!(5, matrix.get_sum(0..2, ..));
    /// ```
    fn rank(&self, index: usize) -> usize {
        // 累積和配列では添字がそのまま要素数を表す。
        self.accum_table[index]
    }
}

/// 範囲内の個数・順位・値の和に関するクエリをサポートする Wavelet Matrix である。
///
/// `WaveletMatrix` より多くの累積情報を保持するため、和を必要としない場合は
/// `WaveletMatrix` を使用する。
///
/// 累積和と和クエリの中間計算には `usize` を使用するため、入力の累積和と各クエリの
/// 中間結果が `usize` の範囲に収まる必要がある。オーバーフローチェックが有効な場合は
/// オーバーフロー時にパニックし、無効な場合は Rust の整数演算に従ってラップする。
#[derive(Clone)]
pub struct WaveletMatrixWithSum {
    /// 値の順位クエリと座標圧縮を担う Wavelet Matrix。
    wavelet_matrix: WaveletMatrix,
    /// 各ビットレベルの 0 側の値について、レベル内順序で作成した累積和。
    accum_table: Vec<AccumulateVector>,
    /// 元の入力順に並ぶ値の累積和。
    accum_v: AccumulateVector,
}

impl WaveletMatrixWithSum {
    /// `usize` のスライスから新しい `WaveletMatrixWithSum` を作成する。
    ///
    /// # Args
    /// - `values`: 元の順序を保ったまま格納する非負整数列。長さは `2^32` 未満で、累積和は
    ///   `usize` の範囲に収まる必要がある。
    ///
    /// # Panics
    /// `values` の長さが `2^32` 以上の場合はデバッグビルドでパニックする。
    /// オーバーフローチェックが有効な場合、構築中に累積和が `usize` の範囲を超えてもパニックする。
    ///
    /// # Returns
    /// 順位クエリと範囲和クエリを行えるデータ構造を返す。
    ///
    /// # Complexity
    /// - 時間計算量: O(N log N + N log(U + 1)) である。$N$ は要素数、$U$ は異なる値の個数である。
    /// - 空間計算量: O(N log(U + 1)) である。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::ds::wavelet_matrix_with_sum::WaveletMatrixWithSum;
    ///
    /// let matrix = WaveletMatrixWithSum::new(&[5, 2, 7, 3]);
    /// assert_eq!(5, matrix.get_sum(1..4, 2..=3));
    /// ```
    pub fn new(values: &[usize]) -> Self {
        // 順位探索用のビット列と値の座標圧縮を共有する Wavelet Matrix を先に構築する。
        let wavelet_matrix = WaveletMatrix::new(values);
        // 各レベルの分割順と揃えるため、入力値を圧縮後の順位へ変換する。
        let mut compress = values
            .iter()
            .map(|&x| wavelet_matrix.sorted_v.partition_point(|&y| y < x))
            .collect::<Vec<_>>();
        let mut accum_table = Vec::with_capacity(wavelet_matrix.height);

        // ビットレベルごとに 0 側の値の累積和を作り、次のレベル用に順位列を安定分割する。
        for i in (0..wavelet_matrix.height).rev() {
            // このレベルで 0 側へ進む値だけを元の値で記録する。
            // 1 側を選ぶクエリでは、この和を加えて 0 側全体を飛ばす。
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
            // ビットレベルの順序を Wavelet Matrix と一致させ、次の下位ビットを処理する。
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
    ///
    /// # Returns
    /// 構築時に渡されたシーケンスの長さを返す。
    ///
    /// # Complexity
    /// - 時間計算量: O(1) である。
    /// - 空間計算量: O(1) である。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::ds::wavelet_matrix_with_sum::WaveletMatrixWithSum;
    ///
    /// let matrix = WaveletMatrixWithSum::new(&[4, 2, 4]);
    /// assert_eq!(3, matrix.len());
    /// ```
    pub fn len(&self) -> usize {
        self.wavelet_matrix.len()
    }

    /// 元のシーケンスが空であるかを返す。
    ///
    /// # Returns
    /// 要素数が 0 の場合は `true`、それ以外の場合は `false` を返す。
    ///
    /// # Complexity
    /// - 時間計算量: O(1) である。
    /// - 空間計算量: O(1) である。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::ds::wavelet_matrix_with_sum::WaveletMatrixWithSum;
    ///
    /// assert!(WaveletMatrixWithSum::new(&[]).is_empty());
    /// assert!(!WaveletMatrixWithSum::new(&[1]).is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.wavelet_matrix.is_empty()
    }

    /// 元のシーケンスの `index` 番目の値を返す。
    ///
    /// # Args
    /// - `index`: 0 始まりの要素位置。
    ///
    /// # Returns
    /// 指定位置の値を `Some` で返す。位置が範囲外の場合は `None` を返す。
    ///
    /// # Complexity
    /// - 時間計算量: O(log U) である。$U$ は異なる値の個数である。
    /// - 空間計算量: O(1) である。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::ds::wavelet_matrix_with_sum::WaveletMatrixWithSum;
    ///
    /// let matrix = WaveletMatrixWithSum::new(&[7, 3]);
    /// assert_eq!(Some(7), matrix.get(0));
    /// assert_eq!(None, matrix.get(2));
    /// ```
    pub fn get(&self, index: usize) -> Option<usize> {
        self.wavelet_matrix.get(index)
    }

    /// `index_range` と `value_range` の両方に含まれる要素数を返す。
    ///
    /// # Args
    /// - `index_range`: 調べるインデックス範囲。境界の包含・排他を指定でき、範囲外の端点は
    ///   シーケンス長に丸められる。逆転した範囲は空として扱う。
    /// - `value_range`: 数える値の範囲。包含・排他・無制限の境界を指定できる。
    ///
    /// # Returns
    /// 両方の範囲を満たす要素数を返す。
    ///
    /// # Complexity
    /// - 時間計算量: O(log U) である。$U$ は異なる値の個数である。
    /// - 空間計算量: O(1) である。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::ds::wavelet_matrix_with_sum::WaveletMatrixWithSum;
    ///
    /// let matrix = WaveletMatrixWithSum::new(&[5, 2, 7, 3]);
    /// assert_eq!(2, matrix.count(.., 3..=5));
    /// ```
    pub fn count<I, V>(&self, index_range: I, value_range: V) -> usize
    where
        I: RangeBounds<usize>,
        V: RangeBounds<usize>,
    {
        self.wavelet_matrix.count(index_range, value_range)
    }

    /// `index_range` と `value_range` に含まれる要素を昇順に並べたときの `k` 番目の値を返す。
    ///
    /// # Args
    /// - `index_range`: 調べるインデックス範囲。境界の包含・排他を指定でき、範囲外の端点は
    ///   シーケンス長に丸められる。逆転した範囲は空として扱う。
    /// - `value_range`: 順位付けの対象とする値の範囲。包含・排他・無制限の境界を指定できる。
    /// - `k`: 昇順に並べたときの 0 始まりの順位。
    ///
    /// # Returns
    /// 指定順位の値を `Some` で返す。該当する要素がない場合は `None` を返す。
    ///
    /// # Complexity
    /// - 時間計算量: O(log U) である。$U$ は異なる値の個数である。
    /// - 空間計算量: O(1) である。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::ds::wavelet_matrix_with_sum::WaveletMatrixWithSum;
    ///
    /// let matrix = WaveletMatrixWithSum::new(&[5, 2, 7, 3]);
    /// assert_eq!(Some(3), matrix.get_kth_smallest(.., 2..=7, 1));
    /// ```
    pub fn get_kth_smallest<I, V>(&self, index_range: I, value_range: V, k: usize) -> Option<usize>
    where
        I: RangeBounds<usize>,
        V: RangeBounds<usize>,
    {
        self.wavelet_matrix
            .get_kth_smallest(index_range, value_range, k)
    }

    /// `index_range` と `value_range` に含まれる要素を降順に並べたときの `k` 番目の値を返す。
    ///
    /// # Args
    /// - `index_range`: 調べるインデックス範囲。境界の包含・排他を指定でき、範囲外の端点は
    ///   シーケンス長に丸められる。逆転した範囲は空として扱う。
    /// - `value_range`: 順位付けの対象とする値の範囲。包含・排他・無制限の境界を指定できる。
    /// - `k`: 降順に並べたときの 0 始まりの順位。
    ///
    /// # Returns
    /// 指定順位の値を `Some` で返す。該当する要素がない場合は `None` を返す。
    ///
    /// # Complexity
    /// - 時間計算量: O(log U) である。$U$ は異なる値の個数である。
    /// - 空間計算量: O(1) である。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::ds::wavelet_matrix_with_sum::WaveletMatrixWithSum;
    ///
    /// let matrix = WaveletMatrixWithSum::new(&[5, 2, 7, 3]);
    /// assert_eq!(Some(5), matrix.get_kth_largest(.., 2..=7, 1));
    /// ```
    pub fn get_kth_largest<I, V>(&self, index_range: I, value_range: V, k: usize) -> Option<usize>
    where
        I: RangeBounds<usize>,
        V: RangeBounds<usize>,
    {
        self.wavelet_matrix
            .get_kth_largest(index_range, value_range, k)
    }

    /// `index_range` 内の `upper` 未満の要素の和を返す。
    ///
    /// # Args
    /// - `index_range`: 調べるインデックス範囲。境界の包含・排他を指定でき、範囲外の端点は
    ///   シーケンス長に丸められる。逆転した範囲は空として扱う。
    /// - `upper`: 排他的な上限値。
    ///
    /// # Returns
    /// 範囲内で値が `upper` 未満となる要素の和を返す。
    ///
    /// # Panics
    /// オーバーフローチェックが有効な場合、中間計算が `usize` の範囲を超えるとパニックする。
    /// チェックが無効な場合、オーバーフローした演算はラップする。
    ///
    /// # Complexity
    /// - 時間計算量: O(log U) である。$U$ は異なる値の個数である。
    /// - 空間計算量: O(1) である。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::ds::wavelet_matrix_with_sum::WaveletMatrixWithSum;
    ///
    /// let matrix = WaveletMatrixWithSum::new(&[5, 2, 7, 3]);
    /// assert_eq!(5, matrix.get_sum_less_than(1..4, 5));
    /// ```
    pub fn get_sum_less_than<I>(&self, index_range: I, upper: usize) -> usize
    where
        I: RangeBounds<usize>,
    {
        self.get_sum(index_range, ..upper)
    }

    /// `index_range` 内の `lower` 以上の要素の和を返す。
    ///
    /// # Args
    /// - `index_range`: 調べるインデックス範囲。境界の包含・排他を指定でき、範囲外の端点は
    ///   シーケンス長に丸められる。逆転した範囲は空として扱う。
    /// - `lower`: 包含的な下限値。
    ///
    /// # Returns
    /// 範囲内で値が `lower` 以上となる要素の和を返す。
    ///
    /// # Panics
    /// オーバーフローチェックが有効な場合、中間計算が `usize` の範囲を超えるとパニックする。
    /// チェックが無効な場合、オーバーフローした演算はラップする。
    ///
    /// # Complexity
    /// - 時間計算量: O(log U) である。$U$ は異なる値の個数である。
    /// - 空間計算量: O(1) である。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::ds::wavelet_matrix_with_sum::WaveletMatrixWithSum;
    ///
    /// let matrix = WaveletMatrixWithSum::new(&[5, 2, 7, 3]);
    /// assert_eq!(12, matrix.get_sum_more_than(.., 5));
    /// ```
    pub fn get_sum_more_than<I>(&self, index_range: I, lower: usize) -> usize
    where
        I: RangeBounds<usize>,
    {
        self.get_sum(index_range, lower..)
    }

    /// `index_range` と `value_range` の両方に含まれる要素の和を返す。
    ///
    /// # Args
    /// - `index_range`: 調べるインデックス範囲。境界の包含・排他を指定でき、範囲外の端点は
    ///   シーケンス長に丸められる。逆転した範囲は空として扱う。
    /// - `value_range`: 合計対象とする値の範囲。包含・排他・無制限の境界を指定できる。
    ///
    /// # Returns
    /// 両方の範囲を満たす値の和を返す。
    ///
    /// # Panics
    /// オーバーフローチェックが有効な場合、中間計算が `usize` の範囲を超えるとパニックする。
    /// チェックが無効な場合、オーバーフローした演算はラップする。
    ///
    /// # Complexity
    /// - 時間計算量: O(log U) である。$U$ は異なる値の個数である。
    /// - 空間計算量: O(1) である。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::ds::wavelet_matrix_with_sum::WaveletMatrixWithSum;
    ///
    /// let matrix = WaveletMatrixWithSum::new(&[5, 2, 7, 3]);
    /// assert_eq!(5, matrix.get_sum(1..4, 2..=3));
    /// ```
    pub fn get_sum<I, V>(&self, index_range: I, value_range: V) -> usize
    where
        I: RangeBounds<usize>,
        V: RangeBounds<usize>,
    {
        // インデックス範囲と値範囲を、Wavelet Matrix 内で扱う半開区間へ正規化する。
        let (l, r) = wavelet_matrix_range::normalize_index_range(index_range, self.len());
        let (lower, upper) =
            wavelet_matrix_range::normalize_value_range(value_range, &self.wavelet_matrix.sorted_v);
        if upper <= lower {
            return 0;
        }
        // 上限未満の和から下限未満の和を引き、指定された値範囲だけを残す。
        self.get_sum_less_than_compressed(l, r, upper)
            - self.get_sum_less_than_compressed(l, r, lower)
    }

    /// `index_range` 内の `k` 個の最小要素の和を返す。
    ///
    /// # Args
    /// - `index_range`: 要素を選ぶインデックス範囲。境界の包含・排他を指定でき、範囲外の
    ///   端点はシーケンス長に丸められる。逆転した範囲は空として扱う。
    /// - `k`: 昇順で先頭から合計する要素数。範囲内の要素数以下でなければならない。
    ///
    /// # Returns
    /// 最小の `k` 個の和を `Some` で返す。`k` が範囲内の要素数を超える場合は `None` を返す。
    /// `k == 0` の場合は `Some(0)` を返す。
    ///
    /// # Panics
    /// オーバーフローチェックが有効な場合、中間計算が `usize` の範囲を超えるとパニックする。
    /// チェックが無効な場合、オーバーフローした演算はラップする。
    ///
    /// # Complexity
    /// - 時間計算量: O(log U) である。$U$ は異なる値の個数である。
    /// - 空間計算量: O(1) である。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::ds::wavelet_matrix_with_sum::WaveletMatrixWithSum;
    ///
    /// let matrix = WaveletMatrixWithSum::new(&[5, 2, 7, 3]);
    /// assert_eq!(Some(5), matrix.get_sum_k_smallest(.., 2));
    /// assert_eq!(None, matrix.get_sum_k_smallest(.., 5));
    /// ```
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

        // 各レベルで 0 側の件数と順位を比べ、選択した側に含まれる値の和を集計する。
        let mut result = 0;
        let mut value = 0;
        let mut remaining = k;
        for (i, (bit, sum)) in (0..self.wavelet_matrix.height).rev().zip(
            self.wavelet_matrix
                .bit_table
                .iter()
                .zip(self.accum_table.iter()),
        ) {
            let rank_l = bit.rank(l);
            let rank_r = bit.rank(r);
            let zeros = (r - l) - (rank_r - rank_l);
            if remaining < zeros {
                // 求める k 個が 0 側に収まるため、1 を除いた区間へ移る。
                l -= rank_l;
                r -= rank_r;
            } else {
                // 0 側を全て採用し、その値の和を加えてから残りの順位を 1 側へ移す。
                result += sum.rank(r) - sum.rank(l);
                let zeros_total = bit.len() - bit.rank(bit.len());
                l = rank_l + zeros_total;
                r = rank_r + zeros_total;
                value |= 1_usize << i;
                remaining -= zeros;
            }
        }
        // 全レベルを通過した残りは同じ値なので、値と要素数を掛けて加算する。
        result += remaining * self.wavelet_matrix.sorted_v[value];
        Some(result)
    }

    /// `index_range` 内の `k` 個の最大要素の和を返す。
    ///
    /// # Args
    /// - `index_range`: 要素を選ぶインデックス範囲。境界の包含・排他を指定でき、範囲外の
    ///   端点はシーケンス長に丸められる。逆転した範囲は空として扱う。
    /// - `k`: 降順で先頭から合計する要素数。範囲内の要素数以下でなければならない。
    ///
    /// # Returns
    /// 最大の `k` 個の和を `Some` で返す。`k` が範囲内の要素数を超える場合は `None` を返す。
    /// `k == 0` の場合は `Some(0)` を返す。
    ///
    /// # Panics
    /// オーバーフローチェックが有効な場合、中間計算が `usize` の範囲を超えるとパニックする。
    /// チェックが無効な場合、オーバーフローした演算はラップする。
    ///
    /// # Complexity
    /// - 時間計算量: O(log U) である。$U$ は異なる値の個数である。
    /// - 空間計算量: O(1) である。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::ds::wavelet_matrix_with_sum::WaveletMatrixWithSum;
    ///
    /// let matrix = WaveletMatrixWithSum::new(&[5, 2, 7, 3]);
    /// assert_eq!(Some(12), matrix.get_sum_k_largest(.., 2));
    /// ```
    pub fn get_sum_k_largest<I>(&self, index_range: I, k: usize) -> Option<usize>
    where
        I: RangeBounds<usize> + Clone,
    {
        // 全体の和から、最小の `n - k` 個の和を引くと最大の k 個の和になる。
        let (l, r) = wavelet_matrix_range::normalize_index_range(index_range.clone(), self.len());
        if r - l < k {
            return None;
        }
        Some(self.total_sum(l, r) - self.get_sum_k_smallest(index_range, r - l - k)?)
    }

    /// `index_range` 内の各値 `y` に対する `min(y, x)` の和を返す。
    ///
    /// # Args
    /// - `index_range`: 対象とするインデックス範囲。境界の包含・排他を指定でき、範囲外の
    ///   端点はシーケンス長に丸められる。逆転した範囲は空として扱う。
    /// - `x`: 各値に適用する上限値。
    ///
    /// # Returns
    /// 範囲内の各値 `y` を `min(y, x)` に置き換えた値の和を返す。
    ///
    /// # Panics
    /// オーバーフローチェックが有効な場合、中間計算が `usize` の範囲を超えるとパニックする。
    /// チェックが無効な場合、オーバーフローした演算はラップする。
    ///
    /// # Complexity
    /// - 時間計算量: O(log U) である。$U$ は異なる値の個数である。
    /// - 空間計算量: O(1) である。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::ds::wavelet_matrix_with_sum::WaveletMatrixWithSum;
    ///
    /// let matrix = WaveletMatrixWithSum::new(&[2, 5, 7]);
    /// assert_eq!(10, matrix.get_sum_min(.., 4));
    /// ```
    pub fn get_sum_min<I>(&self, index_range: I, x: usize) -> usize
    where
        I: RangeBounds<usize> + Clone,
    {
        // x 以上の値はすべて x に切り詰め、x 未満の値は元の値のまま合計する。
        self.get_sum_less_than(index_range.clone(), x)
            + x * self.wavelet_matrix.count_more_than(index_range, x)
    }

    /// `index_range` 内の各値 `y` に対する `max(y, x)` の和を返す。
    ///
    /// # Args
    /// - `index_range`: 対象とするインデックス範囲。境界の包含・排他を指定でき、範囲外の
    ///   端点はシーケンス長に丸められる。逆転した範囲は空として扱う。
    /// - `x`: 各値に適用する下限値。
    ///
    /// # Returns
    /// 範囲内の各値 `y` を `max(y, x)` に置き換えた値の和を返す。
    ///
    /// # Panics
    /// オーバーフローチェックが有効な場合、中間計算が `usize` の範囲を超えるとパニックする。
    /// チェックが無効な場合、オーバーフローした演算はラップする。
    ///
    /// # Complexity
    /// - 時間計算量: O(log U) である。$U$ は異なる値の個数である。
    /// - 空間計算量: O(1) である。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::ds::wavelet_matrix_with_sum::WaveletMatrixWithSum;
    ///
    /// let matrix = WaveletMatrixWithSum::new(&[2, 5, 7]);
    /// assert_eq!(16, matrix.get_sum_max(.., 4));
    /// ```
    pub fn get_sum_max<I>(&self, index_range: I, x: usize) -> usize
    where
        I: RangeBounds<usize> + Clone,
    {
        // x 未満の値を x に引き上げ、x 以上の値は元の値のまま合計する。
        x * self.wavelet_matrix.count_less_than(index_range.clone(), x)
            + self.get_sum_more_than(index_range, x)
    }

    /// `index_range` 内の各値 `y` と `x` の絶対差の和を返す。
    ///
    /// # Args
    /// - `index_range`: 対象とするインデックス範囲。境界の包含・排他を指定でき、範囲外の
    ///   端点はシーケンス長に丸められる。逆転した範囲は空として扱う。
    /// - `x`: 絶対差の基準値。
    ///
    /// # Returns
    /// 範囲内の各値 `y` について `|y - x|` を合計した値を返す。
    ///
    /// # Panics
    /// オーバーフローチェックが有効な場合、中間計算が `usize` の範囲を超えるとパニックする。
    /// チェックが無効な場合、オーバーフローした演算はラップする。
    ///
    /// # Complexity
    /// - 時間計算量: O(log U) である。$U$ は異なる値の個数である。
    /// - 空間計算量: O(1) である。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::ds::wavelet_matrix_with_sum::WaveletMatrixWithSum;
    ///
    /// let matrix = WaveletMatrixWithSum::new(&[2, 5, 7]);
    /// assert_eq!(5, matrix.get_sum_abs_diff(.., 5));
    /// ```
    pub fn get_sum_abs_diff<I>(&self, index_range: I, x: usize) -> usize
    where
        I: RangeBounds<usize> + Clone,
    {
        // x 未満と x より大きい値を分け、それぞれの差の総和を計算する。
        let less_count = self.wavelet_matrix.count_less_than(index_range.clone(), x);
        let more_count = self.wavelet_matrix.count_more_than(index_range.clone(), x);
        let less_sum = self.get_sum_less_than(index_range.clone(), x);
        let more_sum = self.get_sum_more_than(index_range, x);
        (x * less_count - less_sum) + (more_sum - x * more_count)
    }

    /// `index_range` 内の各値を `value_range` 内へ収めるための距離の和を返す。
    ///
    /// 値範囲内の値への距離を 0 とし、範囲外の値は最も近い境界までの距離を合計する。
    /// `value_range` が空の場合は `0` を返す。
    ///
    /// # Args
    /// - `index_range`: 対象とするインデックス範囲。境界の包含・排他を指定でき、範囲外の
    ///   端点はシーケンス長に丸められる。逆転した範囲は空として扱う。
    /// - `value_range`: 値を収める先の範囲。包含・排他・無制限の境界を指定できる。
    ///
    /// # Returns
    /// 各値から `value_range` までの距離の総和を返す。値範囲が空の場合は `0` を返す。
    ///
    /// # Panics
    /// オーバーフローチェックが有効な場合、中間計算が `usize` の範囲を超えるとパニックする。
    /// チェックが無効な場合、オーバーフローした演算はラップする。
    ///
    /// # Complexity
    /// - 時間計算量: O(log U) である。$U$ は異なる値の個数である。
    /// - 空間計算量: O(1) である。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::ds::wavelet_matrix_with_sum::WaveletMatrixWithSum;
    ///
    /// let matrix = WaveletMatrixWithSum::new(&[0, 4, 8]);
    /// assert_eq!(6, matrix.get_sum_distance_to_range(.., 3..=5));
    /// ```
    pub fn get_sum_distance_to_range<I, V>(&self, index_range: I, value_range: V) -> usize
    where
        I: RangeBounds<usize> + Clone,
        V: RangeBounds<usize> + Clone,
    {
        // 境界の開閉を整数の包含区間へ変換する。端点の加減算では飽和演算を使い、
        // usize の最小値・最大値でオーバーフローしないようにする。
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
            // 空の値範囲には距離が定義されないため、全要素の寄与を 0 とする。
            return 0;
        }

        // 下限未満と上限超過の要素だけを取り出し、範囲端までの距離を合計する。
        let (l, r) = wavelet_matrix_range::normalize_index_range(index_range.clone(), self.len());
        let below_count = self.wavelet_matrix.count_less_than(l..r, lower);
        let below_sum = self.get_sum_less_than(l..r, lower);
        let total_sum = self.total_sum(l, r);
        let within_sum = self.get_sum(l..r, ..=upper);
        let above_count = r - l - self.wavelet_matrix.count(l..r, ..=upper);
        let above_sum = total_sum - within_sum;
        lower * below_count - below_sum + above_sum - upper * above_count
    }

    /// 正規化済みインデックス範囲 `[l, r)` の全要素の和を返す。
    ///
    /// # Args
    /// - `l`: 範囲の開始位置。
    /// - `r`: 範囲の終了位置。`l <= r <= self.len()` を満たす。
    ///
    /// # Returns
    /// `[l, r)` に含まれる値の和を返す。
    ///
    /// # Complexity
    /// - 時間計算量: O(1) である。
    /// - 空間計算量: O(1) である。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::ds::wavelet_matrix_with_sum::WaveletMatrixWithSum;
    ///
    /// let matrix = WaveletMatrixWithSum::new(&[2, 3, 5]);
    /// assert_eq!(5, matrix.get_sum(0..2, ..));
    /// ```
    fn total_sum(&self, l: usize, r: usize) -> usize {
        // 半開区間の和は、右端と左端の累積和の差で得られる。
        self.accum_v.rank(r) - self.accum_v.rank(l)
    }

    /// 有効なインデックス範囲で、圧縮順位が `upper` 未満となる要素の和を返す。
    ///
    /// # Args
    /// - `l`: 対象となる半開インデックス範囲の開始位置。
    /// - `r`: 対象となる半開インデックス範囲の終了位置。`l <= r <= self.len()` を満たす。
    /// - `upper`: 圧縮順位の排他的上限。
    ///
    /// # Returns
    /// `[l, r)` 内で圧縮順位が `upper` 未満となる要素の和を返す。
    ///
    /// # Complexity
    /// - 時間計算量: O(log U) である。$U$ は異なる値の個数である。
    /// - 空間計算量: O(1) である。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::ds::wavelet_matrix_with_sum::WaveletMatrixWithSum;
    ///
    /// let matrix = WaveletMatrixWithSum::new(&[2, 5, 3]);
    /// assert_eq!(5, matrix.get_sum_less_than(0..3, 5));
    /// ```
    fn get_sum_less_than_compressed(&self, mut l: usize, mut r: usize, upper: usize) -> usize {
        if r <= l {
            return 0;
        }

        let mut result = 0;
        // 上限順位のビットを上位から追い、上限未満と確定した 0 側の和を加える。
        for (i, (bit, sum)) in (0..self.wavelet_matrix.height).rev().zip(
            self.wavelet_matrix
                .bit_table
                .iter()
                .zip(self.accum_table.iter()),
        ) {
            let rank_l = bit.rank(l);
            let rank_r = bit.rank(r);
            if (upper >> i) & 1 == 0 {
                // 上限ビットが 0 のときは 0 側だけが上限未満になり得る。
                l -= rank_l;
                r -= rank_r;
            } else {
                // 上限ビットが 1 のときは 0 側全体を加算して、上限と同じ 1 側を続ける。
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

        /// Scenario: 空のシーケンスに対する和クエリを処理する。
        /// - Given: 空の `WaveletMatrixWithSum` がある。
        /// - When: 空の範囲と 0 個の順位和を求める。
        /// - Then: 空の和として `0` または `Some(0)` を返す。
        #[test]
        fn handles_empty_sequence() {
            // Given
            let sut = WaveletMatrixWithSum::new(&[]);
            // When and Then
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

        /// Scenario: 値範囲、順位、距離に関する和を返す。
        /// - Given: 重複値を含む `WaveletMatrixWithSum` がある。
        /// - When: 各種の和クエリを求める。
        /// - Then: それぞれの定義に従った和を返す。
        #[test]
        fn returns_sums_for_value_queries() {
            // Given
            let sut = create_wavelet_matrix();
            // When and Then
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

        /// Scenario: 要素数を超える個数の順位和を求める。
        /// - Given: 要素数 8 の `WaveletMatrixWithSum` がある。
        /// - When: 9 個の順位和を求める。
        /// - Then: `None` を返す。
        #[test]
        fn returns_none_when_k_exceeds_length() {
            // Given
            let sut = create_wavelet_matrix();
            // When and Then
            assert_eq!(None, sut.get_sum_k_smallest(.., 9));
            assert_eq!(None, sut.get_sum_k_largest(.., 9));
        }

        /// Scenario: 要素数が十分に大きい入力を構築して和クエリを実行する。
        /// - Given: 100,000 個の異なる値を持つシーケンスがある。
        /// - When: `WaveletMatrixWithSum` を構築し、全体和と順位和を求める。
        /// - Then: 現実的な時間で完了し、結果が正しい。
        #[test]
        fn builds_and_sums_large_input() {
            // Given
            let data = (0..100_000).collect::<Vec<_>>();
            let expected_total = 100_000_usize * 99_999 / 2;
            // When
            let sut = WaveletMatrixWithSum::new(&data);
            // Then
            assert_eq!(expected_total, sut.get_sum(.., ..));
            assert_eq!(10 * 11 / 2, sut.get_sum_k_smallest(.., 11).unwrap());
            assert_eq!(99_989 * 11 + 55, sut.get_sum_k_largest(.., 11).unwrap());
        }
    }
}
