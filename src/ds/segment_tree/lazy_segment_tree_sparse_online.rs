//! 必要な節点だけを生成する 1 次元の遅延セグメント木を提供する。
//!
//! 論理上の区間 `[0, len)` の各点は初めモノイドの単位元を持つ。
//! 区間への作用・集約・境界探索は `O(log(len + 1))` 時間で行い、
//! 更新で生成した節点だけを保持する。

use super::super::super::algebra::monoid;
use std::ops::RangeBounds;

/// 子の節点がまだ生成されていないことを表す。
const NONE: usize = usize::MAX;

/// 区間長を受け取ってモノイド値へ作用する写像を表す。
///
/// 既存の `lazy_segment_tree::Hom` は区間長を受け取らない。
/// 未生成区間にも区間加算・区間和を作用させるため、こちらは
/// `apply` に対象区間の長さを渡す。`composition` の順序は
/// `Hom` と同じで、`self` を先、`other` を後に適用する。
/// `apply(op(a, b), len_a + len_b)` と
/// `op(apply(a, len_a), apply(b, len_b))` は一致する必要がある。
///
/// # Examples
/// ```
/// use anmitsu::ds::segment_tree::lazy_segment_tree_sparse_online;
///
/// #[derive(Clone)]
/// struct Add(i64);
/// impl lazy_segment_tree_sparse_online::RangeAction<i64> for Add {
///     fn apply(&self, value: &i64, len: usize) -> i64 {
///         value + self.0 * len as i64
///     }
///     fn composition(&self, other: &Self) -> Self {
///         Add(self.0 + other.0)
///     }
/// }
/// ```
pub trait RangeAction<S>: Clone {
    /// 区間全体の集約値に作用を適用する。
    ///
    /// # Args
    /// - `value` - 作用前の区間集約値。
    /// - `len` - 作用対象の区間に含まれる点数。
    ///
    /// # Returns
    /// 作用後の区間集約値を返す。
    fn apply(&self, value: &S, len: usize) -> S;

    /// `self` を先に、`other` を後に適用した合成作用を返す。
    ///
    /// # Args
    /// - `other` - `self` の後に適用する作用。
    ///
    /// # Returns
    /// `other.apply(&self.apply(value, len), len)` と等価な作用。
    fn composition(&self, other: &Self) -> Self;
}

/// 節点の集約値、未伝播の作用、子の位置を保持する。
struct Node<S, F> {
    /// この節点の区間全体に対する現在の集約値。
    value: S,
    /// 子にはまだ適用していない作用。
    lazy: Option<F>,
    /// 左右の子。未生成なら `NONE`。
    children: [usize; 2],
}

/// 再帰探索中の節点と、その節点が担当する半開区間。
struct Traversal {
    /// 節点の配列内の位置。未生成なら `NONE`。
    node: usize,
    /// 区間の左端。
    lower: usize,
    /// 区間の右端。
    upper: usize,
}

/// 論理範囲に比例する領域を確保しない遅延セグメント木。
///
/// 区間作用は直ちに対象節点の集約値へ反映され、必要になるまで
/// 子には伝播しない。区間集約と境界探索は節点を生成せずに読める。
///
/// # Examples
/// ```
/// use anmitsu::{algebra::monoid, ds::segment_tree::lazy_segment_tree_sparse_online};
///
/// #[derive(Clone)]
/// struct Add(i64);
/// impl lazy_segment_tree_sparse_online::RangeAction<i64> for Add {
///     fn apply(&self, value: &i64, len: usize) -> i64 {
///         value + self.0 * len as i64
///     }
///     fn composition(&self, other: &Self) -> Self {
///         Add(self.0 + other.0)
///     }
/// }
/// let mut seg = lazy_segment_tree_sparse_online::SegmentTreeLazySparseOnline::<
///     monoid::AddMonoid,
///     Add,
/// >::new(1_000_000_000);
/// seg.effect(10..20, Add(3));
/// assert_eq!(30, seg.fold(..));
/// assert_eq!(3, seg.get(15));
/// ```
pub struct SegmentTreeLazySparseOnline<M, F>
where
    M: monoid::Monoid,
{
    len: usize,
    nodes: Vec<Node<M::S, F>>,
}

