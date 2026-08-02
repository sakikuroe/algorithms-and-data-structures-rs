//! 法 998244353 上の形式的べき級数の除算 (シフト) を実装するモジュールである。

impl super::FPS {
    /// `x^k` で割り、低次の項を削除する。
    ///
    /// # Args
    /// - `k`: 割る `x` の指数。
    ///
    /// # Returns
    /// `Self`: シフト後の系列。
    ///
    /// # Constraints
    /// 制約はない。
    ///
    /// # Panics
    /// - この関数はパニックしない。
    ///
    /// # Complexity
    /// - 時間計算量: O(N)。N は元の項数である。
    /// - 空間計算量: O(N)。N は結果の項数である。
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
        // k が項数以上であれば、全ての項が消えるためゼロ多項式になる。
        if k >= self.len() {
            return Self { coeffs: Vec::new() };
        }

        // 係数列を k だけ左にシフトするため、k 以降の部分列を複製する。
        Self {
            coeffs: self.coeffs[k..].to_vec(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::FPS;

    // div_xk のテスト: 戻り値そのものを検証する
    mod div_xk {
        use super::*;

        /// Scenario: x^k で割ると、 k 番目以降の係数が低次側へシフトされる
        /// - Given: 係数 [1, 2, 3] を持つ系列がある
        /// - When: k = 1 で div_xk を呼び出す
        /// - Then: 係数 [2, 3] を持つ系列が返る
        #[test]
        fn shifts_coefficients_down_by_k() {
            // Given
            let sut = FPS::new(vec![1, 2, 3]);
            // When
            let result = sut.div_xk(1);
            // Then
            assert_eq!(FPS::new(vec![2, 3]), result);
        }

        /// Scenario: 境界となる k や系列の状態でも、 低次側へのシフトが正しく行われる
        /// - Given: 空系列や単項の系列、 k が項数と等しい・それを超える場合など、 境界となる組がある
        /// - When: それぞれの組で div_xk を呼び出す
        /// - Then: 各ケースで期待通りの係数列が返る
        #[test]
        fn shifts_coefficients_for_boundary_value_combinations() {
            // Given
            let cases = [
                // k = 0 (シフトなし)
                (vec![1, 2, 3], 0, vec![1, 2, 3]),
                // k が最高次と等しく、 定数項 1 個だけが残る
                (vec![1, 2, 3], 2, vec![3]),
                // k が項数と等しく、 全ての項が消えてゼロ多項式になる
                (vec![1, 2, 3], 3, vec![]),
                // k が項数を超えても、 同様にゼロ多項式になる
                (vec![1, 2, 3], 10, vec![]),
                // 空系列 (ゼロ多項式) は k によらず空系列のまま
                (vec![], 0, vec![]),
                (vec![], 5, vec![]),
                // 単項の系列を k = 0 でシフトすると、 変化しない
                (vec![7], 0, vec![7]),
            ];

            for (coeffs, k, expected) in cases {
                let sut = FPS::new(coeffs);
                // When
                let result = sut.div_xk(k);
                // Then
                assert_eq!(FPS::new(expected), result);
            }
        }
    }
}
