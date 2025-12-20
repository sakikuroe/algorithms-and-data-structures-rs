//! 法 998244353 上の形式的べき級数の逆元 (inv) を実装するモジュールである.

use super::super::convolution;
use super::super::modulo;

impl super::FPS {
    /// `x^degree` まで (含む) の逆元を計算する (疎密を自動選択する).
    ///
    /// # Args
    /// - `degree`: 計算する最高次数 (含む).
    ///
    /// # Returns
    /// `Option<Self>`: 定数項が可逆なときの逆元系列.
    ///
    /// # Constraints
    /// - 定数項は 0 であってはならない.
    ///
    /// # Panics
    /// - この関数はパニックしない.
    ///
    /// # Complexity
    /// - Time complexity: 実行時に `inverse_dense` または `inverse_sparse` を選択する.
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
        if self.should_use_sparse_inverse(degree) {
            self.inverse_sparse(degree)
        } else {
            self.inverse_dense(degree)
        }
    }

    /// `inverse` が疎実装を選択すべきかどうかを判定する.
    ///
    /// # Args
    /// - `degree`: 計算する最高次数 (含む).
    ///
    /// # Returns
    /// `bool`: 疎実装を選ぶなら `true`.
    ///
    /// # Constraints
    /// 制約はない.
    ///
    /// # Panics
    /// - この関数はパニックしない.
    ///
    /// # Complexity
    /// - Time complexity: O(N). ここで N は `self.len()`.
    /// - Space complexity: O(1).
    ///
    /// # Examples
    /// ```rust,ignore
    /// // `pub fn inverse` から呼び出される.
    /// ```
    fn should_use_sparse_inverse(&self, degree: usize) -> bool {
        let len = degree + 1;

        // 疎実装は O(K * len) と見積もる. ここで K は `1..=degree` の非ゼロ項数である.
        // 密実装は NTT 長 `t` に対して O(t log2 t) と見積もる. 定数倍は無視し, f32 で比較する.
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

    /// `x^degree` まで (含む) の逆元を計算する (密な実装).
    ///
    /// # Args
    /// - `degree`: 計算する最高次数 (含む).
    ///
    /// # Returns
    /// `Option<Self>`: 定数項が可逆なときの逆元系列.
    ///
    /// # Constraints
    /// - 定数項は 0 であってはならない.
    ///
    /// # Panics
    /// - この関数はパニックしない.
    ///
    /// # Complexity
    /// - Time complexity: O(K log K). ここで K は `degree + 1`.
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
        #[cfg(target_arch = "x86_64")]
        {
            if std::is_x86_feature_detected!("avx2") {
                return self.inverse_dense_avx2(degree);
            }
        }

        self.inverse_dense_scalar(degree)
    }

    /// AVX2 + Montgomery によって `x^degree` まで (含む) の逆元を計算する.
    ///
    /// # Args
    /// - `degree`: 計算する最高次数 (含む).
    ///
    /// # Returns
    /// `Option<Self>`: 定数項が可逆なときの逆元系列.
    ///
    /// # Constraints
    /// - AVX2 が利用可能な環境でのみ呼び出す.
    /// - 定数項は 0 であってはならない.
    ///
    /// # Panics
    /// - この関数はパニックしない (debug assert のみ).
    ///
    /// # Complexity
    /// - Time complexity: O(K log K). ここで K は `degree + 1`.
    ///
    /// # Examples
    /// ```rust,ignore
    /// // `pub fn inverse_dense` から呼び出される.
    /// ```
    #[cfg(target_arch = "x86_64")]
    fn inverse_dense_avx2(&self, degree: usize) -> Option<Self> {
        use super::super::convolution_mont;

        debug_assert!(std::is_x86_feature_detected!("avx2"));

        let mut poly = self.coeffs.clone();
        let len = degree + 1;
        let constant = *poly.get(0).unwrap_or(&0);
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

            f_vals.clear();
            f_vals.extend(poly.iter().cloned().take(next_len));
            f_vals.resize(next_len, 0);

            g_vals.clear();
            g_vals.extend(inverse_coeffs.iter().cloned());
            g_vals.resize(next_len, 0);

            unsafe {
                convolution_mont::ntt_mont(&mut f_vals);
                convolution_mont::ntt_mont(&mut g_vals);
            }

            let inv_ntt_len =
                convolution_mont::standard_to_mont_scalar(modulo::inv(next_len as u32));

            unsafe {
                convolution_mont::mul_pointwise_mont(&mut f_vals, &mut g_vals);
                convolution_mont::intt_mont(&mut f_vals);
                convolution_mont::mul_scalar_mont(&mut f_vals, inv_ntt_len);
            }

            h_vals.clear();
            h_vals.resize(next_len, 0);
            for i in 0..current_len {
                h_vals[i] = f_vals[current_len + i];
            }

            unsafe {
                convolution_mont::ntt_mont(&mut h_vals);
                convolution_mont::mul_pointwise_mont(&mut h_vals, &mut g_vals);
                convolution_mont::intt_mont(&mut h_vals);
                convolution_mont::mul_scalar_mont(&mut h_vals, inv_ntt_len);
            }

            let mut updated = Vec::with_capacity(next_len);
            updated.extend(inverse_coeffs.iter().cloned());
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
        while inverse_coeffs.last().map_or(false, |c| *c == 0) {
            inverse_coeffs.pop();
        }
        Some(Self {
            coeffs: inverse_coeffs,
        })
    }

    /// `x^degree` まで (含む) の逆元を計算する (非 SIMD 実装).
    ///
    /// # Args
    /// - `degree`: 計算する最高次数 (含む).
    ///
    /// # Returns
    /// `Option<Self>`: 定数項が可逆なときの逆元系列.
    ///
    /// # Constraints
    /// - 定数項は 0 であってはならない.
    ///
    /// # Panics
    /// - この関数はパニックしない.
    ///
    /// # Complexity
    /// - Time complexity: O(K log K). ここで K は `degree + 1`.
    ///
    /// # Examples
    /// ```rust,ignore
    /// // `pub fn inverse_dense` から呼び出される.
    /// ```
    fn inverse_dense_scalar(&self, degree: usize) -> Option<Self> {
        let len = degree + 1;
        let constant = self.get(0);
        if constant == 0 {
            return None;
        }

        if len == 1 {
            return Some(Self {
                coeffs: vec![modulo::inv(constant)],
            });
        }

        let mut inverse_coeffs = vec![modulo::inv(constant)];
        let mut current_len = 1;

        let mut f_vals = Vec::with_capacity(2 * len);
        let mut g_vals = Vec::with_capacity(2 * len);
        let mut h_vals = Vec::with_capacity(2 * len);

        while current_len < len {
            let next_len = (current_len << 1).min(len);
            let ntt_len = current_len << 1;

            f_vals.clear();
            f_vals.extend(self.coeffs.iter().cloned().take(ntt_len));
            f_vals.resize(ntt_len, 0);

            g_vals.clear();
            g_vals.extend(inverse_coeffs.iter().cloned());
            g_vals.resize(ntt_len, 0);

            convolution::ntt(&mut f_vals);
            convolution::ntt(&mut g_vals);

            let inv_ntt_len = modulo::inv(ntt_len as u32);

            for (value, g_value) in f_vals.iter_mut().zip(g_vals.iter()) {
                *value = modulo::mul(*value, *g_value);
            }
            convolution::intt(&mut f_vals);
            f_vals
                .iter_mut()
                .for_each(|value| *value = modulo::mul(*value, inv_ntt_len));

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

    /// 疎な形式的べき級数の逆元を `x^degree` まで (含む) 計算する.
    ///
    /// 非ゼロ係数が少ないと仮定し, `f(x) g(x) = 1` から得られる
    /// `f_0 g_n = -\sum_{1 \le i \le n} f_i g_{n-i}` を用いて次数の小さい順に係数を求める.
    ///
    /// # Args
    /// - `degree`: 計算する最高次数 (含む).
    ///
    /// # Returns
    /// `Option<Self>`: 定数項が可逆なときの逆元系列.
    ///
    /// # Constraints
    /// - 定数項は 0 であってはならない.
    ///
    /// # Panics
    /// - この関数はパニックしない.
    ///
    /// # Complexity
    /// - Time complexity: O(K * (degree + 1)). ここで K は非ゼロ係数の個数.
    /// - Space complexity: O(K + degree).
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

        // 非ゼロ項だけを抽出し, 計算対象を `degree` 以下に限定した疎な表現を用意する.
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
