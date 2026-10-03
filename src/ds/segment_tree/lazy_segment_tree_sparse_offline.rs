//! 更新区間の端点を事前登録する 1 次元の疎な遅延セグメント木を提供する。
//!
//! 登録端点で論理区間を分割し、長さを持つ各区間を葉として管理する。
//! 更新区間の端点は登録済みである必要があるが、集約と境界探索では
//! 任意の端点を指定できる。異なる登録端点数を `K` とすると、
//! 保持領域は `O(K)`、更新と集約は `O(log(K + 1))` 時間である。
//! 境界探索では圧縮区間内の二分探索を含み、
//! `O(log(K + 1) + log(len + 1))` 時間である。

use super::super::super::algebra::monoid;
use std::ops::{Range, RangeBounds};

pub use super::lazy_segment_tree::RangeAction;

/// 配列木の節点と、その節点が担当する圧縮後の区間。
#[derive(Clone, Copy)]
struct Traversal {
    /// 1-indexed 配列木での位置。
    node: usize,
    /// 担当する圧縮後の左端。
    lower: usize,
    /// 担当する圧縮後の右端。
    upper: usize,
}

/// 更新区間の端点を事前登録する疎な遅延セグメント木。
///
/// 論理上の各点は初めモノイドの単位元を持つ。登録した端点の間では
/// 全点が同じ作用を受けるため、未登録位置の値も区間長から計算できる。
/// 区間集約と境界探索は木を変更しない。
///
/// # Examples
/// ```
/// use anmitsu::{algebra::monoid, ds::segment_tree::lazy_segment_tree_sparse_offline};
///
/// #[derive(Clone)]
/// struct Add(i64);
/// impl lazy_segment_tree_sparse_offline::RangeAction<i64> for Add {
///     fn apply(&self, value: &i64, len: usize) -> i64 {
///         value + self.0 * len as i64
///     }
///     fn composition(&self, other: &Self) -> Self {
///         Add(self.0 + other.0)
///     }
/// }
/// let mut seg = lazy_segment_tree_sparse_offline::SegmentTreeLazySparseOffline::<
///     monoid::AddMonoid,
///     Add,
/// >::new(1_000_000_000, [10..20]);
/// seg.effect(10..20, Add(3));
/// assert_eq!(30, seg.fold(..));
/// assert_eq!(12, seg.fold(13..17));
/// assert_eq!(3, seg.get(15));
/// ```
pub struct SegmentTreeLazySparseOffline<M, F>
where
    M: monoid::Monoid,
{
    len: usize,
    /// `0` と `len` を含む、昇順で重複のない更新端点。
    coordinates: Vec<usize>,
    /// 圧縮区間数を 2 の冪に切り上げた葉数。
    size: usize,
    /// 各節点の現在の集約値。0 番目は未使用。
    data: Vec<M::S>,
    /// 子へ未伝播の作用。葉ではその圧縮区間へ適用した作用全体。
    lazy: Vec<Option<F>>,
}

