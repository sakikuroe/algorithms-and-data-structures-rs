/// `u64` の下位 `k` ビットを効率的に抽出するための事前計算されたマスクである.
const MASKS: [u64; u64::BITS as usize] = {
    let mut masks = [0_u64; u64::BITS as usize];
    let mut k = 1_usize;
    // k が 1 から 63 までのマスクを生成する。
    while k < u64::BITS as usize {
        masks[k] = (1_u64 << k) - 1;
        k += 1;
    }
    masks
};

/// 0 と 1 からなるビット列を効率的に格納し, クエリを実行するためのデータ構造である.
///
/// `BitVector` は 64 ビットのブロックごとに 1 の累積和を事前計算することで,
/// 高速な `sum` クエリを可能にする.
#[derive(Clone)]
pub struct BitVector {
    bits: Vec<u64>,

    // 各ブロックの末尾までに含まれる 1 の累積数を保持する。
    cumulative_sums: Vec<u32>,

    // ビット列の長さを保持する。
    len: usize,
}

impl BitVector {
    /// 新しい `BitVector` を作成する.
    ///
    /// # Args
    ///
    /// - `v`: 各要素が `0` または `1` である `u8` のスライスである.
    ///   `v` の長さは `2^{32}` (=4294967296) 未満でなければならない.
    ///
    /// # Returns
    ///
    /// 新しい `BitVector` インスタンスを返す.
    ///
    /// # Panics
    ///
    /// `v` のいずれかの要素が `0` または `1` でない場合にパニックする.
    /// `v` の長さが `2^{32}` 以上の場合にパニックする.
    ///
    /// # Complexity
    ///
    /// - 時間計算量: O(N) である. ここで N は `v` の長さである.
    /// - 空間計算量: O(N) である. ここで N は `v` の長さである.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use anmitsu::ds::bit_vector;
    ///
    /// let bv = bit_vector::BitVector::new(&[1, 0, 1, 1, 0, 1]);
    /// assert_eq!(6, bv.len());
    /// assert_eq!(3, bv.rank(4)); // [1, 0, 1, 1] の和
    /// assert_eq!(4, bv.rank(6)); // [1, 0, 1, 1, 0, 1] の和
    /// assert_eq!(0, bv.rank(0)); // 空列の和
    /// ```
    pub fn new(v: &[u8]) -> Self {
        let len = v.len();
        assert!(
            len < (1 << u32::BITS as usize),
            "Length of v must be less than 2^{{32}}."
        );

        let num_blocks = len / u64::BITS as usize + 1;
        let mut bits = vec![0_u64; num_blocks];
        let mut cumulative_sums = vec![0_u32; num_blocks];
        let mut current_sum = 0_u32;

        // 入力を走査してビット列を構築し、各要素が 0 または 1 であることを検証する。
        for (i, &bit_val) in v.iter().enumerate() {
            // 後続の処理が二値であることを前提とするため、要素が有効なビット値か確認する。
            if bit_val != 0 && bit_val != 1 {
                panic!("Input slice `v` must only contain 0 or 1.");
            }
            // 値が 1 のとき、対応する位置のビットを `u64` のブロックに設定する。
            if bit_val == 1 {
                let block_index = i / u64::BITS as usize;
                let bit_in_block = i % u64::BITS as usize;
                bits[block_index] |= 1_u64 << bit_in_block;
            }
        }

        // 各ブロックより前にある 1 の個数を累積和として記録する。
        for i in 0..num_blocks {
            cumulative_sums[i] = current_sum;
            current_sum += bits[i].count_ones();
        }

        BitVector {
            bits,
            cumulative_sums,
            len,
        }
    }

