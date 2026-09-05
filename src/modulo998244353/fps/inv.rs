//! 法 998244353 上の形式的べき級数の逆元 (inv) を実装するモジュールである。

use super::super::{convolution, modulo};

impl super::FPS {
    /// `x^degree` まで (含む) の逆元を計算する (疎密を自動選択する)。
    ///
    /// # Args
    /// - `degree`: 計算する最高次数 (含む)
    ///
    /// # Returns
    /// `Option<Self>`: 定数項が可逆なときの逆元系列
    ///
    /// # Constraints
    /// - 定数項は 0 であってはならない。
    ///
    /// # Panics
    /// - この関数はパニックしない。
    ///
    /// # Complexity
    /// - 時間計算量: 実行時に `inverse_dense` または `inverse_sparse` を選択する。
    /// - 空間計算量: 実行時に選択される実装に依存する。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::modulo998244353::fps;
    ///
    /// let f = fps::FPS::new(vec![2, 0, 3]);
    /// let inv = f.inverse(6).unwrap();
    /// let product = f.clone() * inv;
    /// assert_eq!(1, product.get(0));
    /// for i in 1..=6 {
    ///     assert_eq!(0, product.get(i));
    /// }
    /// ```
    pub fn inverse(&self, degree: usize) -> Option<Self> {
        // 疎な系列に対しては疎実装を選択し、それ以外は密実装を用いる。
        if self.should_use_sparse_inverse(degree) {
            self.inverse_sparse(degree)
        } else {
            self.inverse_dense(degree)
        }
    }

    /// `inverse` が疎実装を選択すべきかどうかを判定する。
    ///
    /// # Args
    /// - `degree`: 計算する最高次数 (含む)
    ///
    /// # Returns
    /// `bool`: 疎実装を選ぶなら `true`
    ///
    /// # Constraints
    /// 制約はない。
    ///
    /// # Panics
    /// - この関数はパニックしない。
    ///
    /// # Complexity
    /// - 時間計算量: O(N)。N は `self.len()` である。
    /// - 空間計算量: O(1)
    ///
    /// # Examples
    /// ```rust,ignore
    /// // `pub fn inverse` から呼び出される。
    /// ```
    fn should_use_sparse_inverse(&self, degree: usize) -> bool {
        let len = degree + 1;

        // 疎実装は O(K * len) と見積もる。ここで K は `1..=degree` の非ゼロ項数である。
        // 密実装は NTT 長 `t` に対して O(t log2 t) と見積もる。定数倍は無視し、f32 で比較する。
        let non_zero_count = self
            .non_zero_terms_iter()
            .take_while(|(i, _)| *i <= degree)
            .filter(|(i, _)| *i != 0)
            .count();
        let t = len.next_power_of_two();
        let sparse_cost = (len as f32) * (non_zero_count as f32);
        let dense_cost = (t as f32) * (t as f32).log2();

        sparse_cost < dense_cost
    }

    /// `x^degree` まで (含む) の逆元を計算する (密な実装)。
    ///
    /// # Args
    /// - `degree`: 計算する最高次数 (含む)
    ///
    /// # Returns
    /// `Option<Self>`: 定数項が可逆なときの逆元系列
    ///
    /// # Constraints
    /// - 定数項は 0 であってはならない。
    ///
    /// # Panics
    /// - この関数はパニックしない。
    ///
    /// # Complexity
    /// - 時間計算量: O(K log K)。K は `degree + 1` である。
    /// - 空間計算量: O(K)。K は結果の項数である。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::modulo998244353::fps;
    ///
    /// let f = fps::FPS::new(vec![2, 1]);
    /// let inv = f.inverse_dense(4).unwrap();
    /// let product = (f.clone() * inv).coefficients().to_vec();
    /// assert_eq!(1, product[0]);
    /// ```
    pub fn inverse_dense(&self, degree: usize) -> Option<Self> {
        // 実行環境が AVX2 をサポートする場合は、より高速な実装を選択する。
        #[cfg(target_arch = "x86_64")]
        {
            use std::arch;

            if arch::is_x86_feature_detected!("avx2") {
                return self.inverse_dense_avx2(degree);
            }
        }

        self.inverse_dense_scalar(degree)
    }

