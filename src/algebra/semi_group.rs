//! A module that defines the `SemiGroup` trait.
//! `SemiGroup` trait を定義するモジュールである.

use std::cmp;

/// A trait representing a semigroup, which has a single associative binary operation.
/// 半群 (semigroup) を表現する trait であり, 一つの結合的な二項演算を持つ.
pub trait SemiGroup {
    /// The type of the elements in the semigroup.
    /// 半群の要素の型.
    type S;

    /// An associative binary operation.
    /// 結合的な二項演算.
    ///
    /// # Args
    /// - `a`: The first operand.
    ///        第一オペランド.
    /// - `b`: The second operand.
    ///        第二オペランド.
    ///
    /// # Returns
    /// `Self::S`: Returns the result of the operation.
    ///            演算結果を返す.
    fn op(a: &Self::S, b: &Self::S) -> Self::S;
}

/// A semigroup for the minimum operation on `i64`.
/// `i64` 型の最小値を求める半群である.
pub struct MinSemiGroup;

impl SemiGroup for MinSemiGroup {
    type S = i64;
    fn op(a: &Self::S, b: &Self::S) -> Self::S {
        cmp::min(*a, *b)
    }
}

/// A semigroup for the maximum operation on `i64`.
/// `i64` 型の最大値を求める半群である.
pub struct MaxSemiGroup;

impl SemiGroup for MaxSemiGroup {
    type S = i64;
    fn op(a: &Self::S, b: &Self::S) -> Self::S {
        cmp::max(*a, *b)
    }
}

/// A semigroup for the addition operation on `i64`.
/// `i64` 型の加算を行う半群である.
pub struct AddSemiGroup;

impl SemiGroup for AddSemiGroup {
    type S = i64;
    fn op(a: &Self::S, b: &Self::S) -> Self::S {
        *a + *b
    }
}
