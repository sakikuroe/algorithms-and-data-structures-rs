//! 区間アフィン変換・区間和クエリの具象インスタンスを提供する
//! モジュールである。
//!
//! アフィン変換 `f(x) = mx + b` による区間更新と区間和クエリを
//! 組み合わせた遅延セグメント木の具象型を定義する。加算・乗算・
//! 代入・アフィン変換はすべて `(m, b)` のパラメータ違いで表現
//! できるため、合成規則が 1 つの式で閉じる。

use super::super::super::{
    algebra::{monoid, semi_group},
    modulo998244353::modint,
};
use super::lazy_segment_tree;

/// 区間アフィン変換・区間和クエリ用の遅延セグメント木の
/// 型エイリアスである。
///
/// # Examples
/// ```
/// use anmitsu::ds::segment_tree::range_affine_range_sum;
/// use anmitsu::modulo998244353::modint;
///
/// let mut seg = range_affine_range_sum::RangeAffineRangeSum::from_vec(
///     vec![(modint::ModInt998244353::new(1), 1)],
/// );
/// seg.effect(
///     0, 1,
///     range_affine_range_sum::AffineAction::add(
///         modint::ModInt998244353::new(10),
///     ),
/// );
/// let (sum, _) = seg.fold(0, 1);
/// assert_eq!(11, sum.val());
/// ```
pub type RangeAffineRangeSum =
    lazy_segment_tree::SegmentTreeLazyDense<RangeAffineFoldSumMonoid, AffineAction>;

/// 区間和を `(値の総和, 要素数)` のペアとして保持する
/// モノイドである。
///
/// 要素数を合わせて保持することで、アフィン変換
/// `f(x) = mx + b` を区間に適用する際に、定数項 `b` を
/// 要素数倍して加算することができる。
pub struct RangeAffineFoldSumMonoid;

impl semi_group::SemiGroup for RangeAffineFoldSumMonoid {
    type S = (modint::ModInt998244353, usize);

    /// 2 つの区間の和と要素数をそれぞれ加算して結合する。
    fn op(a: &Self::S, b: &Self::S) -> Self::S {
        (a.0 + b.0, a.1 + b.1)
    }
}

impl monoid::Monoid for RangeAffineFoldSumMonoid {
    /// 空区間を表す単位元を返す。値の総和は 0、要素数も 0
    /// である。
    fn id() -> Self::S {
        (modint::ModInt998244353::new(0), 0)
    }
}

/// アフィン変換 `f(x) = m * x + b` を遅延セグメント木の作用
/// として表現する。
///
/// 加算 (`m=1, b=a`)、乗算 (`m=a, b=0`)、
/// 代入 (`m=0, b=u`)、アフィン変換 (`m=a, b=b`) はすべて
/// この 2 パラメータで表現できる。合成は
/// `g(f(x)) = m₂(m₁x + b₁) + b₂ = (m₂m₁)x + (m₂b₁ + b₂)`
/// の 1 つの式で閉じるため、場合分けが不要である。
#[derive(Clone, Copy, Debug)]
pub struct AffineAction {
    /// 一次の係数。既存の値に掛ける倍率に対応する。
    pub m: modint::ModInt998244353,
    /// 定数項。既存の値に加算するオフセットに対応する。
    pub b: modint::ModInt998244353,
}

impl AffineAction {
    /// アフィン変換 `f(x) = m * x + b` を生成する。
    ///
    /// # Args
    /// - `m` - 一次の係数
    /// - `b` - 定数項
    ///
    /// # Returns
    /// 指定された係数と定数項を持つ `AffineAction` を返す。
    ///
    /// # Examples
    /// ```
    /// use anmitsu::ds::segment_tree::range_affine_range_sum;
    /// use anmitsu::modulo998244353::modint;
    ///
    /// let action =
    ///     range_affine_range_sum::AffineAction::affine(
    ///         modint::ModInt998244353::new(2),
    ///         modint::ModInt998244353::new(3),
    ///     );
    /// assert_eq!(2, action.m.val());
    /// assert_eq!(3, action.b.val());
    /// ```
    pub fn affine(m: modint::ModInt998244353, b: modint::ModInt998244353) -> Self {
        AffineAction { m, b }
    }

