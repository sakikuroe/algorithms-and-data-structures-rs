//! 区間ビット演算作用と区間 XOR 和・AND・OR クエリの
//! 具象インスタンスを提供するモジュールである。
//!
//! `u64` 値の列に対して、区間代入・区間 XOR・区間 AND・区間 OR の
//! 4 種類の更新を混在させつつ、区間 XOR 和・AND・OR を
//! $O(\log n)$ で取得できる遅延セグメント木の具象型を定義する。
//!
//! 更新操作は `(and_mask, xor_mask)` の組で統一的に表現される。
//! 各要素 $x$ に対する作用を $f(x) = (x \mathbin{\&} m_a)
//! \oplus m_x$ と定義することで、代入・XOR・AND・OR のすべてを
//! 1 つの合成規則で扱える。
//!
//! | 操作       | `and_mask`     | `xor_mask` | 作用                           |
//! |-----------|----------------|------------|--------------------------------|
//! | 恒等       | `u64::MAX`     | `0`        | $f(x) = x$                    |
//! | 代入 $v$   | `0`            | `v`        | $f(x) = v$                    |
//! | XOR $v$   | `u64::MAX`     | `v`        | $f(x) = x \oplus v$           |
//! | AND $v$   | `v`            | `0`        | $f(x) = x \mathbin{\&} v$     |
//! | OR $v$    | `!v`           | `v`        | $f(x) = x \mathbin{|} v$      |

use super::super::super::algebra::{monoid, semi_group};
use super::lazy_segment_tree;

/// 区間ビット演算作用と区間 XOR 和・AND・OR クエリ用の
/// 遅延セグメント木の型エイリアスである。
///
/// # Examples
/// ```
/// use anmitsu::ds::segment_tree::range_bitwise_xor_and_or::{
///     self, BitwiseAction,
/// };
///
/// let mut seg =
///     range_bitwise_xor_and_or::RangeBitwiseXorAndOr::from_values(
///         vec![0b1010, 0b1100, 0b0110],
///     );
/// // 区間 [0, 3) に 0b1111 で XOR を適用する。
/// seg.effect(0, 3, BitwiseAction::xor(0b1111));
/// let result = seg.fold(0, 3);
/// assert_eq!(0b0101 ^ 0b0011 ^ 0b1001, result.xor);
/// assert_eq!(0b0001, result.and);
/// assert_eq!(0b1111, result.or);
///
/// // 区間 [0, 2) に 0b0011 で OR を適用する。
/// seg.effect(0, 2, BitwiseAction::or(0b0011));
/// let result = seg.fold(0, 3);
/// assert_eq!(0b0111 ^ 0b0011 ^ 0b1001, result.xor);
/// assert_eq!(0b0001, result.and);
/// assert_eq!(0b1111, result.or);
/// ```
pub type RangeBitwiseXorAndOr =
    lazy_segment_tree::SegmentTreeLazyDense<XorAndOrMonoid, BitwiseAction>;

/// 区間内の全要素に対する XOR 和・AND・OR および要素数を
/// 保持するデータである。
///
/// XOR 和は要素数の偶奇で挙動が変わるため、要素数を合わせて
/// 保持することで、代入操作の際に正しく計算できる。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct XorAndOrData {
    /// 区間内の全要素の XOR 和。
    pub xor: u64,
    /// 区間内の全要素の AND。
    pub and: u64,
    /// 区間内の全要素の OR。
    pub or: u64,
    /// 区間内の要素数。
    pub len: usize,
}

/// `XorAndOrData` に対するモノイドの定義。
pub struct XorAndOrMonoid;

impl semi_group::SemiGroup for XorAndOrMonoid {
    type S = XorAndOrData;

    /// 2 つの区間を結合する。
    ///
    /// XOR 和は排他的論理和で、AND は論理積で、OR は論理和で、
    /// 要素数は加算でそれぞれ結合する。
    fn op(a: &Self::S, b: &Self::S) -> Self::S {
        XorAndOrData {
            xor: a.xor ^ b.xor,
            and: a.and & b.and,
            or: a.or | b.or,
            len: a.len + b.len,
        }
    }
}

impl monoid::Monoid for XorAndOrMonoid {
    /// 空区間を表す単位元を返す。
    ///
    /// XOR 和は 0、AND は `u64::MAX` (全ビット 1)、OR は 0、
    /// 要素数は 0 である。これにより、任意の区間と結合しても
    /// 結果が変化しない。
    fn id() -> Self::S {
        XorAndOrData {
            xor: 0,
            and: u64::MAX,
            or: 0,
            len: 0,
        }
    }
}

impl monoid::GeneratedMonoid for XorAndOrMonoid {
    type V = u64;

    /// 1 要素の値をモノイド値に埋め込む。
    ///
    /// 単一要素では XOR 和・AND・OR はすべて元の値に等しく、
    /// 要素数は 1 である。
    fn singleton(v: Self::V) -> Self::S {
        XorAndOrData {
            xor: v,
            and: v,
            or: v,
            len: 1,
        }
    }
}

