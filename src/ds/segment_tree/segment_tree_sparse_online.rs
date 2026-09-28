//! 更新位置を事前に決めない 1 次元の疎なセグメント木を提供する。
//!
//! 節点を必要になったときだけ配列に追加する。未生成の部分木は
//! モノイドの単位元で埋まっているものとして扱う。
//!
//! 更新する座標を構築時に列挙できない場合に使用する。論理上の区間は
//! `[0, len)` であり、更新していない位置は単位元を持つ。
//! `set` は葉だけを変更するため、区間集約や境界探索の前に `build` が必要である。
//! `update` は変更を直ちに祖先へ反映する。
//!
//! 一点操作、区間集約、境界探索は `O(log len)` 時間で動作する。
//! 更新した異なる位置が `T` 個のとき、生成される節点は最大
//! `O(T log len)` 個である。`len = 0` の木も構築できる。

use super::super::super::algebra::monoid;

/// 子がまだ生成されていないことを表す。
const NONE: usize = usize::MAX;

/// 区間の集約値と、左右の子の配列内での位置を保持する。
struct Node<S> {
    /// この節点が担当する区間の集約値。
    value: S,
    /// 左右の子の添字。未生成側には `NONE` を入れる。
    children: [usize; 2],
}

/// 更新位置を事前に指定せずに使用できる疎なセグメント木。
///
/// 論理上の区間は `[0, len)` であり、未設定位置の値は `M::id()` である。
/// `set` は葉だけを変更するため、区間集約や境界探索の前に `build` を呼ぶ。
/// `update` は祖先まで直ちに更新する。
///
/// # Examples
/// ```rust
/// use anmitsu::{algebra::monoid, ds::segment_tree::segment_tree_sparse_online};
///
/// let mut seg = segment_tree_sparse_online::SegmentTreeSparseOnline::<
///     monoid::AddMonoid,
/// >::new(1_000_000_000);
/// seg.update(7, 3);
/// seg.update(999_999_999, 4);
/// assert_eq!(seg.fold(0, 1_000_000_000), 7);
/// ```
pub struct SegmentTreeSparseOnline<M>
where
    M: monoid::Monoid,
{
    len: usize,
    nodes: Vec<Node<M::S>>,
}

