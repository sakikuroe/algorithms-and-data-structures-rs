//! 更新位置を事前に指定する 1 次元の疎なセグメント木を提供する。
//!
//! 登録座標を圧縮し、2 の冪へ切り上げた配列木で集約する。
//! 区間クエリの端点は登録する必要がない。
//!
//! 更新する座標をあらかじめ列挙できる場合に使用する。論理上の区間は
//! `[0, len)` であり、未登録位置はモノイドの単位元を持つ。
//! `set` と `update` は登録済み座標だけに使える。`set` の後に
//! 区間集約や境界探索を行う場合は、先に `build` を呼ぶ。
//!
//! 異なる登録座標数を `K` とすると、一点操作、区間集約、境界探索は
//! `O(log K)` 時間、保持領域は `O(K)` である。`K = 0` の木も構築できる。

use super::super::super::algebra::monoid;

/// 更新対象の座標を構築時に登録する疎なセグメント木。
///
/// 論理上の区間は `[0, len)` であり、登録されていない位置は
/// `M::id()` として扱う。`set` した後は `build` で集約値を更新する。
///
/// # Examples
/// ```rust
/// use anmitsu::{algebra::monoid, ds::segment_tree::segment_tree_sparse_offline};
///
/// let mut seg = segment_tree_sparse_offline::SegmentTreeSparseOffline::<
///     monoid::AddMonoid,
/// >::new(1_000_000_000, [7, 999_999_999]);
/// seg.update(7, 3);
/// seg.update(999_999_999, 4);
/// assert_eq!(seg.fold(0, 1_000_000_000), 7);
/// ```
pub struct SegmentTreeSparseOffline<M>
where
    M: monoid::Monoid,
{
    len: usize,
    coordinates: Vec<usize>,
    size: usize,
    data: Vec<M::S>,
}

