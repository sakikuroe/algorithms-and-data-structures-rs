//! 法 998244353 上の形式的べき級数の減算を実装するモジュールである。

use super::super::modulo;
use std::ops;

impl ops::Sub for super::FPS {
    type Output = super::FPS;

    /// 2 つの形式的べき級数を減算する。
    ///
    /// # Args
    /// - `self`: 左辺の系列
    /// - `rhs`: 右辺の系列
    ///
    /// # Returns
    /// `Self::Output`: 各係数を法 998244353 で減算した結果
    ///
    /// # Constraints
    /// 制約はない。
    ///
    /// # Panics
    /// - この関数はパニックしない。
    ///
    /// # Complexity
    /// - 時間計算量: O(N)。N は大きい方の項数である。
    /// - 空間計算量: O(N)。N は結果の項数である。
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
        // 係数列の長さを大きい方にそろえ、不足分を 0 で埋める。
        if self.coeffs.len() < rhs.coeffs.len() {
            self.coeffs.resize(rhs.coeffs.len(), 0);
        }

        // 右辺の係数を走査し、同次数の係数を法 998244353 上で減算する。
        for (i, coeff) in rhs.coeffs.into_iter().enumerate() {
            self.coeffs[i] = modulo::sub(self.coeffs[i], coeff);
        }

        // 末尾のゼロ係数を取り除き、正規形に保つ。
        self.trim();
        self
    }
}

impl ops::Neg for super::FPS {
    type Output = super::FPS;

    /// 各係数を符号反転した系列を返す。
    ///
    /// # Args
    /// - `self`: 符号を反転する系列
    ///
    /// # Returns
    /// `Self::Output`: 係数を法 998244353 上で反転した系列
    ///
    /// # Constraints
    /// 制約はない。
    ///
    /// # Panics
    /// - この関数はパニックしない。
    ///
    /// # Complexity
    /// - 時間計算量: O(N)。N は項数である。
    /// - 空間計算量: O(N)。N は結果の項数である。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::modulo998244353::fps;
    ///
    /// let fps = fps::FPS::new(vec![1, 2]);
    /// assert_eq!(fps::FPS::new(vec![998244352, 998244351]), -fps);
    /// ```
    fn neg(mut self) -> Self::Output {
        // 各係数を法 998244353 上で加法逆元へ変換する。
        self.coeffs.iter_mut().for_each(|c| *c = modulo::neg(*c));
        self
    }
}

impl ops::SubAssign for super::FPS {
    /// 右辺の系列を減算し、自身を更新する。
    ///
    /// # Args
    /// - `self`: 更新対象となる系列
    /// - `rhs`: 減算する系列
    ///
    /// # Returns
    /// `()`: 自身を更新するだけで値は返さない。
    ///
    /// # Constraints
    /// 制約はない。
    ///
    /// # Panics
    /// - この関数はパニックしない。
    ///
    /// # Complexity
    /// - 時間計算量: O(N)。N は大きい方の項数である。
    /// - 空間計算量: O(N)。N は更新後の項数である。
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
        // 係数列の長さを大きい方にそろえ、不足分を 0 で埋める。
        if self.coeffs.len() < rhs.coeffs.len() {
            self.coeffs.resize(rhs.coeffs.len(), 0);
        }

        // 右辺の係数を走査し、同次数の係数を法 998244353 上で減算する。
        for (i, coeff) in rhs.coeffs.into_iter().enumerate() {
            self.coeffs[i] = modulo::sub(self.coeffs[i], coeff);
        }

        // 末尾のゼロ係数を取り除き、正規形に保つ。
        self.trim();
    }
}

#[cfg(test)]
mod tests {
    use super::super::FPS;
    use super::*;

    /// Background: 係数 [5, 2] を持つ系列と、 係数 [3] を持つ系列の組
    fn create_operands() -> (FPS, FPS) {
        (FPS::new(vec![5, 2]), FPS::new(vec![3]))
    }

    // sub のテスト: 戻り値そのものを検証する
    mod sub {
        use super::*;

        /// Scenario: 項数の異なる 2 つの系列を減算すると、 同次数の係数同士が法 998244353 で減算される
        /// - Given: 係数 [5, 2] を持つ系列と、 係数 [3] を持つ系列がある
        /// - When: 前者から後者を - で減算する
        /// - Then: 係数 [2, 2] を持つ系列が返る
        #[test]
        fn subtracts_coefficients_of_different_length_operands() {
            // Given
            let (sut, rhs) = create_operands();
            // When
            let result = sut - rhs;
            // Then
            assert_eq!(FPS::new(vec![2, 2]), result);
        }

