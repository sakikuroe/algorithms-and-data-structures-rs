//! 法 998244353 上の形式的べき級数の減算を実装するモジュールである.

use super::super::modulo;
use std::ops::{Neg, Sub, SubAssign};

impl Sub for super::FPS {
    type Output = super::FPS;

    /// 2 つの形式的べき級数を減算する.
    ///
    /// # Args
    /// - `self`: 左辺の系列.
    /// - `rhs`: 右辺の系列.
    ///
    /// # Returns
    /// `Self::Output`: 各係数を法 998244353 で減算した結果.
    ///
    /// # Constraints
    /// 制約はない.
    ///
    /// # Panics
    /// - この関数はパニックしない.
    ///
    /// # Complexity
    /// - Time complexity: O(N). N は大きい方の項数.
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::modulo998244353::fps;
    ///
    /// let a = fps::FPS::new(vec![5, 2]);
    /// let b = fps::FPS::new(vec![3]);
    /// assert_eq!(fps::FPS::new(vec![2, 2]), a - b);
    /// ```
    fn sub(mut self, rhs: Self) -> Self::Output {
        if self.coeffs.len() < rhs.coeffs.len() {
            self.coeffs.resize(rhs.coeffs.len(), 0);
        }
        for (i, coeff) in rhs.coeffs.into_iter().enumerate() {
            self.coeffs[i] = modulo::sub(self.coeffs[i], coeff);
        }
        self.trim();
        self
    }
}

impl Neg for super::FPS {
    type Output = super::FPS;

    /// 各係数を符号反転した系列を返す.
    ///
    /// # Args
    /// - `self`: 符号を反転する系列.
    ///
    /// # Returns
    /// `Self::Output`: 係数を法 998244353 上で反転した系列.
    ///
    /// # Constraints
    /// 制約はない.
    ///
    /// # Panics
    /// - この関数はパニックしない.
    ///
    /// # Complexity
    /// - Time complexity: O(N). N は項数.
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::modulo998244353::fps;
    ///
    /// let fps = fps::FPS::new(vec![1, 2]);
    /// assert_eq!(fps::FPS::new(vec![998244352, 998244351]), -fps);
    /// ```
    fn neg(mut self) -> Self::Output {
        self.coeffs.iter_mut().for_each(|c| *c = modulo::neg(*c));
        self
    }
}

impl SubAssign for super::FPS {
    /// 右辺の系列を減算し, 自身を更新する.
    ///
    /// # Args
    /// - `self`: 更新対象となる系列.
    /// - `rhs`: 減算する系列.
    ///
    /// # Returns
    /// `()`: 自身を更新するだけで値は返さない.
    ///
    /// # Constraints
    /// 制約はない.
    ///
    /// # Panics
    /// - この関数はパニックしない.
    ///
    /// # Complexity
    /// - Time complexity: O(N). N は大きい方の項数.
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::modulo998244353::fps;
    ///
    /// let mut a = fps::FPS::new(vec![5, 2]);
    /// let b = fps::FPS::new(vec![3]);
    /// a -= b;
    /// assert_eq!(fps::FPS::new(vec![2, 2]), a);
    /// ```
    fn sub_assign(&mut self, rhs: Self) {
        if self.coeffs.len() < rhs.coeffs.len() {
            self.coeffs.resize(rhs.coeffs.len(), 0);
        }
        for (i, coeff) in rhs.coeffs.into_iter().enumerate() {
            self.coeffs[i] = modulo::sub(self.coeffs[i], coeff);
        }
        self.trim();
    }
}