    /// AVX2 + Montgomery によって `x^degree` まで (含む) の逆元を計算する。
    ///
    /// # Args
    /// - `degree`: 計算する最高次数 (含む)
    ///
    /// # Returns
    /// `Option<Self>`: 定数項が可逆なときの逆元系列
    ///
    /// # Constraints
    /// - AVX2 が利用可能な環境でのみ呼び出す。
    /// - 定数項は 0 であってはならない。
    ///
    /// # Panics
    /// - この関数はパニックしない (debug assert のみ)。
    ///
    /// # Complexity
    /// - 時間計算量: O(K log K)。K は `degree + 1` である。
    /// - 空間計算量: O(K)。K は結果の項数である。
    ///
    /// # Examples
    /// ```rust,ignore
    /// // `pub fn inverse_dense` から呼び出される。
    /// ```
    #[cfg(target_arch = "x86_64")]
    fn inverse_dense_avx2(&self, degree: usize) -> Option<Self> {
        use super::super::convolution_mont;
        use std::arch;

        debug_assert!(arch::is_x86_feature_detected!("avx2"));

        // 逆元の計算は、Newton 法により精度を 2 倍ずつ増やしていく。
        let mut poly = self.coeffs.clone();
        let len = degree + 1;
        let constant = *poly.first().unwrap_or(&0);
        if constant == 0 {
            return None;
        }

        if len == 1 {
            return Some(Self {
                coeffs: vec![modulo::inv(constant)],
            });
        }

        let mut inverse_coeffs = vec![convolution_mont::standard_to_mont_scalar(modulo::inv(
            constant,
        ))];
        poly.resize(poly.len().next_power_of_two(), 0);
        unsafe {
            convolution_mont::standard_to_mont(&mut poly);
        }

        let mut current_len = 1;

        let mut f_vals = Vec::with_capacity(2 * len);
        let mut g_vals = Vec::with_capacity(2 * len);
        let mut h_vals = Vec::with_capacity(2 * len);

        while current_len < len {
            let next_len = 2 * current_len;

            // f を `next_len` まで取り出し、NTT により値域へ変換する。
            f_vals.clear();
            f_vals.extend(poly.iter().copied().take(next_len));
            f_vals.resize(next_len, 0);

            // g (現在の逆元近似) を `next_len` へ拡張し、NTT により値域へ変換する。
            g_vals.clear();
            g_vals.extend(inverse_coeffs.iter().copied());
            g_vals.resize(next_len, 0);

            unsafe {
                convolution_mont::ntt_mont(&mut f_vals);
                convolution_mont::ntt_mont(&mut g_vals);
            }

            let inv_ntt_len =
                convolution_mont::standard_to_mont_scalar(modulo::inv(next_len as u32));

            unsafe {
                convolution_mont::mul_pointwise_mont(&mut f_vals, &g_vals);
                convolution_mont::intt_mont(&mut f_vals);
                convolution_mont::mul_scalar_mont(&mut f_vals, inv_ntt_len);
            }

            // f*g の上半分から、Newton 更新に必要な項を抽出する。
            h_vals.clear();
            h_vals.resize(next_len, 0);
            h_vals[..current_len].copy_from_slice(&f_vals[current_len..current_len + current_len]);

            unsafe {
                convolution_mont::ntt_mont(&mut h_vals);
                convolution_mont::mul_pointwise_mont(&mut h_vals, &g_vals);
                convolution_mont::intt_mont(&mut h_vals);
                convolution_mont::mul_scalar_mont(&mut h_vals, inv_ntt_len);
            }

            let mut updated = Vec::with_capacity(next_len);
            updated.extend(inverse_coeffs.iter().copied());
            updated.resize(next_len, 0);
            for i in 0..current_len {
                updated[current_len + i] = modulo::neg(h_vals[i]);
            }

            inverse_coeffs = updated;
            current_len = next_len;
        }

        unsafe {
            convolution_mont::mont_to_standard(&mut inverse_coeffs);
        }
        inverse_coeffs.truncate(len);
        let mut res = Self {
            coeffs: inverse_coeffs,
        };
        res.trim();
        Some(res)
    }