    /// 区間加算 `f(x) = x + a` を生成する。
    ///
    /// # Args
    /// - `a` - 加算する値
    ///
    /// # Returns
    /// `m = 1, b = a` の `AffineAction` を返す。
    ///
    /// # Examples
    /// ```
    /// use anmitsu::ds::segment_tree::range_affine_range_sum;
    /// use anmitsu::modulo998244353::modint;
    ///
    /// let action =
    ///     range_affine_range_sum::AffineAction::add(
    ///         modint::ModInt998244353::new(5),
    ///     );
    /// assert_eq!(1, action.m.val());
    /// assert_eq!(5, action.b.val());
    /// ```
    pub fn add(a: modint::ModInt998244353) -> Self {
        AffineAction {
            m: modint::ModInt998244353::new(1),
            b: a,
        }
    }

    /// 区間乗算 `f(x) = a * x` を生成する。
    ///
    /// # Args
    /// - `a` - 乗算する値
    ///
    /// # Returns
    /// `m = a, b = 0` の `AffineAction` を返す。
    ///
    /// # Examples
    /// ```
    /// use anmitsu::ds::segment_tree::range_affine_range_sum;
    /// use anmitsu::modulo998244353::modint;
    ///
    /// let action =
    ///     range_affine_range_sum::AffineAction::mul(
    ///         modint::ModInt998244353::new(3),
    ///     );
    /// assert_eq!(3, action.m.val());
    /// assert_eq!(0, action.b.val());
    /// ```
    pub fn mul(a: modint::ModInt998244353) -> Self {
        AffineAction {
            m: a,
            b: modint::ModInt998244353::new(0),
        }
    }

    /// 区間代入 `f(x) = u` を生成する。
    ///
    /// # Args
    /// - `u` - 代入する値
    ///
    /// # Returns
    /// `m = 0, b = u` の `AffineAction` を返す。
    ///
    /// # Examples
    /// ```
    /// use anmitsu::ds::segment_tree::range_affine_range_sum;
    /// use anmitsu::modulo998244353::modint;
    ///
    /// let action =
    ///     range_affine_range_sum::AffineAction::assign(
    ///         modint::ModInt998244353::new(7),
    ///     );
    /// assert_eq!(0, action.m.val());
    /// assert_eq!(7, action.b.val());
    /// ```
    pub fn assign(u: modint::ModInt998244353) -> Self {
        AffineAction {
            m: modint::ModInt998244353::new(0),
            b: u,
        }
    }
}

impl lazy_segment_tree::Hom<(modint::ModInt998244353, usize)> for AffineAction {
    /// 区間の集約値 `(総和, 要素数)` にアフィン変換を適用する。
    ///
    /// 各要素 `x_i` に `m * x_i + b` を適用した場合の総和は
    /// `m * Σx_i + b * n` であるため、要素数 `n` を用いて
    /// 計算する。
    ///
    /// # Args
    /// - `x` - 区間の集約値 `(総和, 要素数)` への参照
    ///
    /// # Returns
    /// アフィン変換適用後の `(総和, 要素数)` を返す。
    /// 要素数は変化しない。
    fn f(&self, x: &(modint::ModInt998244353, usize)) -> (modint::ModInt998244353, usize) {
        // 要素数を ModInt に変換し、定数項の寄与を計算する。
        let len = modint::ModInt998244353::new(x.1 as u64);
        (self.m * x.0 + self.b * len, x.1)
    }

    /// 2 つのアフィン変換を合成する。
    ///
    /// `self` を先に適用し、`other` を後から適用する。
    /// `other(self(x)) = other.m * (self.m * x + self.b)
    ///   + other.b`
    /// `= (other.m * self.m) * x + (other.m * self.b
    ///   + other.b)`
    ///
    /// # Args
    /// - `other` - `self` の後に適用するアフィン変換
    ///
    /// # Returns
    /// 合成されたアフィン変換を返す。
    fn composition(&self, other: &Self) -> Self {
        AffineAction {
            m: other.m * self.m,
            b: other.m * self.b + other.b,
        }
    }
}
