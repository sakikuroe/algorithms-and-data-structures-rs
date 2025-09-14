const MASKS: [u64; 64] = {
    let mut masks = [0_u64; 64];
    let mut k = 1_usize;
    while k < 64 {
        masks[k] = (1_u64 << k) - 1;
        k += 1;
    }
    masks
};

/// A data structure to efficiently store and query a sequence of bits (0s and 1s).
/// 0 と 1 からなるビット列を効率的に格納し, クエリを実行するためのデータ構造である.
///
/// The `BitVector` precomputes cumulative sums of 1s in blocks of 64 bits
/// to allow for fast `sum` queries.
/// `BitVector` は 64 ビットのブロックごとに 1 の累積和を事前計算することで,
/// 高速な `sum` クエリを可能にする.
#[derive(Clone)]
pub struct BitVector {
    bits: Vec<u64>,

    // Stores the sum of 1s up to the end of each block
    cumulative_sums: Vec<u32>,

    // Length of the BitVector
    len: usize,
}

impl BitVector {
    /// Creates a new `BitVector`.
    /// 新しい `BitVector` を作成する.
    ///
    /// # Args
    ///
    /// * `v`: A slice of `u8` where each element is either `0` or `1`.
    ///        The length of `v` must be less than 2^{32} (=4294967296)
    ///   `v`: 各要素が `0 ` または `1 ` である `u8` のスライスである.
    ///        `v` の長さは 2^{32} (=4294967296) 未満でなければならない.
    ///
    /// # Returns
    ///
    /// A new `BitVector` instance.
    /// 新しい `BitVector` インスタンスを返す.
    ///
    /// # Panics
    ///
    /// Panics if any element in `v` is not `0` or `1`.
    /// Panics if the length of `v` is greater than or equal to 2^{32}.
    /// `v` のいずれかの要素が `0 ` または `1 ` でない場合にパニックする.
    /// `v` の長さが 2^{32} 以上の場合にパニックする.
    ///
    /// # Complexity
    ///
    /// Time: O(N), where N is the length of `v`.
    /// 時間計算量: O(N). ここで N は `v` の長さである.
    ///
    /// Space: O(N), where N is the length of `v`.
    /// 空間計算量: O(N). ここで N は `v` の長さである.
    ///
    /// # Examples
    ///
    /// ```
    /// use anmitsu::ds::bit_vector;
    ///
    /// let bv = bit_vector::BitVector::new(&[1, 0, 1, 1, 0, 1]);
    /// assert_eq!(6, bv.len());
    /// assert_eq!(3, bv.rank(4)); // sum([1, 0, 1, 1])
    /// assert_eq!(4, bv.rank(6)); // sum([1, 0, 1, 1, 0, 1])
    /// assert_eq!(0, bv.rank(0)); // sum([])
    /// ```
    pub fn new(v: &[u8]) -> Self {
        let len = v.len();
        assert!(len < (1 << 32), "Length of v must be less than 2^{{32}}");

        let num_blocks = len / 64 + 1;
        let mut bits = vec![0_u64; num_blocks];
        let mut cumulative_sums = vec![0_u32; num_blocks];
        let mut current_sum = 0_u32;

        for (i, &bit_val) in v.iter().enumerate() {
            if bit_val != 0 && bit_val != 1 {
                panic!("Input slice `v` must only contain 0 or 1.");
            }
            if bit_val == 1 {
                let block_index = i / 64;
                let bit_in_block = i % 64;
                bits[block_index] |= 1_u64 << bit_in_block;
            }
        }

        for i in 0..num_blocks {
            current_sum += bits[i].count_ones();
            cumulative_sums[i] = current_sum;
        }

        BitVector {
            bits,
            cumulative_sums,
            len,
        }
    }

    /// Returns the number of 1s in the range `v[0..r) (sum of v[0..r) )`.
    /// 範囲 `v[0..r)` における `1 ` の数 (v[0..r) の和) を返す.
    ///
    /// # Args
    ///
    /// * `r`: The upper bound of the range. `r` must be less than or equal to `len()`.
    ///   `r`: 範囲の上限である. `r` は `len() ` 以下でなければならない.
    ///
    /// # Returns
    ///
    /// The sum of 1s in `v[0..r)`. Returns `0` if `r` is `0`.
    /// `v[0..r)` における `1 ` の合計を返す. `r` が `0 ` の場合は `0 ` を返す.
    ///
    /// # Panics
    ///
    /// Panics if `r > len()`.
    /// `r > len() ` の場合にパニックする.
    ///
    /// # Complexity
    ///
    /// Time: O(1) due to precomputation.
    /// 時間計算量: 事前計算により O(1).
    ///
    /// Space: O(1) for the query itself.
    /// 空間計算量: クエリ自体は O(1).
    ///
    /// # Examples
    ///
    /// ```
    /// use anmitsu::ds::bit_vector;
    ///
    /// let bv = bit_vector::BitVector::new(&[1, 0, 1, 1, 0, 1, 0, 0]);
    /// assert_eq!(bv.rank(0), 0);
    /// assert_eq!(bv.rank(1), 1); // v[0] = 1
    /// assert_eq!(bv.rank(3), 2); // v[0..3] = [1, 0, 1], sum = 2
    /// assert_eq!(bv.rank(6), 4); // v[0..6] = [1, 0, 1, 1, 0, 1], sum = 4
    /// assert_eq!(bv.rank(8), 4); // v[0..8] = [1, 0, 1, 1, 0, 1, 0, 0], sum = 4
    /// ```
    pub fn rank(&self, r: usize) -> usize {
        if r == 0 {
            return 0;
        }

        if r > self.len {
            panic!(
                "r ({}) cannot be greater than the length of the BitVector ({})",
                r, self.len
            );
        }

        // Calculate the block index to efficiently access the precomputed cumulative sums and bit data.
        let block_index = r / 64;

        let mut res = 0;
        // Add the cumulative sum of 1s from all preceding full blocks.
        if block_index > 0 {
            res += self.cumulative_sums[block_index - 1];
        }
        // Add the number of 1s from the partial current block, up to the r-th bit,
        // using MASKS to isolate the relevant bits.
        res += (self.bits[block_index] & MASKS[r % 64]).count_ones();
        res as usize
    }

    /// Returns the length of the BitVector.
    /// BitVector の長さを返す.
    ///
    /// # Returns
    ///
    /// The length of the original bit sequence.
    /// 元のビット列の長さを返す.
    ///
    /// # Complexity
    ///
    /// Time: O(1)
    /// 時間計算量: O(1).
    ///
    /// Space: O(1)
    /// 空間計算量: O(1).
    ///
    /// # Examples
    ///
    /// ```
    /// use anmitsu::ds::bit_vector;
    ///
    /// let bv = bit_vector::BitVector::new(&[1, 0, 1]);
    /// assert_eq!(bv.len(), 3);
    /// ```
    pub fn len(&self) -> usize {
        self.len
    }

    /// Checks if the BitVector is empty.
    /// BitVector が空かどうかを確認する.
    ///
    /// # Returns
    ///
    /// `true` if the bit vector is empty, `false` otherwise.
    /// BitVector が空の場合は `true`, そうでない場合は `false` を返す.
    ///
    /// # Complexity
    ///
    /// Time: O(1)
    /// 時間計算量: O(1).
    ///
    /// Space: O(1)
    /// 空間計算量: O(1).
    ///
    /// # Examples
    ///
    /// ```
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