    /// 範囲 `v[0..r)` における `1` の数 ( `v[0..r)` の和) を返す.
    ///
    /// # Args
    ///
    /// - `r`: 範囲の上限である. `r` は `len()` 以下でなければならない.
    ///
    /// # Returns
    ///
    /// `v[0..r)` における `1` の合計を返す. `r` が `0` の場合は `0` を返す.
    ///
    /// # Panics
    ///
    /// `r > len()` の場合にパニックする.
    ///
    /// # Complexity
    ///
    /// - 時間計算量: 事前計算により O(1) である.
    /// - 空間計算量: クエリ自体は O(1) である.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use anmitsu::ds::bit_vector;
    ///
    /// let bv = bit_vector::BitVector::new(&[1, 0, 1, 1, 0, 1, 0, 0]);
    /// assert_eq!(bv.rank(0), 0);
    /// assert_eq!(bv.rank(1), 1); // v[0] = 1
    /// assert_eq!(bv.rank(3), 2); // v[0..3] = [1, 0, 1] の和は 2
    /// assert_eq!(bv.rank(6), 4); // v[0..6] = [1, 0, 1, 1, 0, 1] の和は 4
    /// assert_eq!(bv.rank(8), 4); // v[0..8] = [1, 0, 1, 1, 0, 1, 0, 0] の和は 4
    /// ```
    pub fn rank(&self, r: usize) -> usize {
        if r == 0 {
            return 0;
        }

        if r > self.len {
            panic!(
                "r ({}) cannot be greater than the length of the BitVector ({}).",
                r, self.len
            );
        }

        // 事前計算した累積和とビット列を効率よく参照するため、対象ブロックを求める。
        let block_index = r / u64::BITS as usize;

        let mut res = self.cumulative_sums[block_index];
        // `MASKS` で対象範囲のビットを取り出し、ブロック内にある 1 の個数を加算する。
        res += (self.bits[block_index] & MASKS[r % u64::BITS as usize]).count_ones();
        res as usize
    }

    /// 64 ビット単位にパックされたワードから `BitVector` を作成する。
    ///
    /// `words` は列のビットをリトルエンディアン順に保持する。つまり、要素 `k` は
    /// ワード `k / 64` のビット `k % 64` に対応する。必要なワード数に満たない場合は、
    /// 不足分を 0 で補う。
    /// `new` が `0` と `1` を要素ごとに受け取ってワードへ詰めるのに対し、このメソッドは
    /// 既にワードへパックされたビット列を受け取る。Wavelet Matrix の構築時に使用する。
    ///
    /// # Args
    /// - `words`: ビット列を 64 ビット単位で格納したワード列。
    /// - `len`: 作成するビット列の長さ。`2^32` 未満でなければならない。
    ///
    /// # Returns
    /// `words` から作成した `BitVector` を返す。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::ds::bit_vector;
    ///
    /// let bits = bit_vector::BitVector::new(&[1, 0, 1, 0]);
    /// assert_eq!(2, bits.rank(4));
    /// ```
    pub(super) fn from_words(mut words: Vec<u64>, len: usize) -> Self {
        debug_assert!(len < (1 << u32::BITS as usize));
        // rank で末尾境界を参照できるよう、必要なブロック数までワード列を揃える。
        words.resize(len / u64::BITS as usize + 1, 0);
        // 各ブロックの開始位置より前にある 1 の個数を保存する。
        let mut cumulative_sums = vec![0_u32; words.len()];
        let mut acc = 0_u32;
        for (i, &word) in words.iter().enumerate() {
            cumulative_sums[i] = acc;
            // 次のブロックの開始時点で使うため、現在のワードの 1 の個数を累積する。
            acc += word.count_ones();
        }
        Self {
            bits: words,
            cumulative_sums,
            len,
        }
    }

    /// 2 つの位置までに含まれる `1` の累積数を返す。
    ///
    /// `l` と `r` は半開区間 `[l, r)` の端点であり、それぞれの位置までに含まれる
    /// `1` の数を返す。このメソッドは Wavelet Matrix の範囲クエリから使用する。
    ///
    /// # Args
    /// - `l`: 累積数を求める左端の位置。`r` 以下でなければならない。
    /// - `r`: 累積数を求める右端の位置。ビット列の長さ以下でなければならない。
    ///
    /// # Returns
    /// `[0, l)` と `[0, r)` に含まれる `1` の数を順に返す。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::ds::bit_vector;
    ///
    /// let bits = bit_vector::BitVector::new(&[1, 0, 1, 1]);
    /// assert_eq!(2, bits.rank(3));
    /// assert_eq!(3, bits.rank(4));
    /// ```
    #[inline(always)]
    pub(super) fn rank_pair(&self, l: usize, r: usize) -> (usize, usize) {
        debug_assert!(l <= r);
        debug_assert!(r <= self.len);

        // 各端点を含むブロックの累積値に、そのブロック内の 1 の数を足して rank を得る。
        let l_block = l / u64::BITS as usize;
        let r_block = r / u64::BITS as usize;
        (
            (self.cumulative_sums[l_block]
                + (self.bits[l_block] & MASKS[l % u64::BITS as usize]).count_ones())
                as usize,
            (self.cumulative_sums[r_block]
                + (self.bits[r_block] & MASKS[r % u64::BITS as usize]).count_ones())
                as usize,
        )
    }