    /// `x^degree` まで (含む) の逆元を計算する (非 SIMD 実装)。
    ///
    /// # Args
    /// - `degree`: 計算する最高次数 (含む)
    ///
    /// # Returns
    /// `Option<Self>`: 定数項が可逆なときの逆元系列
    ///
    /// # Constraints
    /// - 定数項は 0 であってはならない。
    ///
    /// # Panics
    /// - この関数はパニックしない。
    ///
    /// # Complexity
    /// - 時間計算量: O(K log K)。K は `degree + 1` である。
    /// - 空間計算量: O(K)。K は結果の項数である。
    ///
    /// # Examples
    /// ```rust,ignore
    /// // `pub fn inverse_dense` から呼び出される。
    /// ```
    fn inverse_dense_scalar(&self, degree: usize) -> Option<Self> {
        let len = degree + 1;
        let constant = self.get(0);
        if constant == 0 {
            return None;
        }

        // 次数 0 だけを求める場合、逆元は定数項の逆数である。
        if len == 1 {
            return Some(Self {
                coeffs: vec![modulo::inv(constant)],
            });
        }

        // Newton 法の初期値として、定数項の逆数を用いる。
        let mut inverse_coeffs = vec![modulo::inv(constant)];
        let mut current_len = 1;

        let mut f_vals = Vec::with_capacity(2 * len);
        let mut g_vals = Vec::with_capacity(2 * len);
        let mut h_vals = Vec::with_capacity(2 * len);

        while current_len < len {
            let next_len = (current_len << 1).min(len);
            let ntt_len = current_len << 1;

            // f を `ntt_len` まで取り出し、NTT により値域へ変換する。
            f_vals.clear();
            f_vals.extend(self.coeffs.iter().cloned().take(ntt_len));
            f_vals.resize(ntt_len, 0);

            // g (現在の逆元近似) を `ntt_len` へ拡張し、NTT により値域へ変換する。
            g_vals.clear();
            g_vals.extend(inverse_coeffs.iter().cloned());
            g_vals.resize(ntt_len, 0);

            convolution::ntt(&mut f_vals);
            convolution::ntt(&mut g_vals);

            let inv_ntt_len = modulo::inv(ntt_len as u32);

            // 値域で f*g を計算し、係数域へ戻した後に長さで正規化する。
            for (value, g_value) in f_vals.iter_mut().zip(g_vals.iter()) {
                *value = modulo::mul(*value, *g_value);
            }
            convolution::intt(&mut f_vals);
            f_vals
                .iter_mut()
                .for_each(|value| *value = modulo::mul(*value, inv_ntt_len));

            // f*g の上半分を抜き出し、g との積から Newton 更新項を求める。
            h_vals.clear();
            h_vals.resize(ntt_len, 0);
            for i in 0..current_len {
                if current_len + i < ntt_len {
                    h_vals[i] = f_vals[current_len + i];
                }
            }

            convolution::ntt(&mut h_vals);
            for (value, g_value) in h_vals.iter_mut().zip(g_vals.iter()) {
                *value = modulo::mul(*value, *g_value);
            }
            convolution::intt(&mut h_vals);
            h_vals
                .iter_mut()
                .for_each(|value| *value = modulo::mul(*value, inv_ntt_len));

            let mut updated = Vec::with_capacity(next_len);
            updated.extend(inverse_coeffs.iter().cloned().take(current_len));
            updated.resize(next_len, 0);
            for i in 0..(next_len - current_len) {
                updated[current_len + i] = modulo::neg(h_vals.get(i).copied().unwrap_or(0));
            }

            inverse_coeffs = updated;
            current_len = next_len;
        }

        inverse_coeffs.truncate(len);
        let mut res = Self {
            coeffs: inverse_coeffs,
        };
        res.trim();
        Some(res)
    }