impl<M> SegmentTreeSparseOffline<M>
where
    M: monoid::Monoid,
    M::S: Clone,
{
    /// 更新対象の座標を登録し、すべて単位元の木を作成する。
    ///
    /// # Args
    /// - `len` - 扱う座標範囲 `[0, len)` の長さ。
    /// - `points` - 更新する可能性がある座標。順序や重複は問わない。
    ///
    /// # Returns
    /// 登録座標を圧縮した木を返す。
    ///
    /// # Panics
    /// 登録座標が `len` 以上の場合にパニックする。
    ///
    /// # Complexity
    /// 登録数を `P`、異なる登録座標数を `K` とすると、時間は
    /// $O(P \log P + K)$、空間は $O(K)$。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::{algebra::monoid, ds::segment_tree::segment_tree_sparse_offline};
    /// let seg = segment_tree_sparse_offline::SegmentTreeSparseOffline::<
    ///     monoid::AddMonoid,
    /// >::new(10, [7, 2, 7]);
    /// assert_eq!(seg.len(), 10);
    /// ```
    pub fn new(len: usize, points: impl IntoIterator<Item = usize>) -> Self {
        let mut coordinates = points.into_iter().collect::<Vec<_>>();
        assert!(
            coordinates.iter().all(|&idx| idx < len),
            "registered index out of bounds"
        );
        coordinates.sort_unstable();
        coordinates.dedup();

        // 葉を 2 の冪へ揃えると、各節点が連続する座標区間を表す。
        let size = coordinates.len().max(1).next_power_of_two();
        let capacity = size.checked_mul(2).expect("segment tree is too large");
        Self {
            len,
            coordinates,
            size,
            data: vec![M::id(); capacity],
        }
    }

    /// 論理上の区間長を返す。
    ///
    /// # Returns
    /// 構築時に指定した `len` を返す。
    ///
    /// # Complexity
    /// 時間・空間ともに $O(1)$。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::{algebra::monoid, ds::segment_tree::segment_tree_sparse_offline};
    /// let seg = segment_tree_sparse_offline::SegmentTreeSparseOffline::<
    ///     monoid::AddMonoid,
    /// >::new(8, [2]);
    /// assert_eq!(seg.len(), 8);
    /// ```
    #[must_use]
    pub fn len(&self) -> usize {
        self.len
    }

    /// 論理上の区間が空であるかを返す。
    ///
    /// # Returns
    /// `len == 0` のときに `true` を返す。登録座標が 0 個でも
    /// `len > 0` なら `false` を返す。
    ///
    /// # Complexity
    /// 時間・空間ともに $O(1)$。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::{algebra::monoid, ds::segment_tree::segment_tree_sparse_offline};
    /// let seg = segment_tree_sparse_offline::SegmentTreeSparseOffline::<
    ///     monoid::AddMonoid,
    /// >::new(0, []);
    /// assert!(seg.is_empty());
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// 登録済み座標の葉を設定し、祖先の集約は `build` に委ねる。
    ///
    /// # Args
    /// - `idx` - 登録済みの座標。
    /// - `x` - 新しい値。
    ///
    /// # Panics
    /// `idx` が範囲外、または未登録の場合にパニックする。
    ///
    /// # Complexity
    /// 異なる登録座標数を `K` とすると、時間 $O(\log K)$、
    /// 追加領域 $O(1)$。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::{algebra::monoid, ds::segment_tree::segment_tree_sparse_offline};
    /// let mut seg = segment_tree_sparse_offline::SegmentTreeSparseOffline::<
    ///     monoid::AddMonoid,
    /// >::new(10, [3]);
    /// seg.set(3, 7);
    /// seg.build();
    /// assert_eq!(seg.fold(0, 10), 7);
    /// ```
    pub fn set(&mut self, idx: usize, x: M::S) {
        assert!(idx < self.len, "index out of bounds");
        let rank = self
            .coordinates
            .binary_search(&idx)
            .expect("index is not registered");
        self.data[self.size + rank] = x;
    }

    /// 葉の値を親へ集約する。
    ///
    /// # Complexity
    /// 異なる登録座標数を `K` とすると、時間 $O(K)$、
    /// 追加領域 $O(1)$。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::{algebra::monoid, ds::segment_tree::segment_tree_sparse_offline};
    /// let mut seg = segment_tree_sparse_offline::SegmentTreeSparseOffline::<
    ///     monoid::AddMonoid,
    /// >::new(10, [2, 7]);
    /// seg.set(2, 4);
    /// seg.set(7, 5);
    /// seg.build();
    /// assert_eq!(seg.fold(0, 10), 9);
    /// ```
    pub fn build(&mut self) {
        // 圧縮座標の葉から親へ、座標順を保って値を集約する。
        for node in (1..self.size).rev() {
            self.data[node] = M::op(&self.data[node * 2], &self.data[node * 2 + 1]);
        }
    }

    /// 登録済み座標の葉を変更し、祖先も直ちに更新する。
    ///
    /// # Args
    /// - `idx` - 登録済みの座標。
    /// - `x` - 新しい値。
    ///
    /// # Panics
    /// `idx` が範囲外、または未登録の場合にパニックする。
    ///
    /// # Complexity
    /// 異なる登録座標数を `K` とすると、時間 $O(\log K)$、
    /// 追加領域 $O(1)$。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::{algebra::monoid, ds::segment_tree::segment_tree_sparse_offline};
    /// let mut seg = segment_tree_sparse_offline::SegmentTreeSparseOffline::<
    ///     monoid::AddMonoid,
    /// >::new(10, [3]);
    /// seg.update(3, 7);
    /// assert_eq!(seg.fold(0, 10), 7);
    /// ```
    pub fn update(&mut self, idx: usize, x: M::S) {
        assert!(idx < self.len, "index out of bounds");
        let rank = self
            .coordinates
            .binary_search(&idx)
            .expect("index is not registered");
        let mut node = self.size + rank;
        self.data[node] = x;
        // 葉から根へ進み、変更を受ける祖先だけ再集約する。
        while node > 1 {
            node >>= 1;
            self.data[node] = M::op(&self.data[node * 2], &self.data[node * 2 + 1]);
        }
    }

    /// 一点の値を取得する。
    ///
    /// # Args
    /// - `idx` - 取得対象の座標。
    ///
    /// # Returns
    /// 未登録位置では `M::id()`、登録済み位置では葉の値を返す。
    ///
    /// # Panics
    /// `idx >= len` の場合にパニックする。
    ///
    /// # Complexity
    /// 異なる登録座標数を `K` とすると、時間 $O(\log K)$、
    /// 追加領域 $O(1)$。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::{algebra::monoid, ds::segment_tree::segment_tree_sparse_offline};
    /// let seg = segment_tree_sparse_offline::SegmentTreeSparseOffline::<
    ///     monoid::AddMonoid,
    /// >::new(10, [3]);
    /// assert_eq!(seg.get(4), 0);
    /// ```
    pub fn get(&self, idx: usize) -> M::S {
        assert!(idx < self.len, "index out of bounds");
        self.coordinates
            .binary_search(&idx)
            .map(|rank| self.data[self.size + rank].clone())
            .unwrap_or_else(|_| M::id())
    }

    /// 半開区間 `[l, r)` の値を座標順に集約する。
    ///
    /// # Args
    /// - `l` - 区間の左端。登録不要である。
    /// - `r` - 区間の右端。登録不要である。
    ///
    /// # Returns
    /// 空区間では `M::id()` を返す。
    ///
    /// # Panics
    /// `l > r` または `r > len` の場合にパニックする。
    ///
    /// # Complexity
    /// 異なる登録座標数を `K` とすると、時間 $O(\log K)$、
    /// 追加領域 $O(1)$。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::{algebra::monoid, ds::segment_tree::segment_tree_sparse_offline};
    /// let mut seg = segment_tree_sparse_offline::SegmentTreeSparseOffline::<
    ///     monoid::AddMonoid,
    /// >::new(10, [3, 7]);
    /// seg.update(3, 4);
    /// seg.update(7, 5);
    /// assert_eq!(seg.fold(4, 8), 5);
    /// ```
    pub fn fold(&self, l: usize, r: usize) -> M::S {
        assert!(l <= r && r <= self.len, "range out of bounds");
        // 未登録座標を含む端点を、最初の登録座標の順位へ写す。
        let mut left = self.coordinates.partition_point(|&point| point < l) + self.size;
        let mut right = self.coordinates.partition_point(|&point| point < r) + self.size;
        let mut sum_left = M::id();
        let mut sum_right = M::id();
        while left < right {
            if left & 1 == 1 {
                sum_left = M::op(&sum_left, &self.data[left]);
                left += 1;
            }
            if right & 1 == 1 {
                right -= 1;
                sum_right = M::op(&self.data[right], &sum_right);
            }
            left >>= 1;
            right >>= 1;
        }
        M::op(&sum_left, &sum_right)
    }

    /// `[l, r)` の集約に対して `f` が真である最大の `r` を返す。
    ///
    /// # Args
    /// - `l` - 探索開始位置。登録不要である。
    /// - `f` - 単位元で真となり、右端を延ばすと真から偽にのみ変わる述語。
    ///
    /// # Returns
    /// 最初に条件を満たさなくなる登録座標を返す。最後まで真なら
    /// 論理上の `len` を返す。
    ///
    /// # Panics
    /// `l > len`、または `f(&M::id())` が偽の場合にパニックする。
    ///
    /// # Complexity
    /// 異なる登録座標数を `K` とすると、時間 $O(\log K)$、
    /// 追加領域 $O(1)$。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::{algebra::monoid, ds::segment_tree::segment_tree_sparse_offline};
    /// let mut seg = segment_tree_sparse_offline::SegmentTreeSparseOffline::<
    ///     monoid::AddMonoid,
    /// >::new(10, [3, 7]);
    /// seg.update(3, 4);
    /// assert_eq!(seg.max_right(0, |&sum| sum < 4), 3);
    /// ```
    pub fn max_right<F>(&self, l: usize, f: F) -> usize
    where
        F: Fn(&M::S) -> bool,
    {
        assert!(l <= self.len, "index out of bounds");
        assert!(f(&M::id()), "predicate must accept the identity");
        let rank = self.coordinates.partition_point(|&point| point < l);
        if rank == self.coordinates.len() {
            return self.len;
        }
        let mut node = rank + self.size;
        let mut sum = M::id();
        loop {
            while node & 1 == 0 {
                node >>= 1;
            }
            let next = M::op(&sum, &self.data[node]);
            if !f(&next) {
                while node < self.size {
                    node <<= 1;
                    let candidate = M::op(&sum, &self.data[node]);
                    if f(&candidate) {
                        sum = candidate;
                        node += 1;
                    }
                }
                return self.coordinates[node - self.size];
            }
            sum = next;
            node += 1;
            if node.is_power_of_two() {
                break;
            }
        }
        self.len
    }

    /// `[l, r)` の集約に対して `f` が真である最小の `l` を返す。
    ///
    /// # Args
    /// - `r` - 探索終了位置。登録不要である。
    /// - `f` - 単位元で真となり、左端を縮めると真から偽にのみ変わる述語。
    ///
    /// # Returns
    /// 最初に条件を満たさなくなる登録座標の直後を返す。
    /// 最後まで真なら `0` を返す。
    ///
    /// # Panics
    /// `r > len`、または `f(&M::id())` が偽の場合にパニックする。
    ///
    /// # Complexity
    /// 異なる登録座標数を `K` とすると、時間 $O(\log K)$、
    /// 追加領域 $O(1)$。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::{algebra::monoid, ds::segment_tree::segment_tree_sparse_offline};
    /// let mut seg = segment_tree_sparse_offline::SegmentTreeSparseOffline::<
    ///     monoid::AddMonoid,
    /// >::new(10, [3, 7]);
    /// seg.update(3, 4);
    /// assert_eq!(seg.min_left(10, |&sum| sum < 4), 4);
    /// ```
    pub fn min_left<F>(&self, r: usize, f: F) -> usize
    where
        F: Fn(&M::S) -> bool,
    {
        assert!(r <= self.len, "index out of bounds");
        assert!(f(&M::id()), "predicate must accept the identity");
        let rank = self.coordinates.partition_point(|&point| point < r);
        if rank == 0 {
            return 0;
        }
        let mut node = rank + self.size;
        let mut sum = M::id();
        loop {
            node -= 1;
            while node > 1 && node & 1 == 1 {
                node >>= 1;
            }
            let next = M::op(&self.data[node], &sum);
            if !f(&next) {
                while node < self.size {
                    node = 2 * node + 1;
                    let candidate = M::op(&self.data[node], &sum);
                    if f(&candidate) {
                        sum = candidate;
                        node -= 1;
                    }
                }
                let boundary_rank = node + 1 - self.size;
                return if boundary_rank == 0 {
                    0
                } else {
                    self.coordinates[boundary_rank - 1] + 1
                };
            }
            sum = next;
            if node.is_power_of_two() {
                break;
            }
        }
        0
    }
}