/// 区間ビット演算の作用である。
///
/// 内部的に `(and_mask, xor_mask)` を保持する。各要素 $x$ に対し
/// $f(x) = (x \mathbin{\&} \text{and\_mask}) \oplus \text{xor\_mask}$
/// を適用する。
///
/// 2 つの作用の合成規則は以下のとおりである。$f$ を先に、$g$ を
/// 後から適用するとき、合成 $h = g \circ f$ は
/// $h.\text{and\_mask} = f.\text{and\_mask} \mathbin{\&}
/// g.\text{and\_mask}$、$h.\text{xor\_mask} =
/// (f.\text{xor\_mask} \mathbin{\&} g.\text{and\_mask})
/// \oplus g.\text{xor\_mask}$ となる。
///
/// # Examples
/// ```
/// use anmitsu::ds::segment_tree::range_bitwise_xor_and_or::{
///     self, BitwiseAction,
/// };
///
/// let mut seg =
///     range_bitwise_xor_and_or::RangeBitwiseXorAndOr::from_values(
///         vec![1, 2, 3],
///     );
/// // 全区間を 0xFF に代入する。
/// seg.effect(0, 3, BitwiseAction::assign(0xFF));
/// let result = seg.fold(0, 3);
/// assert_eq!(0xFF, result.and);
/// assert_eq!(0xFF, result.or);
///
/// // 区間 [1, 3) に 0x0F で AND を適用する。
/// seg.effect(1, 3, BitwiseAction::and(0x0F));
/// let result = seg.fold(0, 3);
/// assert_eq!(0x0F, result.and);
/// assert_eq!(0xFF, result.or);
/// ```
#[derive(Clone, Copy, Debug)]
pub struct BitwiseAction {
    /// AND マスク。各要素に対しまずこのマスクで AND を取る。
    and_mask: u64,
    /// XOR マスク。AND の適用後にこのマスクで XOR を取る。
    xor_mask: u64,
}

impl BitwiseAction {
    /// 区間 XOR `f(x) = x ^ v` を生成する。
    ///
    /// # Args
    /// - `v` - 各要素と XOR を取る値
    ///
    /// # Returns
    /// `and_mask = u64::MAX, xor_mask = v` の `BitwiseAction` を返す。
    ///
    /// # Examples
    /// ```
    /// use anmitsu::ds::segment_tree::range_bitwise_xor_and_or::BitwiseAction;
    ///
    /// let action = BitwiseAction::xor(0xFF);
    /// ```
    pub fn xor(v: u64) -> Self {
        BitwiseAction {
            and_mask: u64::MAX,
            xor_mask: v,
        }
    }

    /// 区間代入 `f(x) = v` を生成する。
    ///
    /// # Args
    /// - `v` - 各要素に代入する値
    ///
    /// # Returns
    /// `and_mask = 0, xor_mask = v` の `BitwiseAction` を返す。
    ///
    /// # Examples
    /// ```
    /// use anmitsu::ds::segment_tree::range_bitwise_xor_and_or::BitwiseAction;
    ///
    /// let action = BitwiseAction::assign(42);
    /// ```
    pub fn assign(v: u64) -> Self {
        BitwiseAction {
            and_mask: 0,
            xor_mask: v,
        }
    }

    /// 区間 AND `f(x) = x & v` を生成する。
    ///
    /// # Args
    /// - `v` - 各要素と AND を取る値
    ///
    /// # Returns
    /// `and_mask = v, xor_mask = 0` の `BitwiseAction` を返す。
    ///
    /// # Examples
    /// ```
    /// use anmitsu::ds::segment_tree::range_bitwise_xor_and_or::BitwiseAction;
    ///
    /// let action = BitwiseAction::and(0x0F);
    /// ```
    pub fn and(v: u64) -> Self {
        BitwiseAction {
            and_mask: v,
            xor_mask: 0,
        }
    }

    /// 区間 OR `f(x) = x | v` を生成する。
    ///
    /// # Args
    /// - `v` - 各要素と OR を取る値
    ///
    /// # Returns
    /// `and_mask = !v, xor_mask = v` の `BitwiseAction` を返す。
    /// `(x & !v) ^ v` は、`v` のビットが立っている位置を 1 に、
    /// それ以外を元のまま保持するため `x | v` と等価である。
    ///
    /// # Examples
    /// ```
    /// use anmitsu::ds::segment_tree::range_bitwise_xor_and_or::BitwiseAction;
    ///
    /// let action = BitwiseAction::or(0x0F);
    /// ```
    pub fn or(v: u64) -> Self {
        BitwiseAction {
            and_mask: !v,
            xor_mask: v,
        }
    }
}