    /// 疎な形式的べき級数の逆元を `x^degree` まで (含む) 計算する。
    ///
    /// 非ゼロ係数が少ないと仮定し、`f(x) g(x) = 1` から得られる
    /// `f_0 g_n = -\sum_{1 \le i \le n} f_i g_{n-i}` を用いて次数の小さい順に係数を求める。
    ///
    /// # Args
    /// - `degree`: 計算する最高次数 (含む)
    ///
    /// # Returns
    /// `Option<Self>`: 定数項が可逆なときの逆元系列
    ///
    /// # Constraints
    /// - 定数項は 0 であってはならない。
    ///
    /// # Panics
    /// - この関数はパニックしない。
    ///
    /// # Complexity
    /// - 時間計算量: O(K * (degree + 1))。K は非ゼロ係数の個数である。
    /// - 空間計算量: O(K + degree)
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::modulo998244353::fps;
    ///
    /// let f = fps::FPS::new(vec![2, 0, 3]);
    /// let inv = f.inverse_sparse(4).unwrap();
    /// let product = f.clone() * inv;
    /// assert_eq!(1, product.get(0));
    /// for i in 1..=4 {
    ///     assert_eq!(0, product.get(i));
    /// }
    /// ```
    pub fn inverse_sparse(&self, degree: usize) -> Option<Self> {
        let target_len = degree + 1;
        let constant = self.get(0);
        if constant == 0 {
            return None;
        }

        let inv_const = modulo::inv(constant);

        // 非ゼロ項だけを抽出し、計算対象を `degree` 以下に限定した疎な表現を用意する。
        let sparse_terms = self
            .non_zero_terms_iter()
            .skip(1)
            .take_while(|(i, _)| *i <= degree)
            .collect::<Vec<(usize, u32)>>();

        let mut res = vec![0; target_len];
        res[0] = inv_const;

        for n in 1..target_len {
            let mut acc = 0_u32;
            for &(i, c) in sparse_terms.iter().take_while(|&&(i, _)| i <= n) {
                acc = modulo::sub(acc, modulo::mul(c, res[n - i]));
            }
            res[n] = modulo::mul(acc, inv_const);
        }

        let mut res = Self { coeffs: res };
        res.trim();
        Some(res)
    }
}

#[cfg(test)]
mod tests {
    /// Background: 定数項が非ゼロの密な形式的べき級数 (3 + 4x + 5x^2)。
    fn create_dense_fps() -> super::super::FPS {
        super::super::FPS::new(vec![3, 4, 5])
    }

    /// Background: 非ゼロ項が少ない疎な形式的べき級数 (3 + 5x^2 + 7x^4)。
    fn create_sparse_fps() -> super::super::FPS {
        super::super::FPS::new(vec![3, 0, 5, 0, 7])
    }

    /// Background: 定数項が 0 の形式的べき級数。逆元が存在しない境界値である。
    fn create_fps_with_zero_constant() -> super::super::FPS {
        super::super::FPS::new(vec![0, 1, 2])
    }

    // inverse のテスト: 戻り値そのものを検証する
    mod inverse {
        use super::*;

        /// Scenario: 密な級数では、逆元との積が x^degree まで単位元 (1) に一致する
        /// - Given: 定数項が非ゼロの密な形式的べき級数がある
        /// - When: 逆元を計算し、元の級数と掛け合わせる
        /// - Then: 定数項が 1 になり、1 次から degree 次までの係数は 0 になる
        #[test]
        fn multiplies_to_identity_for_dense_series() {
            // Given
            let sut = create_dense_fps();
            let degree = 5;

            // When
            let inverse = sut.inverse(degree);
            let product = sut.clone() * inverse.unwrap();

            // Then
            assert_eq!(1, product.get(0));
            for i in 1..=degree {
                assert_eq!(0, product.get(i));
            }
        }

        /// Scenario: 疎な級数では、逆元との積が x^degree まで単位元 (1) に一致する
        /// - Given: 非ゼロ項が少ない疎な形式的べき級数がある
        /// - When: 逆元を計算し、元の級数と掛け合わせる
        /// - Then: 定数項が 1 になり、1 次から degree 次までの係数は 0 になる
        #[test]
        fn multiplies_to_identity_for_sparse_series() {
            // Given
            let sut = create_sparse_fps();
            let degree = 6;

            // When
            let inverse = sut.inverse(degree);
            let product = sut.clone() * inverse.unwrap();

            // Then
            assert_eq!(1, product.get(0));
            for i in 1..=degree {
                assert_eq!(0, product.get(i));
            }
        }