impl<M> SegmentTreeSparseOnline<M>
where
    M: monoid::Monoid,
    M::S: Clone,
{
    /// 長さ `len` の木を作成する。
    ///
    /// # Args
    /// - `len` - 扱う座標範囲 `[0, len)` の長さ。
    ///
    /// # Returns
    /// すべての位置が単位元である木を返す。
    ///
    /// # Complexity
    /// 時間・空間ともに $O(1)$。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::{algebra::monoid, ds::segment_tree::segment_tree_sparse_online};
    /// let seg = segment_tree_sparse_online::SegmentTreeSparseOnline::<
    ///     monoid::AddMonoid,
    /// >::new(10);
    /// assert_eq!(seg.len(), 10);
    /// ```
    pub fn new(len: usize) -> Self {
        Self {
            len,
            nodes: vec![Node {
                value: M::id(),
                children: [NONE; 2],
            }],
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
    /// use anmitsu::{algebra::monoid, ds::segment_tree::segment_tree_sparse_online};
    /// let seg = segment_tree_sparse_online::SegmentTreeSparseOnline::<
    ///     monoid::AddMonoid,
    /// >::new(5);
    /// assert_eq!(seg.len(), 5);
    /// ```
    #[must_use]
    pub fn len(&self) -> usize {
        self.len
    }

    /// 論理上の区間が空であるかを返す。
    ///
    /// # Returns
    /// `len == 0` のときに `true` を返す。
    ///
    /// # Complexity
    /// 時間・空間ともに $O(1)$。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::{algebra::monoid, ds::segment_tree::segment_tree_sparse_online};
    /// let seg = segment_tree_sparse_online::SegmentTreeSparseOnline::<
    ///     monoid::AddMonoid,
    /// >::new(0);
    /// assert!(seg.is_empty());
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// 葉の値を設定し、祖先の集約は後続の `build` に委ねる。
    ///
    /// # Args
    /// - `idx` - 設定対象の座標。
    /// - `x` - 新しい値。
    ///
    /// # Panics
    /// `idx >= len` の場合にパニックする。
    ///
    /// # Complexity
    /// 時間・追加領域ともに $O(\log len)$。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::{algebra::monoid, ds::segment_tree::segment_tree_sparse_online};
    /// let mut seg = segment_tree_sparse_online::SegmentTreeSparseOnline::<
    ///     monoid::AddMonoid,
    /// >::new(10);
    /// seg.set(3, 7);
    /// seg.build();
    /// assert_eq!(seg.fold(0, 10), 7);
    /// ```
    pub fn set(&mut self, idx: usize, x: M::S) {
        assert!(idx < self.len, "index out of bounds");
        let mut path = [0; usize::BITS as usize];
        let (leaf, _) = self.locate_or_create(idx, &mut path);
        self.nodes[leaf].value = x;
    }

    /// 生成済みの節点を葉側から再集約する。
    ///
    /// # Complexity
    /// 時間 $O(N)$、追加領域 $O(1)$。`N` は生成済み節点数である。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::{algebra::monoid, ds::segment_tree::segment_tree_sparse_online};
    /// let mut seg = segment_tree_sparse_online::SegmentTreeSparseOnline::<
    ///     monoid::AddMonoid,
    /// >::new(10);
    /// seg.set(2, 4);
    /// seg.set(7, 5);
    /// seg.build();
    /// assert_eq!(seg.fold(0, 10), 9);
    /// ```
    pub fn build(&mut self) {
        // 子は必ず親より後に追加されるため、逆順に走査すれば底から集約できる。
        for idx in (0..self.nodes.len()).rev() {
            if self.nodes[idx].children != [NONE; 2] {
                self.pull(idx);
            }
        }
    }

    /// 葉の値を変更し、祖先の集約値も直ちに更新する。
    ///
    /// # Args
    /// - `idx` - 更新対象の座標。
    /// - `x` - 新しい値。
    ///
    /// # Panics
    /// `idx >= len` の場合にパニックする。
    ///
    /// # Complexity
    /// 時間・追加領域ともに $O(\log len)$。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::{algebra::monoid, ds::segment_tree::segment_tree_sparse_online};
    /// let mut seg = segment_tree_sparse_online::SegmentTreeSparseOnline::<
    ///     monoid::AddMonoid,
    /// >::new(10);
    /// seg.update(3, 7);
    /// assert_eq!(seg.fold(0, 10), 7);
    /// ```
    pub fn update(&mut self, idx: usize, x: M::S) {
        assert!(idx < self.len, "index out of bounds");
        let mut path = [0; usize::BITS as usize];
        let (leaf, depth) = self.locate_or_create(idx, &mut path);
        self.nodes[leaf].value = x;
        for &parent in path[..depth].iter().rev() {
            self.pull(parent);
        }
    }

    /// 一点の値を取得する。
    ///
    /// # Args
    /// - `idx` - 取得対象の座標。
    ///
    /// # Returns
    /// 未設定位置では `M::id()`、設定済み位置では葉の値を返す。
    ///
    /// # Panics
    /// `idx >= len` の場合にパニックする。
    ///
    /// # Complexity
    /// 時間 $O(\log len)$、追加領域 $O(1)$。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::{algebra::monoid, ds::segment_tree::segment_tree_sparse_online};
    /// let seg = segment_tree_sparse_online::SegmentTreeSparseOnline::<
    ///     monoid::AddMonoid,
    /// >::new(10);
    /// assert_eq!(seg.get(3), 0);
    /// ```
    pub fn get(&self, idx: usize) -> M::S {
        assert!(idx < self.len, "index out of bounds");
        let (mut node, mut lo, mut hi) = (0, 0, self.len);
        while hi - lo > 1 {
            let mid = lo + (hi - lo) / 2;
            let side = usize::from(idx >= mid);
            node = self.nodes[node].children[side];
            if node == NONE {
                return M::id();
            }
            if side == 0 {
                hi = mid;
            } else {
                lo = mid;
            }
        }
        self.nodes[node].value.clone()
    }

    /// 半開区間 `[l, r)` の値を座標順に集約する。
    ///
    /// # Args
    /// - `l` - 区間の左端。
    /// - `r` - 区間の右端。
    ///
    /// # Returns
    /// 空区間では `M::id()` を返す。
    ///
    /// # Panics
    /// `l > r` または `r > len` の場合にパニックする。
    ///
    /// # Complexity
    /// 時間 $O(\log len)$、再帰スタック $O(\log len)$。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::{algebra::monoid, ds::segment_tree::segment_tree_sparse_online};
    /// let mut seg = segment_tree_sparse_online::SegmentTreeSparseOnline::<
    ///     monoid::AddMonoid,
    /// >::new(10);
    /// seg.update(3, 7);
    /// assert_eq!(seg.fold(0, 3), 0);
    /// assert_eq!(seg.fold(3, 4), 7);
    /// ```
    pub fn fold(&self, l: usize, r: usize) -> M::S {
        assert!(l <= r && r <= self.len, "range out of bounds");
        self.fold_node(0, 0, self.len, l, r)
    }

    /// `[l, r)` の集約に対して `f` が真である最大の `r` を返す。
    ///
    /// # Args
    /// - `l` - 探索開始位置。
    /// - `f` - 単位元で真となり、右端を延ばすと真から偽にのみ変わる述語。
    ///
    /// # Returns
    /// 条件を満たす最大の右端を返す。最後まで真なら `len` を返す。
    ///
    /// # Panics
    /// `l > len`、または `f(&M::id())` が偽の場合にパニックする。
    ///
    /// # Complexity
    /// 時間・再帰スタックともに $O(\log len)$。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::{algebra::monoid, ds::segment_tree::segment_tree_sparse_online};
    /// let mut seg = segment_tree_sparse_online::SegmentTreeSparseOnline::<
    ///     monoid::AddMonoid,
    /// >::new(10);
    /// seg.update(3, 4);
    /// assert_eq!(seg.max_right(0, |&sum| sum < 4), 3);
    /// ```
    pub fn max_right<F>(&self, l: usize, f: F) -> usize
    where
        F: Fn(&M::S) -> bool,
    {
        assert!(l <= self.len, "index out of bounds");
        assert!(f(&M::id()), "predicate must accept the identity");
        let mut sum = M::id();
        self.max_right_node(0, 0, self.len, l, &f, &mut sum)
            .unwrap_or(self.len)
    }

    /// `[l, r)` の集約に対して `f` が真である最小の `l` を返す。
    ///
    /// # Args
    /// - `r` - 探索終了位置。
    /// - `f` - 単位元で真となり、左端を縮めると真から偽にのみ変わる述語。
    ///
    /// # Returns
    /// 条件を満たす最小の左端を返す。最後まで真なら `0` を返す。
    ///
    /// # Panics
    /// `r > len`、または `f(&M::id())` が偽の場合にパニックする。
    ///
    /// # Complexity
    /// 時間・再帰スタックともに $O(\log len)$。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::{algebra::monoid, ds::segment_tree::segment_tree_sparse_online};
    /// let mut seg = segment_tree_sparse_online::SegmentTreeSparseOnline::<
    ///     monoid::AddMonoid,
    /// >::new(10);
    /// seg.update(3, 4);
    /// assert_eq!(seg.min_left(10, |&sum| sum < 4), 4);
    /// ```
    pub fn min_left<F>(&self, r: usize, f: F) -> usize
    where
        F: Fn(&M::S) -> bool,
    {
        assert!(r <= self.len, "index out of bounds");
        assert!(f(&M::id()), "predicate must accept the identity");
        let mut sum = M::id();
        self.min_left_node(0, 0, self.len, r, &f, &mut sum)
            .unwrap_or(0)
    }

    /// 葉までの経路を必要に応じて作り、祖先を `path` に記録する。
    ///
    /// `set` と `update` が共有する節点生成の規則をここに集める。
    /// 新しい子は親より大きい添字で保存されるため、`build` は
    /// 節点を逆順に走査して親を集約できる。
    ///
    /// # Args
    /// - `idx` - 範囲内と確認済みの更新座標。
    /// - `path` - 祖先の添字を書き込む領域。最大深さ分の長さが必要。
    ///
    /// # Returns
    /// 葉の添字と、`path` に書き込んだ祖先の個数を返す。
    fn locate_or_create(
        &mut self,
        idx: usize,
        path: &mut [usize; usize::BITS as usize],
    ) -> (usize, usize) {
        let (mut node, mut lo, mut hi, mut depth) = (0, 0, self.len, 0);
        while hi - lo > 1 {
            let mid = lo + (hi - lo) / 2;
            let side = usize::from(idx >= mid);
            path[depth] = node;
            depth += 1;
            if self.nodes[node].children[side] == NONE {
                self.nodes[node].children[side] = self.nodes.len();
                self.nodes.push(Node {
                    value: M::id(),
                    children: [NONE; 2],
                });
            }
            node = self.nodes[node].children[side];
            if side == 0 {
                hi = mid;
            } else {
                lo = mid;
            }
        }
        (node, depth)
    }

    /// 子の集約値から親の集約値を計算する。
    ///
    /// 未生成の子は単位元とみなし、片方だけ存在するときは
    /// モノイド演算を省いて存在する子の値を複製する。
    ///
    /// # Args
    /// - `idx` - 集約する内部節点の添字。子の値は更新済みであること。
    fn pull(&mut self, idx: usize) {
        let [left, right] = self.nodes[idx].children;
        // 未生成側は単位元なので、片側だけなら演算を省略できる。
        self.nodes[idx].value = match (left, right) {
            (NONE, NONE) => M::id(),
            (NONE, right) => self.nodes[right].value.clone(),
            (left, NONE) => self.nodes[left].value.clone(),
            (left, right) => M::op(&self.nodes[left].value, &self.nodes[right].value),
        };
    }

    /// 必要な部分木だけを訪問し、区間集約を求める。
    ///
    /// # Args
    /// - `node` - `[lo, hi)` を担当する節点の添字。`NONE` は未生成部分木。
    /// - `lo`, `hi` - 現在の節点が担当する半開区間の端点。
    /// - `l`, `r` - 求める半開区間の端点。
    ///
    /// # Returns
    /// 両区間の共通部分の集約値を返す。未生成部分木では単位元を返す。
    fn fold_node(&self, node: usize, lo: usize, hi: usize, l: usize, r: usize) -> M::S {
        if node == NONE || hi <= l || r <= lo {
            return M::id();
        }
        if l <= lo && hi <= r {
            return self.nodes[node].value.clone();
        }
        let mid = lo + (hi - lo) / 2;
        if r <= mid {
            return self.fold_node(self.nodes[node].children[0], lo, mid, l, r);
        }
        if l >= mid {
            return self.fold_node(self.nodes[node].children[1], mid, hi, l, r);
        }
        let [left_child, right_child] = self.nodes[node].children;
        if left_child == NONE {
            return self.fold_node(right_child, mid, hi, l, r);
        }
        if right_child == NONE {
            return self.fold_node(left_child, lo, mid, l, r);
        }
        let left = self.fold_node(left_child, lo, mid, l, r);
        let right = self.fold_node(right_child, mid, hi, l, r);
        M::op(&left, &right)
    }

    /// 左から集約し、最初に述語が偽になる座標を探す。
    ///
    /// # Args
    /// - `node` - `[lo, hi)` を担当する節点の添字。`NONE` は未生成部分木。
    /// - `lo`, `hi` - 現在の節点が担当する半開区間の端点。
    /// - `l` - 探索を始める座標。この位置より左は集約しない。
    /// - `f` - 単位元を受け入れる単調な述語。
    /// - `sum` - 現在までに左から集約した値。訪問後の値に更新する。
    ///
    /// # Returns
    /// 失敗した葉の座標を返す。最後まで真なら `None` を返す。
    fn max_right_node<F>(
        &self,
        node: usize,
        lo: usize,
        hi: usize,
        l: usize,
        f: &F,
        sum: &mut M::S,
    ) -> Option<usize>
    where
        F: Fn(&M::S) -> bool,
    {
        if node == NONE || hi <= l {
            return None;
        }
        if l <= lo {
            let next = M::op(sum, &self.nodes[node].value);
            if f(&next) {
                *sum = next;
                return None;
            }
            if hi - lo == 1 {
                return Some(lo);
            }
        }
        let mid = lo + (hi - lo) / 2;
        self.max_right_node(self.nodes[node].children[0], lo, mid, l, f, sum)
            .or_else(|| self.max_right_node(self.nodes[node].children[1], mid, hi, l, f, sum))
    }

    /// 右から集約し、最初に述語が偽になる座標の直後を探す。
    ///
    /// # Args
    /// - `node` - `[lo, hi)` を担当する節点の添字。`NONE` は未生成部分木。
    /// - `lo`, `hi` - 現在の節点が担当する半開区間の端点。
    /// - `r` - 探索を終える座標。この位置以右は集約しない。
    /// - `f` - 単位元を受け入れる単調な述語。
    /// - `sum` - 現在までに右から集約した値。訪問後の値に更新する。
    ///
    /// # Returns
    /// 失敗した葉の直後の座標を返す。最後まで真なら `None` を返す。
    fn min_left_node<F>(
        &self,
        node: usize,
        lo: usize,
        hi: usize,
        r: usize,
        f: &F,
        sum: &mut M::S,
    ) -> Option<usize>
    where
        F: Fn(&M::S) -> bool,
    {
        if node == NONE || r <= lo {
            return None;
        }
        if hi <= r {
            let next = M::op(&self.nodes[node].value, sum);
            if f(&next) {
                *sum = next;
                return None;
            }
            if hi - lo == 1 {
                return Some(hi);
            }
        }
        let mid = lo + (hi - lo) / 2;
        self.min_left_node(self.nodes[node].children[1], mid, hi, r, f, sum)
            .or_else(|| self.min_left_node(self.nodes[node].children[0], lo, mid, r, f, sum))
    }
}

/// 疎な節点生成、集約順序、境界条件を確認するテスト。
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

    /// 葉の設定後に祖先が再集約されることを確認する。
    mod build {
        use super::*;

        /// Scenario: 離れた位置を設定してから構築すると区間集約に反映される。
        /// - Given: 長さ 17 の空の木がある。
        /// - When: 2 位置を設定して構築する。
        /// - Then: 葉と全区間の値が設定どおりになる。
        #[test]
        fn aggregates_set_values() {
            // Given
            let mut sut = SegmentTreeSparseOnline::<monoid::AddMonoid>::new(17);
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

        /// Scenario: 疎な位置への連続更新後もすべての区間と境界が一致する。
        /// - Given: 長さ 17 の空の木と愚直な配列がある。
        /// - When: 離れた位置に値を更新する。
        /// - Then: 区間和と単調な境界探索が配列の結果に一致する。
        #[test]
        fn matches_naive_ranges_and_boundaries() {
            // Given
            let mut sut = SegmentTreeSparseOnline::<monoid::AddMonoid>::new(17);
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

        /// Scenario: 非可換モノイドでも座標順に集約される。
        /// - Given: 3 つの離れた座標へ文字列を設定した木がある。
        /// - When: 区間集約と左右の境界探索を行う。
        /// - Then: 連結順と元の座標での境界が一致する。
        #[test]
        fn preserves_order_for_noncommutative_monoid() {
            // Given
            let mut sut = SegmentTreeSparseOnline::<ConcatMonoid>::new(10);
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

    /// 座標が存在しない場合の境界を確認する。
    mod empty {
        use super::*;

        /// Scenario: 長さ 0 の木でも空区間と境界探索が使える。
        /// - Given: 長さ 0 の木がある。
        /// - When: 空区間と両方向の境界を調べる。
        /// - Then: 単位元と 0 が返る。
        #[test]
        fn handles_zero_length() {
            // Given
            let mut sut = SegmentTreeSparseOnline::<monoid::AddMonoid>::new(0);
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

        /// Scenario: 最大の `usize` を区間長として扱える。
        /// - Given: 長さが `usize::MAX` の疎な木がある。
        /// - When: 最後の位置を更新する。
        /// - Then: 添字計算があふれず、集約と境界を取得できる。
        #[test]
        fn handles_maximum_coordinate_domain() {
            // Given
            let mut sut = SegmentTreeSparseOnline::<monoid::AddMonoid>::new(usize::MAX);
            // When
            sut.update(usize::MAX - 1, 1);
            // Then
            assert_eq!(1, sut.get(usize::MAX - 1));
            assert_eq!(1, sut.fold(usize::MAX - 1, usize::MAX));
            assert_eq!(usize::MAX - 1, sut.max_right(0, |&sum| sum < 1));
            assert_eq!(usize::MAX, sut.min_left(usize::MAX, |&sum| sum < 1));
        }
    }

    /// 範囲外を受け付けないことを確認する。
    mod bounds {
        use super::*;

        /// Scenario: 更新位置が区間外なら拒否される。
        /// - Given: 長さ 3 の木がある。
        /// - When: 座標 3 を更新する。
        /// - Then: 範囲外としてパニックする。
        #[test]
        #[should_panic(expected = "index out of bounds")]
        fn rejects_out_of_bounds_update() {
            // Given
            let mut sut = SegmentTreeSparseOnline::<monoid::AddMonoid>::new(3);
            // When
            sut.update(3, 1);
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
            let sut = SegmentTreeSparseOnline::<monoid::AddMonoid>::new(3);
            // When
            sut.fold(2, 1);
            // Then
        }
    }
}
