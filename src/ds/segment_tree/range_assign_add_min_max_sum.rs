//! 区間代入・区間加算と区間 min/max/sum クエリの具象
//! インスタンスを提供するモジュールである。
//!
//! `i64` 値の列に対して、区間代入と区間加算の 2 種類の更新を
//! 混在させつつ、区間最小値・最大値・総和を $O(\log n)$ で
//! 取得できる遅延セグメント木の具象型を定義する。
//!
//! 更新操作は `(assign: Option<i64>, add: i64)` の組で統一的に
//! 表現される。代入を先に適用し、その後に加算を適用するため、
//! 区間代入のみ・区間加算のみ・代入と加算の組合せをすべて
//! 1 つの合成規則で扱える。
//!
//! # 注意
//! 総和は `i64` で保持するため、要素数と値の積が `i64` の
//! 範囲を超える場合はオーバーフローする。最小値・最大値は
//! 要素単位の演算のため影響を受けない。総和のオーバーフローを
//! 避ける必要がある場合は、値域と要素数の上限を利用者側で
//! 管理すること。

use super::super::super::algebra::{monoid, semi_group};
use super::lazy_segment_tree;

/// 区間代入・区間加算と区間 min/max/sum クエリ用の遅延
/// セグメント木の型エイリアスである。
///
/// # Examples
/// ```
/// use anmitsu::ds::segment_tree::range_assign_add_min_max_sum::{
///     self, AssignAddAction,
/// };
///
/// let mut seg =
///     range_assign_add_min_max_sum::RangeAssignAddMinMaxSum::from_vec(
///         vec![
///             range_assign_add_min_max_sum::single(3),
///             range_assign_add_min_max_sum::single(1),
///             range_assign_add_min_max_sum::single(4),
///         ],
///     );
/// // 区間 [0, 3) に 10 を加算する。
/// seg.effect(0, 3, AssignAddAction::add(10));
/// let (sum, min, max, _) = seg.fold(0, 3);
/// assert_eq!(38, sum);
/// assert_eq!(11, min);
/// assert_eq!(14, max);
/// ```
pub type RangeAssignAddMinMaxSum =
    lazy_segment_tree::SegmentTreeLazyDense<MinMaxSumMonoid, AssignAddAction>;

/// 1 つの要素を表すモノイド値を生成する。
///
/// # Args
/// - `v` - 要素の値
///
/// # Returns
/// `(sum, min, max, count) = (v, v, v, 1)` を返す。
///
/// # Examples
/// ```
/// use anmitsu::ds::segment_tree::range_assign_add_min_max_sum;
///
/// let elem = range_assign_add_min_max_sum::single(42);
/// assert_eq!((42, 42, 42, 1), elem);
/// ```
pub fn single(v: i64) -> (i64, i64, i64, usize) {
    (v, v, v, 1)
}

/// 区間の総和・最小値・最大値・要素数を保持するモノイドである。
///
/// 要素数を合わせて保持することで、代入操作の際に
/// `代入値 × 要素数` で総和を正しく計算できる。
pub struct MinMaxSumMonoid;

impl semi_group::SemiGroup for MinMaxSumMonoid {
    /// `(sum, min, max, count)` の 4 つ組を保持する。
    type S = (i64, i64, i64, usize);

    /// 2 つの区間を結合する。
    ///
    /// 総和と要素数はそれぞれ加算し、最小値は小さい方、
    /// 最大値は大きい方を採る。
    fn op(a: &Self::S, b: &Self::S) -> Self::S {
        (
            a.0 + b.0,
            std::cmp::min(a.1, b.1),
            std::cmp::max(a.2, b.2),
            a.3 + b.3,
        )
    }
}

impl monoid::Monoid for MinMaxSumMonoid {
    /// 空区間を表す単位元を返す。
    ///
    /// 総和は 0、最小値は `i64::MAX`、最大値は `i64::MIN`、
    /// 要素数は 0 である。これにより、任意の区間と結合しても
    /// 結果が変化しない。
    fn id() -> Self::S {
        (0, i64::MAX, i64::MIN, 0)
    }
}

