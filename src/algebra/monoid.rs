//! A module that defines the `Monoid` trait and common monoid implementations.
//! `Monoid` trait および一般的なモノイドの実装を定義するモジュールである.

use crate::algebra::semi_group;

/// A trait representing a monoid, which is a semigroup with an identity element.
/// モノイド (monoid) を表現する trait であり, 単位元を持つ半群である.
pub trait Monoid: semi_group::SemiGroup {
    /// Returns the identity element of the monoid.
    /// モノイドの単位元を返す.
    ///
    /// # Returns
    /// `Self::S`: The identity element.
    ///            単位元.
    fn id() -> Self::S;
}

/// A monoid for the minimum operation on `i64`.
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

/// A monoid for the maximum operation on `i64`.
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
/// A monoid for the addition operation on `i64`.
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

/// A monoid for the XOR operation on `u64`.
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

/// A monoid for the bitwise AND operation on `u64`.
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

/// A monoid for the bitwise OR operation on `u64`.
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
