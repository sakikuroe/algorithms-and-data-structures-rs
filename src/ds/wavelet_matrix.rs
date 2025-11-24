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
    /// # Constraints
    /// - `v` can contain any `usize` values.
    ///   `v` は任意の `usize` 値を含むことができる.
    /// - The length of `v` must not be excessively large, such that the number of unique
    ///   elements causes `compress.len()` to exceed `usize` limits for bit calculations.
    ///   `v` の長さは, ユニークな要素の数が `compress.len()` を `usize` のビット計算の
    ///   制限を超えるほどに過度に大きくない必要がある.
    ///
    /// # Panics
    /// - This function may panic if `v` is extremely large and contains a vast number of
    ///   unique elements, leading to the `height` calculation (`(1..).find(...)`)
    ///   attempting to shift a bit beyond `usize::BITS - 1`, or if `(1_usize << i)`
    ///   overflows during the search for `height`.
    ///   この関数は, `v` が極めて大きく, 多数のユニークな要素を含み, `height` の計算
    ///   (`(1..).find(...)`) が `usize::BITS - 1` を超えるビットシフトを試みる場合,
    ///   または `height` の検索中に `(1_usize << i)` がオーバーフローするような値である
    ///   場合にパニックする可能性がある.
    ///
    /// # Complexity
    /// - Time complexity: O(N log U), where N is the length of `v` and U is the number of
    ///   distinct values in `v`.
    ///                          ここで N は `v` の長さ, U は `v` のユニークな値の数である.
    /// - Space complexity: O(N log U), where N is the length of `v` and U is the number of
    ///   distinct values in `v`.
    ///                           ここで N は `v` の長さ, U は `v` のユニークな値の数である.
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::ds::wavelet_matrix::WaveletMatrix;
    ///
    /// let data = vec![10, 5, 20, 15, 5, 10, 25];
    /// let wm = WaveletMatrix::new(&data);
    ///
    /// // Query count of elements < 15 in range [0, 7)
    /// // Original sequence: [10, 5, 20, 15, 5, 10, 25]
    /// // Elements less than 15: 10, 5, 5, 10. Total count is 4.
    /// assert_eq!(4, wm.count_less_than(0, 7, 15));
    ///
    /// // Query count of elements >= 10 in range [0, 4)
    /// // Subsequence: [10, 5, 20, 15]
    /// // Elements greater than or equal to 10: 10, 20, 15. Total count is 3.
    /// assert_eq!(3, wm.count_more_than(0, 4, 10));
    ///
    /// // Query count of elements in [5, 15) in range [1, 6)
    /// // Subsequence: [5, 20, 15, 5, 10]
    /// // Elements in [5, 15): 5, 5, 10. Total count is 3.
    /// assert_eq!(3, wm.count(1, 6, 5, 15));
    /// ```
    pub fn new(v: &[usize]) -> Self {
        // Arrange
        let mut sorted_v = v.to_vec();
        sorted_v.sort_unstable();
        sorted_v.dedup();

        // Compress values to indices (0 to unique_count - 1).
        // `partition_point` returns the index where elements are no longer less than `x`,
        // effectively finding the rank of `x` in `sorted_v`.
        let mut compress = v
            .iter()
            .map(|&x| sorted_v.partition_point(|&y| y < x))
            .collect::<Vec<_>>();

        // Calculate height: number of bits required to represent the maximum compressed value.
        // `compress.len()` is the number of unique elements (U).
        // `(1..)` finds the smallest `i` such that `2^i >= U + 1`.
        // `1_usize << i` explicitly specifies `usize` for the bit shift to avoid type issues.
        let height = (1..)
            .find(|i| (1_usize << i) >= compress.len() + 1)
            .unwrap()
            + 1;

        // Initialize bit_table; type is inferred from `BitVector::new`.
        let mut bit_table = Vec::new();

        // Build the bit_table layer by layer, from most significant bit to least significant bit.
        for i in (0..height).rev() {
            // Act: Create a BitVector for the current bit position.
            // This BitVector stores the i-th bit of each compressed value.
            bit_table.push(bit_vector::BitVector::new(
                &compress
                    .iter()
                    .map(|&x| ((x >> i) & 1) as u8)
                    .collect::<Vec<_>>(),
            ));

            // Rearrange `compress` to move elements with 0 at the i-th bit to the front,
            // and elements with 1 to the back. This prepares for the next level.
            compress = compress
                .iter()
                .filter(|&x| ((x >> i) & 1) == 0)
                .chain(compress.iter().filter(|&x| ((x >> i) & 1) == 1))
                .cloned()
                .collect::<Vec<_>>();
        }

        // Assert
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
    /// # Constraints
    /// - `l` and `r` must be valid indices within the original sequence length
    ///   (`0 <= l, r <= original_len`).
    ///   `l` と `r` は元のシーケンスの長さ内の有効なインデックスである必要がある
    /// - `upper` can be any `usize` value.
    ///   `upper` は任意の `usize` 値である.
    ///
    /// # Panics
    /// - None.
    /// - なし.
    ///
    /// # Complexity
    /// - Time complexity: O(log U), where U is the number of distinct values in the original sequence.
    ///                          ここで U は元のシーケンスにおけるユニークな値の数である.
    pub fn count_less_than(&self, mut l: usize, mut r: usize, upper: usize) -> usize {
        if r <= l {
            return 0;
        }

        // Arrange
        // Find the compressed index for `upper`. All values less than `upper`
        // will have compressed indices less than `upper_idx`.
        let upper_idx = self.sorted_v.partition_point(|&x| x < upper);
        let mut res = 0;

        // Act & Assert
        // Iterate through bit layers from MSB to LSB.
        for (i, bit) in (0..self.height).rev().zip(self.bit_table.iter()) {
            // Calculate ranks of `l` and `r` (count of 1s up to l/r) in the current bit_vector.
            let rank_l = bit.rank(l);
            let rank_r = bit.rank(r);

            // Determine which branch to take based on the i-th bit of `upper_idx`.
            if ((upper_idx >> i) & 1) == 0 {
                // If the i-th bit of `upper_idx` is 0, we only search in the 0-block.
                // Update `l` and `r` to reflect the new range in the 0-block.
                l -= rank_l;
                r -= rank_r;
            } else {
                // If the i-th bit of `upper_idx` is 1, all elements in the 0-block (whose i-th bit is 0)
                // are less than the current target path. Add their count to `res`.
                res += (r - l) - (rank_r - rank_l);

                // Update `l` and `r` to reflect the new range in the 1-block.
                // `len_zeros_upto_end` is the count of 0s in the entire bit_vector up to its end.
                let len_zeros_upto_end = bit.len() - bit.rank(bit.len());
                l = rank_l + len_zeros_upto_end;
                r = rank_r + len_zeros_upto_end;
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
    /// # Constraints
    /// - `l` and `r` must be valid indices within the original sequence length
    ///   `l` x `r` は元のシーケンスの長さ内の有効なインデックスである必要がある
    ///   (`0 <= l , r <= original_len`).
    /// - `lower` can be any `usize` value.
    ///   `lower` は任意の `usize` 値である.
    ///
    /// # Panics
    /// - None.
    /// - なし.
    ///
    /// # Complexity
    /// - Time complexity: O(log U), where U is the number of distinct values in the original sequence.
    ///                               ここで U は元のシーケンスにおけるユニークな値の数である.
    pub fn count_more_than(&self, l: usize, r: usize, lower: usize) -> usize {
        if r <= l {
            return 0;
        }

        (r - l) - self.count_less_than(l, r, lower)
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
    /// # Constraints
    /// - `l` and `r` must be valid indices within the original sequence length
    ///   `l` と `r` は元のシーケンスの長さ内の有効なインデックスである必要がある
    ///   (`0 <= l, r <= original_len`).
    /// - `lower` and `upper` can be any `usize` values.
    ///   `lower` と `upper` は任意の `usize` 値である.
    ///
    /// # Panics
    /// - None.
    /// - なし.
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