/// 区間代入と区間加算を統合した作用である。
///
/// 内部的に `(assign: Option<i64>, add: i64)` を保持する。
/// 代入が `Some(v)` の場合は全要素を `v` に置き換えた後に
/// `add` を加算し、`None` の場合は `add` のみを加算する。
/// 2 つの作用の合成規則は以下のとおりである。
///
/// - 後から代入が来る場合: 前の作用はすべて上書きされる。
/// - 後から加算のみの場合: `add` を加算し、`assign` は
///   前の作用を引き継ぐ。
///
/// # Examples
/// ```
/// use anmitsu::ds::segment_tree::range_assign_add_min_max_sum::{
///     self, AssignAddAction,
/// };
///
/// let mut seg =
///     range_assign_add_min_max_sum::RangeAssignAddMinMaxSum::from_vec(
///         vec![
///             range_assign_add_min_max_sum::single(1),
///             range_assign_add_min_max_sum::single(2),
///             range_assign_add_min_max_sum::single(3),
///         ],
///     );
/// // 全区間を 10 に代入する。
/// seg.effect(0, 3, AssignAddAction::assign(10));
/// assert_eq!((30, 10, 10, 3), seg.fold(0, 3));
///
/// // 区間 [1, 3) に 5 を加算する。
/// seg.effect(1, 3, AssignAddAction::add(5));
/// assert_eq!((40, 10, 15, 3), seg.fold(0, 3));
/// ```
#[derive(Clone, Copy, Debug)]
pub struct AssignAddAction {
    /// 代入値。`Some(v)` は全要素を `v` に置き換えることを表し、
    /// `None` は代入を行わないことを表す。
    assign: Option<i64>,
    /// 加算値。代入の後に (または代入なしで) 各要素に加算する値。
    add: i64,
}

impl AssignAddAction {
    /// 区間加算 `f(x) = x + a` を生成する。
    ///
    /// # Args
    /// - `a` - 各要素に加算する値
    ///
    /// # Returns
    /// `assign = None, add = a` の `AssignAddAction` を返す。
    ///
    /// # Examples
    /// ```
    /// use anmitsu::ds::segment_tree::range_assign_add_min_max_sum::AssignAddAction;
    ///
    /// let action = AssignAddAction::add(5);
    /// ```
    pub fn add(a: i64) -> Self {
        AssignAddAction {
            assign: None,
            add: a,
        }
    }

    /// 区間代入 `f(x) = v` を生成する。
    ///
    /// # Args
    /// - `v` - 各要素に代入する値
    ///
    /// # Returns
    /// `assign = Some(v), add = 0` の `AssignAddAction` を返す。
    ///
    /// # Examples
    /// ```
    /// use anmitsu::ds::segment_tree::range_assign_add_min_max_sum::AssignAddAction;
    ///
    /// let action = AssignAddAction::assign(42);
    /// ```
    pub fn assign(v: i64) -> Self {
        AssignAddAction {
            assign: Some(v),
            add: 0,
        }
    }
}

impl lazy_segment_tree::Hom<(i64, i64, i64, usize)> for AssignAddAction {
    /// 区間の集約値 `(sum, min, max, count)` に作用を適用する。
    ///
    /// 代入がある場合は全要素を代入値に置き換えた上で加算値を
    /// 足し、代入がない場合は各要素に加算値のみを足す。
    /// 要素数 0 の空区間には作用しない。
    ///
    /// # Args
    /// - `x` - 区間の集約値への参照
    ///
    /// # Returns
    /// 作用適用後の集約値を返す。要素数は変化しない。
    fn f(&self, x: &(i64, i64, i64, usize)) -> (i64, i64, i64, usize) {
        let (sum, min, max, count) = *x;
        if count == 0 {
            return *x;
        }
        match self.assign {
            Some(v) => {
                // 代入後に加算する。全要素が同一値になるため、
                // min と max は等しくなる。
                let val = v + self.add;
                (val * count as i64, val, val, count)
            }
            None => {
                // 加算のみ。各統計量に加算値を反映する。
                (
                    sum + self.add * count as i64,
                    min + self.add,
                    max + self.add,
                    count,
                )
            }
        }
    }