impl lazy_segment_tree::Hom<XorAndOrData> for BitwiseAction {
    /// 区間の集約値に作用を適用する。
    ///
    /// 各要素に `f(x) = (x & and_mask) ^ xor_mask` を適用した後の
    /// 集約値を計算する。AND マスクによるビット消去の後、XOR
    /// マスクで反転する 2 段階で処理し、XOR のビットが立っている
    /// 位置では AND と OR が入れ替わる性質を利用する。要素数 0 の
    /// 空区間には作用しない。
    ///
    /// # Args
    /// - `x` - 区間の集約値への参照
    ///
    /// # Returns
    /// 作用適用後の集約値を返す。要素数は変化しない。
    fn f(&self, x: &XorAndOrData) -> XorAndOrData {
        if x.len == 0 {
            return x.clone();
        }

        // AND マスク適用後の中間集約値を求める。
        let mid_and = x.and & self.and_mask;
        let mid_or = x.or & self.and_mask;

        // XOR マスクが立っているビットでは AND と OR が入れ替わる。
        let v = self.xor_mask;
        let new_and = (mid_and & !v) | (!mid_or & v);
        let new_or = (mid_or & !v) | (!mid_and & v);

        // XOR 和: 保持されるビットの XOR と、XOR マスクの偶奇分。
        let parity = if x.len % 2 == 1 { v } else { 0 };
        let new_xor = (x.xor & self.and_mask) ^ parity;

        XorAndOrData {
            xor: new_xor,
            and: new_and,
            or: new_or,
            len: x.len,
        }
    }