    /// `BitVector` の長さを返す。
    ///
    /// # Returns
    ///
    /// 元のビット列の長さを返す。
    ///
    /// # Complexity
    ///
    /// - 時間計算量: O(1) である。
    /// - 空間計算量: O(1) である。
    ///
    /// # Examples
    ///
    /// ```rust
    /// use anmitsu::ds::bit_vector;
    ///
    /// let bv = bit_vector::BitVector::new(&[1, 0, 1]);
    /// assert_eq!(bv.len(), 3);
    /// ```
    pub fn len(&self) -> usize {
        self.len
    }

    /// `BitVector` が空かどうかを確認する。
    ///
    /// # Returns
    ///
    /// `BitVector` が空の場合は `true`、そうでない場合は `false` を返す。
    ///
    /// # Complexity
    ///
    /// - 時間計算量: O(1) である。
    /// - 空間計算量: O(1) である。
    ///
    /// # Examples
    ///
    /// ```rust
    /// use anmitsu::ds::bit_vector;
    ///
    /// let bv_empty = bit_vector::BitVector::new(&[]);
    /// assert!(bv_empty.is_empty());
    ///
    /// let bv_not_empty = bit_vector::BitVector::new(&[0, 1]);
    /// assert!(!bv_not_empty.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // len のテスト: 戻り値を検証する。
    mod len {
        use super::*;

        /// 状況: 生成時に渡したスライスの長さを返す (正常系 + 境界値)。
        /// - 前提: 要素数が異なる複数のスライスがある (空、単一要素、複数要素)。
        /// - 操作: 各スライスから `BitVector` を生成し、`len()` を呼ぶ。
        /// - 結果: 各ケースでスライスの長さが返る。
        #[test]
        fn returns_length_of_input_slice() {
            // 前提
            let cases = [(vec![], 0_usize), (vec![0], 1), (vec![1, 0, 1, 1, 0], 5)];
            // 操作と結果
            for (input, expected) in cases {
                let sut = BitVector::new(&input);
                assert_eq!(expected, sut.len());
            }
        }
    }

    // is_empty のテスト: 戻り値を検証する。
    mod is_empty {
        use super::*;

        /// 状況: スライスが空かどうかに応じて判定を返す (正常系 + 境界値)。
        /// - 前提: 空のスライスと、要素数が異なる複数の非空スライスがある。
        /// - 操作: 各スライスから `BitVector` を生成し、`is_empty()` を呼ぶ。
        /// - 結果: 空のスライスに対しては `true`、非空のスライスに対しては `false` が返る。
        #[test]
        fn returns_whether_empty() {
            // 前提
            let cases = [
                (vec![], true),
                (vec![0], false),
                (vec![1, 0, 1, 1, 0], false),
            ];
            // 操作と結果
            for (input, expected) in cases {
                let sut = BitVector::new(&input);
                assert_eq!(expected, sut.is_empty());
            }
        }
    }

    // rank のテスト: 戻り値を検証する。
    mod rank {
        use super::*;

        /// 状況: 空の `BitVector` に対して `rank(0)` は `0` になる (境界値)。
        /// - 前提: 空のスライスから生成した `BitVector` がある。
        /// - 操作: `rank(0)` を呼ぶ。
        /// - 結果: `0` が返る。
        #[test]
        fn returns_zero_for_empty_bit_vector() {
            // 前提
            let sut = BitVector::new(&[]);
            // 操作
            let result = sut.rank(0);
            // 結果
            assert_eq!(0, result);
        }

