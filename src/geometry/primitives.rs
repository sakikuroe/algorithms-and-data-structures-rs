//! 整数座標の点・ベクトルと、長さ・距離・積の計算
//!
//! 点の差はベクトル、点の平行移動は点を返します。座標の演算子はすべて
//! 結果が `i64` に収まることを検査し、範囲外なら debug・release ともパニックします。
//! 二乗距離は、各座標の差が `i64` に収まらない二点にも使えます。
//! ただし、距離の二乗が `u128` を超えるとパニックします。
//! 点の引き算 `a - b` は、右辺の点 b から左辺の点 a へ向かうベクトルです。
//!
//! ```
//! use anmitsu::geometry::primitives;
//! let origin = primitives::Point2 { x: 3_i64, y: 5 };
//! let step = primitives::Vector2 { x: 2_i64, y: -1 };
//! let moved = origin + step;
//! assert_eq!(step, moved - origin);
//! assert_eq!(5, origin.distance_squared(moved));
//! ```

use super::kernel;
use std::{cmp, ops};

/// 二次元の点です。等値とハッシュは座標によって決まります。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Point2<T> {
    /// x 座標。
    pub x: T,
    /// y 座標。
    pub y: T,
}

/// 長さを持つ二次元のベクトルです。同方向でも座標が異なれば別の値です。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Vector2<T> {
    /// x 成分。
    pub x: T,
    /// y 成分。
    pub y: T,
}

impl Point2<i64> {
    /// 二点間の距離の二乗を取得します。
    ///
    /// # Args
    /// - `other`: もう一方の点。
    /// # Returns
    /// 非負の二乗距離。
    /// # Panics
    /// 結果が u128 に収まらない場合。
    /// # Complexity
    /// 時間・追加領域とも O(1)。
    pub fn distance_squared(self, other: Self) -> u128 {
        let delta = kernel::difference((self.x, self.y), (other.x, other.y));
        kernel::dot(delta, delta).magnitude()
    }
}

impl Vector2<i64> {
    /// 内積の値を取得します。
    ///
    /// # Args
    /// - `other`: もう一方のベクトル。
    /// # Returns
    /// `self.x * other.x + self.y * other.y`。
    /// # Panics
    /// 結果が i128 に収まらない場合。
    /// # Complexity
    /// 時間・追加領域とも O(1)。
    pub fn dot(self, other: Self) -> i128 {
        kernel::dot(self.components(), other.components()).to_i128()
    }

    /// 外積の値を取得します。反時計回りの組が正です。
    ///
    /// # Args
    /// - `other`: もう一方のベクトル。
    /// # Returns
    /// `self.x * other.y - self.y * other.x`。
    /// # Complexity
    /// 時間・追加領域とも O(1)。
    pub fn cross(self, other: Self) -> i128 {
        kernel::cross(self.components(), other.components()).to_i128()
    }

    /// 内積 `self.x * other.x + self.y * other.y` の符号だけを返します。
    ///
    /// # Args
    ///
    /// - `other`: 内積を取るもう一方のベクトル。ゼロも受け付けます。
    ///
    /// # Returns
    ///
    /// 負なら `Less`、ゼロなら `Equal`、正なら `Greater`。
    /// どちらかがゼロベクトルでも `Equal` です。90 度の角をなすかの判定には、
    /// 両方が非ゼロであることも確認してください。
    /// 非ゼロ同士での角度との対応は [`Direction::dot_sign`](super::direction::Direction::dot_sign) と同じです。
    ///
    /// # Complexity
    ///
    /// 時間・追加領域とも O(1)。
    #[inline]
    pub fn dot_sign(self, other: Self) -> cmp::Ordering {
        kernel::dot_sign(self.components(), other.components())
    }

    /// 外積 `self.x * other.y - self.y * other.x` の符号だけを返します。
    ///
    /// # Args
    ///
    /// - `other`: 外積の右側に置くベクトル。ゼロも受け付けます。
    ///
    /// # Returns
    ///
    /// 負なら `Less`、ゼロなら `Equal`、正なら `Greater`。
    /// `Equal` には、どちらかがゼロの場合、同方向の場合、逆方向の場合がすべて含まれます。
    /// 非ゼロ同士での回転方向との対応は [`Direction::cross_sign`](super::direction::Direction::cross_sign) を参照してください。
    ///
    /// # Complexity
    ///
    /// 時間・追加領域とも O(1)。
    #[inline]
    pub fn cross_sign(self, other: Self) -> cmp::Ordering {
        kernel::cross_sign(self.components(), other.components())
    }

    /// 長さの二乗を取得します。
    ///
    /// # Returns
    /// `x * x + y * y`。i64 の全成分範囲で u128 に収まります。
    /// # Complexity
    /// 時間・追加領域とも O(1)。
    pub fn norm_squared(self) -> u128 {
        kernel::dot(self.components(), self.components()).magnitude()
    }

