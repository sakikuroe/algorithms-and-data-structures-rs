//! 長さを持たない非ゼロの有向方向と、逆方向を同一視するキー
//!
//! `Direction` の等値・ハッシュ・順序は正の整数倍に依存しません。
//! 順序は (-π, π] の反時計回り順です。ゼロは構築時に `None` として分離します。

use super::super::math::number_theory;
use super::{kernel, primitives};
use std::{cmp, f64};

/// 正の整数倍を同じ値とする、非ゼロの有向方向です。
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
    /// ベクトルから方向を構築します。
    ///
    /// # Args
    /// - `vector`: 長さを除いて正規化するベクトル。
    /// # Returns
    /// 非ゼロなら方向、ゼロなら None。
    /// # Complexity
    /// 成分絶対値の上限を C として O(log C) 時間、O(1) 領域。
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

    /// 逆方向を返します。内部の広い成分で反転するので失敗しません。
    ///
    /// # Returns
    /// 180 度反転した方向。
    /// # Complexity
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

    /// 内積の値の範囲に依存せず、符号を返します。
    ///
    /// # Args
    /// - `other`: 比較する非ゼロ方向。
    /// # Returns
    /// 鈍角・直角・鋭角に対応する Less・Equal・Greater。
    /// # Complexity
    /// 時間・領域とも O(1)。
    #[inline]
    pub fn dot_sign(self, other: Self) -> cmp::Ordering {
        kernel::dot_sign(self.components(), other.components())
    }

    /// 外積の値の範囲に依存せず、符号を返します。
    ///
    /// # Args
    /// - `other`: 比較する非ゼロ方向。
    /// # Returns
    /// 外積の負・ゼロ・正に対応する比較結果。
    /// # Complexity
    /// 時間・領域とも O(1)。
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
    /// (-π, π] の偏角順を比較します。
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
