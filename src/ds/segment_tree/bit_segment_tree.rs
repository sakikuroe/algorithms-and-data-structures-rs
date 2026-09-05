//! 0/1 列に対する区間 set/flip と各種区間クエリの具象
//! インスタンスを提供するモジュールである。
//!
//! 0/1 のみからなる列に対して、区間代入 (set 0 / set 1) と
//! 区間反転 (flip) の 2 種類の更新を混在させつつ、以下の
//! クエリを $O(\log n)$ で取得できる遅延セグメント木の具象型を
//! 定義する。
//!
//! - **区間和** (1 の個数): [`BitRangeData::ones`]
//! - **0/1 転倒数**: [`BitRangeData::inv`]
//! - **連続 1 の最大長**: [`BitRangeData::longest_ones`]
//! - **連続 0 の最大長**: [`BitRangeData::longest_zeros`]
//! - **AND / OR / XOR 和**: [`BitRangeData::and`],
//!   [`BitRangeData::or`], [`BitRangeData::xor`]
//!
//! 更新操作は `(assign: Option<bool>, flip: bool)` の組で
//! 表現される。代入を先に適用し、その後に反転を適用するため、
//! 代入のみ・反転のみ・代入と反転の組合せをすべて 1 つの
//! 合成規則で扱える。

use super::super::super::algebra::{monoid, semi_group};
use super::lazy_segment_tree;

/// 0/1 列に対する区間 set/flip クエリ用の遅延セグメント木の
/// 型エイリアスである。
///
/// # Examples
/// ```
/// use anmitsu::ds::segment_tree::bit_segment_tree::{
///     BitAction, BitSegTree,
/// };
///
/// // [1, 0, 1, 1, 0] で構築する。
/// let mut seg = BitSegTree::from_values(
///     vec![true, false, true, true, false],
/// );
/// assert_eq!(3, seg.fold(0, 5).ones);
///
/// // 区間 [1, 4) を反転する。[1, 1, 0, 0, 0]
/// seg.effect(1, 4, BitAction::flip());
/// assert_eq!(2, seg.fold(0, 5).ones);
/// assert_eq!(2, seg.fold(0, 5).longest_ones);
/// ```
pub type BitSegTree =
    lazy_segment_tree::SegmentTreeLazyDense<BitRangeMonoid, BitAction>;

/// 0/1 列の区間に関する統計情報を保持する構造体である。
///
/// 遅延セグメント木の各ノードがこの構造体を保持し、区間の
/// 結合・作用の適用時に正しく更新される。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BitRangeData {
    /// 区間の長さ。
    pub len: usize,
    /// 区間内の 1 の個数。
    pub ones: usize,
    /// 0/1 転倒数。`i < j` かつ `a[i] = 1, a[j] = 0` である
    /// ペアの個数。
    pub inv: u64,
    /// 左端から連続する 1 の個数。
    pub prefix_ones: usize,
    /// 右端から連続する 1 の個数。
    pub suffix_ones: usize,
    /// 区間内の連続する 1 の最大長。
    pub longest_ones: usize,
    /// 左端から連続する 0 の個数。
    pub prefix_zeros: usize,
    /// 右端から連続する 0 の個数。
    pub suffix_zeros: usize,
    /// 区間内の連続する 0 の最大長。
    pub longest_zeros: usize,
}

impl BitRangeData {
    /// 区間内の全要素が 1 であるかを返す。
    ///
    /// 空区間に対しては `true` を返す (AND の単位元)。
    pub fn and(&self) -> bool {
        self.ones == self.len
    }

    /// 区間内に 1 が 1 つ以上存在するかを返す。
    ///
    /// 空区間に対しては `false` を返す (OR の単位元)。
    pub fn or(&self) -> bool {
        self.ones > 0
    }

    /// 区間内の 1 の個数の偶奇を返す。
    ///
    /// 空区間に対しては `false` を返す (XOR の単位元)。
    pub fn xor(&self) -> bool {
        self.ones % 2 == 1
    }