/// 座標圧縮、集約順序、境界条件を確認するテスト。
#[cfg(test)]
mod tests {
    use super::super::super::super::algebra::semi_group;
    use super::*;

    /// 文字列を左から右へ連結する非可換モノイド。
    struct ConcatMonoid;

    impl semi_group::SemiGroup for ConcatMonoid {
        type S = String;

        /// 左右の文字列を座標順に連結する。
        ///
        /// # Args
        /// - `a` - 左区間の文字列。
        /// - `b` - 右区間の文字列。
        ///
        /// # Returns
        /// `a` の後に `b` を連結した文字列を返す。
        fn op(a: &Self::S, b: &Self::S) -> Self::S {
            let mut result = String::with_capacity(a.len() + b.len());
            result.push_str(a);
            result.push_str(b);
            result
        }
    }

    impl monoid::Monoid for ConcatMonoid {
        /// 連結の単位元である空文字列を返す。
        ///
        /// # Returns
        /// 空文字列を返す。
        fn id() -> Self::S {
            String::new()
        }
    }

    /// 圧縮した葉が正しく集約されることを確認する。
    mod build {
        use super::*;

        /// Scenario: 未整列かつ重複した登録座標から構築できる。
        /// - Given: 座標 16 と 1 を重複を含めて登録した木がある。
        /// - When: 両座標を設定して構築する。
        /// - Then: 葉と全区間の値が設定どおりになる。
        #[test]
        fn sorts_deduplicates_and_aggregates() {
            // Given
            let mut sut = SegmentTreeSparseOffline::<monoid::AddMonoid>::new(17, [16, 1, 16]);
            // When
            sut.set(1, 2);
            sut.set(16, 3);
            sut.build();
            // Then
            assert_eq!(17, sut.len());
            assert!(!sut.is_empty());
            assert_eq!(2, sut.get(1));
            assert_eq!(0, sut.get(2));
            assert_eq!(5, sut.fold(0, 17));
            sut.update(16, 4);
            assert_eq!(6, sut.fold(0, 17));
        }
    }

