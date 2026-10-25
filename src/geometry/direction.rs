//! 長さを持たない非ゼロの有向方向と、逆方向を同一視するキー
//!
//! 同じ向きのベクトルをまとめたり、方向でソートしたりするときに使います。
//! `(2, 4)` と `(1, 2)` から作った [`Direction`] は同じ値です。
//! `(-1, -2)` は逆方向なので別の値ですが、[`LineDirection`] では同じキーになります。
//!
//! 正の x 軸を 0 として、反時計回りに測る角度を (-π, π] の範囲で比較します。
//! 軸上なら南・東・北・西の順です。ゼロベクトルには向きがないため、構築時に
//! `None` を返します。

use super::super::math::number_theory;
use super::{kernel, primitives};
use std::{cmp, f64};

/// ベクトルの長さを区別せず、向きだけを比較するための型です。
///
/// `(2, 4)` と `(1, 2)` は同じ方向、`(-1, -2)` はその逆方向として扱います。
/// 同じ方向なら等値・ハッシュも等しく、ゼロベクトルからは構築できません。
/// `Ord` は正の x 軸を 0 とする角度 (-π, π] の順なので、軸上なら南・東・北・西の順です。
/// この順番で先に来る方を `<` として比較できます。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Direction {
    /// 互いに素な x 成分です。
    x: i128,
    /// 互いに素な y 成分です。
    y: i128,
}

/// 有向方向とその逆方向を同一視するキーです。直線の位置は含みません。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LineDirection {
    /// 逆向きも同じ符号へ正規化した x 成分です。
    x: i128,
    /// 逆向きも同じ符号へ正規化した y 成分です。
    y: i128,
}

impl Direction {
    /// 非ゼロのベクトルから、長さを区別しない方向の値を作ります。
    ///
    /// 例えば `(2, 4)` と `(1, 2)` は同じ `Direction` になり、`(-1, -2)` は逆方向になります。
    /// 同じ向きのベクトルを等値比較やハッシュのキーでまとめるときに使えます。
    ///
    /// # Args
    ///
    /// - `vector`: 向きを取り出すベクトル。
    ///
    /// # Returns
    ///
    /// 非ゼロならその方向を表す `Some(Direction)`、ゼロベクトルなら `None`。
    ///
    /// # Complexity
    ///
    /// 成分絶対値の上限を C として O(log C) 時間、O(1) 領域。
    ///
    /// # Examples
    ///
    /// ```
    /// use anmitsu::geometry::{direction, primitives};
    /// let ray = |x, y| direction::Direction::from_vector(primitives::Vector2 { x, y });
    /// assert_eq!(ray(1, 2), ray(2, 4));
    /// assert_ne!(ray(1, 2), ray(-1, -2));
    /// assert!(ray(0, 0).is_none());
    /// ```
    pub fn from_vector(vector: primitives::Vector2<i64>) -> Option<Self> {
        Self::from_components((i128::from(vector.x), i128::from(vector.y)))
    }

    /// 座標差が i64 を超える二点間でも方向を構築します。
    ///
    /// # Args
    /// - `from`, `to`: 始点と終点。
    /// # Returns
    /// 始点から終点への方向。同一点の場合だけ None。
    /// # Complexity
    /// 差の絶対値の上限を C として O(log C) 時間、O(1) 領域。
    pub fn between(from: primitives::Point2<i64>, to: primitives::Point2<i64>) -> Option<Self> {
        Self::from_components(kernel::difference((from.x, from.y), (to.x, to.y)))
    }

    /// 180 度反転した逆方向を返します。
    ///
    /// どの `Direction` でもパニックせずに反転できます。
    ///
    /// # Returns
    ///
    /// 東向きなら西向き、北向きなら南向きのように、元とは反対を向く方向。
    ///
    /// # Complexity
    ///
    /// 時間・領域とも O(1)。
    #[inline]
    pub fn opposite(self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }

    /// 反時計回りを正として 90 度単位で回転します。
    ///
    /// # Args
    /// - `turns`: 4 を法として解釈する回転数。
    /// # Returns
    /// 回転後の方向。負の回転数も受け付けます。
    /// # Complexity
    /// 時間・領域とも O(1)。
    #[inline]
    pub fn rotate_quarter_turns(self, turns: i32) -> Self {
        match turns.rem_euclid(4) {
            0 => self,
            1 => Self {
                x: -self.y,
                y: self.x,
            },
            2 => self.opposite(),
            _ => Self {
                x: self.y,
                y: -self.x,
            },
        }
    }

    /// 逆方向も同一視する集約キーを返します。
    ///
    /// # Returns
    /// 位置を持たない無向方向。
    /// # Complexity
    /// 時間・領域とも O(1)。
    #[inline]
    pub fn unoriented(self) -> LineDirection {
        let ray = if self.x < 0 || self.x == 0 && self.y < 0 {
            self.opposite()
        } else {
            self
        };
        LineDirection { x: ray.x, y: ray.y }
    }