        /// 状況: 全要素が `0` のとき、任意の範囲での `rank` は常に `0` になる (境界値)。
        /// - 前提: 長さ 100 の、全要素が `0` の `BitVector` がある。
        /// - 操作: `0` から `len()` までの各 `r` で `rank(r)` を呼ぶ。
        /// - 結果: すべて `0` が返る。
        #[test]
        fn returns_zero_for_all_zero_bit_vector() {
            // 前提
            let sut = BitVector::new(&[0; 100]);
            // 操作と結果
            for r in 0..=100 {
                assert_eq!(0, sut.rank(r));
            }
        }

        /// 状況: 全要素が `1` のとき、`rank(r)` は `r` に等しくなる (境界値)。
        /// - 前提: 長さ 100 の、全要素が `1` の `BitVector` がある。
        /// - 操作: `0` から `len()` までの各 `r` で `rank(r)` を呼ぶ。
        /// - 結果: 各 `r` に対して `r` 自身が返る。
        #[test]
        fn returns_r_for_all_one_bit_vector() {
            // 前提
            let sut = BitVector::new(&[1; 100]);
            // 操作と結果
            for r in 0..=100 {
                assert_eq!(r, sut.rank(r));
            }
        }

        /// 状況: 64 ビットのブロック境界をまたぐ場合でも、累積和を用いた `rank` が
        /// 正しく計算される (境界値)。
        /// - 前提: 3 ブロック分 (長さ 192) の `BitVector` があり、各ブロックの
        ///   先頭と末尾のビットのみが `1` になっている。
        /// - 操作: 各ブロックの境界の前後で `rank` を呼ぶ。
        /// - 結果: 各位置までの `1` の累積個数が正しく返る。
        #[test]
        fn matches_expected_values_across_block_boundaries() {
            // 前提
            // ちょうど 3 ブロック分の長さを用意し、各ブロックの先頭と末尾のみ 1 にする。
            let mut v = vec![0; 192];
            v[0] = 1; // ブロック 0 の先頭
            v[63] = 1; // ブロック 0 の末尾
            v[64] = 1; // ブロック 1 の先頭
            v[127] = 1; // ブロック 1 の末尾
            v[128] = 1; // ブロック 2 の先頭
            v[191] = 1; // ブロック 2 の末尾
            let sut = BitVector::new(&v);
            // 操作と結果
            assert_eq!(192, sut.len());
            assert_eq!(0, sut.rank(0));
            assert_eq!(1, sut.rank(1)); // v[0] を含む rank(1)
            assert_eq!(1, sut.rank(63)); // v[63] の前の rank(63)
            assert_eq!(2, sut.rank(64)); // v[63] を含む rank(64)
            assert_eq!(3, sut.rank(65)); // v[64] を含む rank(65)
            assert_eq!(3, sut.rank(127)); // v[127] の前の rank(127)
            assert_eq!(4, sut.rank(128)); // v[127] を含む rank(128)
            assert_eq!(5, sut.rank(129)); // v[128] を含む rank(129)
            assert_eq!(5, sut.rank(191)); // v[191] の前の rank(191)
            assert_eq!(6, sut.rank(192)); // v[191] を含む rank(192), 全長の合計
        }

        /// 状況: `r` が `len()` を超える場合はパニックする (異常系)。
        /// - 前提: 長さ 3 の `BitVector` がある。
        /// - 操作: `rank(4)` を呼ぶ。
        /// - 結果: パニックする。
        #[test]
        #[should_panic(expected = "cannot be greater than the length of the BitVector")]
        fn panics_when_r_greater_than_len() {
            // 前提
            let sut = BitVector::new(&[1, 0, 1]);
            // 操作と結果 (パニック)
            let _ = sut.rank(4);
        }

        /// 状況: 空の `BitVector` に対しても、`r > len()` ならパニックする
        /// (異常系 + 境界値)。
        /// - 前提: 空のスライスから生成した `BitVector` がある。
        /// - 操作: `rank(1)` を呼ぶ。
        /// - 結果: パニックする。
        #[test]
        #[should_panic(expected = "cannot be greater than the length of the BitVector")]
        fn panics_when_r_greater_than_len_for_empty_bit_vector() {
            // 前提
            let sut = BitVector::new(&[]);
            // 操作と結果 (パニック)
            let _ = sut.rank(1);
        }
    }
}
