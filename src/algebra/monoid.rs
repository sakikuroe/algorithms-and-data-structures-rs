//! `Monoid` trait および一般的なモノイドの実装を定義するモジュールである.

use crate::algebra::semi_group;

/// モノイド (monoid) を表現する trait であり, 単位元を持つ半群である.
pub trait Monoid: semi_group::SemiGroup {
    /// モノイドの単位元を返す.
    ///
    /// # Returns
    /// `Self::S`: 単位元.
    fn id() -> Self::S;
}

/// `i64` 型の最小値を求めるモノイドである.
pub struct MinMonoid;

impl semi_group::SemiGroup for MinMonoid {
    type S = i64;
    fn op(a: &Self::S, b: &Self::S) -> Self::S {
        std::cmp::min(*a, *b)
    }
}

impl Monoid for MinMonoid {
    fn id() -> Self::S {
        std::i64::MAX
    }
}

/// `i64` 型の最大値を求めるモノイドである.
pub struct MaxMonoid;

impl semi_group::SemiGroup for MaxMonoid {
    type S = i64;
    fn op(a: &Self::S, b: &Self::S) -> Self::S {
        std::cmp::max(*a, *b)
    }
}

impl Monoid for MaxMonoid {
    fn id() -> Self::S {
        std::i64::MIN
    }
}

/// `i64` 型の加算を行うモノイドである.
pub struct AddMonoid;

impl semi_group::SemiGroup for AddMonoid {
    type S = i64;
    fn op(a: &Self::S, b: &Self::S) -> Self::S {
        *a + *b
    }
}

impl Monoid for AddMonoid {
    fn id() -> Self::S {
        0
    }
}

/// `u64` 型の排他的論理和 (XOR) を行うモノイドである.
pub struct XorMonoid;

impl semi_group::SemiGroup for XorMonoid {
    type S = u64;
    fn op(a: &Self::S, b: &Self::S) -> Self::S {
        *a ^ *b
    }
}

impl Monoid for XorMonoid {
    fn id() -> Self::S {
        0
    }
}

/// `u64` 型のビット単位の論理積 (AND) を行うモノイドである.
pub struct AndMonoid;

impl semi_group::SemiGroup for AndMonoid {
    type S = u64;
    fn op(a: &Self::S, b: &Self::S) -> Self::S {
        *a & *b
    }
}

impl Monoid for AndMonoid {
    fn id() -> Self::S {
        std::u64::MAX
    }
}

/// `u64` 型のビット単位の論理和 (OR) を行うモノイドである.
pub struct OrMonoid;

impl semi_group::SemiGroup for OrMonoid {
    type S = u64;
    fn op(a: &Self::S, b: &Self::S) -> Self::S {
        *a | *b
    }
}

impl Monoid for OrMonoid {
    fn id() -> Self::S {
        0
    }
}

/// `MOD` を法とする一次関数 `f(x) = ax + b` の合成を行うモノイドである.
///
/// このモジュール内の他のモノイドと異なり, この演算は可換**ではない**.
/// 2つの関数を合成する順序を入れ替えると, 一般に異なる関数になる.
/// `op(a, b)` は「まず `a` を適用し, その結果に `b` を適用する」
/// (すなわち `op(a, b)` は合成 `b ∘ a` を表す) と定義する. この
/// 「左から順に `a`, 続けて `b` を適用する」という規約は, セグメント木が
/// 区間を畳み込む際に要素を結合する順序と一致するため, 関数列に対する
/// パス合成クエリと相性がよい.
pub struct AffineMonoid<const MOD: u64>;

impl<const MOD: u64> semi_group::SemiGroup for AffineMonoid<MOD> {
    /// `(a, b)` は `f(x) = ax + b mod MOD` を表す.
    type S = (u64, u64);
    fn op(a: &Self::S, b: &Self::S) -> Self::S {
        let (a1, b1) = *a;
        let (a2, b2) = *b;
        // オーバーフローを避けるため、乗算は u128 で行う。
        let new_a = (u128::from(a2) * u128::from(a1) % u128::from(MOD)) as u64;
        let new_b = ((u128::from(a2) * u128::from(b1) + u128::from(b2)) % u128::from(MOD)) as u64;
        (new_a, new_b)
    }
}

impl<const MOD: u64> Monoid for AffineMonoid<MOD> {
    fn id() -> Self::S {
        (1, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algebra::semi_group::SemiGroup;

    /// Background: 998244353 を法とする `AffineMonoid`。
    type Sut = AffineMonoid<998244353>;

    // op のテスト: 戻り値そのもの (合成後の関数) を検証する。
    mod op {
        use super::*;

        /// Scenario: 2つの一次関数を合成すると、先に適用した関数を内側に
        /// 持つ合成関数になる。
        /// - Given: f(x) = 2x + 1 と g(x) = 3x + 5 がある。
        /// - When: `op(f, g)` (f を先に適用し、その結果に g を適用する) を
        ///   求める。
        /// - Then: g(f(x)) = 3(2x+1)+5 = 6x+8 に対応する (6, 8) になる。
        #[test]
        fn composes_first_operand_as_inner_function() {
            // Given
            let f: (u64, u64) = (2, 1);
            let g: (u64, u64) = (3, 5);
            // When
            let result = Sut::op(&f, &g);
            // Then
            assert_eq!((6, 8), result);
        }

        /// Scenario: 合成順序を入れ替えると、一般に異なる関数になる
        /// (非可換であることの確認)。
        /// - Given: f(x) = 2x + 1 と g(x) = 3x + 5 がある。
        /// - When: `op(f, g)` と `op(g, f)` の両方を求める。
        /// - Then: 2つの結果が一致しない。
        #[test]
        fn is_not_commutative() {
            // Given
            let f: (u64, u64) = (2, 1);
            let g: (u64, u64) = (3, 5);
            // When
            let fg = Sut::op(&f, &g);
            let gf = Sut::op(&g, &f);
            // Then
            assert_ne!(fg, gf);
        }

        /// Scenario: 法を超える値は正しく剰余される。
        /// - Given: 合成結果が法 998244353 を超えるような、大きな係数を
        ///   持つ2つの一次関数がある。
        /// - When: それらを合成する。
        /// - Then: 合成後の係数がいずれも法未満に収まる。
        #[test]
        fn reduces_result_modulo_mod() {
            // Given
            let f: (u64, u64) = (998244352, 998244352);
            let g: (u64, u64) = (998244352, 998244352);
            // When
            let result = Sut::op(&f, &g);
            // Then
            assert!(result.0 < 998244353);
            assert!(result.1 < 998244353);
        }
    }

    // id のテスト: 戻り値そのもの (恒等関数) を検証する。
    mod id {
        use super::*;

        /// Scenario: 単位元は恒等関数 f(x) = x であり、他の関数と合成
        /// しても、その関数を変化させない。
        /// - Given: f(x) = 7x + 3 と、単位元がある。
        /// - When: `op(id, f)` と `op(f, id)` の両方を求める。
        /// - Then: いずれも f 自身と一致する。
        #[test]
        fn does_not_change_other_function_when_composed() {
            // Given
            let f: (u64, u64) = (7, 3);
            let id = Sut::id();
            // When
            let id_then_f = Sut::op(&id, &f);
            let f_then_id = Sut::op(&f, &id);
            // Then
            assert_eq!(f, id_then_f);
            assert_eq!(f, f_then_id);
        }
    }
}