    /// 区間内の 0 の個数を返す。
    pub fn zeros(&self) -> usize {
        self.len - self.ones
    }
}

/// 0/1 列の区間統計を保持するモノイドである。
pub struct BitRangeMonoid;

impl semi_group::SemiGroup for BitRangeMonoid {
    type S = BitRangeData;

    /// 2 つの隣接区間を結合する。
    ///
    /// 1 の個数は加算、転倒数は左区間の 1 と右区間の 0 の
    /// ペアを追加し、連続長は接合部をまたぐケースを考慮する。
    fn op(a: &Self::S, b: &Self::S) -> Self::S {
        BitRangeData {
            len: a.len + b.len,
            ones: a.ones + b.ones,
            inv: a.inv + b.inv + a.ones as u64 * b.zeros() as u64,
            prefix_ones: if a.prefix_ones == a.len {
                a.len + b.prefix_ones
            } else {
                a.prefix_ones
            },
            suffix_ones: if b.suffix_ones == b.len {
                b.len + a.suffix_ones
            } else {
                b.suffix_ones
            },
            longest_ones: *[
                a.longest_ones,
                b.longest_ones,
                a.suffix_ones + b.prefix_ones,
            ]
            .iter()
            .max()
            .unwrap(),
            prefix_zeros: if a.prefix_zeros == a.len {
                a.len + b.prefix_zeros
            } else {
                a.prefix_zeros
            },
            suffix_zeros: if b.suffix_zeros == b.len {
                b.len + a.suffix_zeros
            } else {
                b.suffix_zeros
            },
            longest_zeros: *[
                a.longest_zeros,
                b.longest_zeros,
                a.suffix_zeros + b.prefix_zeros,
            ]
            .iter()
            .max()
            .unwrap(),
        }
    }
}

impl monoid::Monoid for BitRangeMonoid {
    /// 空区間を表す単位元を返す。
    fn id() -> Self::S {
        BitRangeData {
            len: 0,
            ones: 0,
            inv: 0,
            prefix_ones: 0,
            suffix_ones: 0,
            longest_ones: 0,
            prefix_zeros: 0,
            suffix_zeros: 0,
            longest_zeros: 0,
        }
    }
}

impl monoid::GeneratedMonoid for BitRangeMonoid {
    type V = bool;

    /// 1 要素の値をモノイド値に埋め込む。
    ///
    /// # Examples
    /// ```
    /// use anmitsu::algebra::monoid::GeneratedMonoid;
    /// use anmitsu::ds::segment_tree::bit_segment_tree::BitRangeMonoid;
    ///
    /// let one = BitRangeMonoid::singleton(true);
    /// assert_eq!(1, one.ones);
    /// assert_eq!(0, one.zeros());
    ///
    /// let zero = BitRangeMonoid::singleton(false);
    /// assert_eq!(0, zero.ones);
    /// assert_eq!(1, zero.zeros());
    /// ```
    fn singleton(v: Self::V) -> Self::S {
        if v {
            BitRangeData {
                len: 1,
                ones: 1,
                inv: 0,
                prefix_ones: 1,
                suffix_ones: 1,
                longest_ones: 1,
                prefix_zeros: 0,
                suffix_zeros: 0,
                longest_zeros: 0,
            }
        } else {
            BitRangeData {
                len: 1,
                ones: 0,
                inv: 0,
                prefix_ones: 0,
                suffix_ones: 0,
                longest_ones: 0,
                prefix_zeros: 1,
                suffix_zeros: 1,
                longest_zeros: 1,
            }
        }
    }
}

