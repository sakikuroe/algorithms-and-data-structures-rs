//! 法 998244353 上の形式的べき級数の除算 (シフト) を実装するモジュールである.

impl super::FPS {
    /// `x^k` で割り, 低次の項を削除する.
    ///
    /// # Args
    /// - `k`: 割る `x` の指数.
    ///
    /// # Returns
    /// `Self`: シフト後の系列.
    ///
    /// # Constraints
    /// 制約はない.
    ///
    /// # Panics
    /// - この関数はパニックしない.
    ///
    /// # Complexity
    /// - Time complexity: O(N).
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::modulo998244353::fps;
    ///
    /// let fps = fps::FPS::new(vec![1, 2, 3]);
    /// let shifted = fps.div_xk(1);
    /// assert_eq!(2, shifted.get(0));
    /// assert_eq!(3, shifted.get(1));
    /// ```
    pub fn div_xk(&self, k: usize) -> Self {
        if k >= self.len() {
            return Self { coeffs: Vec::new() };
        }
        Self {
            coeffs: self.coeffs[k..].to_vec(),
        }
    }
}