impl<M, F> SegmentTreeLazySparseOffline<M, F>
where
    M: monoid::Monoid,
    M::S: Clone,
    F: RangeAction<M::S>,
{
    /// 更新候補区間の端点を登録し、全点が単位元の木を作成する。
    ///
    /// `0` と `len` は自動で登録する。順序と重複は問わない。
    ///
    /// # Args
    /// - `len` - 論理上の区間 `[0, len)` の長さ。
    /// - `ranges` - 後から更新する可能性がある半開区間。
    ///
    /// # Returns
    /// 登録端点で分割した圧縮区間を持つ木を返す。
    ///
    /// # Panics
    /// 更新候補区間が範囲外か逆順の場合、または配列長が
    /// `usize` に収まらない場合にパニックする。
    ///
    /// # Complexity
    /// 候補区間数を `P`、異なる端点数を `K` とすると、
    /// 時間 `O(P log(P + 1) + K)`、空間 `O(K)`。
    pub fn new(len: usize, ranges: impl IntoIterator<Item = Range<usize>>) -> Self {
        let mut coordinates = vec![0, len];
        for range in ranges {
            assert!(
                range.start <= range.end && range.end <= len,
                "registered range out of bounds"
            );
            coordinates.push(range.start);
            coordinates.push(range.end);
        }
        coordinates.sort_unstable();
        coordinates.dedup();
        let cells = coordinates.len() - 1;
        let size = cells
            .max(1)
            .checked_next_power_of_two()
            .expect("too many registered ranges");
        let capacity = size.checked_mul(2).expect("too many registered ranges");
        let mut lazy = Vec::with_capacity(capacity);
        lazy.resize_with(capacity, || None);
        Self {
            len,
            coordinates,
            size,
            data: vec![M::id(); capacity],
            lazy,
        }
    }

    /// 論理上の区間長を返す。
    #[must_use]
    pub fn len(&self) -> usize {
        self.len
    }

    /// 論理上の区間が空であるかを返す。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// 登録済み端点を持つ区間に作用を適用する。
    ///
    /// 空区間は登録の有無によらず何もしない。
    ///
    /// # Args
    /// - `range` - 更新区間。`left..right`、`left..=right`、`..` などを指定できる。
    /// - `effect` - 現在の値へ後から適用する作用。
    ///
    /// # Panics
    /// 範囲外、逆順、または非空区間の端点が未登録の場合にパニックする。
    ///
    /// # Complexity
    /// 異なる登録端点数を `K` とすると、時間 `O(log(K + 1))`、
    /// 追加領域 `O(log(K + 1))`。
    pub fn effect(&mut self, range: impl RangeBounds<usize>, effect: F) {
        let (left, right) = super::range_bounds::normalize(range, self.len, "index");
        if left == right {
            return;
        }
        let left_rank = self
            .coordinates
            .binary_search(&left)
            .expect("range endpoint is not registered");
        let right_rank = self
            .coordinates
            .binary_search(&right)
            .expect("range endpoint is not registered");
        self.effect_node(
            Traversal {
                node: 1,
                lower: 0,
                upper: self.size,
            },
            left_rank,
            right_rank,
            &effect,
        );
    }

    /// 任意の端点で指定した区間の集約値を返す。
    ///
    /// # Args
    /// - `range` - 集約区間。`left..right`、`left..=right`、`..` などを指定できる。
    ///
    /// # Returns
    /// 区間の集約値。空区間では単位元を返す。
    ///
    /// # Panics
    /// 範囲外または逆順の場合にパニックする。
    ///
    /// # Complexity
    /// 異なる登録端点数を `K` とすると、時間と再帰スタックは
    /// `O(log(K + 1))`。
    pub fn fold(&self, range: impl RangeBounds<usize>) -> M::S {
        let (left, right) = super::range_bounds::normalize(range, self.len, "index");
        if left == right {
            return M::id();
        }
        self.fold_node(
            Traversal {
                node: 1,
                lower: 0,
                upper: self.size,
            },
            left,
            right,
            None,
        )
    }

    /// 点の現在値を返す。
    ///
    /// # Args
    /// - `index` - 取得する点の座標。
    ///
    /// # Returns
    /// 更新された値、または単位元を返す。
    ///
    /// # Panics
    /// `index >= len` の場合にパニックする。
    pub fn get(&self, index: usize) -> M::S {
        assert!(index < self.len, "index out of bounds");
        self.fold(index..index + 1)
    }

    /// `[left, right)` の集約値が述語を満たす最大の `right` を返す。
    ///
    /// # Args
    /// - `left` - 探索を開始する位置。
    /// - `predicate` - 単位元を受け入れ、区間を広げると真から偽にのみ変わる述語。
    ///
    /// # Returns
    /// 条件を満たす最大の右端。最後まで真なら `len` を返す。
    ///
    /// # Panics
    /// `left > len`、または述語が単位元を拒否するとパニックする。
    ///
    /// # Complexity
    /// `K` を異なる登録端点数とすると、時間は
    /// `O(log(K + 1) + log(len + 1))`。
    pub fn max_right<P>(&self, left: usize, predicate: P) -> usize
    where
        P: Fn(&M::S) -> bool,
    {
        assert!(left <= self.len, "index out of bounds");
        assert!(predicate(&M::id()), "predicate must accept the identity");
        let mut sum = M::id();
        self.max_right_node(
            Traversal {
                node: 1,
                lower: 0,
                upper: self.size,
            },
            left,
            None,
            &predicate,
            &mut sum,
        )
        .unwrap_or(self.len)
    }

    /// `[left, right)` の集約値が述語を満たす最小の `left` を返す。
    ///
    /// # Args
    /// - `right` - 探索を終了する位置。
    /// - `predicate` - 単位元を受け入れ、区間を広げると真から偽にのみ変わる述語。
    ///
    /// # Returns
    /// 条件を満たす最小の左端。最後まで真なら `0` を返す。
    ///
    /// # Panics
    /// `right > len`、または述語が単位元を拒否するとパニックする。
    ///
    /// # Complexity
    /// `K` を異なる登録端点数とすると、時間は
    /// `O(log(K + 1) + log(len + 1))`。
    pub fn min_left<P>(&self, right: usize, predicate: P) -> usize
    where
        P: Fn(&M::S) -> bool,
    {
        assert!(right <= self.len, "index out of bounds");
        assert!(predicate(&M::id()), "predicate must accept the identity");
        let mut sum = M::id();
        self.min_left_node(
            Traversal {
                node: 1,
                lower: 0,
                upper: self.size,
            },
            right,
            None,
            &predicate,
            &mut sum,
        )
        .unwrap_or(0)
    }

    /// 節点の集約値を更新し、未伝播作用を時系列順に合成する。
    fn apply_node(&mut self, node: usize, length: usize, effect: &F) {
        self.data[node] = effect.apply(&self.data[node], length);
        self.lazy[node] = Some(match self.lazy[node].take() {
            Some(old) => old.composition(effect),
            None => effect.clone(),
        });
    }

    /// 親の未伝播作用を、実際の長さを持つ左右の子へ渡す。
    fn push(&mut self, range: Traversal) {
        let Some(effect) = self.lazy[range.node].take() else {
            return;
        };
        let cells = self.coordinates.len() - 1;
        let middle = range.lower + (range.upper - range.lower) / 2;
        let left_length =
            self.coordinates[middle.min(cells)] - self.coordinates[range.lower.min(cells)];
        let right_length =
            self.coordinates[range.upper.min(cells)] - self.coordinates[middle.min(cells)];
        if left_length > 0 {
            self.apply_node(range.node * 2, left_length, &effect);
        }
        if right_length > 0 {
            self.apply_node(range.node * 2 + 1, right_length, &effect);
        }
    }

    /// 圧縮区間の更新経路をたどり、帰りがけに親を再集約する。
    fn effect_node(&mut self, range: Traversal, left: usize, right: usize, effect: &F) {
        if left <= range.lower && range.upper <= right {
            let cells = self.coordinates.len() - 1;
            let length =
                self.coordinates[range.upper.min(cells)] - self.coordinates[range.lower.min(cells)];
            self.apply_node(range.node, length, effect);
            return;
        }
        let middle = range.lower + (range.upper - range.lower) / 2;
        self.push(range);
        if left < middle {
            self.effect_node(
                Traversal {
                    node: range.node * 2,
                    lower: range.lower,
                    upper: middle,
                },
                left,
                right,
                effect,
            );
        }
        if middle < right {
            self.effect_node(
                Traversal {
                    node: range.node * 2 + 1,
                    lower: middle,
                    upper: range.upper,
                },
                left,
                right,
                effect,
            );
        }
        // 非可換なモノイドでも座標順を維持する。
        self.data[range.node] = M::op(&self.data[range.node * 2], &self.data[range.node * 2 + 1]);
    }

    /// 節点の古い作用の後に祖先の新しい作用を合成する。
    fn descend_effect(&self, node: usize, ancestor: Option<F>) -> Option<F> {
        match (self.lazy[node].as_ref(), ancestor) {
            (Some(old), Some(new)) => Some(old.composition(&new)),
            (Some(old), None) => Some(old.clone()),
            (None, inherited) => inherited,
        }
    }

    /// 未登録端点で切れた葉と祖先の未伝播作用を含めて集約する。
    fn fold_node(&self, range: Traversal, left: usize, right: usize, ancestor: Option<F>) -> M::S {
        let cells = self.coordinates.len() - 1;
        let lower = self.coordinates[range.lower.min(cells)];
        let upper = self.coordinates[range.upper.min(cells)];
        if right <= lower || upper <= left {
            return M::id();
        }
        if left <= lower && upper <= right {
            let value = self.data[range.node].clone();
            return match ancestor {
                Some(effect) => effect.apply(&value, upper - lower),
                None => value,
            };
        }
        if range.upper - range.lower == 1 {
            // 葉の全点は同じ作用を受けているため、部分区間を単位元から復元できる。
            let length = right.min(upper) - left.max(lower);
            return match self.descend_effect(range.node, ancestor) {
                Some(effect) => effect.apply(&M::id(), length),
                None => M::id(),
            };
        }
        let middle = range.lower + (range.upper - range.lower) / 2;
        let inherited = self.descend_effect(range.node, ancestor);
        let left_value = self.fold_node(
            Traversal {
                node: range.node * 2,
                lower: range.lower,
                upper: middle,
            },
            left,
            right,
            inherited.clone(),
        );
        let right_value = self.fold_node(
            Traversal {
                node: range.node * 2 + 1,
                lower: middle,
                upper: range.upper,
            },
            left,
            right,
            inherited,
        );
        M::op(&left_value, &right_value)
    }

    /// 左から集約し、述語が初めて偽になる位置を探す。
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
        let cells = self.coordinates.len() - 1;
        let lower = self.coordinates[range.lower.min(cells)];
        let upper = self.coordinates[range.upper.min(cells)];
        if upper <= left || lower == upper {
            return None;
        }
        if left <= lower {
            let value = match &ancestor {
                Some(effect) => effect.apply(&self.data[range.node], upper - lower),
                None => self.data[range.node].clone(),
            };
            let next = M::op(sum, &value);
            if predicate(&next) {
                *sum = next;
                return None;
            }
        }
        if range.upper - range.lower == 1 {
            let start = lower.max(left);
            let effect = self.descend_effect(range.node, ancestor);
            let rest = match &effect {
                Some(effect) => effect.apply(&M::id(), upper - start),
                None => M::id(),
            };
            let next = M::op(sum, &rest);
            if predicate(&next) {
                *sum = next;
                return None;
            }

            // 長い圧縮区間の内部では、述語が真である最大の右端を求める。
            let mut good = start;
            let mut bad = upper;
            while bad - good > 1 {
                let middle = good + (bad - good) / 2;
                let value = match &effect {
                    Some(effect) => effect.apply(&M::id(), middle - start),
                    None => M::id(),
                };
                if predicate(&M::op(sum, &value)) {
                    good = middle;
                } else {
                    bad = middle;
                }
            }
            return Some(good);
        }
        let middle = range.lower + (range.upper - range.lower) / 2;
        let inherited = self.descend_effect(range.node, ancestor);
        self.max_right_node(
            Traversal {
                node: range.node * 2,
                lower: range.lower,
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
                    node: range.node * 2 + 1,
                    lower: middle,
                    upper: range.upper,
                },
                left,
                inherited,
                predicate,
                sum,
            )
        })
    }

    /// 右から集約し、述語が初めて偽になる位置の直後を探す。
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
        let cells = self.coordinates.len() - 1;
        let lower = self.coordinates[range.lower.min(cells)];
        let upper = self.coordinates[range.upper.min(cells)];
        if right <= lower || lower == upper {
            return None;
        }
        if upper <= right {
            let value = match &ancestor {
                Some(effect) => effect.apply(&self.data[range.node], upper - lower),
                None => self.data[range.node].clone(),
            };
            let next = M::op(&value, sum);
            if predicate(&next) {
                *sum = next;
                return None;
            }
        }
        if range.upper - range.lower == 1 {
            let end = upper.min(right);
            let effect = self.descend_effect(range.node, ancestor);
            let rest = match &effect {
                Some(effect) => effect.apply(&M::id(), end - lower),
                None => M::id(),
            };
            let next = M::op(&rest, sum);
            if predicate(&next) {
                *sum = next;
                return None;
            }

            // 圧縮区間内で述語が真になる最小の左端を求める。
            let mut bad = lower;
            let mut good = end;
            while good - bad > 1 {
                let middle = bad + (good - bad) / 2;
                let value = match &effect {
                    Some(effect) => effect.apply(&M::id(), end - middle),
                    None => M::id(),
                };
                if predicate(&M::op(&value, sum)) {
                    good = middle;
                } else {
                    bad = middle;
                }
            }
            return Some(good);
        }
        let middle = range.lower + (range.upper - range.lower) / 2;
        let inherited = self.descend_effect(range.node, ancestor);
        self.min_left_node(
            Traversal {
                node: range.node * 2 + 1,
                lower: middle,
                upper: range.upper,
            },
            right,
            inherited.clone(),
            predicate,
            sum,
        )
        .or_else(|| {
            self.min_left_node(
                Traversal {
                    node: range.node * 2,
                    lower: range.lower,
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
    use super::super::super::super::algebra::semi_group;
    use super::*;
    use std::iter;

    /// 区間の各値に同じ整数を加える作用。
    #[derive(Clone)]
    struct Add(i64);

    impl RangeAction<i64> for Add {
        /// 区間和には増分に区間長を掛けて加える。
        fn apply(&self, value: &i64, len: usize) -> i64 {
            value + self.0 * len as i64
        }

        /// 二つの加算をまとめる。
        fn composition(&self, other: &Self) -> Self {
            Self(self.0 + other.0)
        }
    }

    /// 区間の各値を `a * x + b` に変換する作用。
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

        /// 古い作用を先に、新しい作用を後に合成する。
        fn composition(&self, other: &Self) -> Self {
            Self {
                a: other.a * self.a,
                b: other.a * self.b + other.b,
            }
        }
    }

    /// 未登録位置と作用の合成を確認する。
    mod effect {
        use super::*;

        /// Scenario: 未登録位置を含む更新後も任意端点で集約できる。
        /// - Given: 更新区間の端点だけを登録した長さ 12 の木がある。
        /// - When: 重なる二つの区間に加算を適用する。
        /// - Then: 登録端点の途中を含む集約と点取得が正しい。
        #[test]
        fn folds_inside_compressed_cells() {
            // Given
            let mut sut =
                SegmentTreeLazySparseOffline::<monoid::AddMonoid, Add>::new(12, [2..8, 4..6]);
            // When
            sut.effect(2..8, Add(3));
            sut.effect(4..6, Add(2));
            // Then
            assert_eq!(22, sut.fold(..));
            assert_eq!(16, sut.fold(3..7));
            assert_eq!(11, sut.fold(5..=7));
            assert_eq!(5, sut.get(5));
            assert_eq!(0, sut.get(1));
        }

        /// Scenario: 非可換な作用も適用順に合成される。
        /// - Given: 全域と部分区間の端点を登録した木がある。
        /// - When: 全域、部分区間、全域の順にアフィン変換する。
        /// - Then: 最後の集約値が操作順に計算した結果と一致する。
        #[test]
        fn composes_affine_actions_in_time_order() {
            // Given
            let mut sut =
                SegmentTreeLazySparseOffline::<monoid::AddMonoid, Affine>::new(6, [0..6, 1..5]);
            // When
            sut.effect(.., Affine { a: 1, b: 2 });
            sut.effect(1..5, Affine { a: 3, b: 1 });
            sut.effect(.., Affine { a: 2, b: 4 });
            // Then
            assert_eq!(8, sut.get(0));
            assert_eq!(18, sut.get(3));
            assert_eq!(88, sut.fold(..));
            assert_eq!(36, sut.fold(2..4));
        }

        /// Scenario: 複数の更新後も素朴な配列の区間和と一致する。
        /// - Given: 長さ 14 の木と配列がある。
        /// - When: 事前登録した区間へ加算を繰り返す。
        /// - Then: すべての半開区間の和が配列と一致する。
        #[test]
        fn matches_naive_range_sums() {
            // Given
            let ranges = [(0, 14), (1, 9), (3, 12), (4, 8), (10, 14)];
            let mut sut = SegmentTreeLazySparseOffline::<monoid::AddMonoid, Add>::new(
                14,
                ranges.map(|(left, right)| left..right),
            );
            let mut values = [0_i64; 14];
            // When
            for step in 0..15 {
                let (left, right) = ranges[step % ranges.len()];
                let delta = step as i64 - 6;
                sut.effect(left..right, Add(delta));
                for value in &mut values[left..right] {
                    *value += delta;
                }

                // Then
                for query_left in 0..=14 {
                    for query_right in query_left..=14 {
                        let expected = values[query_left..query_right].iter().sum::<i64>();
                        assert_eq!(expected, sut.fold(query_left..query_right));
                    }
                }
            }
        }
    }

    /// 圧縮区間内も含む境界探索を確認する。
    mod boundary {
        use super::*;

        /// Scenario: 境界が長い圧縮区間の内部にあっても探せる。
        /// - Given: 全域だけを登録した長さ 10 億の木がある。
        /// - When: 全点に 3 を加算する。
        /// - Then: 両方向の探索は圧縮区間内の点を返す。
        #[test]
        fn searches_within_a_long_compressed_cell() {
            // Given
            let mut sut = SegmentTreeLazySparseOffline::<monoid::AddMonoid, Add>::new(
                1_000_000_000,
                iter::once(0..1_000_000_000),
            );
            // When
            sut.effect(.., Add(3));
            // Then
            assert_eq!(43, sut.max_right(10, |&sum| sum <= 100));
            assert_eq!(467, sut.min_left(500, |&sum| sum <= 100));
            assert_eq!(6, sut.fold(11..13));
            assert_eq!(2, sut.data.len());
        }

        /// Scenario: 更新と境界探索は非負値の配列走査に一致する。
        /// - Given: 長さ 14 の木と配列がある。
        /// - When: 端点登録済みの区間へ正の値を加算する。
        /// - Then: すべての開始・終了位置の境界が一致する。
        #[test]
        fn matches_naive_boundaries() {
            // Given
            let ranges = [(0, 14), (1, 9), (3, 12), (4, 8), (10, 14)];
            let mut sut = SegmentTreeLazySparseOffline::<monoid::AddMonoid, Add>::new(
                14,
                ranges.map(|(left, right)| left..right),
            );
            let mut values = [0_i64; 14];
            // When
            for (left, right) in ranges {
                sut.effect(left..right, Add(2));
                for value in &mut values[left..right] {
                    *value += 2;
                }
            }
            // Then
            for limit in [1, 3, 8, 20, 100] {
                for left in 0..=14 {
                    let mut expected = left;
                    let mut sum = 0;
                    while expected < 14 && sum + values[expected] < limit {
                        sum += values[expected];
                        expected += 1;
                    }
                    assert_eq!(expected, sut.max_right(left, |&value| value < limit));
                }
                for right in 0..=14 {
                    let mut expected = right;
                    let mut sum = 0;
                    while expected > 0 && sum + values[expected - 1] < limit {
                        expected -= 1;
                        sum += values[expected];
                    }
                    assert_eq!(expected, sut.min_left(right, |&value| value < limit));
                }
            }
        }

        /// 文字列を左から順に連結するモノイド。
        struct Concat;

        impl semi_group::SemiGroup for Concat {
            type S = String;

            /// 左右の順序を保って文字列を結合する。
            fn op(left: &Self::S, right: &Self::S) -> Self::S {
                left.clone() + right
            }
        }

        impl monoid::Monoid for Concat {
            /// 空文字列を単位元として返す。
            fn id() -> Self::S {
                String::new()
            }
        }

        /// 区間中の各文字を一つの文字へ置き換える作用。
        #[derive(Clone)]
        struct Fill(char);

        impl RangeAction<String> for Fill {
            /// 区間長と同じ個数の文字を返す。
            fn apply(&self, _value: &String, len: usize) -> String {
                self.0.to_string().repeat(len)
            }

            /// 後から指定された文字を優先する。
            fn composition(&self, other: &Self) -> Self {
                other.clone()
            }
        }

        /// Scenario: 非可換な連結でも集約順序と境界が保たれる。
        /// - Given: 更新区間の端点を登録した長さ 5 の木がある。
        /// - When: 全域と部分区間へ順に文字を設定する。
        /// - Then: 集約結果と両方向の境界が文字列の順序に一致する。
        #[test]
        fn preserves_order_for_noncommutative_monoid() {
            // Given
            let mut sut = SegmentTreeLazySparseOffline::<Concat, Fill>::new(5, [0..5, 1..4, 2..3]);
            // When
            sut.effect(.., Fill('x'));
            sut.effect(1..4, Fill('a'));
            sut.effect(2..3, Fill('b'));
            // Then
            assert_eq!("xabax", sut.fold(..));
            assert_eq!("aba", sut.fold(1..4));
            assert_eq!(2, sut.max_right(0, |value| !value.contains("ab")));
            assert_eq!(3, sut.min_left(5, |value| !value.contains("ba")));
        }
    }

    /// 空区間、範囲外、未登録の更新端点を確認する。
    mod bounds {
        use super::*;

        /// Scenario: 空の木は空区間と境界探索を受け付ける。
        /// - Given: 更新候補のない長さ 0 の木がある。
        /// - When: 空区間へ作用し、集約と境界探索を行う。
        /// - Then: 単位元と境界 0 が返る。
        #[test]
        fn handles_empty_tree() {
            // Given
            let mut sut = SegmentTreeLazySparseOffline::<monoid::AddMonoid, Add>::new(0, []);
            // When
            sut.effect(.., Add(3));
            // Then
            assert!(sut.is_empty());
            assert_eq!(0, sut.len());
            assert_eq!(0, sut.fold(..));
            assert_eq!(0, sut.max_right(0, |&sum| sum < 1));
            assert_eq!(0, sut.min_left(0, |&sum| sum < 1));
        }

        /// Scenario: 未登録位置の空区間更新は何もしない。
        /// - Given: 両端以外の更新端点を登録していない木がある。
        /// - When: 未登録位置で空区間へ作用する。
        /// - Then: 木の集約値は変わらない。
        #[test]
        fn ignores_empty_range_without_registration() {
            // Given
            let mut sut = SegmentTreeLazySparseOffline::<monoid::AddMonoid, Add>::new(10, []);
            // When
            sut.effect(4..4, Add(7));
            // Then
            assert_eq!(0, sut.fold(..));
        }

        /// Scenario: 包含端点で登録済みの区間を更新できる。
        /// - Given: 先頭の二点を更新候補として登録した木がある。
        /// - When: 包含端点で二点へ加算する。
        /// - Then: 先頭の二点だけが変わる。
        #[test]
        fn accepts_inclusive_and_unbounded_bounds() {
            // Given
            let mut sut =
                SegmentTreeLazySparseOffline::<monoid::AddMonoid, Add>::new(5, iter::once(0..2));
            // When
            sut.effect(..=1, Add(4));
            // Then
            assert_eq!(8, sut.fold(..));
            assert_eq!(4, sut.fold(1..=1));
            assert_eq!(0, sut.fold(2..));
        }

        /// Scenario: 未登録の更新端点は拒否する。
        /// - Given: 端点 2 と 8 を登録した木がある。
        /// - When: 未登録の端点 3 から更新する。
        /// - Then: 端点の未登録を示すパニックになる。
        #[test]
        #[should_panic(expected = "range endpoint is not registered")]
        fn rejects_unregistered_update_endpoint() {
            // Given
            let mut sut =
                SegmentTreeLazySparseOffline::<monoid::AddMonoid, Add>::new(10, iter::once(2..8));
            // When
            sut.effect(3..8, Add(1));
        }

        /// Scenario: 範囲外の更新候補は構築時に拒否する。
        /// - Given: 論理長 10 を指定する。
        /// - When: 右端 11 の候補区間を登録する。
        /// - Then: 範囲外を示すパニックになる。
        #[test]
        #[should_panic(expected = "registered range out of bounds")]
        fn rejects_out_of_bounds_registration() {
            // When
            let _ =
                SegmentTreeLazySparseOffline::<monoid::AddMonoid, Add>::new(10, iter::once(4..11));
        }
    }
}