    /// 反時計回りを正として、90 度単位で回転します。
    ///
    /// # Args
    /// - `turns`: 4 を法として解釈する回転数。負の値も受け付けます。
    /// # Returns
    /// 回転後のベクトル。
    /// # Panics
    /// 回転後の成分が i64 に収まらない場合。
    /// # Complexity
    /// 時間・追加領域とも O(1)。
    pub fn rotate_quarter_turns(self, turns: i32) -> Self {
        let (x, y) = self.components();
        let (x, y) = match turns.rem_euclid(4) {
            0 => (x, y),
            1 => (-y, x),
            2 => (-x, -y),
            _ => (y, -x),
        };
        Self {
            x: i64::try_from(x).expect("rotated x does not fit i64"),
            y: i64::try_from(y).expect("rotated y does not fit i64"),
        }
    }

    /// 内部の積和へ渡す広い成分を返します。
    fn components(self) -> (i128, i128) {
        (i128::from(self.x), i128::from(self.y))
    }
}

impl ops::Add<Vector2<i64>> for Point2<i64> {
    type Output = Self;
    /// ベクトルによる平行移動を、成分ごとに検査します。
    fn add(self, rhs: Vector2<i64>) -> Self {
        Self {
            x: self.x.checked_add(rhs.x).expect("point x overflow"),
            y: self.y.checked_add(rhs.y).expect("point y overflow"),
        }
    }
}
impl ops::Sub<Vector2<i64>> for Point2<i64> {
    type Output = Self;
    /// ベクトルを逆向きに移動し、結果を検査します。
    fn sub(self, rhs: Vector2<i64>) -> Self {
        Self {
            x: self.x.checked_sub(rhs.x).expect("point x overflow"),
            y: self.y.checked_sub(rhs.y).expect("point y overflow"),
        }
    }
}
impl ops::Sub for Point2<i64> {
    type Output = Vector2<i64>;
    /// 右辺の点 `rhs` から左辺の点 `self` へ向かうベクトルを返します。
    fn sub(self, rhs: Self) -> Self::Output {
        Vector2 {
            x: self
                .x
                .checked_sub(rhs.x)
                .expect("point difference x overflow"),
            y: self
                .y
                .checked_sub(rhs.y)
                .expect("point difference y overflow"),
        }
    }
}
impl ops::AddAssign<Vector2<i64>> for Point2<i64> {
    /// 検査済みの平行移動の結果を代入します。
    fn add_assign(&mut self, rhs: Vector2<i64>) {
        *self = *self + rhs;
    }
}
impl ops::SubAssign<Vector2<i64>> for Point2<i64> {
    /// 検査済みの逆向きの平行移動の結果を代入します。
    fn sub_assign(&mut self, rhs: Vector2<i64>) {
        *self = *self - rhs;
    }
}
impl ops::Add for Vector2<i64> {
    type Output = Self;
    /// 成分ごとの和を検査します。
    fn add(self, rhs: Self) -> Self {
        Self {
            x: self.x.checked_add(rhs.x).expect("vector x overflow"),
            y: self.y.checked_add(rhs.y).expect("vector y overflow"),
        }
    }
}
impl ops::Sub for Vector2<i64> {
    type Output = Self;
    /// 成分ごとの差を検査します。
    fn sub(self, rhs: Self) -> Self {
        Self {
            x: self.x.checked_sub(rhs.x).expect("vector x overflow"),
            y: self.y.checked_sub(rhs.y).expect("vector y overflow"),
        }
    }
}
impl ops::Neg for Vector2<i64> {
    type Output = Self;
    /// 各成分の符号を反転し、最小値のオーバーフローも検出します。
    fn neg(self) -> Self {
        Self {
            x: self.x.checked_neg().expect("vector x overflow"),
            y: self.y.checked_neg().expect("vector y overflow"),
        }
    }
}
impl ops::Mul<i64> for Vector2<i64> {
    type Output = Self;
    /// 整数倍の結果を成分ごとに検査します。
    fn mul(self, rhs: i64) -> Self {
        Self {
            x: self.x.checked_mul(rhs).expect("vector x overflow"),
            y: self.y.checked_mul(rhs).expect("vector y overflow"),
        }
    }
}
impl ops::AddAssign for Vector2<i64> {
    /// 検査済みの和を代入します。
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}
impl ops::SubAssign for Vector2<i64> {
    /// 検査済みの差を代入します。
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}
impl ops::MulAssign<i64> for Vector2<i64> {
    /// 検査済みの整数倍を代入します。
    fn mul_assign(&mut self, rhs: i64) {
        *self = *self * rhs;
    }
}