        /// Scenario: 定数項が 0 の級数には逆元が存在しない
        /// - Given: 定数項が 0 の形式的べき級数がある
        /// - When: 逆元を計算する
        /// - Then: None が返る
        #[test]
        fn returns_none_when_constant_term_is_zero() {
            // Given
            let sut = create_fps_with_zero_constant();

            // When
            let result = sut.inverse(4);

            // Then
            assert!(result.is_none());
        }

        /// Scenario: degree が 0 のとき、逆元は定数項の逆数のみからなる 1 項になる
        /// - Given: 定数項が非ゼロの形式的べき級数がある
        /// - When: degree = 0 で逆元を計算する
        /// - Then: 元の級数との積の定数項が 1 になる
        #[test]
        fn returns_reciprocal_constant_when_degree_is_zero() {
            // Given
            let sut = create_dense_fps();

            // When
            let inverse = sut.inverse(0);

            // Then
            let product = sut.clone() * inverse.unwrap();
            assert_eq!(1, product.get(0));
        }
    }

    // inverse_dense のテスト: 戻り値そのものを検証する
    mod inverse_dense {
        use super::*;

        /// Scenario: 密実装でも、逆元との積が x^degree まで単位元 (1) に一致する
        /// - Given: 定数項が非ゼロの密な形式的べき級数がある
        /// - When: 密実装で逆元を計算し、元の級数と掛け合わせる
        /// - Then: 定数項が 1 になり、1 次から degree 次までの係数は 0 になる
        #[test]
        fn multiplies_to_identity_for_dense_series() {
            // Given
            let sut = create_dense_fps();
            let degree = 5;

            // When
            let inverse = sut.inverse_dense(degree);
            let product = sut.clone() * inverse.unwrap();

            // Then
            assert_eq!(1, product.get(0));
            for i in 1..=degree {
                assert_eq!(0, product.get(i));
            }
        }

        /// Scenario: 定数項が 0 の級数には逆元が存在しない
        /// - Given: 定数項が 0 の形式的べき級数がある
        /// - When: 密実装で逆元を計算する
        /// - Then: None が返る
        #[test]
        fn returns_none_when_constant_term_is_zero() {
            // Given
            let sut = create_fps_with_zero_constant();

            // When
            let result = sut.inverse_dense(4);

            // Then
            assert!(result.is_none());
        }
    }

    // inverse_sparse のテスト: 戻り値そのものを検証する
    mod inverse_sparse {
        use super::*;

        /// Scenario: 疎実装でも、逆元との積が x^degree まで単位元 (1) に一致する
        /// - Given: 非ゼロ項が少ない疎な形式的べき級数がある
        /// - When: 疎実装で逆元を計算し、元の級数と掛け合わせる
        /// - Then: 定数項が 1 になり、1 次から degree 次までの係数は 0 になる
        #[test]
        fn multiplies_to_identity_for_sparse_series() {
            // Given
            let sut = create_sparse_fps();
            let degree = 6;

            // When
            let inverse = sut.inverse_sparse(degree);
            let product = sut.clone() * inverse.unwrap();

            // Then
            assert_eq!(1, product.get(0));
            for i in 1..=degree {
                assert_eq!(0, product.get(i));
            }
        }

        /// Scenario: 定数項が 0 の級数には逆元が存在しない
        /// - Given: 定数項が 0 の形式的べき級数がある
        /// - When: 疎実装で逆元を計算する
        /// - Then: None が返る
        #[test]
        fn returns_none_when_constant_term_is_zero() {
            // Given
            let sut = create_fps_with_zero_constant();

            // When
            let result = sut.inverse_sparse(4);

            // Then
            assert!(result.is_none());
        }

        /// Scenario: 疎実装の結果は、自動選択された実装の結果と一致する
        /// - Given: 非ゼロ項が少ない疎な形式的べき級数がある
        /// - When: 疎実装と自動選択でそれぞれ逆元を計算する
        /// - Then: 両者の結果は等しい
        #[test]
        fn matches_auto_selected_inverse_for_sparse_series() {
            // Given
            let sut = create_sparse_fps();
            let degree = 6;

            // When
            let sparse = sut.inverse_sparse(degree).unwrap();
            let auto = sut.inverse(degree).unwrap();

            // Then
            assert_eq!(auto, sparse);
        }
    }
}