    /// 区間集約と境界探索が愚直な配列と一致することを確認する。
    mod queries {
        use super::*;

        /// Scenario: 登録座標の隙間を含むすべての区間と境界が一致する。
        /// - Given: 長さ 17 の木に 5 座標を登録している。
        /// - When: 登録済み座標を繰り返し更新する。
        /// - Then: 区間和と単調な境界探索が愚直な配列に一致する。
        #[test]
        fn matches_naive_ranges_and_boundaries() {
            // Given
            let mut sut = SegmentTreeSparseOffline::<monoid::AddMonoid>::new(17, [16, 1, 9, 4, 7]);
            let mut values = [0_i64; 17];
            // When
            for (idx, value) in [(1, 2), (9, 3), (16, 4), (1, 5), (9, 0)] {
                sut.update(idx, value);
                values[idx] = value;
                // Then
                for l in 0..=values.len() {
                    for r in l..=values.len() {
                        assert_eq!(values[l..r].iter().sum::<i64>(), sut.fold(l, r));
                    }
                    for limit in [1, 3, 5, 8, 12] {
                        let mut expected = l;
                        let mut sum = 0;
                        while expected < values.len() && sum + values[expected] < limit {
                            sum += values[expected];
                            expected += 1;
                        }
                        assert_eq!(expected, sut.max_right(l, |&x| x < limit));
                    }
                }
                for r in 0..=values.len() {
                    for limit in [1, 3, 5, 8, 12] {
                        let mut expected = r;
                        let mut sum = 0;
                        while expected > 0 && sum + values[expected - 1] < limit {
                            expected -= 1;
                            sum += values[expected];
                        }
                        assert_eq!(expected, sut.min_left(r, |&x| x < limit));
                    }
                }
            }
        }

