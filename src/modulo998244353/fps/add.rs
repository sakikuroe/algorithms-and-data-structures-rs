//! 法 998244353 上の形式的べき級数の加算を実装するモジュールである。

use super::super::modulo;
use std::ops;

impl ops::Add for super::FPS {
    type Output = super::FPS;

    /// 2 つの形式的べき級数を加算する。
    ///
    /// # Args
    /// - `self`: 左辺の系列。
    /// - `rhs`: 右辺の系列。
    ///
    /// # Returns
    /// `Self::Output`: 各係数を法 998244353 で加算した結果。
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
    /// let a = fps::FPS::new(vec![1, 2]);
    /// let b = fps::FPS::new(vec![3]);
    /// assert_eq!(fps::FPS::new(vec![4, 2]), a + b);
    /// ```
    fn add(mut self, rhs: Self) -> Self::Output {
        // 係数列の長さを大きい方にそろえ、不足分を 0 で埋める。
        if self.coeffs.len() < rhs.coeffs.len() {
            self.coeffs.resize(rhs.coeffs.len(), 0);
        }

        // 右辺の係数を走査し、同次数の係数を法 998244353 上で加算する。
        for (i, coeff) in rhs.coeffs.into_iter().enumerate() {
            self.coeffs[i] = modulo::add(self.coeffs[i], coeff);
        }

        // 末尾のゼロ係数を取り除き、正規形に保つ。
        self.trim();
        self
    }
}

impl ops::AddAssign for super::FPS {
    /// 右辺の系列を加算し、自身を更新する。
    ///
    /// # Args
    /// - `self`: 更新対象となる系列。
    /// - `rhs`: 加算する系列。
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
    /// let mut a = fps::FPS::new(vec![1, 2]);
    /// let b = fps::FPS::new(vec![3]);
    /// a += b;
    /// assert_eq!(fps::FPS::new(vec![4, 2]), a);
    /// ```
    fn add_assign(&mut self, rhs: Self) {
        // 係数列の長さを大きい方にそろえ、不足分を 0 で埋める。
        if self.coeffs.len() < rhs.coeffs.len() {
            self.coeffs.resize(rhs.coeffs.len(), 0);
        }

        // 右辺の係数を走査し、同次数の係数を法 998244353 上で加算する。
        for (i, coeff) in rhs.coeffs.into_iter().enumerate() {
            self.coeffs[i] = modulo::add(self.coeffs[i], coeff);
        }

        // 末尾のゼロ係数を取り除き、正規形に保つ。
        self.trim();
    }
}

#[cfg(test)]
mod tests {
    use super::super::FPS;
    use super::*;

    /// Background: 係数 [1, 2] を持つ系列と、 係数 [3] を持つ系列の組
    fn create_operands() -> (FPS, FPS) {
        (FPS::new(vec![1, 2]), FPS::new(vec![3]))
    }

    // add のテスト: 戻り値そのものを検証する
    mod add {
        use super::*;

        /// Scenario: 項数の異なる 2 つの系列を加算すると、 同次数の係数同士が法 998244353 で加算される
        /// - Given: 係数 [1, 2] を持つ系列と、 係数 [3] を持つ系列がある
        /// - When: 両者を + で加算する
        /// - Then: 係数 [4, 2] を持つ系列が返る
        #[test]
        fn sums_coefficients_of_different_length_operands() {
            // Given
            let (sut, rhs) = create_operands();
            // When
            let result = sut + rhs;
            // Then
            assert_eq!(FPS::new(vec![4, 2]), result);
        }

        /// Scenario: 境界値を含む組み合わせで加算しても、 各項が法 998244353 で正しく加算され、 末尾のゼロは正規化される
        /// - Given: 空系列、 単項の系列、 MOD-1 付近の係数など、 境界となる係数列の組がある
        /// - When: それぞれの組を + で加算する
        /// - Then: 各ケースで期待通りに正規化された係数列が返る
        #[test]
        fn sums_coefficients_for_boundary_value_combinations() {
            // Given
            let cases = [
                // 両辺が空系列 (ゼロ多項式同士の加算)
                (vec![], vec![], vec![]),
                // 左辺が空系列
                (vec![], vec![7], vec![7]),
                // 右辺が空系列
                (vec![7], vec![], vec![7]),
                // 単項同士で、 和が法を超えて折り返す (wraparound)
                (vec![modulo::M - 1], vec![2], vec![1]),
                // MOD-1 同士の加算 (法未満に収まる境界)
                (
                    vec![modulo::M - 1],
                    vec![modulo::M - 1],
                    vec![modulo::M - 2],
                ),
                // 最高次の係数同士が打ち消し合い、 結果の項数が減る (末尾トリム)
                (vec![1, 2], vec![0, modulo::M - 2], vec![1]),
            ];

            for (a_coeffs, b_coeffs, expected) in cases {
                let sut = FPS::new(a_coeffs);
                let rhs = FPS::new(b_coeffs);
                // When
                let result = sut + rhs;
                // Then
                assert_eq!(FPS::new(expected), result);
            }
        }
    }

    // add_assign のテスト: 呼び出し後の自身の状態変化を検証する
    mod add_assign {
        use super::*;

        /// Scenario: += で加算すると、 自身が加算結果に更新される
        /// - Given: 係数 [1, 2] を持つ系列と、 係数 [3] を持つ系列がある
        /// - When: 前者に後者を += する
        /// - Then: 前者の係数が [4, 2] に更新される
        #[test]
        fn updates_self_to_sum() {
            // Given
            let (mut sut, rhs) = create_operands();
            // When
            sut += rhs;
            // Then
            assert_eq!(FPS::new(vec![4, 2]), sut);
        }

        /// Scenario: 境界値を含む組み合わせで += しても、 add と同じ規則で自身が更新される
        /// - Given: 空系列、 単項の系列、 MOD-1 付近の係数など、 境界となる係数列の組がある
        /// - When: それぞれの組を += で加算する
        /// - Then: 各ケースで期待通りに正規化された係数列に更新される
        #[test]
        fn updates_self_for_boundary_value_combinations() {
            // Given
            let cases = [
                (vec![], vec![], vec![]),
                (vec![], vec![7], vec![7]),
                (vec![7], vec![], vec![7]),
                (vec![modulo::M - 1], vec![2], vec![1]),
                (
                    vec![modulo::M - 1],
                    vec![modulo::M - 1],
                    vec![modulo::M - 2],
                ),
                (vec![1, 2], vec![0, modulo::M - 2], vec![1]),
            ];

            for (a_coeffs, b_coeffs, expected) in cases {
                let mut sut = FPS::new(a_coeffs);
                let rhs = FPS::new(b_coeffs);
                // When
                sut += rhs;
                // Then
                assert_eq!(FPS::new(expected), sut);
            }
        }
    }
}
