//! A segment tree data structure for range queries and updates.
//! `Segment tree` の実装であり, range query と更新をサポートする.

use super::super::super::algebra::monoid::Monoid;

// テストでのみ使用する追加のインポート. `monoid::AddMonoid` や `semi_group::SemiGroup` は
// 実装本体では使用しないため, `#[cfg(test)]` で分離する.
#[cfg(test)]
use super::super::super::algebra::{monoid, semi_group};

/// A dense segment tree that supports range queries and updates.
/// 密な `segment tree` であり, range query と更新をサポートする.
#[derive(Clone)]
pub struct SegmentTreeDense<M>
where
    M: Monoid,
{
    len: usize,
    data: Vec<M::S>,
}

impl<M> SegmentTreeDense<M>
where
    M: Monoid,
    M::S: Clone,
{
    /// Creates a new `SegmentTreeDense` with capacity for `n` elements.
    /// `n` 個の要素に対応する `SegmentTreeDense` を生成する.
    ///
    /// # Args
    /// - `n`: The size (number of leaves) of the segment tree.
    ///   `segment tree` のサイズ (葉の数).
    ///
    /// # Returns
    /// `SegmentTreeDense<M>`: Returns a newly created segment tree instance.
    ///                        新しい `segment tree` のインスタンスを返す.
    ///
    /// # Constraints
    /// No specific constraints on `n`.
    /// `n` に関する制約はない.
    ///
    /// # Complexity
    /// - Time complexity: O(n), where `n` is the size of the segment tree.
    ///   ここで `n` は `segment tree` のサイズである.
    /// - Space complexity: O(n), where `n` is the size of the segment tree.
    ///   ここで `n` は `segment tree` のサイズである.
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::{algebra::monoid, ds::segment_tree::segment_tree_dense};
    /// let seg = segment_tree_dense::SegmentTreeDense::<monoid::AddMonoid>::new(5);
    /// ```
    pub fn new(n: usize) -> Self {
        let len = n;
        // The size of the internal data vector is 2*len - 1 for a complete binary tree.
        // Handle the case where len is 0 to avoid underflow.
        SegmentTreeDense::<M> {
            len,
            data: vec![M::id(); if len == 0 { 0 } else { 2 * len - 1 }],
        }
    }

    /// Returns the size (number of leaves) of this segment tree.
    /// この `segment tree` のサイズ (葉の数) を返す.
    ///
    /// # Returns
    /// `usize`: The size (number of leaves) of the segment tree.
    ///          `segment tree` のサイズ (葉の数).
    ///
    /// # Panics
    /// This function does not panic.
    /// この関数はパニックしない.
    ///
    /// # Complexity
    /// - Time complexity: O(1).
    /// - Space complexity: O(1).
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::{algebra::monoid, ds::segment_tree::segment_tree_dense};
    /// let seg = segment_tree_dense::SegmentTreeDense::<monoid::AddMonoid>::new(10);
    /// assert_eq!(seg.len(), 10);
    /// ```
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns true if the segment tree is empty.
    /// この `segment tree` が空であるかどうかを返す.
    ///
    /// # Returns
    /// `bool`: True if the segment tree contains no elements.
    ///         `segment tree` が要素を含まない場合に真.
    ///
    /// # Panics
    /// This function does not panic.
    /// この関数はパニックしない.
    ///
    /// # Complexity
    /// - Time complexity: O(1).
    /// - Space complexity: O(1).
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::{algebra::monoid, ds::segment_tree::segment_tree_dense};
    /// let seg = segment_tree_dense::SegmentTreeDense::<monoid::AddMonoid>::new(0);
    /// assert!(seg.is_empty());
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Sets the value at index `idx` to `x`.
    /// The update is lazy; use `build` or `update` to propagate changes to parent nodes.
    /// インデックス `idx` の値を `x` にセットする.
    /// この更新は遅延実行されるため, 上位ノードへ変更を反映するには `build` または `update` を使用する.
    ///
    /// # Args
    /// - `idx`: The index to set.
    ///   セット対象のインデックス.
    /// - `x`: The new value.
    ///   新しい値.
    ///
    /// # Panics
    /// Panics if `idx` >= `self.len()`.
    /// `idx` が `self.len()` 以上の場合にパニックする.
    ///
    /// # Complexity
    /// - Time complexity: O(1).
    /// - Space complexity: O(1).
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::{algebra::monoid, ds::segment_tree::segment_tree_dense};
    /// let mut seg = segment_tree_dense::SegmentTreeDense::<monoid::AddMonoid>::new(3);
    /// seg.set(0, 1);
    /// seg.set(1, 2);
    /// seg.set(2, 3);
    /// // `set` is lazy. Use `build` to apply changes.
    /// seg.build();
    /// assert_eq!(seg.fold(0, 3), 6);
    /// ```
    pub fn set(&mut self, mut idx: usize, x: M::S) {
        assert!(
            idx < self.len(),
            "index out of bounds: the len is {} but the index is {}",
            self.len(),
            idx
        );
        // Calculate the position in the data vector corresponding to the leaf node.
        idx += self.len - 1;
        self.data[idx] = x;
    }

    /// Builds the segment tree by propagating the leaves' values up to their parent nodes.
    /// 葉に設定された値を親ノードへ伝播させて, `segment tree` を構築する.
    ///
    /// # Panics
    /// This function does not panic.
    /// この関数はパニックしない.
    ///
    /// # Complexity
    /// - Time complexity: O(n), where `n` is the size of the segment tree.
    ///   ここで `n` は `segment tree` のサイズである.
    /// - Space complexity: O(1).
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::{algebra::monoid, ds::segment_tree::segment_tree_dense};
    /// let mut seg = segment_tree_dense::SegmentTreeDense::<monoid::AddMonoid>::new(3);
    /// seg.set(0, 5);
    /// seg.set(1, 10);
    /// seg.set(2, 15);
    /// seg.build();
    /// assert_eq!(seg.fold(0, 3), 30);
    /// ```
    pub fn build(&mut self) {
        // Iterate from the last parent node down to the root.
        for idx in (0..self.len - 1).rev() {
            // Update parent node with the result of the monoid operation on its children.
            self.data[idx] = M::op(&self.data[2 * idx + 1], &self.data[2 * idx + 2]);
        }
    }

    /// Updates the value at index `idx` to `x` and propagates this change up the tree.
    /// インデックス `idx` の値を `x` に更新し, 上位ノードへ変更を反映する.
    ///
    /// # Args
    /// - `idx`: The index to update.
    ///   更新対象のインデックス.
    /// - `x`: The new value.
    ///   新しい値.
    ///
    /// # Panics
    /// Panics if `idx` >= `self.len()`.
    /// `idx` が `self.len()` 以上の場合にパニックする.
    ///
    /// # Complexity
    /// - Time complexity: O(log(n)), where `n` is the size of the segment tree.
    ///   ここで `n` は `segment tree` のサイズである.
    /// - Space complexity: O(1).
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::{algebra::monoid, ds::segment_tree::segment_tree_dense};
    /// let mut seg = segment_tree_dense::SegmentTreeDense::<monoid::AddMonoid>::new(3);
    /// seg.set(0, 1);
    /// seg.set(1, 2);
    /// seg.set(2, 3);
    /// seg.build();
    /// assert_eq!(seg.fold(0, 3), 6);
    /// seg.update(1, 10);
    /// assert_eq!(seg.fold(0, 3), 14);
    /// ```
    pub fn update(&mut self, mut idx: usize, x: M::S) {
        assert!(
            idx < self.len(),
            "index out of bounds: the len is {} but the index is {}",
            self.len(),
            idx
        );
        // Calculate leaf position and update its value.
        idx += self.len - 1;
        self.data[idx] = x;
        // Climb up the tree updating parent nodes.
        while idx > 0 {
            idx = (idx - 1) / 2;
            self.data[idx] = M::op(&self.data[2 * idx + 1], &self.data[2 * idx + 2]);
        }
    }

    /// Gets the value at index `idx`.
    /// インデックス `idx` の値を取得する.
    ///
    /// # Args
    /// - `idx`: The index to retrieve.
    ///   値を取得するインデックス.
    ///
    /// # Returns
    /// `M::S`: The value at `idx`.
    ///         `idx` の値.
    ///
    /// # Panics
    /// Panics if `idx` >= `self.len()`.
    /// `idx` が `self.len()` 以上の場合にパニックする.
    ///
    /// # Complexity
    /// - Time complexity: O(1).
    /// - Space complexity: O(1).
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::{algebra::monoid, ds::segment_tree::segment_tree_dense};
    /// let mut seg = segment_tree_dense::SegmentTreeDense::<monoid::AddMonoid>::new(3);
    /// seg.set(0, 10);
    /// seg.build();
    /// assert_eq!(seg.get(0), 10);
    /// ```
    pub fn get(&self, mut idx: usize) -> M::S {
        assert!(
            idx < self.len(),
            "index out of bounds: the len is {} but the index is {}",
            self.len(),
            idx
        );
        // Calculate leaf position and return its value.
        idx += self.len - 1;
        self.data[idx].clone()
    }

    /// Performs a range fold (query) on the interval `[l, r)`.
    /// This operation aggregates the elements in the specified range using the monoid's binary operation `op`.
    /// For example, if the operation is addition, this calculates `data[l] + data[l + 1] + ... + data[r - 1]`.
    /// 区間 `[l, r)` 上の値に対して `fold` (畳み込み) を行う `query` を実行する.
    /// この操作は, 指定された範囲の要素をモノイドの二項演算 `op` を用いて集約する.
    /// 例えば, 演算が加算の場合, `data[l] + data[l+1] + ... + data[r-1]` を計算する.
    ///
    /// # Args
    /// - `l`: The start index of the range (inclusive).
    ///   `query` 区間の開始インデックス (含む).
    /// - `r`: The end index of the range (exclusive).
    ///   `query` 区間の終了インデックス (含まない).
    ///
    /// # Returns
    /// `M::S`: The folded result of the interval `[l, r)`. It is the identity element `M::id()` if the range is empty.
    ///         区間 `[l, r)` の畳み込み結果. 区間が空の場合, 単位元 `M::id()` となる.
    ///
    /// # Panics
    /// Panics if `r > self.len()`.
    /// `r > self.len()` の場合にパニックする.
    ///
    /// # Complexity
    /// - Time complexity: O(log(n)), where `n` is the size of the segment tree.
    ///   ここで `n` は `segment tree` のサイズである.
    /// - Space complexity: O(1).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use anmitsu::{algebra::monoid, ds::segment_tree::segment_tree_dense};
    ///
    /// let mut seg = segment_tree_dense::SegmentTreeDense::<monoid::AddMonoid>::new(5);
    /// seg.set(0, 1);
    /// seg.set(1, 10);
    /// seg.set(2, 100);
    /// seg.set(3, 1000);
    /// seg.set(4, 10000);
    /// seg.build();
    /// assert_eq!(seg.fold(1, 3), 110);
    /// ```
    pub fn fold(&self, mut l: usize, mut r: usize) -> M::S {
        if l >= r {
            return M::id();
        }

        assert!(
            r <= self.len(),
            "index out of bounds: r must be less than or equal to the len (r: {}, len: {})",
            r,
            self.len()
        );

        // Map logical indices to internal data array indices.
        l += self.len - 1;
        r += self.len - 1;

        let mut sum_l = M::id();
        let mut sum_r = M::id();

        // Fold elements within [l, r).
        while l < r {
            if l.is_multiple_of(2) {
                sum_l = M::op(&sum_l, &self.data[l]);
            }
            if r.is_multiple_of(2) {
                sum_r = M::op(&self.data[r - 1], &sum_r);
            }
            l /= 2;
            r = (r - 1) / 2;
        }

        M::op(&sum_l, &sum_r)
    }

    // Check if k is outside of leaf index or satisfies a particular condition
    fn is_good_node(k: usize, len: usize) -> bool {
        if k >= len {
            true
        } else {
            let d = k.leading_zeros() - len.leading_zeros();
            len >> d != k || len >> d << d == len
        }
    }

    /// Finds the maximum `r` in `[l, self.len()]` such that `f` applied to the fold result
    /// from `[l, r)` is `true`. Returns `self.len()` if no further extension is possible.
    /// 区間 `[l, self.len()]` 内で, `[l, r)` の `fold` 結果に対して述語 `f` が `true` を返すような
    /// 最大の `r` を探索する. 条件を満たす `r` がこれ以上存在しない場合は, `self.len()` を返す.
    ///
    /// # Args
    /// - `l`: The start index of the range.
    ///   範囲の開始インデックス.
    /// - `f`: A function that takes a reference to `M::S` and returns a boolean.
    ///   `M::S` への参照を受け取り, 真偽値を返す関数.
    ///
    /// # Returns
    /// `usize`: The maximum `r` such that `f(fold(l, r))` is `true`.
    ///          `f(fold(l, r))` が `true` となる最大の `r`.
    ///
    /// # Panics
    /// Panics if `f(&M::id())` is false or `l > self.len()`.
    /// `f(&M::id())` が `false` の場合, または `l > self.len()` の場合にパニックする.
    ///
    /// # Complexity
    /// - Time complexity: O(log(n)), where `n` is the size of the segment tree.
    ///   ここで `n` は `segment tree` のサイズである.
    /// - Space complexity: O(1).
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::{algebra::monoid, ds::segment_tree::segment_tree_dense};
    /// let mut seg = segment_tree_dense::SegmentTreeDense::<monoid::AddMonoid>::new(5);
    /// for i in 0..5 {
    ///     seg.set(i, i as i64 + 1);
    /// }
    /// seg.build();
    /// // Find `r` from `l=1` where sum of `[1, r)` is less than 10.
    /// // [1, 4) -> 2 + 3 + 4 = 9 (< 10)
    /// // [1, 5) -> 2 + 3 + 4 + 5 = 14 (>= 10)
    /// let r = seg.max_right(1, |&sum| sum < 10);
    /// assert_eq!(r, 4);
    /// ```
    pub fn max_right<F>(&self, mut l: usize, f: F) -> usize
    where
        F: Fn(&M::S) -> bool,
    {
        assert!(
            f(&M::id()),
            "predicate must be true for the identity element"
        );
        assert!(
            l <= self.len(),
            "index out of bounds: l must be less than or equal to the len (l: {}, len: {})",
            l,
            self.len()
        );

        // If the full range `[l, self.len())` satisfies f, return self.len().
        if l == self.len() || f(&self.fold(l, self.len())) {
            return self.len();
        }

        // Map to internal index and initialize sum.
        l += self.len();
        let mut sum = M::id();
        // Iteratively push the boundary to the right until we find the maximum r.
        loop {
            // Move up to the parent if l is the right child.
            while l & 1 == 0 && Self::is_good_node(l >> 1, self.len()) {
                l >>= 1;
            }
            // If the predicate fails with the next node, we've found the boundary.
            if !f(&M::op(&sum, &self.data[l - 1])) {
                while l < self.len() {
                    l <<= 1;
                    let t = M::op(&sum, &self.data[l - 1]);
                    // If adding the left child's value is still valid, move right.
                    if f(&t) {
                        sum = t;
                        l += 1;
                    }
                }
                // Convert internal index back to logical index.
                return l - self.len();
            }
            // Otherwise, include the current node and move to the next.
            sum = M::op(&sum, &self.data[l - 1]);
            l += 1;
        }
    }

    /// Finds the minimum `l` in `[0, r]` such that `f` applied to the fold result
    /// from `[l, r)` is `true`. Returns `0` if it cannot move further left.
    /// 区間 `[0, r]` 内で, `[l, r)` の `fold` 結果に対して述語 `f` が `true` を返すような
    /// 最小の `l` を探索する. 条件を満たす `l` がこれ以上存在しない場合は, `0` を返す.
    ///
    /// # Args
    /// - `r`: The end index of the range.
    ///   範囲の終了インデックス.
    /// - `f`: A function that takes a reference to `M::S` and returns a boolean.
    ///   `M::S` への参照を受け取り, 真偽値を返す関数.
    ///
    /// # Returns
    /// `usize`: The minimum `l` such that `f(fold(l, r))` is `true`.
    ///          `f(fold(l, r))` が `true` となる最小の `l`.
    ///
    /// # Panics
    /// Panics if `f(&M::id())` is false or `r > self.len()`.
    /// `f(&M::id())` が `false` の場合, または `r > self.len()` の場合にパニックする.
    ///
    /// # Complexity
    /// - Time complexity: O(log(n)), where `n` is the size of the segment tree.
    ///   ここで `n` は `segment tree` のサイズである.
    /// - Space complexity: O(1).
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::{algebra::monoid, ds::segment_tree::segment_tree_dense};
    /// let mut seg = segment_tree_dense::SegmentTreeDense::<monoid::AddMonoid>::new(5);
    /// for i in 0..5 {
    ///     seg.set(i, i as i64 + 1);
    /// }
    /// seg.build();
    /// // Find `l` from `r=4` where sum of `[l, 4)` is less than 10.
    /// // [1, 4) -> 2 + 3 + 4 = 9 (< 10)
    /// // [0, 4) -> 1 + 2 + 3 + 4 = 10 (>= 10)
    /// let l = seg.min_left(4, |&sum| sum < 10);
    /// assert_eq!(l, 1);
    /// ```
    pub fn min_left<L>(&mut self, mut r: usize, f: L) -> usize
    where
        L: Fn(&M::S) -> bool,
    {
        assert!(
            f(&M::id()),
            "predicate must be true for the identity element"
        );
        assert!(
            r <= self.len(),
            "index out of bounds: r must be less than or equal to the len (r: {}, len: {})",
            r,
            self.len()
        );

        // If the full range `[0, r)` satisfies f, return 0.
        if r == 0 || f(&self.fold(0, r)) {
            return 0;
        }

        // Map to internal index and initialize sum.
        let mut sum = M::id();
        r += self.len;
        // Iteratively shrink the boundary from the right.
        loop {
            r -= 1;
            // Move up to parent if r is the left child.
            while !Self::is_good_node(r, self.len()) {
                r = r * 2 + 1;
            }
            while r & 1 != 0 && Self::is_good_node(r >> 1, self.len()) {
                r >>= 1;
            }
            // If the predicate fails with the next node, we've found the boundary.
            if !f(&M::op(&self.data[r - 1], &sum)) {
                while r < self.len {
                    r = r * 2 + 1;
                    let t = M::op(&self.data[r - 1], &sum);
                    // If including the right child is still valid, move left.
                    if f(&t) {
                        sum = t;
                        r -= 1;
                    }
                }
                // Convert internal index back to logical index.
                return r + 1 - self.len;
            }
            // Otherwise, include the current node and move to the next.
            sum = M::op(&self.data[r - 1], &sum);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Background: `initial_data` を葉に設定し, `build` まで済ませた `SegmentTreeDense` を用意する。
    fn create_dense_tree<M>(initial_data: &[M::S]) -> SegmentTreeDense<M>
    where
        M: Monoid,
        M::S: Clone,
    {
        let mut sut = SegmentTreeDense::<M>::new(initial_data.len());
        for (i, val) in initial_data.iter().enumerate() {
            sut.set(i, val.clone());
        }
        sut.build();
        sut
    }

    /// Background: `initial_data` を持つ `SegmentTreeDense` と, 同じデータを持つ
    /// `NaiveSegmentTree` (期待値算出用のオラクル) の組を用意する。
    fn create_dense_and_naive<M>(
        initial_data: &[M::S],
    ) -> (SegmentTreeDense<M>, NaiveSegmentTree<M>)
    where
        M: Monoid,
        M::S: Clone,
    {
        let n = initial_data.len();
        let mut sut = SegmentTreeDense::<M>::new(n);
        let mut naive = NaiveSegmentTree::<M>::new(n);
        for (i, val) in initial_data.iter().enumerate() {
            sut.set(i, val.clone());
            naive.set(i, val.clone());
        }
        sut.build();
        (sut, naive)
    }

    /// `segment tree` の愚直な実装であり, `SegmentTreeDense` の正当性を確かめるための
    /// 期待値算出用のオラクルとして用いる。
    struct NaiveSegmentTree<M>
    where
        M: Monoid,
    {
        data: Vec<M::S>,
    }

    impl<M> NaiveSegmentTree<M>
    where
        M: Monoid,
        M::S: Clone,
    {
        fn new(n: usize) -> Self {
            Self {
                data: vec![M::id(); n],
            }
        }

        fn len(&self) -> usize {
            self.data.len()
        }

        fn set(&mut self, idx: usize, x: M::S) {
            self.data[idx] = x;
        }

        fn update(&mut self, idx: usize, x: M::S) {
            self.data[idx] = x;
        }

        fn get(&self, idx: usize) -> M::S {
            self.data[idx].clone()
        }

        fn fold(&self, l: usize, r: usize) -> M::S {
            assert!(r <= self.len());
            if l >= r {
                return M::id();
            }

            let mut res = M::id();
            for i in l..r {
                res = M::op(&res, &self.data[i]);
            }
            res
        }

        fn max_right<F>(&self, l: usize, f: F) -> usize
        where
            F: Fn(&M::S) -> bool,
        {
            assert!(
                f(&M::id()),
                "predicate must be true for the identity element"
            );
            assert!(l <= self.len());

            let mut sum = M::id();
            for r in l..self.len() {
                let next_sum = M::op(&sum, &self.data[r]);
                if !f(&next_sum) {
                    return r;
                }
                sum = next_sum;
            }
            self.len()
        }

        fn min_left<L>(&self, r: usize, f: L) -> usize
        where
            L: Fn(&M::S) -> bool,
        {
            assert!(
                f(&M::id()),
                "predicate must be true for the identity element"
            );
            assert!(r <= self.len());

            let mut sum = M::id();
            for l in (0..r).rev() {
                let next_sum = M::op(&self.data[l], &sum);
                if !f(&next_sum) {
                    return l + 1;
                }
                sum = next_sum;
            }
            0
        }
    }

    // new のテスト: 生成直後の状態 (len) を検証する。
    mod new {
        use super::*;

        /// Scenario: 指定したサイズで生成すると, `len` がそのサイズを返す。
        /// - Given: サイズ `0` および `10` がある。
        /// - When: `SegmentTreeDense::new` で `segment tree` を生成する。
        /// - Then: `len()` が指定したサイズと一致する。
        #[test]
        fn sets_len_to_given_size() {
            // Given
            let cases = [0_usize, 10];
            // When, Then
            for n in cases {
                let sut = SegmentTreeDense::<monoid::AddMonoid>::new(n);
                assert_eq!(n, sut.len());
            }
        }
    }

    // set のテスト: 異常系を検証する。 正常系は get, fold などのテストの前提条件として
    // 間接的に検証されている。
    mod set {
        use super::*;

        /// Scenario: 範囲外のインデックスを指定するとパニックする (異常系)。
        /// - Given: サイズ `5` の `sut` がある。
        /// - When: `set(5, 1)` を呼ぶ。
        /// - Then: パニックする。
        #[test]
        #[should_panic(expected = "index out of bounds")]
        fn panics_when_index_out_of_bounds() {
            // Given
            let mut sut = SegmentTreeDense::<monoid::AddMonoid>::new(5);
            // When, Then (panic)
            sut.set(5, 1);
        }
    }

    // get のテスト: 戻り値, および異常系を検証する。
    mod get {
        use super::*;

        /// Scenario: 設定した葉の値をそのまま取得できる。
        /// - Given: `[1, 10, 100, 1000, 10000]` を設定し `build` した `sut` がある。
        /// - When: 各インデックスに対して `get` を呼ぶ。
        /// - Then: 設定した値がそのまま返る。
        #[test]
        fn returns_value_set_at_leaf() {
            // Given
            let initial_data = vec![1, 10, 100, 1000, 10000];
            let sut = create_dense_tree::<monoid::AddMonoid>(&initial_data);
            // When, Then
            for (i, &expected) in initial_data.iter().enumerate() {
                assert_eq!(expected, sut.get(i));
            }
        }

        /// Scenario: 範囲外のインデックスを指定するとパニックする (異常系)。
        /// - Given: サイズ `5` の `sut` がある。
        /// - When: `get(5)` を呼ぶ。
        /// - Then: パニックする。
        #[test]
        #[should_panic(expected = "index out of bounds")]
        fn panics_when_index_out_of_bounds() {
            // Given
            let sut = SegmentTreeDense::<monoid::AddMonoid>::new(5);
            // When, Then (panic)
            let _ = sut.get(5);
        }
    }

    // fold のテスト: 戻り値, 境界値, および異常系を検証する。
    mod fold {
        use super::*;

        /// Scenario: 典型的な複数の区間に対して, 愚直実装と同じ畳み込み結果を返す。
        /// - Given: `[1, 10, 100, 1000, 10000]` を持つ `sut` と, 同じデータの `naive` オラクルがある。
        /// - When: `[0, n]` の範囲に含まれる全ての `(i, j)` の組で `fold(i, j)` を求める。
        /// - Then: 各組について `naive.fold` の結果と一致する。
        #[test]
        fn matches_naive_for_all_ranges() {
            // Given
            let initial_data = vec![1, 10, 100, 1000, 10000];
            let n = initial_data.len();
            let (sut, naive) = create_dense_and_naive::<monoid::AddMonoid>(&initial_data);
            // When, Then
            for i in 0..=n {
                for j in i..=n {
                    assert_eq!(naive.fold(i, j), sut.fold(i, j));
                }
            }
        }

        /// Scenario: 空区間 (`l >= r`) を畳み込むと単位元を返す (境界値)。
        /// - Given: サイズ `5` の `sut` がある。
        /// - When: `(0, 0)`, `(3, 3)`, `(5, 5)` のいずれかの区間で `fold` を呼ぶ。
        /// - Then: いずれも単位元 (`AddMonoid::id()`) が返る。
        #[test]
        fn returns_identity_for_empty_range() {
            // Given
            let sut = SegmentTreeDense::<monoid::AddMonoid>::new(5);
            let cases = [(0_usize, 0_usize), (3, 3), (5, 5)];
            // When, Then
            for (l, r) in cases {
                assert_eq!(monoid::AddMonoid::id(), sut.fold(l, r));
            }
        }

        /// Scenario: 範囲外 (`r > len`) を指定するとパニックする (異常系)。
        /// - Given: サイズ `5` の `sut` がある。
        /// - When: `fold(0, 6)` を呼ぶ。
        /// - Then: パニックする。
        #[test]
        #[should_panic(expected = "index out of bounds")]
        fn panics_when_r_exceeds_len() {
            // Given
            let sut = SegmentTreeDense::<monoid::AddMonoid>::new(5);
            // When, Then (panic)
            let _ = sut.fold(0, 6);
        }
    }

    // update のテスト: 状態変化, および異常系を検証する。
    mod update {
        use super::*;

        /// Scenario: 値を更新すると, 対応する葉および祖先ノードに変更が反映される。
        /// - Given: `[1, 2, 3, 4, 5]` を持つ `sut` と, 同じデータの `naive` オラクルがある。
        /// - When: インデックス `2` の値を `10` に更新する。
        /// - Then: `get(2)` が `10` を返し, 更新後の `fold` の結果も `naive` と一致する。
        #[test]
        fn updates_leaf_and_propagates_to_ancestors() {
            // Given
            let initial_data = vec![1, 2, 3, 4, 5];
            let n = initial_data.len();
            let (mut sut, mut naive) = create_dense_and_naive::<monoid::AddMonoid>(&initial_data);
            // When
            sut.update(2, 10);
            naive.update(2, 10);
            // Then
            assert_eq!(10, sut.get(2));
            assert_eq!(naive.fold(0, n), sut.fold(0, n));
            assert_eq!(naive.fold(1, 4), sut.fold(1, 4));
        }

        /// Scenario: 範囲外のインデックスを指定するとパニックする (異常系)。
        /// - Given: サイズ `5` の `sut` がある。
        /// - When: `update(5, 1)` を呼ぶ。
        /// - Then: パニックする。
        #[test]
        #[should_panic(expected = "index out of bounds")]
        fn panics_when_index_out_of_bounds() {
            // Given
            let mut sut = SegmentTreeDense::<monoid::AddMonoid>::new(5);
            // When, Then (panic)
            sut.update(5, 1);
        }
    }

    // max_right のテスト: 戻り値を検証する。
    mod max_right {
        use super::*;

        /// Scenario: 開始位置と述語の組み合わせに応じて, 期待通りの最大 `r` を返す。
        /// - Given: `[1, 2, 3, 4, 5]` を持つ `sut` がある。
        /// - When: 複数の `(l, 述語)` の組で `max_right` を求める。
        /// - Then: 各ケースで期待する `r` が返る。
        #[test]
        fn returns_expected_r_for_various_predicates() {
            // Given
            let initial_data = vec![1, 2, 3, 4, 5];
            let n = initial_data.len();
            let sut = create_dense_tree::<monoid::AddMonoid>(&initial_data);
            // 述語の型を関数ポインタに統一するため、`as fn` で変換する。
            // 複雑な型注釈を避け、型推論で配列の型を定める。
            let cases = [
                // l=1 から総和が 10 未満: [1,4) = 2+3+4=9, [1,5)=14
                (1, (|&sum: &i64| sum < 10) as fn(&i64) -> bool, 4),
                // l=0 から総和が 6 以下: [0,3) = 1+2+3=6, [0,4)=10
                (0, (|&sum: &i64| sum <= 6) as fn(&i64) -> bool, 3),
                // 述語が常に true の場合は末尾まで伸びる。
                (0, (|&_sum: &i64| true) as fn(&i64) -> bool, n),
                // 述語が単位元に対してのみ true の場合は開始位置から動かない。
                (0, (|&sum: &i64| sum == 0) as fn(&i64) -> bool, 0),
                // 開始位置が末尾の場合は末尾がそのまま返る。
                (n, (|&_sum: &i64| true) as fn(&i64) -> bool, n),
            ];
            // When, Then
            for (l, f, expected) in cases {
                assert_eq!(expected, sut.max_right(l, f));
            }
        }
    }

    // min_left のテスト: 戻り値を検証する。
    mod min_left {
        use super::*;

        /// Scenario: 終了位置と述語の組み合わせに応じて, 期待通りの最小 `l` を返す。
        /// - Given: `[1, 2, 3, 4, 5]` を持つ `sut` がある。
        /// - When: 複数の `(r, 述語)` の組で `min_left` を求める。
        /// - Then: 各ケースで期待する `l` が返る。
        #[test]
        fn returns_expected_l_for_various_predicates() {
            // Given
            let initial_data = vec![1, 2, 3, 4, 5];
            let n = initial_data.len();
            let mut sut = create_dense_tree::<monoid::AddMonoid>(&initial_data);
            // 述語の型を関数ポインタに統一するため、`as fn` で変換する。
            // 複雑な型注釈を避け、型推論で配列の型を定める。
            let cases = [
                // r=4 まで総和が 10 未満: [1,4) = 2+3+4=9, [0,4)=10
                (4, (|&sum: &i64| sum < 10) as fn(&i64) -> bool, 1),
                // r=5 まで総和が 15 以下: [0,5) = 15
                (5, (|&sum: &i64| sum <= 15) as fn(&i64) -> bool, 0),
                // 述語が常に true の場合は先頭まで縮む。
                (n, (|&_sum: &i64| true) as fn(&i64) -> bool, 0),
                // 述語が単位元に対してのみ true の場合は終了位置から動かない。
                (n, (|&sum: &i64| sum == 0) as fn(&i64) -> bool, n),
                // 終了位置が先頭の場合は先頭がそのまま返る。
                (0, (|&_sum: &i64| true) as fn(&i64) -> bool, 0),
            ];
            // When, Then
            for (r, f, expected) in cases {
                assert_eq!(expected, sut.min_left(r, f));
            }
        }
    }

    // 非可換なモノイド (行列積) を用いたランダムテスト:
    // 愚直実装 (NaiveSegmentTree) との比較を通じて, 多数の操作列にわたる整合性を検証する。
    mod randomized_matrix_monoid {
        use super::*;
        use rand::{self, Rng};
        use std::mem;

        /// テスト用の 2x2 行列。
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        struct Matrix2x2 {
            mat: [[u64; 2]; 2],
        }

        /// 行列積を演算とするモノイド。 非可換な演算での挙動を検証するために用いる。
        #[derive(Clone)]
        struct MatrixMulMonoid;

        impl semi_group::SemiGroup for MatrixMulMonoid {
            type S = Matrix2x2;

            fn op(a: &Self::S, b: &Self::S) -> Self::S {
                let mut res = Matrix2x2 { mat: [[0; 2]; 2] };
                for i in 0..2 {
                    for j in 0..2 {
                        for k in 0..2 {
                            res.mat[i][j] = res.mat[i][j]
                                .saturating_add(a.mat[i][k].saturating_mul(b.mat[k][j]));
                        }
                    }
                }
                res
            }
        }

        impl Monoid for MatrixMulMonoid {
            fn id() -> Self::S {
                Matrix2x2 {
                    mat: [[1, 0], [0, 1]],
                }
            }
        }

        // ランダムな 2x2 行列を生成する。
        fn random_matrix(rng: &mut impl Rng) -> Matrix2x2 {
            Matrix2x2 {
                mat: [
                    [rng.random_range(1..=5), rng.random_range(1..=5)],
                    [rng.random_range(1..=5), rng.random_range(1..=5)],
                ],
            }
        }

        /// Scenario: 非可換なモノイド (行列積) に対しても, ランダムな操作列を通して
        /// 愚直実装と同じ結果を返し続ける。
        /// - Given: サイズ `N` のランダムな行列で初期化した `sut` と `naive` オラクルがある。
        /// - When: `update`, `fold`, `max_right`, `min_left` のいずれかをランダムに選び,
        ///   `Q` 回操作する。
        /// - Then: 操作のたびに `sut` と `naive` の結果が一致する。
        #[test]
        fn matches_naive_for_random_operations() {
            // Given
            const N: usize = 10000;
            const Q: usize = 10000;
            let mut rng = rand::rng();
            let initial_data = (0..N).map(|_| random_matrix(&mut rng)).collect::<Vec<_>>();
            let (mut sut, mut naive) = create_dense_and_naive::<MatrixMulMonoid>(&initial_data);

            // When, Then
            for _ in 0..Q {
                match rng.random_range(0..4) {
                    0 => {
                        // update の一致を検証する。
                        let idx = rng.random_range(0..N);
                        let val = random_matrix(&mut rng);
                        sut.update(idx, val);
                        naive.update(idx, val);
                        assert_eq!(naive.get(idx), sut.get(idx));
                    }
                    1 => {
                        // fold の一致を検証する。
                        let mut l = rng.random_range(0..=N);
                        let mut r = rng.random_range(0..=N);
                        if l > r {
                            mem::swap(&mut l, &mut r);
                        }
                        assert_eq!(naive.fold(l, r), sut.fold(l, r));
                    }
                    2 => {
                        // max_right の一致を検証する。
                        let l = rng.random_range(0..=N);
                        let threshold = 1_000_000;
                        let f = |m: &Matrix2x2| m.mat[0][0] < threshold;
                        assert_eq!(naive.max_right(l, f), sut.max_right(l, f));
                    }
                    3 => {
                        // min_left の一致を検証する。
                        let r = rng.random_range(0..=N);
                        let threshold = 1_000_000;
                        let f = |m: &Matrix2x2| m.mat[0][0] < threshold;
                        assert_eq!(naive.min_left(r, f), sut.min_left(r, f));
                    }
                    _ => unreachable!(),
                }
            }
        }
    }
}