        /// Scenario: 非可換モノイドでも元の座標順に集約される。
        /// - Given: 3 つの離れた座標に文字列を登録した木がある。
        /// - When: 区間集約と左右の境界探索を行う。
        /// - Then: 連結順と元の座標での境界が一致する。
        #[test]
        fn preserves_order_for_noncommutative_monoid() {
            // Given
            let mut sut = SegmentTreeSparseOffline::<ConcatMonoid>::new(10, [7, 1, 4]);
            sut.set(1, "a".to_owned());
            sut.set(4, "b".to_owned());
            sut.set(7, "c".to_owned());
            sut.build();
            // When
            let all = sut.fold(0, 10);
            let suffix = sut.fold(2, 8);
            let right = sut.max_right(0, |s| "ab".starts_with(s));
            let left = sut.min_left(10, |s| "bc".ends_with(s));
            // Then
            assert_eq!("abc", all);
            assert_eq!("bc", suffix);
            assert_eq!(7, right);
            assert_eq!(2, left);
        }
    }

    /// 登録座標がない場合も扱えることを確認する。
    mod empty {
        use super::*;

        /// Scenario: 長さ 0 の木でも空区間と境界探索が使える。
        /// - Given: 長さ 0 で登録座標のない木がある。
        /// - When: 空区間と両方向の境界を調べる。
        /// - Then: 単位元と 0 が返る。
        #[test]
        fn handles_zero_length() {
            // Given
            let mut sut = SegmentTreeSparseOffline::<monoid::AddMonoid>::new(0, []);
            sut.build();
            // When
            let fold = sut.fold(0, 0);
            let right = sut.max_right(0, |&sum| sum == 0);
            let left = sut.min_left(0, |&sum| sum == 0);
            // Then
            assert!(sut.is_empty());
            assert_eq!(0, fold);
            assert_eq!(0, right);
            assert_eq!(0, left);
        }