    /// 二方向の小さいほうの角度が 90 度より小さいか、等しいか、大きいかを判定します。
    ///
    /// 内積の値そのものを取得せずに符号だけを返すため、値の表現範囲に依存しません。
    ///
    /// # Args
    ///
    /// - `other`: `self` との角度を調べる方向。
    ///
    /// # Returns
    ///
    /// 二方向の小さいほうの角度を θ としたとき、次の値を返します。
    ///
    /// - `Less`: 90 度より大きく 180 度以下。逆方向も含みます。
    /// - `Equal`: 90 度。
    /// - `Greater`: 0 度以上 90 度未満。同じ方向も含みます。
    ///
    /// # Complexity
    ///
    /// 時間・領域とも O(1)。
    ///
    /// # Examples
    ///
    /// ```
    /// use anmitsu::geometry::{direction, primitives};
    /// use std::cmp;
    /// let ray = |x, y| direction::Direction::from_vector(primitives::Vector2 { x, y }).unwrap();
    /// let east = ray(1, 0);
    /// assert_eq!(cmp::Ordering::Greater, east.dot_sign(east));
    /// assert_eq!(cmp::Ordering::Equal, east.dot_sign(ray(0, 1)));
    /// assert_eq!(cmp::Ordering::Less, east.dot_sign(east.opposite()));
    /// ```
    #[inline]
    pub fn dot_sign(self, other: Self) -> cmp::Ordering {
        kernel::dot_sign(self.components(), other.components())
    }

    /// `self` から `other` への回転方向を、外積の符号で判定します。
    ///
    /// 二方向を表すベクトルを a、b としたとき、`a.x * b.y - a.y * b.x` の符号です。
    /// 値そのものを取得せずに符号だけを返すため、値の表現範囲に依存しません。
    ///
    /// # Args
    ///
    /// - `other`: `self` から見た回転方向を調べる方向。
    ///
    /// # Returns
    ///
    /// - `Greater`: `self` から反時計回りに、0 度より大きく 180 度未満で到達します。
    /// - `Less`: `self` から時計回りに、0 度より大きく 180 度未満で到達します。
    /// - `Equal`: 同じ方向、または逆方向です。この結果だけでは両者を区別しません。
    ///
    /// # Complexity
    ///
    /// 時間・領域とも O(1)。
    ///
    /// # Examples
    ///
    /// ```
    /// use anmitsu::geometry::{direction, primitives};
    /// use std::cmp;
    /// let ray = |x, y| direction::Direction::from_vector(primitives::Vector2 { x, y }).unwrap();
    /// let east = ray(1, 0);
    /// assert_eq!(cmp::Ordering::Greater, east.cross_sign(ray(0, 1)));
    /// assert_eq!(cmp::Ordering::Less, east.cross_sign(ray(0, -1)));
    /// assert_eq!(cmp::Ordering::Equal, east.cross_sign(east));
    /// assert_eq!(cmp::Ordering::Equal, east.cross_sign(east.opposite()));
    /// ```
    #[inline]
    pub fn cross_sign(self, other: Self) -> cmp::Ordering {
        kernel::cross_sign(self.components(), other.components())
    }

    /// 小さい方の角度を [0, π] のラジアンで返します。
    ///
    /// # Args
    /// - `other`: 角度をなす方向。
    /// # Returns
    /// 有限な近似値。同方向は 0、逆方向は PI。
    /// # Complexity
    /// 時間・領域とも O(1)。
    #[inline]
    pub fn angle_to(self, other: Self) -> f64 {
        if self == other {
            return 0.0;
        }
        if self.opposite() == other {
            return f64::consts::PI;
        }
        // 外積の打ち消しを整数で処理した後に実数化します。
        let cross = kernel::cross(self.components(), other.components())
            .to_f64()
            .abs();
        let dot = kernel::dot(self.components(), other.components()).to_f64();
        cross.atan2(dot)
    }

    /// 差の絶対値が u64::MAX 以下の成分を約分します。
    ///
    /// # Args
    /// - `x`, `y`: 広い二点差または i64 のベクトル成分。
    /// # Returns
    /// ゼロだけ None、ほかは互いに素な有向方向。
    /// # Complexity
    /// 成分絶対値の上限を C として O(log C)。
    fn from_components((x, y): (i128, i128)) -> Option<Self> {
        let divisor = number_theory::gcd(x.unsigned_abs(), y.unsigned_abs());
        if divisor == 0 {
            return None;
        }
        let divisor = divisor as i128;
        Some(Self {
            x: x / divisor,
            y: y / divisor,
        })
    }

    /// 判定用の非公開の広い成分を返します。
    ///
    /// # Returns
    /// 正規化した二成分。各絶対値は u64::MAX 以下です。
    /// # Complexity
    /// O(1)。
    #[inline]
    pub(super) fn components(self) -> (i128, i128) {
        (self.x, self.y)
    }
}

impl cmp::Ord for Direction {
    /// 正の x 軸から反時計回りに測る角度を、(-π, π] の範囲で比較します。
    ///
    /// 南・東・北・西の順に並びます。`self` の角度が小さければ `Less`、
    /// 大きければ `Greater`、同じ方向なら `Equal` を返します。
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        kernel::argument_cmp(self.components(), other.components())
    }
}
impl cmp::PartialOrd for Direction {
    /// 正規化した方向の全順序を返します。
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}
