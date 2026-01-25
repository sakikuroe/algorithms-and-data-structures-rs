//! 法 998244353 上の形式的べき級数の加算を実装するモジュールである.

use super::super::modulo;
use std::ops;

impl ops::Add for super::FPS {
    type Output = super::FPS;

    /// 2 つの形式的べき級数を加算する.
    ///
    /// # Args
    /// - `self`: 左辺の系列.
    /// - `rhs`: 右辺の系列.
    ///
    /// # Returns
    /// `Self::Output`: 各係数を法 998244353 で加算した結果.
    ///
    /// # Constraints
    /// 制約はない.
    ///
    /// # Panics
    /// - この関数はパニックしない.
    ///
    /// # Complexity
    /// - 時間計算量: O(N). N は大きい方の項数である.
    /// - 空間計算量: O(N). N は結果の項数である.
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::modulo998244353::fps;
    ///
    /// let a = fps::FPS::new(vec![1, 2]);
    /// let b = fps::FPS::new(vec![3]);
    /// assert_eq!(fps::FPS::new(vec![4, 2]), a + b);
    /// ```
    fn add(mut self, rhs: Self) -> Self::Output {
        // 係数列の長さを大きい方にそろえ, 不足分を 0 で埋める.
        if self.coeffs.len() < rhs.coeffs.len() {
            self.coeffs.resize(rhs.coeffs.len(), 0);
        }

        // 右辺の係数を走査し, 同次数の係数を法 998244353 上で加算する.
        for (i, coeff) in rhs.coeffs.into_iter().enumerate() {
            self.coeffs[i] = modulo::add(self.coeffs[i], coeff);
        }

        // 末尾のゼロ係数を取り除き, 正規形に保つ.
        self.trim();
        self
    }
}

impl ops::AddAssign for super::FPS {
    /// 右辺の系列を加算し, 自身を更新する.
    ///
    /// # Args
    /// - `self`: 更新対象となる系列.
    /// - `rhs`: 加算する系列.
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
    /// - 時間計算量: O(N). N は大きい方の項数である.
    /// - 空間計算量: O(N). N は更新後の項数である.
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::modulo998244353::fps;
    ///
    /// let mut a = fps::FPS::new(vec![1, 2]);
    /// let b = fps::FPS::new(vec![3]);
    /// a += b;
    /// assert_eq!(fps::FPS::new(vec![4, 2]), a);
    /// ```
    fn add_assign(&mut self, rhs: Self) {
        // 係数列の長さを大きい方にそろえ, 不足分を 0 で埋める.
        if self.coeffs.len() < rhs.coeffs.len() {
            self.coeffs.resize(rhs.coeffs.len(), 0);
        }

        // 右辺の係数を走査し, 同次数の係数を法 998244353 上で加算する.
        for (i, coeff) in rhs.coeffs.into_iter().enumerate() {
            self.coeffs[i] = modulo::add(self.coeffs[i], coeff);
        }

        // 末尾のゼロ係数を取り除き, 正規形に保つ.
        self.trim();
    }
}