        /// Scenario: 登録座標がなくても論理上の長さが維持される。
        /// - Given: 長さ 10 で登録座標のない木がある。
        /// - When: 値と境界を調べる。
        /// - Then: 未登録位置は単位元で、境界は元の区間端になる。
        #[test]
        fn handles_no_registered_points() {
            // Given
            let sut = SegmentTreeSparseOffline::<monoid::AddMonoid>::new(10, []);
            // When
            let point = sut.get(4);
            let fold = sut.fold(2, 9);
            let right = sut.max_right(2, |&sum| sum == 0);
            let left = sut.min_left(9, |&sum| sum == 0);
            // Then
            assert!(!sut.is_empty());
            assert_eq!(0, point);
            assert_eq!(0, fold);
            assert_eq!(10, right);
            assert_eq!(0, left);
        }

        /// Scenario: 最大の `usize` を区間長として扱える。
        /// - Given: 最後の座標だけを登録した木がある。
        /// - When: その座標を更新する。
        /// - Then: 境界を元の座標で取得できる。
        #[test]
        fn handles_maximum_coordinate_domain() {
            // Given
            let mut sut =
                SegmentTreeSparseOffline::<monoid::AddMonoid>::new(usize::MAX, [usize::MAX - 1]);
            // When
            sut.update(usize::MAX - 1, 1);
            // Then
            assert_eq!(1, sut.get(usize::MAX - 1));
            assert_eq!(1, sut.fold(usize::MAX - 1, usize::MAX));
            assert_eq!(usize::MAX - 1, sut.max_right(0, |&sum| sum < 1));
            assert_eq!(usize::MAX, sut.min_left(usize::MAX, |&sum| sum < 1));
        }
    }

    /// 未登録位置と範囲外を受け付けないことを確認する。
    mod bounds {
        use super::*;

        /// Scenario: 未登録位置への更新は拒否される。
        /// - Given: 座標 1 だけを登録した木がある。
        /// - When: 座標 2 を更新する。
        /// - Then: 未登録としてパニックする。
        #[test]
        #[should_panic(expected = "index is not registered")]
        fn rejects_unregistered_update() {
            // Given
            let mut sut = SegmentTreeSparseOffline::<monoid::AddMonoid>::new(3, [1]);
            // When
            sut.update(2, 1);
            // Then
        }

        /// Scenario: 範囲外の登録座標は拒否される。
        /// - Given: 長さ 3 を指定する。
        /// - When: 座標 3 を登録する。
        /// - Then: 範囲外としてパニックする。
        #[test]
        #[should_panic(expected = "registered index out of bounds")]
        fn rejects_out_of_bounds_registration() {
            // Given
            let len = 3;
            // When
            let _sut = SegmentTreeSparseOffline::<monoid::AddMonoid>::new(len, [3]);
            // Then
        }

        /// Scenario: 逆向きの区間は拒否される。
        /// - Given: 長さ 3 の木がある。
        /// - When: `[2, 1)` を集約する。
        /// - Then: 範囲外としてパニックする。
        #[test]
        #[should_panic(expected = "range out of bounds")]
        fn rejects_reversed_range() {
            // Given
            let sut = SegmentTreeSparseOffline::<monoid::AddMonoid>::new(3, [1]);
            // When
            sut.fold(2, 1);
            // Then
        }
    }
}