/// 0/1 列に対する区間 set/flip 操作を表す作用である。
///
/// 内部的に `(assign: Option<bool>, flip: bool)` を保持する。
/// `assign` が `Some(v)` の場合は全要素を `v` に置き換えた後に
/// `flip` が `true` であれば反転する。2 つの作用の合成規則は
/// 以下のとおりである。
///
/// - 後から代入が来る場合: 前の作用は完全に上書きされる。
/// - 後が反転のみの場合: 代入は前の作用を引き継ぎ、
///   反転フラグを XOR で累積する。
#[derive(Clone, Copy, Debug)]
pub struct BitAction {
    /// 代入値。`Some(v)` は全要素を `v` に置き換えることを
    /// 表し、`None` は代入を行わないことを表す。
    assign: Option<bool>,
    /// 反転フラグ。`true` の場合、代入の後に (または代入なしで)
    /// 全要素を反転する。
    flip: bool,
}

impl BitAction {
    /// 区間代入 (set 0 / set 1) を生成する。
    ///
    /// # Args
    /// - `v` - 代入する値
    ///
    /// # Returns
    /// `assign = Some(v), flip = false` の `BitAction` を返す。
    ///
    /// # Examples
    /// ```
    /// use anmitsu::ds::segment_tree::bit_segment_tree::BitAction;
    ///
    /// let set_one = BitAction::set(true);
    /// let set_zero = BitAction::set(false);
    /// ```
    pub fn set(v: bool) -> Self {
        BitAction {
            assign: Some(v),
            flip: false,
        }
    }

    /// 区間反転 (flip) を生成する。
    ///
    /// # Returns
    /// `assign = None, flip = true` の `BitAction` を返す。
    ///
    /// # Examples
    /// ```
    /// use anmitsu::ds::segment_tree::bit_segment_tree::BitAction;
    ///
    /// let flip = BitAction::flip();
    /// ```
    pub fn flip() -> Self {
        BitAction {
            assign: None,
            flip: true,
        }
    }
}

impl lazy_segment_tree::Hom<BitRangeData> for BitAction {
    /// 区間データに作用を適用する。
    ///
    /// 代入がある場合は全要素を代入値に置き換え、さらに
    /// 反転フラグが立っていれば反転する。代入がない場合は
    /// 反転フラグに応じて反転のみ行う。
    fn f(&self, x: &BitRangeData) -> BitRangeData {
        let len = x.len;
        let mut result = match self.assign {
            Some(true) => BitRangeData {
                len,
                ones: len,
                inv: 0,
                prefix_ones: len,
                suffix_ones: len,
                longest_ones: len,
                prefix_zeros: 0,
                suffix_zeros: 0,
                longest_zeros: 0,
            },
            Some(false) => BitRangeData {
                len,
                ones: 0,
                inv: 0,
                prefix_ones: 0,
                suffix_ones: 0,
                longest_ones: 0,
                prefix_zeros: len,
                suffix_zeros: len,
                longest_zeros: len,
            },
            None => *x,
        };
        if self.flip {
            result = BitRangeData {
                len: result.len,
                ones: result.len - result.ones,
                inv: result.ones as u64 * result.zeros() as u64 - result.inv,
                prefix_ones: result.prefix_zeros,
                suffix_ones: result.suffix_zeros,
                longest_ones: result.longest_zeros,
                prefix_zeros: result.prefix_ones,
                suffix_zeros: result.suffix_ones,
                longest_zeros: result.longest_ones,
            };
        }
        result
    }