impl<M, F> SegmentTreeLazySparseOnline<M, F>
where
    M: monoid::Monoid,
    M::S: Clone,
    F: RangeAction<M::S>,
{
    /// 全点が単位元である論理上の区間を作成する。
    ///
    /// # Args
    /// - `len` - 論理上の区間 `[0, len)` の長さ。
    ///
    /// # Returns
    /// 根節点だけを持つ木を返す。
    ///
    /// # Complexity
    /// 時間・空間ともに `O(1)`。
    pub fn new(len: usize) -> Self {
        Self {
            len,
            nodes: vec![Node {
                value: M::id(),
                lazy: None,
                children: [NONE; 2],
            }],
        }
    }

    /// 論理上の区間長を返す。
    ///
    /// # Returns
    /// 構築時に指定した `len` を返す。
    #[must_use]
    pub fn len(&self) -> usize {
        self.len
    }

    /// 論理上の区間が空であるかを返す。
    ///
    /// # Returns
    /// `len == 0` なら真を返す。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// 指定した区間に作用を適用する。
    ///
    /// # Args
    /// - `range` - 更新する区間。`left..right`、`left..=right`、`..` などを指定できる。
    /// - `effect` - 現在の値へ後から適用する作用。
    ///
    /// # Panics
    /// 端点が範囲外か、順序が逆の場合にパニックする。
    ///
    /// # Complexity
    /// 時間・追加領域ともに `O(log(len + 1))`。空区間では `O(1)`。
    pub fn effect(&mut self, range: impl RangeBounds<usize>, effect: F) {
        let (left, right) = super::range_bounds::normalize(range, self.len, "index");
        if left < right {
            self.effect_node(0, 0, self.len, left, right, &effect);
        }
    }

    /// 指定した区間の集約値を返す。
    ///
    /// # Args
    /// - `range` - 集約する区間。`left..right`、`left..=right`、`..` などを指定できる。
    ///
    /// # Returns
    /// 区間の集約値。空区間では単位元を返す。
    ///
    /// # Panics
    /// 端点が範囲外か、順序が逆の場合にパニックする。
    ///
    /// # Complexity
    /// 時間 `O(log(len + 1))`、再帰スタック `O(log(len + 1))`。
    pub fn fold(&self, range: impl RangeBounds<usize>) -> M::S {
        let (left, right) = super::range_bounds::normalize(range, self.len, "index");
        if left == right {
            return M::id();
        }
        self.fold_node(0, 0, self.len, left, right, None)
    }

    /// 点の現在値を返す。
    ///
    /// # Args
    /// - `index` - 取得する点の座標。
    ///
    /// # Returns
    /// 現在値。未更新なら単位元を返す。
    ///
    /// # Panics
    /// `index >= len` の場合にパニックする。
    pub fn get(&self, index: usize) -> M::S {
        assert!(index < self.len, "index out of bounds");
        self.fold(index..index + 1)
    }

    /// `[left, right)` の集約値を述語が受け入れる最大の `right` を返す。
    ///
    /// # Args
    /// - `left` - 探索を開始する位置。
    /// - `predicate` - 単位元を受け入れ、区間を広げると真から偽にのみ変わる述語。
    ///
    /// # Returns
    /// 条件を満たす最大の右端。最後まで真なら `len` を返す。
    ///
    /// # Panics
    /// `left > len`、または単位元を述語が拒否するとパニックする。
    pub fn max_right<P>(&self, left: usize, predicate: P) -> usize
    where
        P: Fn(&M::S) -> bool,
    {
        assert!(left <= self.len, "index out of bounds");
        assert!(predicate(&M::id()), "predicate must accept the identity");
        let mut sum = M::id();
        self.max_right_node(
            Traversal {
                node: 0,
                lower: 0,
                upper: self.len,
            },
            left,
            None,
            &predicate,
            &mut sum,
        )
        .unwrap_or(self.len)
    }

    /// `[left, right)` の集約値を述語が受け入れる最小の `left` を返す。
    ///
    /// # Args
    /// - `right` - 探索を終了する位置。
    /// - `predicate` - 単位元を受け入れ、区間を広げると真から偽にのみ変わる述語。
    ///
    /// # Returns
    /// 条件を満たす最小の左端。最後まで真なら `0` を返す。
    ///
    /// # Panics
    /// `right > len`、または単位元を述語が拒否するとパニックする。
    pub fn min_left<P>(&self, right: usize, predicate: P) -> usize
    where
        P: Fn(&M::S) -> bool,
    {
        assert!(right <= self.len, "index out of bounds");
        assert!(predicate(&M::id()), "predicate must accept the identity");
        let mut sum = M::id();
        self.min_left_node(
            Traversal {
                node: 0,
                lower: 0,
                upper: self.len,
            },
            right,
            None,
            &predicate,
            &mut sum,
        )
        .unwrap_or(0)
    }

    /// 節点の集約値を更新し、子への未伝播作用を時系列順に合成する。
    fn apply_node(&mut self, node: usize, length: usize, effect: &F) {
        self.nodes[node].value = effect.apply(&self.nodes[node].value, length);
        self.nodes[node].lazy = Some(match self.nodes[node].lazy.take() {
            Some(old) => old.composition(effect),
            None => effect.clone(),
        });
    }

    /// 未伝播作用を左右の子へ適用する。未生成の子は単位元で作る。
    fn push(&mut self, node: usize, left_length: usize, right_length: usize) {
        let Some(effect) = self.nodes[node].lazy.take() else {
            return;
        };
        for (side, length) in [left_length, right_length].into_iter().enumerate() {
            let mut child = self.nodes[node].children[side];
            if child == NONE {
                child = self.nodes.len();
                self.nodes.push(Node {
                    value: M::id(),
                    lazy: None,
                    children: [NONE; 2],
                });
                self.nodes[node].children[side] = child;
            }
            self.apply_node(child, length, &effect);
        }
    }

    /// 区間更新に必要な経路を生成し、帰りがけに親を再集約する。
    fn effect_node(
        &mut self,
        node: usize,
        lower: usize,
        upper: usize,
        left: usize,
        right: usize,
        effect: &F,
    ) {
        if left <= lower && upper <= right {
            self.apply_node(node, upper - lower, effect);
            return;
        }
        let middle = lower + (upper - lower) / 2;
        self.push(node, middle - lower, upper - middle);
        if left < middle {
            let mut child = self.nodes[node].children[0];
            if child == NONE {
                child = self.nodes.len();
                self.nodes.push(Node {
                    value: M::id(),
                    lazy: None,
                    children: [NONE; 2],
                });
                self.nodes[node].children[0] = child;
            }
            self.effect_node(child, lower, middle, left, right, effect);
        }
        if middle < right {
            let mut child = self.nodes[node].children[1];
            if child == NONE {
                child = self.nodes.len();
                self.nodes.push(Node {
                    value: M::id(),
                    lazy: None,
                    children: [NONE; 2],
                });
                self.nodes[node].children[1] = child;
            }
            self.effect_node(child, middle, upper, left, right, effect);
        }
        let [left_child, right_child] = self.nodes[node].children;
        self.nodes[node].value = match (left_child, right_child) {
            (NONE, NONE) => M::id(),
            (NONE, right_child) => self.nodes[right_child].value.clone(),
            (left_child, NONE) => self.nodes[left_child].value.clone(),
            (left_child, right_child) => M::op(
                &self.nodes[left_child].value,
                &self.nodes[right_child].value,
            ),
        };
    }

    /// 祖先からの新しい作用を節点の古い未伝播作用の後に合成する。
    fn descend_effect(&self, node: usize, ancestor: Option<F>) -> Option<F> {
        let local = if node == NONE {
            None
        } else {
            self.nodes[node].lazy.as_ref()
        };
        match (local, ancestor) {
            (Some(old), Some(new)) => Some(old.composition(&new)),
            (Some(old), None) => Some(old.clone()),
            (None, inherited) => inherited,
        }
    }

    /// 未生成部分木と祖先の未伝播作用を含めて区間を集約する。
    fn fold_node(
        &self,
        node: usize,
        lower: usize,
        upper: usize,
        left: usize,
        right: usize,
        ancestor: Option<F>,
    ) -> M::S {
        if right <= lower || upper <= left {
            return M::id();
        }
        if left <= lower && upper <= right {
            let value = if node == NONE {
                M::id()
            } else {
                self.nodes[node].value.clone()
            };
            return match ancestor {
                Some(effect) => effect.apply(&value, upper - lower),
                None => value,
            };
        }
        let middle = lower + (upper - lower) / 2;
        let children = if node == NONE {
            [NONE; 2]
        } else {
            self.nodes[node].children
        };
        let inherited = self.descend_effect(node, ancestor);
        if right <= middle {
            return self.fold_node(children[0], lower, middle, left, right, inherited);
        }
        if left >= middle {
            return self.fold_node(children[1], middle, upper, left, right, inherited);
        }
        let left_value = self.fold_node(children[0], lower, middle, left, right, inherited.clone());
        let right_value = self.fold_node(children[1], middle, upper, left, right, inherited);
        M::op(&left_value, &right_value)
    }

    /// 左から集約し、述語が初めて偽になる点を探す。
    fn max_right_node<P>(
        &self,
        range: Traversal,
        left: usize,
        ancestor: Option<F>,
        predicate: &P,
        sum: &mut M::S,
    ) -> Option<usize>
    where
        P: Fn(&M::S) -> bool,
    {
        let Traversal { node, lower, upper } = range;
        if upper <= left {
            return None;
        }
        if left <= lower {
            let value = if node == NONE {
                M::id()
            } else {
                self.nodes[node].value.clone()
            };
            let value = match &ancestor {
                Some(effect) => effect.apply(&value, upper - lower),
                None => value,
            };
            let next = M::op(sum, &value);
            if predicate(&next) {
                *sum = next;
                return None;
            }
            if upper - lower == 1 {
                return Some(lower);
            }
        }
        let middle = lower + (upper - lower) / 2;
        let children = if node == NONE {
            [NONE; 2]
        } else {
            self.nodes[node].children
        };
        let inherited = self.descend_effect(node, ancestor);
        self.max_right_node(
            Traversal {
                node: children[0],
                lower,
                upper: middle,
            },
            left,
            inherited.clone(),
            predicate,
            sum,
        )
        .or_else(|| {
            self.max_right_node(
                Traversal {
                    node: children[1],
                    lower: middle,
                    upper,
                },
                left,
                inherited,
                predicate,
                sum,
            )
        })
    }

    /// 右から集約し、述語が初めて偽になる点の直後を探す。
    fn min_left_node<P>(
        &self,
        range: Traversal,
        right: usize,
        ancestor: Option<F>,
        predicate: &P,
        sum: &mut M::S,
    ) -> Option<usize>
    where
        P: Fn(&M::S) -> bool,
    {
        let Traversal { node, lower, upper } = range;
        if right <= lower {
            return None;
        }
        if upper <= right {
            let value = if node == NONE {
                M::id()
            } else {
                self.nodes[node].value.clone()
            };
            let value = match &ancestor {
                Some(effect) => effect.apply(&value, upper - lower),
                None => value,
            };
            let next = M::op(&value, sum);
            if predicate(&next) {
                *sum = next;
                return None;
            }
            if upper - lower == 1 {
                return Some(upper);
            }
        }
        let middle = lower + (upper - lower) / 2;
        let children = if node == NONE {
            [NONE; 2]
        } else {
            self.nodes[node].children
        };
        let inherited = self.descend_effect(node, ancestor);
        self.min_left_node(
            Traversal {
                node: children[1],
                lower: middle,
                upper,
            },
            right,
            inherited.clone(),
            predicate,
            sum,
        )
        .or_else(|| {
            self.min_left_node(
                Traversal {
                    node: children[0],
                    lower,
                    upper: middle,
                },
                right,
                inherited,
                predicate,
                sum,
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 区間の各値に同じ整数を加える作用。
    #[derive(Clone)]
    struct Add(i64);

    impl RangeAction<i64> for Add {
        /// 和には増分に区間長を掛けて加える。
        fn apply(&self, value: &i64, len: usize) -> i64 {
            value + self.0 * len as i64
        }

        /// 二つの加算をまとめる。
        fn composition(&self, other: &Self) -> Self {
            Self(self.0 + other.0)
        }
    }

    /// 区間の各値を `a * x + b` に置き換える作用。
    #[derive(Clone)]
    struct Affine {
        /// 乗数。
        a: i64,
        /// 加数。
        b: i64,
    }

    impl RangeAction<i64> for Affine {
        /// 区間和に各点のアフィン変換をまとめて適用する。
        fn apply(&self, value: &i64, len: usize) -> i64 {
            self.a * value + self.b * len as i64
        }

        /// 古い作用 `self` の後に `other` を合成する。
        fn composition(&self, other: &Self) -> Self {
            Self {
                a: other.a * self.a,
                b: other.a * self.b + other.b,
            }
        }
    }

    /// 未生成区間への作用と集約を確認する。
    mod effect {
        use super::*;

        /// Scenario: 全域作用の後に一部だけ変更しても、他の点の値を保つ。
        /// - Given: まだ根しかない長さ 8 の木がある。
        /// - When: 全域に加算し、一部の区間へさらに加算する。
        /// - Then: 未生成だった点も区間長に応じた値を持つ。
        #[test]
        fn applies_to_uncreated_ranges() {
            // Given
            let mut sut = SegmentTreeLazySparseOnline::<monoid::AddMonoid, Add>::new(8);
            // When
            sut.effect(.., Add(2));
            assert_eq!(1, sut.nodes.len());
            sut.effect(3..6, Add(5));
            // Then
            assert_eq!(31, sut.fold(..));
            assert_eq!(21, sut.fold(3..6));
            assert_eq!(2, sut.get(0));
            assert_eq!(7, sut.get(5));
            assert_eq!(2, sut.get(7));
        }

        /// Scenario: 作用順序が異なるアフィン変換も正しい。
        /// - Given: 値がすべて 0 の木がある。
        /// - When: 全域・一部・全域の順に作用する。
        /// - Then: 古い作用から新しい作用への順番で値が決まる。
        #[test]
        fn composes_noncommutative_actions_in_time_order() {
            // Given
            let mut sut = SegmentTreeLazySparseOnline::<monoid::AddMonoid, Affine>::new(4);
            // When
            sut.effect(.., Affine { a: 1, b: 2 });
            sut.effect(1..3, Affine { a: 3, b: 1 });
            sut.effect(.., Affine { a: 2, b: 4 });
            // Then
            assert_eq!(8, sut.get(0));
            assert_eq!(18, sut.get(1));
            assert_eq!(18, sut.get(2));
            assert_eq!(8, sut.get(3));
            assert_eq!(52, sut.fold(..));
        }

        /// Scenario: 複数の区間更新後も素朴な配列と一致する。
        /// - Given: 長さ 9 の木と配列がある。
        /// - When: 重なり合う区間への加算を繰り返す。
        /// - Then: すべての半開区間の集約値が一致する。
        #[test]
        fn matches_naive_range_sums() {
            // Given
            let mut sut = SegmentTreeLazySparseOnline::<monoid::AddMonoid, Add>::new(9);
            let mut values = [0_i64; 9];
            // When
            for step in 0..24 {
                let left = step * 7 % 10;
                let right = left + step * 5 % (10 - left);
                let delta = step as i64 - 8;
                sut.effect(left..right, Add(delta));
                for value in &mut values[left..right] {
                    *value += delta;
                }

                // Then
                for query_left in 0..=9 {
                    for query_right in query_left..=9 {
                        let expected = values[query_left..query_right].iter().sum::<i64>();
                        assert_eq!(expected, sut.fold(query_left..query_right));
                    }
                }
            }
        }
    }

    /// 境界探索が未生成区間と未伝播作用を扱えることを確認する。
    mod boundary {
        use super::*;

        /// Scenario: 両側の探索は配列で求めた境界に一致する。
        /// - Given: 長さ 9 の木と非負値の配列がある。
        /// - When: 区間加算後、各位置から合計の閾値を探す。
        /// - Then: 左右の境界が素朴な走査と一致する。
        #[test]
        fn searches_both_directions() {
            // Given
            let mut sut = SegmentTreeLazySparseOnline::<monoid::AddMonoid, Add>::new(9);
            let mut values = [0_i64; 9];
            // When
            for (left, right, delta) in [(0, 9, 2), (2, 7, 3), (4, 5, 6)] {
                sut.effect(left..right, Add(delta));
                for value in &mut values[left..right] {
                    *value += delta;
                }
            }
            let before = sut.nodes.len();

            // Then
            for limit in [1, 3, 8, 20, 100] {
                for left in 0..=9 {
                    let mut expected = left;
                    let mut sum = 0;
                    while expected < 9 && sum + values[expected] < limit {
                        sum += values[expected];
                        expected += 1;
                    }
                    assert_eq!(expected, sut.max_right(left, |&value| value < limit));
                }
                for right in 0..=9 {
                    let mut expected = right;
                    let mut sum = 0;
                    while expected > 0 && sum + values[expected - 1] < limit {
                        expected -= 1;
                        sum += values[expected];
                    }
                    assert_eq!(expected, sut.min_left(right, |&value| value < limit));
                }
            }
            assert_eq!(before, sut.nodes.len());
        }

        /// Scenario: 広い論理範囲でも未生成部分の位置を探せる。
        /// - Given: 長さ 10 億の木がある。
        /// - When: 末尾の一点へ作用する。
        /// - Then: 境界探索が末尾の座標を返す。
        #[test]
        fn searches_large_sparse_range() {
            // Given
            let mut sut = SegmentTreeLazySparseOnline::<monoid::AddMonoid, Add>::new(1_000_000_000);
            // When
            sut.effect(999_999_999..1_000_000_000, Add(7));
            // Then
            assert_eq!(999_999_999, sut.max_right(0, |&sum| sum < 7));
            assert_eq!(1_000_000_000, sut.min_left(1_000_000_000, |&sum| sum < 7));
            assert_eq!(0, sut.fold(..999_999_999));
        }
    }

    /// 空区間と範囲外を確認する。
    mod bounds {
        use super::*;

        /// Scenario: 包含端点と無制限端点で作用と集約を指定できる。
        /// - Given: 長さ 4 の空の木がある。
        /// - When: 先頭と末尾を異なる範囲表記で更新する。
        /// - Then: 集約は指定された端点を正しく含む。
        #[test]
        fn accepts_inclusive_and_unbounded_ranges() {
            // Given
            let mut sut = SegmentTreeLazySparseOnline::<monoid::AddMonoid, Add>::new(4);
            // When
            sut.effect(..=1, Add(3));
            sut.effect(2.., Add(2));
            // Then
            assert_eq!(10, sut.fold(..));
            assert_eq!(5, sut.fold(1..=2));
            assert_eq!(4, sut.fold(2..));
        }

        /// Scenario: 長さ 0 の木は空区間の作用と集約を受け付ける。
        /// - Given: 長さ 0 の木がある。
        /// - When: 空区間に作用して集約・境界探索する。
        /// - Then: 集約は単位元、境界は 0 を返す。
        #[test]
        fn handles_empty_tree() {
            // Given
            let mut sut = SegmentTreeLazySparseOnline::<monoid::AddMonoid, Add>::new(0);
            // When
            sut.effect(.., Add(3));
            // Then
            assert!(sut.is_empty());
            assert_eq!(0, sut.len());
            assert_eq!(0, sut.fold(..));
            assert_eq!(0, sut.max_right(0, |&sum| sum < 1));
            assert_eq!(0, sut.min_left(0, |&sum| sum < 1));
            assert_eq!(1, sut.nodes.len());
        }

        /// Scenario: 範囲外の右端を拒否する。
        /// - Given: 長さ 4 の木がある。
        /// - When: 右端 5 まで作用する。
        /// - Then: 範囲外としてパニックする。
        #[test]
        #[should_panic(expected = "range out of bounds")]
        fn rejects_out_of_bounds_range() {
            // Given
            let mut sut = SegmentTreeLazySparseOnline::<monoid::AddMonoid, Add>::new(4);
            // When
            sut.effect(0..5, Add(1));
        }

        /// Scenario: 単位元を拒否する述語は境界探索に使用できない。
        /// - Given: 長さ 4 の木がある。
        /// - When: 偽の述語で境界探索する。
        /// - Then: 述語条件違反としてパニックする。
        #[test]
        #[should_panic(expected = "predicate must accept the identity")]
        fn rejects_predicate_on_identity() {
            // Given
            let sut = SegmentTreeLazySparseOnline::<monoid::AddMonoid, Add>::new(4);
            // When
            sut.max_right(0, |_| false);
        }
    }
}
