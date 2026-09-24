use std::ops::{Range, RangeBounds};

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
    /// `usize` のスライスから `WaveletMatrix` を構築する。
    ///
    /// 入力値を座標圧縮し、圧縮値の各ビットを上位から順に並べたビット列を構築する。
    ///
    /// # Args
    /// - `v`: 元の順序を保ったまま格納する値のスライス。
    ///
    /// # Returns
    /// `v` の要素を格納した `WaveletMatrix` を返す。
    ///
    /// # Complexity
    /// - 時間計算量: $O(N \log U)$ である。$N$ は `v` の長さ、$U$ は異なる値の個数である。
    /// - 空間計算量: $O(N \log U)$ である。各ビットレベルに `N` 個のビットを保持する。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::ds::wavelet_matrix::WaveletMatrix;
    ///
    /// let values = [10, 5, 20, 15, 5, 10, 25];
    /// let matrix = WaveletMatrix::new(&values);
    /// assert_eq!(4, matrix.count_less_than(0..7, 15));
    /// assert_eq!(3, matrix.count_more_than(0..4, 10));
    /// assert_eq!(3, matrix.count(1..6, 5..15));
    /// ```
    pub fn new(v: &[usize]) -> Self {
        // 値を昇順に並べて重複を除き、元の値と圧縮後の順位を対応付ける。
        let mut sorted_v = v.to_vec();
        sorted_v.sort_unstable();
        sorted_v.dedup();

        // 順位を各ビットレベルで扱えるよう、各値をソート済み配列上の添字に変換する。
        let mut compress = v
            .iter()
            .map(|&x| sorted_v.partition_point(|&y| y < x))
            .collect::<Vec<_>>();
        // 全ての圧縮順位を表現できるビット数を使う。空入力でも構築処理を一貫させるため、
        // その場合は 1 レベルを設ける。
        let height = if sorted_v.is_empty() {
            1
        } else {
            usize::BITS as usize - sorted_v.len().leading_zeros() as usize
        };
        // 各レベルのビット列と、0 側の要素数を上位ビットから順に保存する。
        let mut bit_table = Vec::with_capacity(height);
        let mut zero_counts = Vec::with_capacity(height);
        // 安定分割の出力を保持し、次のレベルでも再利用するバッファ。
        let mut next = vec![0_usize; compress.len()];

        // 上位ビットから順に、ビット列の作成と 0 側・1 側への安定分割を行う。
        for i in (0..height).rev() {
            // 1 側の開始位置を決めるため、このレベルで 0 となる要素数を数える。
            let num_zeros = compress.iter().filter(|&&x| ((x >> i) & 1) == 0).count();
            // ビットをワード単位に詰め、rank クエリ用の領域を確保する。
            let mut words = vec![0_u64; compress.len() / u64::BITS as usize + 1];
            // 0 と 1 の要素はそれぞれの領域内で入力順を保って配置する。
            let mut zero_pos = 0_usize;
            let mut one_pos = num_zeros;
            // 現在処理している 64 ビットワードと、その保存先を追跡する。
            let mut word = 0_u64;
            let mut word_index = 0_usize;
            for (pos, &x) in compress.iter().enumerate() {
                // 0 の順位は前方へ、1 の順位は 0 の領域の後方へ書き込み、
                // 同時に元の並びで 1 だった位置をビット列へ記録する。
                if ((x >> i) & 1) == 0 {
                    next[zero_pos] = x;
                    zero_pos += 1;
                } else {
                    next[one_pos] = x;
                    one_pos += 1;
                    word |= 1_u64 << (pos & 63);
                }
                // ワードが 64 ビット埋まったら保存して、次のワードを初期化する。
                if (pos & 63) == 63 {
                    words[word_index] = word;
                    word_index += 1;
                    word = 0;
                }
            }
            // 最後のワードが 64 ビット未満の場合も、残ったビットを保存する。
            if (compress.len() & 63) != 0 {
                words[word_index] = word;
            }
            // 次のレベルで各要素を正しい区間へ写せるよう、0 の総数とビット列を保存する。
            zero_counts.push(num_zeros);
            bit_table.push(bit_vector::BitVector::from_words(words, compress.len()));
            // 今作成した安定分割済み配列を次の下位ビットの入力にする。
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

    /// 元のシーケンスに含まれる要素数を返す。
    ///
    /// # Returns
    /// 構築時に渡されたスライスの長さを返す。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::ds::wavelet_matrix::WaveletMatrix;
    ///
    /// let matrix = WaveletMatrix::new(&[4, 2, 4]);
    /// assert_eq!(3, matrix.len());
    /// ```
    pub fn len(&self) -> usize {
        self.len
    }

    /// 元のシーケンスに要素が含まれないかを返す。
    ///
    /// # Returns
    /// 要素数が 0 の場合は `true`、それ以外の場合は `false` を返す。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::ds::wavelet_matrix::WaveletMatrix;
    ///
    /// let empty = WaveletMatrix::new(&[]);
    /// let non_empty = WaveletMatrix::new(&[1]);
    /// assert!(empty.is_empty());
    /// assert!(!non_empty.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// 元のシーケンスの `index` 番目の値を返す。
    ///
    /// 範囲外のインデックスに対しては `None` を返す。
    ///
    /// # Args
    /// - `index`: 0 始まりの要素位置。
    ///
    /// # Returns
    /// 指定位置の値を `Some` で返す。位置が範囲外の場合は `None` を返す。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::ds::wavelet_matrix::WaveletMatrix;
    ///
    /// let matrix = WaveletMatrix::new(&[7, 3]);
    /// assert_eq!(Some(7), matrix.get(0));
    /// assert_eq!(None, matrix.get(2));
    /// ```
    pub fn get(&self, index: usize) -> Option<usize> {
        if index >= self.len {
            return None;
        }

        // 各レベルで位置を 0 側または 1 側へ写しながら、圧縮値のビットを復元する。
        let mut position = index;
        let mut compressed_value = 0;
        for (level, (i, bit)) in (0..self.height)
            .rev()
            .zip(self.bit_table.iter())
            .enumerate()
        {
            // 区間 `[position, position + 1)` の rank 差から、対象要素の現在ビットを判定する。
            let rank = bit.rank(position);
            let next_rank = bit.rank(position + 1);
            let is_one = next_rank != rank;
            let zeros = self.zero_counts[level];
            if is_one {
                // 1 側は全体の 0 の領域の後ろにあるため、0 の総数を加えて位置を移す。
                compressed_value |= 1_usize << i;
                position = rank + zeros;
            } else {
                // 0 側では、それより前にある 1 を除いた位置が新しい位置になる。
                position -= rank;
            }
        }

        Some(self.sorted_v[compressed_value])
    }

    /// 値の圧縮インデックスが `upper` 未満となる要素の個数を返す。
    fn count_less_than_compressed(&self, mut l: usize, mut r: usize, upper: usize) -> usize {
        // 空区間や圧縮順位 0 未満には該当要素がない。
        if r <= l || upper == 0 {
            return 0;
        }
        // 圧縮値はつねに `sorted_v.len()` 未満なので、上限が種類数以上なら
        // 区間内の全要素が条件を満たす。
        if upper >= self.sorted_v.len() {
            return r - l;
        }

        let mut result = 0;
        // 上限の各ビットを上位からたどり、上限より小さい側へ確定した 0 の個数を加算する。
        for (level, (i, bit)) in (0..self.height)
            .rev()
            .zip(self.bit_table.iter())
            .enumerate()
        {
            let rank_l = bit.rank(l);
            let rank_r = bit.rank(r);
            if (upper >> i) & 1 == 0 {
                // 上限ビットが 0 なら 1 側は上限以上なので、0 側だけを続けて調べる。
                l -= rank_l;
                r -= rank_r;
            } else {
                // 上限ビットが 1 なら現在範囲の 0 側は全て上限未満なので、その個数を足す。
                result += (r - l) - (rank_r - rank_l);
                let zeros = self.zero_counts[level];
                // 上限と同じ 1 側へ進み、残りの下位ビットを比較する。
                l = rank_l + zeros;
                r = rank_r + zeros;
            }
        }
        result
    }

    /// 共通する上位ビットの走査を共有し、圧縮値が `[lower, upper)` に含まれる要素数を返す。
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
        // 下限と上限のビット列が分岐するまでは共通の区間を走査する。
        let mut diverged = false;

        for (level, (i, bit)) in (0..self.height)
            .rev()
            .zip(self.bit_table.iter())
            .enumerate()
        {
            if !diverged && ((lower >> i) & 1) == ((upper >> i) & 1) {
                // 共通するビットでは区間を一度だけ写し、両境界の累積数を同じだけ更新する。
                let rank_l = bit.rank(l);
                let rank_r = bit.rank(r);
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
                // 最初に境界ビットが異なる位置で、下限と上限の探索区間を分離する。
                diverged = true;
                lower_l = l;
                lower_r = r;
                upper_l = l;
                upper_r = r;
            }

            let lower_rank_l = bit.rank(lower_l);
            let lower_rank_r = bit.rank(lower_r);
            if (lower >> i) & 1 == 0 {
                // 下限が 0 なら下限未満の値を増やさず、0 側へ進む。
                lower_l -= lower_rank_l;
                lower_r -= lower_rank_r;
            } else {
                // 下限が 1 なら、0 側にある値はすべて下限未満として数える。
                lower_result += (lower_r - lower_l) - (lower_rank_r - lower_rank_l);
                let zeros_total = self.zero_counts[level];
                lower_l = lower_rank_l + zeros_total;
                lower_r = lower_rank_r + zeros_total;
            }

            let upper_rank_l = bit.rank(upper_l);
            let upper_rank_r = bit.rank(upper_r);
            if (upper >> i) & 1 == 0 {
                // 上限が 0 なら上限未満の値を増やさず、0 側へ進む。
                upper_l -= upper_rank_l;
                upper_r -= upper_rank_r;
            } else {
                // 上限が 1 なら、0 側にある値はすべて上限未満として数える。
                upper_result += (upper_r - upper_l) - (upper_rank_r - upper_rank_l);
                let zeros_total = self.zero_counts[level];
                upper_l = upper_rank_l + zeros_total;
                upper_r = upper_rank_r + zeros_total;
            }
        }

        // 上限未満の個数から下限未満の個数を引き、半開値範囲内の個数を得る。
        upper_result - lower_result
    }

    /// `index_range` に含まれる `upper` 未満の要素数を返す。
    ///
    /// # Args
    /// - `index_range`: 要素を調べるインデックス範囲。
    /// - `upper`: 比較に用いる排他的な上限値。
    ///
    /// # Returns
    /// 範囲内で値が `upper` 未満となる要素の個数を返す。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::ds::wavelet_matrix::WaveletMatrix;
    ///
    /// let matrix = WaveletMatrix::new(&[5, 2, 7, 3]);
    /// assert_eq!(2, matrix.count_less_than(1..4, 5));
    /// ```
    pub fn count_less_than<I>(&self, index_range: I, upper: usize) -> usize
    where
        I: RangeBounds<usize>,
    {
        let (l, r) = wavelet_matrix_range::normalize_index_range(index_range, self.len);
        let upper = self.sorted_v.partition_point(|&x| x < upper);
        self.count_less_than_compressed(l, r, upper)
    }

    /// `index_range` に含まれる `lower` 以上の要素数を返す。
    ///
    /// # Args
    /// - `index_range`: 要素を調べるインデックス範囲。
    /// - `lower`: 比較に用いる包含的な下限値。
    ///
    /// # Returns
    /// 範囲内で値が `lower` 以上となる要素の個数を返す。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::ds::wavelet_matrix::WaveletMatrix;
    ///
    /// let matrix = WaveletMatrix::new(&[5, 2, 7, 3]);
    /// assert_eq!(1, matrix.count_more_than(1..4, 5));
    /// ```
    pub fn count_more_than<I>(&self, index_range: I, lower: usize) -> usize
    where
        I: RangeBounds<usize>,
    {
        let (l, r) = wavelet_matrix_range::normalize_index_range(index_range, self.len);
        let lower = self.sorted_v.partition_point(|&x| x < lower);
        (r - l) - self.count_less_than_compressed(l, r, lower)
    }

    /// インデックス範囲と値範囲の両方に含まれる要素数を返す。
    ///
    /// # Args
    /// - `index_range`: 要素を調べるインデックス範囲。
    /// - `value_range`: 数える値の範囲。
    ///
    /// # Returns
    /// 両方の範囲を満たす要素の個数を返す。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::ds::wavelet_matrix::WaveletMatrix;
    ///
    /// let matrix = WaveletMatrix::new(&[5, 2, 7, 3]);
    /// assert_eq!(2, matrix.count(.., 3..=5));
    /// ```
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
    ///
    /// # Args
    /// - `index_range`: 要素を調べるインデックス範囲。
    /// - `value_range`: 順位付けの対象とする値の範囲。
    /// - `k`: 昇順に並べたときの 0 始まりの順位。
    ///
    /// # Returns
    /// 該当する順位の値を `Some` で返す。該当する要素がない場合は `None` を返す。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::ds::wavelet_matrix::WaveletMatrix;
    ///
    /// let matrix = WaveletMatrix::new(&[5, 2, 7, 3]);
    /// assert_eq!(Some(3), matrix.get_kth_smallest(.., 2..=7, 1));
    /// ```
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
    ///
    /// # Args
    /// - `index_range`: 要素を調べるインデックス範囲。
    /// - `value_range`: 順位付けの対象とする値の範囲。
    /// - `k`: 降順に並べたときの 0 始まりの順位。
    ///
    /// # Returns
    /// 該当する順位の値を `Some` で返す。該当する要素がない場合は `None` を返す。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::ds::wavelet_matrix::WaveletMatrix;
    ///
    /// let matrix = WaveletMatrix::new(&[5, 2, 7, 3]);
    /// assert_eq!(Some(5), matrix.get_kth_largest(.., 2..=7, 1));
    /// ```
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
        // 各レベルで 0 側の要素数と順位 `k` を比べ、目的の部分木を選び続ける。
        for (level, (i, bit)) in (0..self.height)
            .rev()
            .zip(self.bit_table.iter())
            .enumerate()
        {
            let rank_l = bit.rank(l);
            let rank_r = bit.rank(r);
            let zeros = (r - l) - (rank_r - rank_l);
            if k < zeros {
                // k 番目が 0 側にあるため、1 の累積数を除いて 0 側の範囲へ移る。
                l -= rank_l;
                r -= rank_r;
            } else {
                // 0 側を飛ばして 1 側へ進み、順位から 0 側の要素数を差し引く。
                let zeros_total = self.zero_counts[level];
                l = rank_l + zeros_total;
                r = rank_r + zeros_total;
                compressed_value |= 1_usize << i;
                k -= zeros;
            }
        }
        Some(self.sorted_v[compressed_value])
    }

    /// 複数の半開区間それぞれの `k` 番目に小さい値をまとめて返す。
    ///
    /// 各クエリは `(index_range, k)` の組であり、`index_range` 内の `k` 番目
    /// （0 始まり）に小さい値を返す。値域は全体とする。クエリを16件ずつ
    /// まとめて各レベルを処理することで、ビット列のキャッシュ再利用を高める。
    ///
    /// # Args
    /// - `queries`: 半開インデックス範囲と、その範囲内で求める 0 始まりの順位の組。
    ///
    /// # Returns
    /// 入力順に各クエリの順位要素を格納したベクターを返す。
    ///
    /// # Panics
    /// 範囲外のインデックスや、区間長以上の `k` を指定した場合にパニックする。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::ds::wavelet_matrix::WaveletMatrix;
    ///
    /// let matrix = WaveletMatrix::new(&[5, 2, 7, 3]);
    /// let queries = [(0..4, 0), (1..4, 1)];
    /// assert_eq!(vec![2, 3], matrix.get_kth_smallest_batch(&queries));
    /// ```
    pub fn get_kth_smallest_batch(&self, queries: &[(Range<usize>, usize)]) -> Vec<usize> {
        const CHUNK: usize = 16;
        let mut answers = Vec::with_capacity(queries.len());
        // 各チャンク内のクエリ状態 `[l, r, k, 圧縮値]` を固定長配列でまとめて処理する。
        for chunk in queries.chunks(CHUNK) {
            let mut states = [[0_usize; 4]; CHUNK];
            // クエリ範囲と順位を状態へ格納し、指定条件を満たさない入力を検証する。
            for (state, (range, k)) in states.iter_mut().zip(chunk.iter()) {
                assert!(range.start <= range.end && range.end <= self.len);
                assert!(*k < range.end - range.start);
                state[0] = range.start;
                state[1] = range.end;
                state[2] = *k;
            }
            // 同じレベルのビット列を連続して参照し、全クエリを並行して順位探索する。
            for (level, (i, bit)) in (0..self.height)
                .rev()
                .zip(self.bit_table.iter())
                .enumerate()
            {
                let zeros_total = self.zero_counts[level];
                for state in states[..chunk.len()].iter_mut() {
                    // 0 側の要素数を数え、順位が含まれる側へ区間を写す。
                    let rank_l = bit.rank(state[0]);
                    let rank_r = bit.rank(state[1]);
                    let zeros = (state[1] - state[0]) - (rank_r - rank_l);
                    if state[2] < zeros {
                        state[0] -= rank_l;
                        state[1] -= rank_r;
                    } else {
                        state[0] = rank_l + zeros_total;
                        state[1] = rank_r + zeros_total;
                        state[3] |= 1_usize << i;
                        state[2] -= zeros;
                    }
                }
            }
            // 復元した圧縮順位を元の値へ戻し、入力と同じ順序で結果へ追加する。
            answers.extend(
                states[..chunk.len()]
                    .iter()
                    .map(|state| self.sorted_v[state[3]]),
            );
        }
        answers
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 背景: 重複値を含むシーケンスから構築した `WaveletMatrix`。
    fn create_wavelet_matrix() -> WaveletMatrix {
        WaveletMatrix::new(&[5, 4, 8, 6, 0, 7, 2, 5])
    }

    /// 背景: `usize::MAX` を含むシーケンスから構築した `WaveletMatrix`。
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
            // When and Then
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
            // When and Then
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
            // When and Then
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
            // When and Then
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
            // When and Then
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
            // When and Then
            assert_eq!(None, sut.get_kth_smallest(.., 8..8, 0));
            assert_eq!(None, sut.get_kth_largest(.., .., 8));
        }

        /// Scenario: 一括クエリが単発クエリと一致する。
        /// - Given: 重複値を含む `WaveletMatrix` がある。
        /// - When: チャンク境界をまたぐ件数の一括クエリを実行する。
        /// - Then: 単発の `get_kth_smallest` と同じ値を返す。
        #[test]
        fn batch_matches_single_queries() {
            // Given
            let sut = create_wavelet_matrix();
            let queries = (0..20)
                .map(|t| {
                    let l = t % 8;
                    let len = 1 + (t % (8 - l));
                    (l..l + len, t % len)
                })
                .collect::<Vec<_>>();
            // When
            let results = sut.get_kth_smallest_batch(&queries);
            // Then
            assert_eq!(20, results.len());
            for ((range, k), answer) in queries.iter().zip(results.iter()) {
                assert_eq!(sut.get_kth_smallest(range.clone(), .., *k), Some(*answer));
            }
            assert!(sut.get_kth_smallest_batch(&[]).is_empty());
        }
    }
}