    /// 2 つの作用を合成する。
    ///
    /// `self` を先に適用し、`other` を後から適用する。
    /// `other` に代入がある場合は `self` の作用を完全に
    /// 上書きする。`other` が反転のみの場合は `self` の
    /// 代入を引き継ぎ、反転フラグを XOR で累積する。
    fn composition(&self, other: &Self) -> Self {
        match other.assign {
            Some(v) => BitAction {
                assign: Some(v),
                flip: other.flip,
            },
            None => BitAction {
                assign: self.assign,
                flip: self.flip ^ other.flip,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algebra::{
        monoid::{GeneratedMonoid, Monoid},
        semi_group::SemiGroup,
    };
    use crate::ds::segment_tree::lazy_segment_tree::Hom;

    fn s(v: bool) -> BitRangeData {
        BitRangeMonoid::singleton(v)
    }

    /// Background: [1, 0, 1, 1, 0] で構築した遅延セグメント木
    fn create_seg() -> BitSegTree {
        BitSegTree::from_values(
            vec![true, false, true, true, false],
        )
    }

    // BitRangeMonoid のテスト: モノイド演算を検証する。
    mod bit_range_monoid {
        use super::*;

        // op のテスト: 戻り値を検証する。
        mod op {
            use super::*;

            /// Scenario: 2 区間を結合すると、ones は加算され、
            /// inv は左の 1 と右の 0 のペア分が追加される。
            /// - Given: [1, 0] と [1, 0] がある。
            /// - When: op で結合する。
            /// - Then: ones=2, inv=1+1+1*1=3 になる。
            #[test]
            fn merges_ones_and_inversions() {
                // Given
                let a = BitRangeMonoid::op(&s(true), &s(false));
                let b = BitRangeMonoid::op(&s(true), &s(false));
                // When
                let result = BitRangeMonoid::op(&a, &b);
                // Then
                assert_eq!(4, result.len);
                assert_eq!(2, result.ones);
                assert_eq!(3, result.inv);
            }

            /// Scenario: 連続長は接合部をまたぐケースを考慮する。
            /// - Given: [1, 1] と [1, 0] がある。
            /// - When: op で結合する。
            /// - Then: longest_ones=3, prefix_ones=3 になる。
            #[test]
            fn merges_consecutive_runs_across_boundary() {
                // Given
                let a = BitRangeMonoid::op(&s(true), &s(true));
                let b = BitRangeMonoid::op(&s(true), &s(false));
                // When
                let result = BitRangeMonoid::op(&a, &b);
                // Then
                assert_eq!(3, result.longest_ones);
                assert_eq!(3, result.prefix_ones);
                assert_eq!(0, result.suffix_ones);
            }
        }

        // id のテスト: 戻り値を検証する。
        mod id {
            use super::*;

            /// Scenario: 単位元は任意の区間と結合しても
            /// 結果を変えない。
            /// - Given: 単位元と [1, 0, 1] がある。
            /// - When: op で結合する。
            /// - Then: [1, 0, 1] のまま変化しない。
            #[test]
            fn does_not_change_other_when_composed() {
                // Given
                let id = BitRangeMonoid::id();
                let a = BitRangeMonoid::op(
                    &BitRangeMonoid::op(&s(true), &s(false)),
                    &s(true),
                );
                // When / Then
                assert_eq!(a, BitRangeMonoid::op(&id, &a));
                assert_eq!(a, BitRangeMonoid::op(&a, &id));
            }
        }
    }

    // BitAction のテスト: 作用の適用と合成を検証する。
    mod bit_action {
        use super::*;

        // composition のテスト: 戻り値を検証する。
        mod composition {
            use super::*;

            /// Scenario: 合成結果が逐次適用と一致することを
            /// 全パターンで検証する。
            /// - Given: set(false), set(true), flip の全組合せ。
            /// - When: composition で合成し、結果を適用する。
            /// - Then: 逐次適用した結果と一致する。
            #[test]
            fn matches_sequential_application() {
                // Given
                let actions = [
                    BitAction::set(false),
                    BitAction::set(true),
                    BitAction::flip(),
                ];
                let x = BitRangeMonoid::op(
                    &BitRangeMonoid::op(&s(true), &s(false)),
                    &s(true),
                );
                for &f in &actions {
                    for &g in &actions {
                        // When
                        let composed = f.composition(&g);
                        // Then
                        assert_eq!(
                            g.f(&f.f(&x)),
                            composed.f(&x),
                            "f={:?}, g={:?}",
                            f,
                            g,
                        );
                    }
                }
            }

            /// Scenario: flip を 2 回合成すると恒等作用になる。
            /// - Given: flip 2 つがある。
            /// - When: composition で合成する。
            /// - Then: 元のデータと同じになる。
            #[test]
            fn double_flip_is_identity() {
                // Given
                let f = BitAction::flip();
                let x = BitRangeMonoid::op(&s(true), &s(false));
                // When
                let composed = f.composition(&f);
                // Then
                assert_eq!(x, composed.f(&x));
            }
        }
    }

    // fold のテスト: 遅延セグメント木全体の動作を検証する。
    mod fold {
        use super::*;

        /// Scenario: 初期値から各種統計を取得できる。
        /// - Given: [1, 0, 1, 1, 0] で構築したセグメント木がある。
        /// - When: 全区間を fold する。
        /// - Then: ones=3, inv=4, longest_ones=2 になる。
        #[test]
        fn returns_initial_statistics() {
            // Given
            let mut sut = create_seg();
            // When
            let result = sut.fold(0, 5);
            // Then
            assert_eq!(3, result.ones);
            assert_eq!(4, result.inv);
            assert_eq!(2, result.longest_ones);
            assert_eq!(1, result.longest_zeros);
        }

        /// Scenario: AND/OR/XOR が正しく導出される。
        /// - Given: [1, 0, 1, 1, 0] で構築したセグメント木がある。
        /// - When: 全区間と部分区間の fold を取得する。
        /// - Then: 導出値が正しい。
        #[test]
        fn derived_and_or_xor() {
            // Given
            let mut sut = create_seg();
            // When / Then
            let all = sut.fold(0, 5);
            assert!(!all.and());
            assert!(all.or());
            assert!(all.xor());

            let sub = sut.fold(2, 4);
            assert!(sub.and());
            assert!(sub.or());
            assert!(!sub.xor());
        }

        /// Scenario: 空区間を fold すると単位元が返る。
        /// - Given: セグメント木がある。
        /// - When: [2, 2) を fold する。
        /// - Then: 単位元が返る。
        #[test]
        fn returns_identity_for_empty_range() {
            // Given
            let mut sut = create_seg();
            // When
            let result = sut.fold(2, 2);
            // Then
            assert_eq!(BitRangeMonoid::id(), result);
        }

        /// Scenario: 要素数 1 の木で fold できる。
        /// - Given: [true] で構築したセグメント木がある。
        /// - When: [0, 1) を fold する。
        /// - Then: ones=1 になる。
        #[test]
        fn single_element_tree() {
            // Given
            let mut sut =
                BitSegTree::from_values(vec![true]);
            // When
            let result = sut.fold(0, 1);
            // Then
            assert_eq!(s(true), result);
        }

        /// Scenario: 空ベクタから構築した木で空区間を fold
        /// すると単位元が返る。
        /// - Given: 空ベクタで構築したセグメント木がある。
        /// - When: [0, 0) を fold する。
        /// - Then: 単位元が返る。
        #[test]
        fn empty_tree() {
            // Given
            let mut sut = BitSegTree::from_vec(vec![]);
            // When
            let result = sut.fold(0, 0);
            // Then
            assert_eq!(BitRangeMonoid::id(), result);
        }

        /// Scenario: 全要素が同一値の場合の連続長が正しい。
        /// - Given: [1, 1, 1, 1] で構築したセグメント木がある。
        /// - When: 全区間を fold する。
        /// - Then: longest_ones=4, longest_zeros=0 になる。
        #[test]
        fn all_same_value() {
            // Given
            let mut sut = BitSegTree::from_values(
                vec![true; 4],
            );
            // When
            let result = sut.fold(0, 4);
            // Then
            assert_eq!(4, result.longest_ones);
            assert_eq!(0, result.longest_zeros);
            assert!(result.and());
        }
    }

    // effect のテスト: 区間更新後の状態変化を検証する。
    mod effect {
        use super::*;

        /// Scenario: 区間反転後に統計が正しく更新される。
        /// - Given: [1, 0, 1, 1, 0] に [1, 4) を反転する。
        /// - When: 全区間を fold する。
        /// - Then: [1, 1, 0, 0, 0] になり ones=2 になる。
        #[test]
        fn flip_updates_statistics() {
            // Given
            let mut sut = create_seg();
            sut.effect(1, 4, BitAction::flip());
            // When
            let result = sut.fold(0, 5);
            // Then
            assert_eq!(2, result.ones);
            assert_eq!(2, result.longest_ones);
            assert_eq!(3, result.longest_zeros);
        }

        /// Scenario: set(true) で全要素を 1 にできる。
        /// - Given: [1, 0, 1, 1, 0] に全区間を set(true) する。
        /// - When: 全区間を fold する。
        /// - Then: ones=5, longest_ones=5, inv=0 になる。
        #[test]
        fn set_true_makes_all_ones() {
            // Given
            let mut sut = create_seg();
            sut.effect(0, 5, BitAction::set(true));
            // When
            let result = sut.fold(0, 5);
            // Then
            assert_eq!(5, result.ones);
            assert_eq!(5, result.longest_ones);
            assert_eq!(0, result.inv);
            assert!(result.and());
        }

        /// Scenario: set(false) で全要素を 0 にできる。
        /// - Given: [1, 0, 1, 1, 0] に全区間を set(false) する。
        /// - When: 全区間を fold する。
        /// - Then: ones=0, longest_zeros=5 になる。
        #[test]
        fn set_false_makes_all_zeros() {
            // Given
            let mut sut = create_seg();
            sut.effect(0, 5, BitAction::set(false));
            // When
            let result = sut.fold(0, 5);
            // Then
            assert_eq!(0, result.ones);
            assert_eq!(5, result.longest_zeros);
            assert!(!result.or());
        }

        /// Scenario: set の後に flip を適用すると逆になる。
        /// - Given: 全区間を set(true) した後、全区間を flip する。
        /// - When: 全区間を fold する。
        /// - Then: 全要素が 0 になる。
        #[test]
        fn set_then_flip_inverts() {
            // Given
            let mut sut = create_seg();
            sut.effect(0, 5, BitAction::set(true));
            sut.effect(0, 5, BitAction::flip());
            // When
            let result = sut.fold(0, 5);
            // Then
            assert_eq!(0, result.ones);
        }

        /// Scenario: 空区間への effect は状態を変えない。
        /// - Given: セグメント木がある。
        /// - When: 空区間 [2, 2) に flip を適用する。
        /// - Then: fold 結果が変化しない。
        #[test]
        fn empty_range_effect_is_noop() {
            // Given
            let mut sut = create_seg();
            let before = sut.fold(0, 5);
            // When
            sut.effect(2, 2, BitAction::flip());
            sut.effect(2, 2, BitAction::set(true));
            // Then
            assert_eq!(before, sut.fold(0, 5));
        }

        /// Scenario: 要素数 1 の木に flip を適用できる。
        /// - Given: [true] で構築したセグメント木がある。
        /// - When: [0, 1) を flip する。
        /// - Then: ones=0 になる。
        #[test]
        fn flip_on_single_element_tree() {
            // Given
            let mut sut =
                BitSegTree::from_values(vec![true]);
            // When
            sut.effect(0, 1, BitAction::flip());
            // Then
            assert_eq!(0, sut.fold(0, 1).ones);
        }

        /// Scenario: 要素数 1 の木に set を適用できる。
        /// - Given: [false] で構築したセグメント木がある。
        /// - When: [0, 1) を set(true) する。
        /// - Then: ones=1 になる。
        #[test]
        fn set_on_single_element_tree() {
            // Given
            let mut sut =
                BitSegTree::from_values(vec![false]);
            // When
            sut.effect(0, 1, BitAction::set(true));
            // Then
            assert_eq!(1, sut.fold(0, 1).ones);
        }

        /// Scenario: 転倒数が区間操作後に正しく更新される。
        /// - Given: [1, 1, 0, 0] (inv=4) に [0, 2) を flip する。
        /// - When: 全区間を fold する。
        /// - Then: [0, 0, 0, 0] になり inv=0 になる。
        #[test]
        fn inversion_count_after_operations() {
            // Given
            let mut sut = BitSegTree::from_values(
                vec![true, true, false, false],
            );
            assert_eq!(4, sut.fold(0, 4).inv);
            // When
            sut.effect(0, 2, BitAction::flip());
            // Then
            assert_eq!(0, sut.fold(0, 4).inv);
        }
    }

    // ランダムテスト: ナイーブ実装との照合を検証する。
    mod random {
        use super::*;
        use rand::Rng;

        /// ナイーブな配列実装。区間操作を愚直に O(n) で行う。
        struct Naive {
            a: Vec<bool>,
        }

        impl Naive {
            fn new(a: Vec<bool>) -> Self {
                Naive { a }
            }

            fn set(&mut self, l: usize, r: usize, v: bool) {
                for x in &mut self.a[l..r] {
                    *x = v;
                }
            }

            fn flip(&mut self, l: usize, r: usize) {
                for x in &mut self.a[l..r] {
                    *x = !*x;
                }
            }

            fn fold(&self, l: usize, r: usize) -> BitRangeData {
                if l == r {
                    return BitRangeMonoid::id();
                }
                let slice = &self.a[l..r];
                let len = slice.len();
                let ones = slice.iter().filter(|&&v| v).count();

                // 転倒数を愚直に計算する。
                let mut inv = 0_u64;
                let mut ones_so_far = 0_u64;
                for &v in slice {
                    if v {
                        ones_so_far += 1;
                    } else {
                        inv += ones_so_far;
                    }
                }

                // 連続長を愚直に計算する。
                let mut prefix_ones = 0;
                for &v in slice {
                    if v { prefix_ones += 1; } else { break; }
                }
                let mut suffix_ones = 0;
                for &v in slice.iter().rev() {
                    if v { suffix_ones += 1; } else { break; }
                }
                let mut longest_ones = 0;
                let mut cur = 0;
                for &v in slice {
                    if v { cur += 1; } else { cur = 0; }
                    longest_ones = std::cmp::max(longest_ones, cur);
                }

                let mut prefix_zeros = 0;
                for &v in slice {
                    if !v { prefix_zeros += 1; } else { break; }
                }
                let mut suffix_zeros = 0;
                for &v in slice.iter().rev() {
                    if !v { suffix_zeros += 1; } else { break; }
                }
                let mut longest_zeros = 0;
                cur = 0;
                for &v in slice {
                    if !v { cur += 1; } else { cur = 0; }
                    longest_zeros = std::cmp::max(longest_zeros, cur);
                }

                BitRangeData {
                    len,
                    ones,
                    inv,
                    prefix_ones,
                    suffix_ones,
                    longest_ones,
                    prefix_zeros,
                    suffix_zeros,
                    longest_zeros,
                }
            }
        }

        /// Scenario: ランダムな操作列に対して、遅延セグメント木と
        /// ナイーブ実装の fold 結果が一致する。
        /// - Given: ランダムな初期値で構築した遅延セグメント木と
        ///   ナイーブ実装がある。
        /// - When: ランダムな set/flip/fold を繰り返す。
        /// - Then: すべての fold 結果が一致する。
        #[test]
        fn matches_naive_implementation() {
            let mut rng = rand::rng();
            let n = 50;
            let q = 500;

            for _ in 0..20 {
                // Given
                let init: Vec<bool> =
                    (0..n).map(|_| rng.random_bool(0.5)).collect();
                let mut sut = BitSegTree::from_values(init.clone());
                let mut naive = Naive::new(init);

                for _ in 0..q {
                    let l = rng.random_range(0..n);
                    let r = rng.random_range(l..=n);

                    match rng.random_range(0..4) {
                        // When: set(false)
                        0 => {
                            sut.effect(l, r, BitAction::set(false));
                            naive.set(l, r, false);
                        }
                        // When: set(true)
                        1 => {
                            sut.effect(l, r, BitAction::set(true));
                            naive.set(l, r, true);
                        }
                        // When: flip
                        2 => {
                            sut.effect(l, r, BitAction::flip());
                            naive.flip(l, r);
                        }
                        // When: fold
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
