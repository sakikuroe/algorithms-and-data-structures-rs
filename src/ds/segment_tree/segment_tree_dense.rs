//! A segment tree data structure for range queries and updates.
//! `Segment tree` の実装であり, range query と更新をサポートする.

use super::super::super::algebra::monoid::Monoid;

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
    ///        `segment tree` のサイズ (葉の数).
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
    ///                          ここで `n` は `segment tree` のサイズである.
    /// - Space complexity: O(n), where `n` is the size of the segment tree.
    ///                           ここで `n` は `segment tree` のサイズである.
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

    /// Sets the value at index `idx` to `x`.
    /// The update is lazy; use `build` or `update` to propagate changes to parent nodes.
    /// インデックス `idx` の値を `x` にセットする.
    /// この更新は遅延実行されるため, 上位ノードへ変更を反映するには `build` または `update` を使用する.
    ///
    /// # Args
    /// - `idx`: The index to set.
    ///          セット対象のインデックス.
    /// - `x`: The new value.
    ///        新しい値.
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
    ///                          ここで `n` は `segment tree` のサイズである.
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
    ///          更新対象のインデックス.
    /// - `x`: The new value.
    ///        新しい値.
    ///
    /// # Panics
    /// Panics if `idx` >= `self.len()`.
    /// `idx` が `self.len()` 以上の場合にパニックする.
    ///
    /// # Complexity
    /// - Time complexity: O(log(n)), where `n` is the size of the segment tree.
    ///                               ここで `n` は `segment tree` のサイズである.
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
    ///          値を取得するインデックス.
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
    ///        `query` 区間の開始インデックス (含む).
    /// - `r`: The end index of the range (exclusive).
    ///        `query` 区間の終了インデックス (含まない).
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
    ///                               ここで `n` は `segment tree` のサイズである.
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
            if l % 2 == 0 {
                sum_l = M::op(&sum_l, &self.data[l]);
            }
            if r % 2 == 0 {
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
    ///        範囲の開始インデックス.
    /// - `f`: A function that takes a reference to `M::S` and returns a boolean.
    ///        `M::S` への参照を受け取り, 真偽値を返す関数.
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
    ///                               ここで `n` は `segment tree` のサイズである.
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
    ///        範囲の終了インデックス.
    /// - `f`: A function that takes a reference to `M::S` and returns a boolean.
    ///        `M::S` への参照を受け取り, 真偽値を返す関数.
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
    ///                               ここで `n` は `segment tree` のサイズである.
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