    /// 2 つの作用を合成する。
    ///
    /// `self` を先に適用し、`other` を後から適用する合成作用を
    /// 返す。$g(f(x)) = ((x \mathbin{\&} f.m_a) \oplus f.m_x)
    /// \mathbin{\&} g.m_a) \oplus g.m_x$ を展開すると、
    /// $h.m_a = f.m_a \mathbin{\&} g.m_a$、$h.m_x =
    /// (f.m_x \mathbin{\&} g.m_a) \oplus g.m_x$ になる。
    ///
    /// # Args
    /// - `other` - `self` の後に適用する作用
    ///
    /// # Returns
    /// 合成された作用を返す。
    fn composition(&self, other: &Self) -> Self {
        BitwiseAction {
            and_mask: self.and_mask & other.and_mask,
            xor_mask: (self.xor_mask & other.and_mask) ^ other.xor_mask,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algebra::{monoid::Monoid, semi_group::SemiGroup};
    use crate::ds::segment_tree::lazy_segment_tree::Hom;

    /// Background: 要素数 5、値 [0b1010, 0b1100, 0b0110, 0b0011, 0b1001]
    /// の遅延セグメント木
    fn create_seg() -> RangeBitwiseXorAndOr {
        RangeBitwiseXorAndOr::from_values(vec![0b1010, 0b1100, 0b0110, 0b0011, 0b1001])
    }

    // XorAndOrMonoid のテスト: モノイド演算を検証する。
    mod xor_and_or_monoid {
        use super::*;

        // op のテスト: 戻り値を検証する。
        mod op {
            use super::*;

            /// Scenario: 2 区間を結合すると、XOR は排他的論理和、
            /// AND は論理積、OR は論理和、要素数は加算になる。
            /// - Given: (0b101, 0b001, 0b111, 2) と (0b110, 0b010, 0b110, 3)
            ///   がある。
            /// - When: op で結合する。
            /// - Then: XOR=0b011, AND=0b000, OR=0b111, len=5 になる。
            #[test]
            fn merges_xor_and_or_count() {
                // Given
                let a = XorAndOrData {
                    xor: 0b101,
                    and: 0b001,
                    or: 0b111,
                    len: 2,
                };
                let b = XorAndOrData {
                    xor: 0b110,
                    and: 0b010,
                    or: 0b110,
                    len: 3,
                };
                // When
                let result = XorAndOrMonoid::op(&a, &b);
                // Then
                assert_eq!(0b011, result.xor);
                assert_eq!(0b000, result.and);
                assert_eq!(0b111, result.or);
                assert_eq!(5, result.len);
            }
        }

        // id のテスト: 戻り値を検証する。
        mod id {
            use super::*;

            /// Scenario: 単位元は任意の区間と結合しても結果を
            /// 変えない。
            /// - Given: 単位元と (0b101, 0b001, 0b111, 3) がある。
            /// - When: op で結合する。
            /// - Then: 元の値のまま変化しない。
            #[test]
            fn does_not_change_other_when_composed() {
                // Given
                let id = XorAndOrMonoid::id();
                let a = XorAndOrData {
                    xor: 0b101,
                    and: 0b001,
                    or: 0b111,
                    len: 3,
                };
                // When / Then
                assert_eq!(a, XorAndOrMonoid::op(&id, &a));
                assert_eq!(a, XorAndOrMonoid::op(&a, &id));
            }
        }
    }

    // BitwiseAction のテスト: 作用の適用と合成を検証する。
    mod bitwise_action {
        use super::*;

        // f のテスト: 戻り値を検証する。
        mod f {
            use super::*;

            /// Scenario: XOR 作用は AND と OR のビットを正しく
            /// 入れ替える。
            /// - Given: 要素 {0b1010, 0b1100} の区間
            ///   (xor=0b0110, and=0b1000, or=0b1110, len=2) に
            ///   XOR 0b1111 を適用する。
            /// - When: f を呼ぶ。
            /// - Then: 要素は {0b0101, 0b0011} になり、
            ///   xor=0b0110, and=0b0001, or=0b0111 になる。
            #[test]
            fn xor_swaps_and_or_bits() {
                // Given
                let x = XorAndOrData {
                    xor: 0b0110,
                    and: 0b1000,
                    or: 0b1110,
                    len: 2,
                };
                let sut = BitwiseAction::xor(0b1111);
                // When
                let result = sut.f(&x);
                // Then
                assert_eq!(0b0110, result.xor);
                assert_eq!(0b0001, result.and);
                assert_eq!(0b0111, result.or);
            }

            /// Scenario: 代入作用は全要素を同一値に置き換える。
            /// - Given: 3 要素の区間に 0b1010 を代入する。
            /// - When: f を呼ぶ。
            /// - Then: XOR=0b1010 (奇数個)、AND=OR=0b1010 になる。
            #[test]
            fn assigns_uniform_value() {
                // Given
                let x = XorAndOrData {
                    xor: 0b0110,
                    and: 0b0000,
                    or: 0b1110,
                    len: 3,
                };
                let sut = BitwiseAction::assign(0b1010);
                // When
                let result = sut.f(&x);
                // Then
                assert_eq!(0b1010, result.xor);
                assert_eq!(0b1010, result.and);
                assert_eq!(0b1010, result.or);
            }

            /// Scenario: 偶数個の要素に代入すると XOR 和は 0 になる。
            /// - Given: 4 要素の区間に 0b1111 を代入する。
            /// - When: f を呼ぶ。
            /// - Then: XOR=0, AND=OR=0b1111 になる。
            #[test]
            fn assign_even_count_gives_zero_xor() {
                // Given
                let x = XorAndOrData {
                    xor: 0,
                    and: 0,
                    or: 0,
                    len: 4,
                };
                let sut = BitwiseAction::assign(0b1111);
                // When
                let result = sut.f(&x);
                // Then
                assert_eq!(0, result.xor);
                assert_eq!(0b1111, result.and);
                assert_eq!(0b1111, result.or);
            }

            /// Scenario: AND 作用は各要素のビットを消去する。
            /// - Given: 要素 {0b1010, 0b1110} の区間
            ///   (xor=0b0100, and=0b1010, or=0b1110, len=2) に
            ///   AND 0b1100 を適用する。
            /// - When: f を呼ぶ。
            /// - Then: 要素は {0b1000, 0b1100} になり、
            ///   xor=0b0100, and=0b1000, or=0b1100 になる。
            #[test]
            fn and_clears_bits() {
                // Given
                let x = XorAndOrData {
                    xor: 0b0100,
                    and: 0b1010,
                    or: 0b1110,
                    len: 2,
                };
                let sut = BitwiseAction::and(0b1100);
                // When
                let result = sut.f(&x);
                // Then
                assert_eq!(0b0100, result.xor);
                assert_eq!(0b1000, result.and);
                assert_eq!(0b1100, result.or);
            }

            /// Scenario: OR 作用は各要素のビットを立てる。
            /// - Given: 要素 {0b1010, 0b1100} の区間
            ///   (xor=0b0110, and=0b1000, or=0b1110, len=2) に
            ///   OR 0b0101 を適用する。
            /// - When: f を呼ぶ。
            /// - Then: 要素は {0b1111, 0b1101} になり、
            ///   xor=0b0010, and=0b1101, or=0b1111 になる。
            #[test]
            fn or_sets_bits() {
                // Given
                let x = XorAndOrData {
                    xor: 0b0110,
                    and: 0b1000,
                    or: 0b1110,
                    len: 2,
                };
                let sut = BitwiseAction::or(0b0101);
                // When
                let result = sut.f(&x);
                // Then
                assert_eq!(0b0010, result.xor);
                assert_eq!(0b1101, result.and);
                assert_eq!(0b1111, result.or);
            }

            /// Scenario: 空区間 (len=0) には作用しない。
            /// - Given: 単位元に XOR 0xFF を適用する。
            /// - When: f を呼ぶ。
            /// - Then: 単位元のまま変化しない。
            #[test]
            fn no_effect_on_empty_range() {
                // Given
                let x = XorAndOrMonoid::id();
                let sut = BitwiseAction::xor(0xFF);
                // When
                let result = sut.f(&x);
                // Then
                assert_eq!(x, result);
            }
        }

        // composition のテスト: 戻り値を検証する。
        mod composition {
            use super::*;

            /// Scenario: XOR の後に XOR を合成すると、XOR 値が
            /// 排他的論理和で累積される。
            /// - Given: xor(0b1010) の後に xor(0b0110) を適用する。
            /// - When: composition で合成する。
            /// - Then: 合成結果が逐次適用と同じになる。
            #[test]
            fn xor_then_xor_accumulates() {
                // Given
                let f = BitwiseAction::xor(0b1010);
                let g = BitwiseAction::xor(0b0110);
                let x = XorAndOrData {
                    xor: 0b111,
                    and: 0b001,
                    or: 0b111,
                    len: 3,
                };
                // When
                let composed = f.composition(&g);
                // Then
                assert_eq!(g.f(&f.f(&x)), composed.f(&x));
            }

            /// Scenario: XOR の後に代入を合成すると、前の XOR は
            /// 消え、代入のみが残る。
            /// - Given: xor(0b1111) の後に assign(0b1010) を適用する。
            /// - When: composition で合成する。
            /// - Then: 合成結果が逐次適用と同じになる。
            #[test]
            fn assign_overwrites_previous_xor() {
                // Given
                let f = BitwiseAction::xor(0b1111);
                let g = BitwiseAction::assign(0b1010);
                let x = XorAndOrData {
                    xor: 0b111,
                    and: 0b001,
                    or: 0b111,
                    len: 3,
                };
                // When
                let composed = f.composition(&g);
                // Then
                assert_eq!(g.f(&f.f(&x)), composed.f(&x));
            }

            /// Scenario: 代入の後に XOR を合成すると、代入値に
            /// XOR が累積する。
            /// - Given: assign(0b1010) の後に xor(0b1111) を適用する。
            /// - When: composition で合成する。
            /// - Then: 合成結果が逐次適用と同じになる。
            #[test]
            fn xor_after_assign_accumulates() {
                // Given
                let f = BitwiseAction::assign(0b1010);
                let g = BitwiseAction::xor(0b1111);
                let x = XorAndOrData {
                    xor: 0b111,
                    and: 0b001,
                    or: 0b111,
                    len: 3,
                };
                // When
                let composed = f.composition(&g);
                // Then
                assert_eq!(g.f(&f.f(&x)), composed.f(&x));
            }

            /// Scenario: 代入の後に代入を合成すると、後の代入が
            /// 前の代入を完全に上書きする。
            /// - Given: assign(0b1010) の後に assign(0b0101) を適用する。
            /// - When: composition で合成する。
            /// - Then: 合成結果が逐次適用と同じになる。
            #[test]
            fn assign_overwrites_previous_assign() {
                // Given
                let f = BitwiseAction::assign(0b1010);
                let g = BitwiseAction::assign(0b0101);
                let x = XorAndOrData {
                    xor: 0b111,
                    and: 0b001,
                    or: 0b111,
                    len: 3,
                };
                // When
                let composed = f.composition(&g);
                // Then
                assert_eq!(g.f(&f.f(&x)), composed.f(&x));
            }

            /// Scenario: AND の後に OR を合成すると正しく合成
            /// される。
            /// - Given: and(0b1100) の後に or(0b0011) を適用する。
            /// - When: composition で合成する。
            /// - Then: 合成結果が逐次適用と同じになる。
            #[test]
            fn and_then_or_composes() {
                // Given
                let f = BitwiseAction::and(0b1100);
                let g = BitwiseAction::or(0b0011);
                let x = XorAndOrData {
                    xor: 0b111,
                    and: 0b001,
                    or: 0b111,
                    len: 3,
                };
                // When
                let composed = f.composition(&g);
                // Then
                assert_eq!(g.f(&f.f(&x)), composed.f(&x));
            }

            /// Scenario: OR の後に AND を合成すると正しく合成
            /// される。
            /// - Given: or(0b0011) の後に and(0b1100) を適用する。
            /// - When: composition で合成する。
            /// - Then: 合成結果が逐次適用と同じになる。
            #[test]
            fn or_then_and_composes() {
                // Given
                let f = BitwiseAction::or(0b0011);
                let g = BitwiseAction::and(0b1100);
                let x = XorAndOrData {
                    xor: 0b111,
                    and: 0b001,
                    or: 0b111,
                    len: 3,
                };
                // When
                let composed = f.composition(&g);
                // Then
                assert_eq!(g.f(&f.f(&x)), composed.f(&x));
            }

            /// Scenario: XOR の後に AND を合成すると正しく合成
            /// される。
            /// - Given: xor(0b1010) の後に and(0b1100) を適用する。
            /// - When: composition で合成する。
            /// - Then: 合成結果が逐次適用と同じになる。
            #[test]
            fn xor_then_and_composes() {
                // Given
                let f = BitwiseAction::xor(0b1010);
                let g = BitwiseAction::and(0b1100);
                let x = XorAndOrData {
                    xor: 0b111,
                    and: 0b001,
                    or: 0b111,
                    len: 3,
                };
                // When
                let composed = f.composition(&g);
                // Then
                assert_eq!(g.f(&f.f(&x)), composed.f(&x));
            }

            /// Scenario: OR の後に XOR を合成すると正しく合成
            /// される。
            /// - Given: or(0b0011) の後に xor(0b1010) を適用する。
            /// - When: composition で合成する。
            /// - Then: 合成結果が逐次適用と同じになる。
            #[test]
            fn or_then_xor_composes() {
                // Given
                let f = BitwiseAction::or(0b0011);
                let g = BitwiseAction::xor(0b1010);
                let x = XorAndOrData {
                    xor: 0b111,
                    and: 0b001,
                    or: 0b111,
                    len: 3,
                };
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

        /// Scenario: 初期値から全区間の XOR/AND/OR を取得できる。
        /// - Given: [0b1010, 0b1100, 0b0110, 0b0011, 0b1001] で
        ///   構築したセグメント木がある。
        /// - When: 全区間を fold する。
        /// - Then: XOR/AND/OR/len が正しく集約される。
        #[test]
        fn returns_initial_aggregation() {
            // Given
            let mut sut = create_seg();
            // When
            let result = sut.fold(0, 5);
            // Then
            let expected_xor = 0b1010;
            let expected_and = 0;
            let expected_or = 0b1010 | 0b1100 | 0b0110 | 0b0011 | 0b1001;
            assert_eq!(expected_xor, result.xor);
            assert_eq!(expected_and, result.and);
            assert_eq!(expected_or, result.or);
            assert_eq!(5, result.len);
        }

        /// Scenario: 部分区間の fold が正しく集約される。
        /// - Given: 上記のセグメント木がある。
        /// - When: [1, 4) を fold する。
        /// - Then: [0b1100, 0b0110, 0b0011] の集約結果が返る。
        #[test]
        fn returns_partial_aggregation() {
            // Given
            let mut sut = create_seg();
            // When
            let result = sut.fold(1, 4);
            // Then
            assert_eq!(0b1100 ^ 0b0110 ^ 0b0011, result.xor);
            assert_eq!(0b1100 & 0b0110 & 0b0011, result.and);
            assert_eq!(0b1100 | 0b0110 | 0b0011, result.or);
            assert_eq!(3, result.len);
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
            assert_eq!(XorAndOrMonoid::id(), result);
        }

        /// Scenario: 要素数 1 の木で fold できる。
        /// - Given: [0xDEAD] で構築したセグメント木がある。
        /// - When: [0, 1) を fold する。
        /// - Then: XOR=AND=OR=0xDEAD, len=1 になる。
        #[test]
        fn single_element_tree() {
            // Given
            let mut sut = RangeBitwiseXorAndOr::from_values(vec![0xDEAD]);
            // When
            let result = sut.fold(0, 1);
            // Then
            assert_eq!(0xDEAD, result.xor);
            assert_eq!(0xDEAD, result.and);
            assert_eq!(0xDEAD, result.or);
            assert_eq!(1, result.len);
        }

        /// Scenario: 全要素が 0 の木を正しく fold できる。
        /// - Given: [0, 0, 0] で構築したセグメント木がある。
        /// - When: 全区間を fold する。
        /// - Then: XOR=AND=OR=0, len=3 になる。
        #[test]
        fn all_zeros() {
            // Given
            let mut sut = RangeBitwiseXorAndOr::from_values(vec![0, 0, 0]);
            // When
            let result = sut.fold(0, 3);
            // Then
            assert_eq!(0, result.xor);
            assert_eq!(0, result.and);
            assert_eq!(0, result.or);
            assert_eq!(3, result.len);
        }
    }

    // effect のテスト: 区間更新後の状態変化を検証する。
    mod effect {
        use super::*;

        /// Scenario: 区間 XOR 後に AND/OR が正しく反映される。
        /// - Given: [0b1010, 0b1100, 0b0110, 0b0011, 0b1001] に
        ///   区間 [1, 4) へ 0b1111 で XOR を適用する。
        /// - When: 区間 [1, 4) を fold する。
        /// - Then: 要素が [0b0011, 0b1001, 0b1100] に変化した
        ///   集約結果が得られる。
        #[test]
        fn xor_updates_statistics() {
            // Given
            let mut sut = create_seg();
            sut.effect(1, 4, BitwiseAction::xor(0b1111));
            // When
            let result = sut.fold(1, 4);
            // Then
            assert_eq!(0b0011 ^ 0b1001 ^ 0b1100, result.xor);
            assert_eq!(0b0011 & 0b1001 & 0b1100, result.and);
            assert_eq!(0b0011 | 0b1001 | 0b1100, result.or);
        }

        /// Scenario: 区間代入後に全要素が代入値に置き換わる。
        /// - Given: 上記のセグメント木に区間 [1, 4) へ 0b0101 を
        ///   代入する。
        /// - When: 全区間と部分区間を fold する。
        /// - Then: [1, 4) は全要素 0b0101 になり、影響外の要素は
        ///   変わらない。
        #[test]
        fn assign_replaces_all_elements() {
            // Given
            let mut sut = create_seg();
            sut.effect(1, 4, BitwiseAction::assign(0b0101));
            // When / Then
            let part = sut.fold(1, 4);
            assert_eq!(0b0101, part.xor);
            assert_eq!(0b0101, part.and);
            assert_eq!(0b0101, part.or);
            assert_eq!(3, part.len);
        }

        /// Scenario: 区間 AND でビットが消去される。
        /// - Given: 上記のセグメント木に全区間へ AND 0b1100 を
        ///   適用する。
        /// - When: 全区間を fold する。
        /// - Then: 各要素の下位 2 ビットが消え、AND/OR が正しく
        ///   更新される。
        #[test]
        fn and_clears_lower_bits() {
            // Given
            let mut sut = create_seg();
            sut.effect(0, 5, BitwiseAction::and(0b1100));
            // When
            let result = sut.fold(0, 5);
            // Then
            let vals = [
                0b1010 & 0b1100,
                0b1100 & 0b1100,
                0b0110 & 0b1100,
                0b0011 & 0b1100,
                0b1001 & 0b1100,
            ];
            assert_eq!(vals.iter().fold(0, |a, b| a ^ b), result.xor);
            assert_eq!(vals.iter().fold(u64::MAX, |a, b| a & b), result.and);
            assert_eq!(vals.iter().fold(0, |a, b| a | b), result.or);
        }

        /// Scenario: 区間 OR でビットが立つ。
        /// - Given: 上記のセグメント木に全区間へ OR 0b0001 を
        ///   適用する。
        /// - When: 全区間を fold する。
        /// - Then: 各要素の最下位ビットが立ち、AND/OR が正しく
        ///   更新される。
        #[test]
        fn or_sets_lowest_bit() {
            // Given
            let mut sut = create_seg();
            sut.effect(0, 5, BitwiseAction::or(0b0001));
            // When
            let result = sut.fold(0, 5);
            // Then
            let vals = [
                0b1010 | 0b0001,
                0b1100 | 0b0001,
                0b0110 | 0b0001,
                0b0011 | 0b0001,
                0b1001 | 0b0001,
            ];
            assert_eq!(vals.iter().fold(0, |a, b| a ^ b), result.xor);
            assert_eq!(vals.iter().fold(u64::MAX, |a, b| a & b), result.and);
            assert_eq!(vals.iter().fold(0, |a, b| a | b), result.or);
        }

        /// Scenario: 代入の後に XOR を適用すると正しく累積される。
        /// - Given: 上記のセグメント木に全区間へ 0xFF を代入した後、
        ///   同区間に 0x0F で XOR を適用する。
        /// - When: 全区間を fold する。
        /// - Then: 全要素が 0xF0 になり AND=OR=0xF0 になる。
        #[test]
        fn assign_then_xor_composes_correctly() {
            // Given
            let mut sut = create_seg();
            sut.effect(0, 5, BitwiseAction::assign(0xFF));
            sut.effect(0, 5, BitwiseAction::xor(0x0F));
            // When
            let result = sut.fold(0, 5);
            // Then
            assert_eq!(0xF0, result.and);
            assert_eq!(0xF0, result.or);
        }

        /// Scenario: XOR の後に代入を適用すると、XOR は
        /// 上書きされる。
        /// - Given: 上記のセグメント木に全区間へ 0xFF で XOR した後、
        ///   同区間に 0 を代入する。
        /// - When: 全区間を fold する。
        /// - Then: XOR は無効化され全要素 0 になる。
        #[test]
        fn xor_then_assign_overwrites() {
            // Given
            let mut sut = create_seg();
            sut.effect(0, 5, BitwiseAction::xor(0xFF));
            sut.effect(0, 5, BitwiseAction::assign(0));
            // When
            let result = sut.fold(0, 5);
            // Then
            assert_eq!(0, result.xor);
            assert_eq!(0, result.and);
            assert_eq!(0, result.or);
        }

        /// Scenario: 同じ値で 2 回 XOR すると元に戻る。
        /// - Given: セグメント木に全区間へ 0b1111 で XOR を
        ///   2 回適用する。
        /// - When: 全区間を fold する。
        /// - Then: 初期状態と同じ結果になる。
        #[test]
        fn double_xor_is_identity() {
            // Given
            let mut sut = create_seg();
            let before = sut.fold(0, 5);
            sut.effect(0, 5, BitwiseAction::xor(0b1111));
            sut.effect(0, 5, BitwiseAction::xor(0b1111));
            // When
            let after = sut.fold(0, 5);
            // Then
            assert_eq!(before, after);
        }

        /// Scenario: AND の後に OR を適用すると正しく累積される。
        /// - Given: 全区間へ AND 0b1100 の後、OR 0b0011 を適用する。
        /// - When: 全区間を fold する。
        /// - Then: 各要素の下位 2 ビットが消えた後に立てられた
        ///   結果が得られる。
        #[test]
        fn and_then_or_composes_correctly() {
            // Given
            let mut sut = create_seg();
            sut.effect(0, 5, BitwiseAction::and(0b1100));
            sut.effect(0, 5, BitwiseAction::or(0b0011));
            // When
            let result = sut.fold(0, 5);
            // Then
            let vals = [
                (0b1010 & 0b1100) | 0b0011,
                (0b1100 & 0b1100) | 0b0011,
                (0b0110 & 0b1100) | 0b0011,
                0b0011,
                (0b1001 & 0b1100) | 0b0011,
            ];
            assert_eq!(vals.iter().fold(0, |a, b| a ^ b), result.xor);
            assert_eq!(vals.iter().fold(u64::MAX, |a, b| a & b), result.and);
            assert_eq!(vals.iter().fold(0, |a, b| a | b), result.or);
        }

        /// Scenario: 空区間への effect は状態を変えない。
        /// - Given: [1, 2, 3] のセグメント木がある。
        /// - When: 空区間 [1, 1) に各種作用を適用する。
        /// - Then: fold 結果が初期状態と変わらない。
        #[test]
        fn empty_range_effect_is_noop() {
            // Given
            let mut sut = RangeBitwiseXorAndOr::from_values(vec![1, 2, 3]);
            let before = sut.fold(0, 3);
            // When
            sut.effect(1, 1, BitwiseAction::xor(0xFF));
            sut.effect(1, 1, BitwiseAction::assign(0xFF));
            sut.effect(1, 1, BitwiseAction::and(0));
            sut.effect(1, 1, BitwiseAction::or(0xFF));
            // Then
            assert_eq!(before, sut.fold(0, 3));
        }

        /// Scenario: u64::MAX を含む操作が正しく処理される。
        /// - Given: [u64::MAX, 0] で構築したセグメント木がある。
        /// - When: 全区間を fold する。
        /// - Then: XOR=u64::MAX, AND=0, OR=u64::MAX になる。
        #[test]
        fn handles_max_values() {
            // Given
            let mut sut = RangeBitwiseXorAndOr::from_values(vec![u64::MAX, 0]);
            // When
            let result = sut.fold(0, 2);
            // Then
            assert_eq!(u64::MAX, result.xor);
            assert_eq!(0, result.and);
            assert_eq!(u64::MAX, result.or);
        }
    }

    // ランダムテスト: ナイーブ実装との照合を検証する。
    mod random {
        use super::*;
        use rand::Rng;

        /// ナイーブな配列実装。区間操作を愚直に O(n) で行う。
        struct Naive {
            a: Vec<u64>,
        }

        impl Naive {
            fn new(a: Vec<u64>) -> Self {
                Naive { a }
            }

            fn xor(&mut self, l: usize, r: usize, v: u64) {
                for x in &mut self.a[l..r] {
                    *x ^= v;
                }
            }

            fn assign(&mut self, l: usize, r: usize, v: u64) {
                for x in &mut self.a[l..r] {
                    *x = v;
                }
            }

            fn and(&mut self, l: usize, r: usize, v: u64) {
                for x in &mut self.a[l..r] {
                    *x &= v;
                }
            }

            fn or(&mut self, l: usize, r: usize, v: u64) {
                for x in &mut self.a[l..r] {
                    *x |= v;
                }
            }

            fn fold(&self, l: usize, r: usize) -> XorAndOrData {
                if l == r {
                    return XorAndOrMonoid::id();
                }
                let slice = &self.a[l..r];
                XorAndOrData {
                    xor: slice.iter().fold(0, |acc, &x| acc ^ x),
                    and: slice.iter().fold(u64::MAX, |acc, &x| acc & x),
                    or: slice.iter().fold(0, |acc, &x| acc | x),
                    len: slice.len(),
                }
            }
        }

        /// Scenario: ランダムな操作列に対して、遅延セグメント木と
        /// ナイーブ実装の fold 結果が一致する。
        /// - Given: ランダムな初期値で構築した遅延セグメント木と
        ///   ナイーブ実装がある。
        /// - When: ランダムな区間 XOR・区間代入・区間 AND・
        ///   区間 OR・区間 fold を繰り返す。
        /// - Then: すべての fold 結果が一致する。
        #[test]
        fn matches_naive_implementation() {
            let mut rng = rand::rng();
            let n = 50;
            let q = 500;

            for _ in 0..20 {
                // Given
                let init: Vec<u64> = (0..n).map(|_| rng.random_range(0..=0xFFFF)).collect();
                let mut sut = RangeBitwiseXorAndOr::from_values(init.clone());
                let mut naive = Naive::new(init);

                for _ in 0..q {
                    let l = rng.random_range(0..n);
                    let r = rng.random_range(l..=n);

                    match rng.random_range(0..5) {
                        // When: 区間 XOR
                        0 => {
                            let v = rng.random_range(0..=0xFFFF);
                            sut.effect(l, r, BitwiseAction::xor(v));
                            naive.xor(l, r, v);
                        }
                        // When: 区間代入
                        1 => {
                            let v = rng.random_range(0..=0xFFFF);
                            sut.effect(l, r, BitwiseAction::assign(v));
                            naive.assign(l, r, v);
                        }
                        // When: 区間 AND
                        2 => {
                            let v = rng.random_range(0..=0xFFFF);
                            sut.effect(l, r, BitwiseAction::and(v));
                            naive.and(l, r, v);
                        }
                        // When: 区間 OR
                        3 => {
                            let v = rng.random_range(0..=0xFFFF);
                            sut.effect(l, r, BitwiseAction::or(v));
                            naive.or(l, r, v);
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
