//! A segment tree data structure for range queries and updates.
//! Segment tree の実装であり、レンジクエリと更新をサポートする。

use super::super::super::algebra::monoid::Monoid;

/// A dense segment tree that supports range queries and updates.
/// 密なセグメント木であり、レンジクエリと更新をサポートする。
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
    /// `n` 個の要素に対応する `SegmentTreeDense` を生成する。
    ///
    /// # Args
    /// - `n`: The size (number of leaves) of the segment tree.
    ///        セグメント木のサイズ (葉の数)。
    ///
    /// # Returns
    /// `SegmentTreeDense<M>`: Returns a newly created segment tree instance.
    ///                        新しいセグメント木のインスタンスを返す。
    ///
    /// # Panics
    /// If `n == 0`, the behavior is not well-defined.
    /// `n == 0` の場合、この実装では未定義の動作となる可能性がある。
    ///
    /// # Complexity
    /// - Time complexity: O(n),
    ///                    O(n)、ここで n はセグメント木のサイズ。
    /// - Space complexity: O(n),
    ///                     O(n)、ここで n はセグメント木のサイズ。
    ///
    /// # Examples
    /// ```rust
    /// // Pseudocode usage example:
    /// // use your_crate::SegmentTreeDense;
    /// // use your_crate::algebra::monoid::SomeMonoid;
    ///
    /// // let seg = SegmentTreeDense::<SomeMonoid>::new(5);
    /// // assert_eq!(seg.len(), 5);
    /// ```
    pub fn new(n: usize) -> Self {
        let len = n;
        // If n == 0, 2*len - 1 becomes negative, which is invalid for a Vec.
        // We do not handle n == 0 properly, so the user must ensure n > 0.
        SegmentTreeDense::<M> {
            len,
            data: vec![M::id(); 2 * len - 1],
        }
    }

    /// Returns the size of this segment tree.
    /// このセグメント木のサイズを返す。
    ///
    /// # Returns
    /// `usize`: The size of the segment tree.
    ///          セグメント木のサイズ。
    ///
    /// # Panics
    /// This function does not panic.
    /// この関数はパニックしない。
    ///
    /// # Complexity
    /// - Time complexity: O(1),
    ///                    O(1)。
    /// - Space complexity: O(1),
    ///                     O(1)。
    ///
    /// # Examples
    /// ```rust
    /// // let seg = SegmentTreeDense::<SomeMonoid>::new(5);
    /// // assert_eq!(seg.len(), 5);
    /// ```
    pub fn len(&self) -> usize {
        self.len
    }

    /// Sets the value at index `idx` to `x`. The update is lazy; use `build` or `update`
    /// if you want to propagate changes to upper nodes.
    /// インデックス `idx` の値を `x` にセットする。更新は遅延されるため、
    /// 上位ノードへの反映には `build` や `update` を使用する。
    ///
    /// # Args
    /// - `idx`: The index to set.
    ///          セット対象のインデックス。
    /// - `x`: The new value.
    ///        新しい値。
    ///
    /// # Returns
    /// This function does not return anything.
    /// この関数は何も返さない。
    ///
    /// # Panics
    /// Panics if `idx` >= `self.len()`.
    /// `idx` が `self.len()` 以上の場合パニックする。
    ///
    /// # Complexity
    /// - Time complexity: O(log(n)),
    ///                    O(log(n))、ここで n はセグメント木のサイズ。
    /// - Space complexity: O(1).
    ///
    /// # Examples
    /// ```rust
    /// // let mut seg = SegmentTreeDense::<SomeMonoid>::new(5);
    /// // seg.set(0, SomeMonoid::S::default());
    /// ```
    pub fn set(&mut self, mut idx: usize, x: M::S) {
        idx += self.len - 1;
        self.data[idx] = x;
    }

    /// Builds the segment tree by propagating the leaves' values up to their parent nodes.
    /// 葉に設定された値を親ノードへ伝播して、セグメント木を構築する。
    ///
    /// # Args
    /// None.
    /// 引数はない。
    ///
    /// # Returns
    /// This function does not return anything.
    /// この関数は何も返さない。
    ///
    /// # Panics
    /// This function does not panic.
    /// この関数はパニックしない。
    ///
    /// # Complexity
    /// - Time complexity: O(n),
    ///                    O(n)、ここで n はセグメント木のサイズ。
    /// - Space complexity: O(1).
    ///
    /// # Examples
    /// ```rust
    /// // let mut seg = SegmentTreeDense::<SomeMonoid>::new(5);
    /// // // Use set(...) as needed
    /// // seg.build();
    /// ```
    pub fn build(&mut self) {
        for idx in (0..self.len - 1).rev() {
            self.data[idx] = M::op(&self.data[2 * idx + 1], &self.data[2 * idx + 2]);
        }
    }

    /// Updates the value at index `idx` to `x` and propagates this change up the tree.
    /// インデックス `idx` の値を `x` に更新し、上位ノードへ反映する。
    ///
    /// # Args
    /// - `idx`: The index to update.
    ///          更新対象のインデックス。
    /// - `x`: The new value.
    ///        新しい値。
    ///
    /// # Returns
    /// This function does not return anything.
    /// この関数は何も返さない。
    ///
    /// # Panics
    /// Panics if `idx` >= `self.len()`.
    /// `idx` が `self.len()` 以上の場合パニックする。
    ///
    /// # Complexity
    /// - Time complexity: O(log(n)),
    ///                    O(log(n))、ここで n はセグメント木のサイズ。
    /// - Space complexity: O(1).
    ///
    /// # Examples
    /// ```rust
    /// // let mut seg = SegmentTreeDense::<SomeMonoid>::new(5);
    /// // seg.update(0, SomeMonoid::S::default());
    /// ```
    pub fn update(&mut self, mut idx: usize, x: M::S) {
        idx += self.len - 1;
        self.data[idx] = x;
        // Climb up the tree updating parent nodes.
        while idx > 0 {
            idx = (idx - 1) / 2;
            self.data[idx] = M::op(&self.data[2 * idx + 1], &self.data[2 * idx + 2]);
        }
    }

    /// Gets the value at index `idx`.
    /// インデックス `idx` の値を取得する。
    ///
    /// # Args
    /// - `idx`: The index to retrieve.
    ///          値を取得するインデックス。
    ///
    /// # Returns
    /// `M::S`: The value at `idx`.
    ///         `idx` の値。
    ///
    /// # Panics
    /// Panics if `idx` >= `self.len()`.
    /// `idx` が `self.len()` 以上の場合パニックする。
    ///
    /// # Complexity
    /// - Time complexity: O(1),
    ///                    O(1)。
    /// - Space complexity: O(1).
    ///
    /// # Examples
    /// ```rust
    /// // let seg = SegmentTreeDense::<SomeMonoid>::new(5);
    /// // let val = seg.get(0);
    /// ```
    pub fn get(&self, mut idx: usize) -> M::S {
        idx += self.len - 1;
        self.data[idx].clone()
    }

    /// Performs a range fold (query) on the interval `[l, r)`.
    /// 区間 `[l, r)` 上の値をまとめ (fold) するクエリを実行する。
    ///
    /// # Args
    /// - `l`: The start index of the range (inclusive).
    ///        クエリ区間の開始インデックス (含む)。
    /// - `r`: The end index of the range (exclusive).
    ///        クエリ区間の終了インデックス (含まない)。
    ///
    /// # Returns
    /// `M::S`: The folded result of the interval `[l, r)`.
    ///         区間 `[l, r)` の畳み込み結果。
    ///
    /// # Panics
    /// Panics if `l > r` or `r > self.len()`.
    /// `l > r` または `r > self.len()` の場合パニックする。
    ///
    /// # Complexity
    /// - Time complexity: O(log(n)),
    ///                    O(log(n))、ここで n はセグメント木のサイズ。
    /// - Space complexity: O(1).
    ///
    /// # Examples
    /// ```rust
    /// // let mut seg = SegmentTreeDense::<SomeMonoid>::new(5);
    /// // // Suppose we have built the tree with some values...
    /// // let answer = seg.fold(0, 5);
    /// ```
    pub fn fold(&self, mut l: usize, r: usize) -> M::S {
        let mut r = r.min(self.len());
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

    /// Finds the maximum `r` in `[l, self.len()]` such that `f` applied to the fold result
    /// from `[l, r)` is `true`. Returns `self.len()` if no further extension is possible.
    /// 区間 `[l, self.len()]` の中で、[`l`, `r`) の fold 結果に対して `f` が `true` を返す最大の `r`
    /// を探す。区間をこれ以上伸ばせない場合は `self.len()` を返す。
    ///
    /// # Args
    /// - `l`: The start index of the range.
    ///        範囲の開始インデックス。
    /// - `f`: A function that takes a reference to `M::S` and returns a boolean.
    ///        `M::S` への参照を受け取り、真偽値を返す関数。
    ///
    /// # Returns
    /// `usize`: The maximum `r` such that `f(fold(l, r))` is `true`.
    ///          `f(fold(l, r))` が `true` となる最大の `r`。
    ///
    /// # Panics
    /// Panics if `f(&M::id())` is false or `l > self.len()`.
    /// `f(&M::id())` が false の場合、または `l > self.len()` の場合にパニックする。
    ///
    /// # Complexity
    /// - Time complexity: O(log(n)),
    ///                    O(log(n))、ここで n はセグメント木のサイズ。
    /// - Space complexity: O(1).
    ///
    /// # Examples
    /// ```rust
    /// // let seg = SegmentTreeDense::<SomeMonoid>::new(5);
    /// // // Suppose f checks if the segment sum is <= some_value.
    /// // let idx = seg.max_right(0, |val| val <= &SomeMonoid::threshold());
    /// ```
    pub fn max_right<F>(&self, mut l: usize, f: F) -> usize
    where
        F: Fn(&M::S) -> bool,
    {
        fn is_good_node(k: usize, len: usize) -> bool {
            // Check if k is outside of leaf index or satisfies a particular condition
            if k >= len {
                true
            } else {
                let d = k.leading_zeros() - len.leading_zeros();
                len >> d != k || len >> d << d == len
            }
        }

        assert!(f(&M::id()));
        assert!(l <= self.len());

        // If `[l, self.len()]` satisfies f, return self.len().
        if l == self.len() || f(&self.fold(l, self.len())) {
            return self.len();
        }

        l += self.len();
        let mut sum = M::id();
        // Iteratively push the boundary to the right until we find the maximum r.
        loop {
            while l & 1 == 0 && is_good_node(l >> 1, self.len()) {
                l >>= 1;
            }
            if !f(&M::op(&sum, &self.data[l - 1])) {
                while l < self.len() {
                    l <<= 1;
                    let t = M::op(&sum, &self.data[l - 1]);
                    if f(&t) {
                        sum = t;
                        l += 1;
                    }
                }
                return l - self.len();
            }
            sum = M::op(&sum, &self.data[l - 1]);
            l += 1;
        }
    }

    /// Finds the minimum `l` in `[0, r]` such that `f` applied to the fold result
    /// from `[l, r)` is `true`. Returns `0` if it cannot move further left.
    /// 区間 `[0, r]` の中で、[`l`, `r`) の fold 結果に対して `f` が `true` を返す最小の `l`
    /// を探す。これ以上左に伸ばせない場合は `0` を返す。
    ///
    /// # Args
    /// - `r`: The end index of the range.
    ///        範囲の終了インデックス。
    /// - `f`: A function that takes a reference to `M::S` and returns a boolean.
    ///        `M::S` への参照を受け取り、真偽値を返す関数。
    ///
    /// # Returns
    /// `usize`: The minimum `l` such that `f(fold(l, r))` is `true`.
    ///          `f(fold(l, r))` が `true` となる最小の `l`。
    ///
    /// # Panics
    /// Panics if `f(&M::id())` is false or `r > self.len()`.
    /// `f(&M::id())` が false の場合、または `r > self.len()` の場合にパニックする。
    ///
    /// # Complexity
    /// - Time complexity: O(log(n)),
    ///                    O(log(n))、ここで n はセグメント木のサイズ。
    /// - Space complexity: O(1).
    ///
    /// # Examples
    /// ```rust
    /// // let seg = SegmentTreeDense::<SomeMonoid>::new(5);
    /// // // Suppose f checks if the segment sum is <= some_value.
    /// // let idx = seg.min_left(5, |val| val <= &SomeMonoid::threshold());
    /// ```
    pub fn min_left<L>(&mut self, mut r: usize, f: L) -> usize
    where
        L: Fn(&M::S) -> bool,
    {
        fn is_good_node(k: usize, len: usize) -> bool {
            // Similar check to ensure traversal correctness
            if k >= len {
                true
            } else {
                let d = k.leading_zeros() - len.leading_zeros();
                len >> d != k || len >> d << d == len
            }
        }

        assert!(f(&M::id()));
        assert!(r <= self.len());

        // If `[0, r)` satisfies f, return 0.
        if r == 0 || f(&self.fold(0, r)) {
            return 0;
        }

        let mut sum = M::id();
        r += self.len;
        // Iteratively shrink the boundary from the right.
        loop {
            r -= 1;
            while !is_good_node(r, self.len()) {
                r = r * 2 + 1;
            }
            while r & 1 != 0 && is_good_node(r >> 1, self.len()) {
                r >>= 1;
            }
            if !f(&M::op(&sum, &self.data[r - 1])) {
                while r < self.len {
                    r = r * 2 + 1;
                    let t = M::op(&sum, &self.data[r - 1]);
                    if f(&t) {
                        sum = t;
                        r -= 1;
                    }
                }
                return r + 1 - self.len;
            }
            sum = M::op(&sum, &self.data[r - 1]);
        }
    }
}
