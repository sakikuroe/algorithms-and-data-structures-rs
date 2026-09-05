//! `SemiGroup` trait を定義するモジュールである.

use std::cmp;

/// 半群 (semigroup) を表現する trait であり, 一つの結合的な二項演算を持つ.
pub trait SemiGroup {
    /// 半群の要素の型.
    type S;

    /// 結合的な二項演算.
    ///
    /// # Args
    /// - `a` - 第一オペランド.
    /// - `b` - 第二オペランド.
    ///
    /// # Returns
    /// `Self::S` - 演算結果を返す.
    fn op(a: &Self::S, b: &Self::S) -> Self::S;
}

/// `i64` 型の最小値を求める半群である.
pub struct MinSemiGroup;

impl SemiGroup for MinSemiGroup {
    type S = i64;
    fn op(a: &Self::S, b: &Self::S) -> Self::S {
        cmp::min(*a, *b)
    }
}

/// `i64` 型の最大値を求める半群である.
pub struct MaxSemiGroup;

impl SemiGroup for MaxSemiGroup {
    type S = i64;
    fn op(a: &Self::S, b: &Self::S) -> Self::S {
        cmp::max(*a, *b)
    }
}

/// `i64` 型の加算を行う半群である.
pub struct AddSemiGroup;

impl SemiGroup for AddSemiGroup {
    type S = i64;
    fn op(a: &Self::S, b: &Self::S) -> Self::S {
        *a + *b
    }
}