        /// Scenario: 境界値を含む組み合わせで減算しても、 各項が法 998244353 で正しく減算され、 末尾のゼロは正規化される
        /// - Given: 空系列、 単項の系列、 0 や MOD-1 付近の係数など、 境界となる係数列の組がある
        /// - When: それぞれの組を - で減算する
        /// - Then: 各ケースで期待通りに正規化された係数列が返る
        #[test]
        fn subtracts_coefficients_for_boundary_value_combinations() {
            // Given
            let cases = [
                // 両辺が空系列 (ゼロ多項式同士の減算)
                (vec![], vec![], vec![]),
                // 右辺が空系列
                (vec![7], vec![], vec![7]),
                // 単項同士が等しく、 結果がゼロ多項式になる
                (vec![7], vec![7], vec![]),
                // 0 から引くと法未満に折り返す (wraparound)
                (vec![0], vec![1], vec![modulo::M - 1]),
                // MOD-1 から MOD-1 を引くと 0 になり、 末尾がトリムされる
                (vec![modulo::M - 1, 2], vec![modulo::M - 1], vec![0, 2]),
            ];

            for (a_coeffs, b_coeffs, expected) in cases {
                let sut = FPS::new(a_coeffs);
                let rhs = FPS::new(b_coeffs);
                // When
                let result = sut - rhs;
                // Then
                assert_eq!(FPS::new(expected), result);
            }
        }
    }

    // neg のテスト: 戻り値そのものを検証する
    mod neg {
        use super::*;

        /// Scenario: 符号反転すると、 各係数が法 998244353 上の加法逆元になる
        /// - Given: 係数 [1, 2] を持つ系列がある
        /// - When: - で符号反転する
        /// - Then: 係数 [MOD-1, MOD-2] を持つ系列が返る
        #[test]
        fn negates_each_coefficient() {
            // Given
            let sut = FPS::new(vec![1, 2]);
            // When
            let result = -sut;
            // Then
            assert_eq!(FPS::new(vec![modulo::M - 1, modulo::M - 2]), result);
        }

        /// Scenario: 境界値でも符号反転が法 998244353 上で正しく行われる
        /// - Given: 0, 1, MOD-1 を係数に持つ系列がある
        /// - When: - で符号反転する
        /// - Then: 各ケースで期待通りの係数列が返る
        #[test]
        fn negates_coefficients_at_boundary_values() {
            // Given
            let cases = [
                // 0 の符号反転は 0 のまま
                (vec![0, 1], vec![0, modulo::M - 1]),
                // MOD-1 の符号反転は 1 になる
                (vec![modulo::M - 1], vec![1]),
                // 空系列 (ゼロ多項式) の符号反転は空系列のまま
                (vec![], vec![]),
            ];

            for (coeffs, expected) in cases {
                let sut = FPS::new(coeffs);
                // When
                let result = -sut;
                // Then
                assert_eq!(FPS::new(expected), result);
            }
        }
    }

    // sub_assign のテスト: 呼び出し後の自身の状態変化を検証する
    mod sub_assign {
        use super::*;

        /// Scenario: -= で減算すると、 自身が減算結果に更新される
        /// - Given: 係数 [5, 2] を持つ系列と、 係数 [3] を持つ系列がある
        /// - When: 前者に後者を -= する
        /// - Then: 前者の係数が [2, 2] に更新される
        #[test]
        fn updates_self_to_difference() {
            // Given
            let (mut sut, rhs) = create_operands();
            // When
            sut -= rhs;
            // Then
            assert_eq!(FPS::new(vec![2, 2]), sut);
        }

        /// Scenario: 境界値を含む組み合わせで -= しても、 sub と同じ規則で自身が更新される
        /// - Given: 空系列、 単項の系列、 0 や MOD-1 付近の係数など、 境界となる係数列の組がある
        /// - When: それぞれの組を -= で減算する
        /// - Then: 各ケースで期待通りに正規化された係数列に更新される
        #[test]
        fn updates_self_for_boundary_value_combinations() {
            // Given
            let cases = [
                (vec![], vec![], vec![]),
                (vec![7], vec![], vec![7]),
                (vec![7], vec![7], vec![]),
                (vec![0], vec![1], vec![modulo::M - 1]),
                (vec![modulo::M - 1, 2], vec![modulo::M - 1], vec![0, 2]),
            ];

            for (a_coeffs, b_coeffs, expected) in cases {
                let mut sut = FPS::new(a_coeffs);
                let rhs = FPS::new(b_coeffs);
                // When
                sut -= rhs;
                // Then
                assert_eq!(FPS::new(expected), sut);
            }
        }
    }
}