    /// 2 つの作用を合成する。
    ///
    /// `self` を先に適用し、`other` を後から適用する。
    /// `other` に代入がある場合は `self` の作用を完全に
    /// 上書きする。`other` が加算のみの場合は `self` の
    /// 代入を引き継ぎ、加算値を累積する。
    ///
    /// # Args
    /// - `other` - `self` の後に適用する作用
    ///
    /// # Returns
    /// 合成された作用を返す。
    fn composition(&self, other: &Self) -> Self {
        match other.assign {
            // 後から代入が来る場合、前の作用は完全に上書きされる。
            Some(v) => AssignAddAction {
                assign: Some(v),
                add: other.add,
            },
            // 後が加算のみの場合、代入は前の作用を引き継ぎ、
            // 加算値を累積する。
            None => AssignAddAction {
                assign: self.assign,
                add: self.add + other.add,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algebra::{monoid::Monoid, semi_group::SemiGroup};
    use crate::ds::segment_tree::lazy_segment_tree::Hom;

    /// Background: 要素数 5、値 [1, 2, 3, 4, 5] の遅延セグメント木
    fn create_seg() -> RangeAssignAddMinMaxSum {
        RangeAssignAddMinMaxSum::from_vec(
            vec![1, 2, 3, 4, 5].into_iter().map(single).collect(),
        )
    }

    // MinMaxSumMonoid のテスト: モノイド演算を検証する。
    mod min_max_sum_monoid {
        use super::*;

        // op のテスト: 戻り値を検証する。
        mod op {
            use super::*;

            /// Scenario: 2 区間を結合すると、総和は加算され、
            /// min/max はそれぞれ小さい方/大きい方になる。
            /// - Given: (10, 1, 5, 3) と (6, 2, 4, 2) がある。
            /// - When: op で結合する。
            /// - Then: (16, 1, 5, 5) になる。
            #[test]
            fn merges_sum_min_max_count() {
                // Given
                let a = (10_i64, 1_i64, 5_i64, 3_usize);
                let b = (6_i64, 2_i64, 4_i64, 2_usize);
                // When
                let result = MinMaxSumMonoid::op(&a, &b);
                // Then
                assert_eq!((16, 1, 5, 5), result);
            }
        }

        // id のテスト: 戻り値を検証する。
        mod id {
            use super::*;

            /// Scenario: 単位元は任意の区間と結合しても
            /// 結果を変えない。
            /// - Given: 単位元と (10, 1, 5, 3) がある。
            /// - When: op で結合する。
            /// - Then: (10, 1, 5, 3) のまま変化しない。
            #[test]
            fn does_not_change_other_when_composed() {
                // Given
                let id = MinMaxSumMonoid::id();
                let a = (10_i64, 1_i64, 5_i64, 3_usize);
                // When / Then
                assert_eq!(a, MinMaxSumMonoid::op(&id, &a));
                assert_eq!(a, MinMaxSumMonoid::op(&a, &id));
            }
        }
    }

    // AssignAddAction のテスト: 作用の適用と合成を検証する。
    mod assign_add_action {
        use super::*;

        // f のテスト: 戻り値を検証する。
        mod f {
            use super::*;

            /// Scenario: 加算作用は各統計量に値を加える。
            /// - Given: (10, 1, 5, 3) に加算 +2 を適用する。
            /// - When: f を呼ぶ。
            /// - Then: (16, 3, 7, 3) になる。
            #[test]
            fn adds_to_all_statistics() {
                // Given
                let x = (10_i64, 1_i64, 5_i64, 3_usize);
                let sut = AssignAddAction::add(2);
                // When
                let result = sut.f(&x);
                // Then
                assert_eq!((16, 3, 7, 3), result);
            }

            /// Scenario: 代入作用は全要素を同一値に置き換える。
            /// - Given: (10, 1, 5, 3) に代入 7 を適用する。
            /// - When: f を呼ぶ。
            /// - Then: (21, 7, 7, 3) になる。
            #[test]
            fn assigns_uniform_value() {
                // Given
                let x = (10_i64, 1_i64, 5_i64, 3_usize);
                let sut = AssignAddAction::assign(7);
                // When
                let result = sut.f(&x);
                // Then
                assert_eq!((21, 7, 7, 3), result);
            }

            /// Scenario: 空区間 (count=0) には作用しない。
            /// - Given: 単位元に加算 +5 を適用する。
            /// - When: f を呼ぶ。
            /// - Then: 単位元のまま変化しない。
            #[test]
            fn no_effect_on_empty_range() {
                // Given
                let x = MinMaxSumMonoid::id();
                let sut = AssignAddAction::add(5);
                // When
                let result = sut.f(&x);
                // Then
                assert_eq!(x, result);
            }
        }

        // composition のテスト: 戻り値を検証する。
        mod composition {
            use super::*;

            /// Scenario: 加算の後に加算を合成すると、加算値が
            /// 累積される。
            /// - Given: add(3) の後に add(5) を適用する。
            /// - When: composition で合成する。
            /// - Then: add(8) と同じ作用になる。
            #[test]
            fn add_then_add_accumulates() {
                // Given
                let f = AssignAddAction::add(3);
                let g = AssignAddAction::add(5);
                let x = (10_i64, 1_i64, 5_i64, 3_usize);
                // When
                let composed = f.composition(&g);
                // Then
                assert_eq!(g.f(&f.f(&x)), composed.f(&x));
            }

            /// Scenario: 加算の後に代入を合成すると、前の加算は
            /// 消え、代入のみが残る。
            /// - Given: add(3) の後に assign(10) を適用する。
            /// - When: composition で合成する。
            /// - Then: assign(10) と同じ作用になる。
            #[test]
            fn assign_overwrites_previous_add() {
                // Given
                let f = AssignAddAction::add(3);
                let g = AssignAddAction::assign(10);
                let x = (10_i64, 1_i64, 5_i64, 3_usize);
                // When
                let composed = f.composition(&g);
                // Then
                assert_eq!(g.f(&f.f(&x)), composed.f(&x));
            }

            /// Scenario: 代入の後に加算を合成すると、代入値を
            /// 引き継ぎ加算値が累積する。
            /// - Given: assign(10) の後に add(5) を適用する。
            /// - When: composition で合成する。
            /// - Then: 全要素が 15 になる作用と同じになる。
            #[test]
            fn add_after_assign_accumulates() {
                // Given
                let f = AssignAddAction::assign(10);
                let g = AssignAddAction::add(5);
                let x = (10_i64, 1_i64, 5_i64, 3_usize);
                // When
                let composed = f.composition(&g);
                // Then
                assert_eq!(g.f(&f.f(&x)), composed.f(&x));
            }

            /// Scenario: 代入の後に代入を合成すると、後の代入が
            /// 前の代入を完全に上書きする。
            /// - Given: assign(10) の後に assign(20) を適用する。
            /// - When: composition で合成する。
            /// - Then: assign(20) と同じ作用になる。
            #[test]
            fn assign_overwrites_previous_assign() {
                // Given
                let f = AssignAddAction::assign(10);
                let g = AssignAddAction::assign(20);
                let x = (10_i64, 1_i64, 5_i64, 3_usize);
                // When
                let composed = f.composition(&g);
                // Then
                assert_eq!(g.f(&f.f(&x)), composed.f(&x));
            }
        }
    }

    // fold のテスト: 遅延セグメント木全体の動作を検証する。
    mod fold {
        use super::*;

        /// Scenario: 初期値から全区間の sum/min/max を取得できる。
        /// - Given: [1, 2, 3, 4, 5] で構築したセグメント木がある。
        /// - When: 全区間を fold する。
        /// - Then: (15, 1, 5, 5) になる。
        #[test]
        fn returns_initial_aggregation() {
            // Given
            let mut sut = create_seg();
            // When
            let result = sut.fold(0, 5);
            // Then
            assert_eq!((15, 1, 5, 5), result);
        }

        /// Scenario: 部分区間の fold が正しく集約される。
        /// - Given: [1, 2, 3, 4, 5] で構築したセグメント木がある。
        /// - When: [1, 4) を fold する。
        /// - Then: (9, 2, 4, 3) になる。
        #[test]
        fn returns_partial_aggregation() {
            // Given
            let mut sut = create_seg();
            // When
            let result = sut.fold(1, 4);
            // Then
            assert_eq!((9, 2, 4, 3), result);
        }

        /// Scenario: 空区間を fold すると単位元が返る。
        /// - Given: セグメント木がある。
        /// - When: [2, 2) を fold する。
        /// - Then: 単位元 (0, MAX, MIN, 0) が返る。
        #[test]
        fn returns_identity_for_empty_range() {
            // Given
            let mut sut = create_seg();
            // When
            let result = sut.fold(2, 2);
            // Then
            assert_eq!(MinMaxSumMonoid::id(), result);
        }

        /// Scenario: 要素数 1 の木で fold できる。
        /// - Given: [42] で構築したセグメント木がある。
        /// - When: [0, 1) を fold する。
        /// - Then: (42, 42, 42, 1) になる。
        #[test]
        fn single_element_tree() {
            // Given
            let mut sut = RangeAssignAddMinMaxSum::from_vec(
                vec![single(42)],
            );
            // When
            let result = sut.fold(0, 1);
            // Then
            assert_eq!((42, 42, 42, 1), result);
        }

        /// Scenario: 空ベクタから構築した木で空区間を fold すると
        /// 単位元が返る。
        /// - Given: 空ベクタで構築したセグメント木がある。
        /// - When: [0, 0) を fold する。
        /// - Then: 単位元が返る。
        #[test]
        fn empty_tree() {
            // Given
            let mut sut =
                RangeAssignAddMinMaxSum::from_vec(vec![]);
            // When
            let result = sut.fold(0, 0);
            // Then
            assert_eq!(MinMaxSumMonoid::id(), result);
        }

        /// Scenario: 全要素が 0 の木を正しく fold できる。
        /// - Given: [0, 0, 0] で構築したセグメント木がある。
        /// - When: 全区間を fold する。
        /// - Then: (0, 0, 0, 3) になる。
        #[test]
        fn all_zeros() {
            // Given
            let mut sut = RangeAssignAddMinMaxSum::from_vec(
                vec![single(0), single(0), single(0)],
            );
            // When
            let result = sut.fold(0, 3);
            // Then
            assert_eq!((0, 0, 0, 3), result);
        }
    }

    // effect のテスト: 区間更新後の状態変化を検証する。
    mod effect {
        use super::*;

        /// Scenario: 区間加算後に sum/min/max が正しく反映される。
        /// - Given: [1, 2, 3, 4, 5] に区間 [1, 4) へ +10 を適用する。
        /// - When: 全区間と部分区間を fold する。
        /// - Then: 全区間は (45, 1, 14, 5)、[1, 4) は (39, 12, 14, 3)
        ///   になる。
        #[test]
        fn add_updates_statistics() {
            // Given
            let mut sut = create_seg();
            sut.effect(1, 4, AssignAddAction::add(10));
            // When / Then
            assert_eq!((45, 1, 14, 5), sut.fold(0, 5));
            assert_eq!((39, 12, 14, 3), sut.fold(1, 4));
        }

        /// Scenario: 区間代入後に全要素が代入値に置き換わる。
        /// - Given: [1, 2, 3, 4, 5] に区間 [1, 4) へ 0 を代入する。
        /// - When: 全区間と部分区間を fold する。
        /// - Then: 全区間は (6, 0, 5, 5)、[1, 4) は (0, 0, 0, 3)
        ///   になる。
        #[test]
        fn assign_replaces_all_elements() {
            // Given
            let mut sut = create_seg();
            sut.effect(1, 4, AssignAddAction::assign(0));
            // When / Then
            assert_eq!((6, 0, 5, 5), sut.fold(0, 5));
            assert_eq!((0, 0, 0, 3), sut.fold(1, 4));
        }

        /// Scenario: 代入の後に加算を適用すると正しく累積される。
        /// - Given: [1, 2, 3, 4, 5] に区間 [0, 5) へ 10 を代入した後、
        ///   同区間に +3 を加算する。
        /// - When: 全区間を fold する。
        /// - Then: 全要素が 13 になり (65, 13, 13, 5) になる。
        #[test]
        fn assign_then_add_composes_correctly() {
            // Given
            let mut sut = create_seg();
            sut.effect(0, 5, AssignAddAction::assign(10));
            sut.effect(0, 5, AssignAddAction::add(3));
            // When
            let result = sut.fold(0, 5);
            // Then
            assert_eq!((65, 13, 13, 5), result);
        }

        /// Scenario: 加算の後に代入を適用すると、加算は
        /// 上書きされる。
        /// - Given: [1, 2, 3, 4, 5] に区間 [0, 5) へ +100 を加算
        ///   した後、同区間に 0 を代入する。
        /// - When: 全区間を fold する。
        /// - Then: 加算は無効化され (0, 0, 0, 5) になる。
        #[test]
        fn add_then_assign_overwrites() {
            // Given
            let mut sut = create_seg();
            sut.effect(0, 5, AssignAddAction::add(100));
            sut.effect(0, 5, AssignAddAction::assign(0));
            // When
            let result = sut.fold(0, 5);
            // Then
            assert_eq!((0, 0, 0, 5), result);
        }

        /// Scenario: 負の値を含む代入・加算が正しく処理される。
        /// - Given: [1, 2, 3, 4, 5] に区間 [0, 3) へ -5 を代入し、
        ///   区間 [2, 5) へ -1 を加算する。
        /// - When: 全区間を fold する。
        /// - Then: 列は [-5, -5, -6, 3, 4] になり
        ///   (-9, -6, 4, 5) になる。
        #[test]
        fn handles_negative_values() {
            // Given
            let mut sut = create_seg();
            sut.effect(0, 3, AssignAddAction::assign(-5));
            sut.effect(2, 5, AssignAddAction::add(-1));
            // When
            let result = sut.fold(0, 5);
            // Then
            assert_eq!((-9, -6, 4, 5), result);
        }

        /// Scenario: 要素数 1 の区間への操作が正しく処理される。
        /// - Given: [1, 2, 3, 4, 5] に位置 2 のみ (+10) を加算する。
        /// - When: 全区間を fold する。
        /// - Then: (25, 1, 13, 5) になる。
        #[test]
        fn single_element_effect() {
            // Given
            let mut sut = create_seg();
            sut.effect(2, 3, AssignAddAction::add(10));
            // When
            let result = sut.fold(0, 5);
            // Then
            assert_eq!((25, 1, 13, 5), result);
        }

        /// Scenario: 空区間への effect は状態を変えない。
        /// - Given: [1, 2, 3] のセグメント木がある。
        /// - When: 空区間 [1, 1) に加算と代入を適用する。
        /// - Then: fold 結果が初期状態と変わらない。
        #[test]
        fn empty_range_effect_is_noop() {
            // Given
            let mut sut = RangeAssignAddMinMaxSum::from_vec(
                vec![single(1), single(2), single(3)],
            );
            let before = sut.fold(0, 3);
            // When
            sut.effect(1, 1, AssignAddAction::add(100));
            sut.effect(1, 1, AssignAddAction::assign(100));
            // Then
            assert_eq!(before, sut.fold(0, 3));
        }

        /// Scenario: 要素数 1 の木に加算を適用できる。
        /// - Given: [7] で構築したセグメント木がある。
        /// - When: [0, 1) に +3 を加算する。
        /// - Then: (10, 10, 10, 1) になる。
        #[test]
        fn add_on_single_element_tree() {
            // Given
            let mut sut = RangeAssignAddMinMaxSum::from_vec(
                vec![single(7)],
            );
            // When
            sut.effect(0, 1, AssignAddAction::add(3));
            // Then
            assert_eq!((10, 10, 10, 1), sut.fold(0, 1));
        }

        /// Scenario: 要素数 1 の木に代入を適用できる。
        /// - Given: [7] で構築したセグメント木がある。
        /// - When: [0, 1) に 0 を代入する。
        /// - Then: (0, 0, 0, 1) になる。
        #[test]
        fn assign_on_single_element_tree() {
            // Given
            let mut sut = RangeAssignAddMinMaxSum::from_vec(
                vec![single(7)],
            );
            // When
            sut.effect(0, 1, AssignAddAction::assign(0));
            // Then
            assert_eq!((0, 0, 0, 1), sut.fold(0, 1));
        }

        /// Scenario: 要素数 1 の木に代入と加算を連続で
        /// 適用できる。
        /// - Given: [0] で構築したセグメント木がある。
        /// - When: 代入 5 → 加算 +3 → 代入 -1 → 加算 +1 の
        ///   順に適用する。
        /// - Then: 最終値は 0 になり (0, 0, 0, 1) が返る。
        #[test]
        fn mixed_operations_on_single_element_tree() {
            // Given
            let mut sut = RangeAssignAddMinMaxSum::from_vec(
                vec![single(0)],
            );
            // When
            sut.effect(0, 1, AssignAddAction::assign(5));
            sut.effect(0, 1, AssignAddAction::add(3));
            sut.effect(0, 1, AssignAddAction::assign(-1));
            sut.effect(0, 1, AssignAddAction::add(1));
            // Then
            assert_eq!((0, 0, 0, 1), sut.fold(0, 1));
        }

        /// Scenario: 全要素 0 に加算すると正しく反映される。
        /// - Given: [0, 0, 0] で構築したセグメント木がある。
        /// - When: 全区間に +5 を加算する。
        /// - Then: (15, 5, 5, 3) になる。
        #[test]
        fn add_on_all_zeros() {
            // Given
            let mut sut = RangeAssignAddMinMaxSum::from_vec(
                vec![single(0), single(0), single(0)],
            );
            // When
            sut.effect(0, 3, AssignAddAction::add(5));
            // Then
            assert_eq!((15, 5, 5, 3), sut.fold(0, 3));
        }

        /// Scenario: 全要素 0 に 0 を代入しても状態が変わらない。
        /// - Given: [0, 0, 0] で構築したセグメント木がある。
        /// - When: 全区間に 0 を代入する。
        /// - Then: (0, 0, 0, 3) のまま変化しない。
        #[test]
        fn assign_zero_on_all_zeros() {
            // Given
            let mut sut = RangeAssignAddMinMaxSum::from_vec(
                vec![single(0), single(0), single(0)],
            );
            // When
            sut.effect(0, 3, AssignAddAction::assign(0));
            // Then
            assert_eq!((0, 0, 0, 3), sut.fold(0, 3));
        }
    }

    // ランダムテスト: ナイーブ実装との照合を検証する。
    mod random {
        use super::*;
        use rand::Rng;

        /// ナイーブな配列実装。区間操作を愚直に O(n) で行う。
        struct Naive {
            a: Vec<i64>,
        }

        impl Naive {
            fn new(a: Vec<i64>) -> Self {
                Naive { a }
            }

            fn add(&mut self, l: usize, r: usize, v: i64) {
                for x in &mut self.a[l..r] {
                    *x += v;
                }
            }

            fn assign(&mut self, l: usize, r: usize, v: i64) {
                for x in &mut self.a[l..r] {
                    *x = v;
                }
            }

            fn fold(&self, l: usize, r: usize) -> (i64, i64, i64, usize) {
                if l == r {
                    return MinMaxSumMonoid::id();
                }
                let slice = &self.a[l..r];
                (
                    slice.iter().sum(),
                    *slice.iter().min().unwrap(),
                    *slice.iter().max().unwrap(),
                    slice.len(),
                )
            }
        }

        /// Scenario: ランダムな操作列に対して、遅延セグメント木と
        /// ナイーブ実装の fold 結果が一致する。
        /// - Given: ランダムな初期値で構築した遅延セグメント木と
        ///   ナイーブ実装がある。
        /// - When: ランダムな区間加算・区間代入・区間 fold を
        ///   繰り返す。
        /// - Then: すべての fold 結果が一致する。
        #[test]
        fn matches_naive_implementation() {
            let mut rng = rand::rng();
            let n = 50;
            let q = 500;
            let value_range = -100..=100;

            for _ in 0..20 {
                // Given
                let init: Vec<i64> =
                    (0..n).map(|_| rng.random_range(value_range.clone())).collect();
                let mut sut = RangeAssignAddMinMaxSum::from_vec(
                    init.iter().map(|&v| single(v)).collect(),
                );
                let mut naive = Naive::new(init);

                for _ in 0..q {
                    let l = rng.random_range(0..n);
                    let r = rng.random_range(l..=n);

                    match rng.random_range(0..3) {
                        // When: 区間加算
                        0 => {
                            let v = rng.random_range(value_range.clone());
                            sut.effect(l, r, AssignAddAction::add(v));
                            naive.add(l, r, v);
                        }
                        // When: 区間代入
                        1 => {
                            let v = rng.random_range(value_range.clone());
                            sut.effect(l, r, AssignAddAction::assign(v));
                            naive.assign(l, r, v);
                        }
                        // When: 区間 fold
                        _ => {
                            // Then
                            assert_eq!(
                                naive.fold(l, r),
                                sut.fold(l, r),
                                "fold({}, {}) が一致しない",
                                l,
                                r,
                            );
                        }
                    }
                }

                // Then: 最終状態ですべての部分区間が一致する。
                for l in 0..n {
                    for r in l..=n {
                        assert_eq!(
                            naive.fold(l, r),
                            sut.fold(l, r),
                            "最終 fold({}, {}) が一致しない",
                            l,
                            r,
                        );
                    }
                }
            }
        }
    }
}
