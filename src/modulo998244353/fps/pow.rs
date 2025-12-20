//! 法 998244353 上の形式的べき級数のべき乗 (pow) を実装するモジュールである.

use super::super::modulo;

impl super::FPS {
    /// 形式的べき級数を非負整数乗し, `x^degree` まで (含む) を切り詰めて返す
    /// (疎密を自動選択する).
    ///
    /// # Args
    /// - `exponent`: 非負整数冪指数.
    /// - `degree`: 計算する最高次数 (含む).
    ///
    /// # Returns
    /// `Self`: `self(x)^exponent` を `degree` 次まで切り詰めた形式的べき級数.
    ///
    /// # Constraints
    /// - `exponent > 0` のとき `degree + 1 < 998244353`.
    ///
    /// # Panics
    /// - `exponent > 0` かつ `degree + 1 >= 998244353` のとき.
    ///
    /// # Complexity
    /// - Time complexity: 実行時に疎実装または密実装を選択する.
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::modulo998244353::fps;
    ///
    /// let f = fps::FPS::new(vec![0, 1, 1]);
    /// let result = f.pow(3, 4);
    /// assert_eq!(1, result.get(3));
    /// ```
    pub fn pow(&self, exponent: usize, degree: usize) -> Self {
        let target_len = degree + 1;
        if exponent == 0 {
            return Self { coeffs: vec![1] };
        }
        assert!(
            target_len < modulo::M as usize,
            "pow requires degree + 1 < modulus"
        );

        if self.should_use_sparse_pow(exponent, degree) {
            self.pow_sparse(exponent, degree)
        } else {
            self.pow_dense(exponent, degree)
        }
    }

    /// `pow` が疎実装を選択すべきかどうかを判定する.
    ///
    /// # Args
    /// - `exponent`: 非負整数冪指数.
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
    /// // `pub fn pow` から呼び出される.
    /// ```
    fn should_use_sparse_pow(&self, exponent: usize, degree: usize) -> bool {
        let target_len = degree + 1;
        if exponent == 0 || target_len <= 1 {
            return true;
        }

        let mut iter = self.non_zero_terms_iter().take_while(|(i, _)| *i <= degree);

        let Some((min_non_zero_degree, _)) = iter.next() else {
            return true;
        };

        let total_degree_shift = exponent.saturating_mul(min_non_zero_degree);
        if total_degree_shift >= target_len {
            return true;
        }

        // `pow_sparse` の主要コストは, 正規化後の長さ `L` と非ゼロ項数 `K` に対して O(L * K)
        // とみなせるため, それを見積もって密実装のコストと比較する.
        let normalized_len = target_len - total_degree_shift;
        let max_degree = min_non_zero_degree + normalized_len - 1;
        let non_zero_count = iter
            .filter(|(i, _)| *i <= max_degree)
            .filter(|(i, _)| *i != min_non_zero_degree)
            .count();

        let sparse_cost = (normalized_len as f32) * (non_zero_count as f32);

        // 密実装は, 正規化後の長さに対して NTT 長 `t` を用いた O(t log2 t) と見積もる.
        // 定数倍は無視し, f32 で比較する.
        let t = (2 * normalized_len).next_power_of_two();
        let dense_cost = (t as f32) * (t as f32).log2();

        sparse_cost < dense_cost
    }

    /// 形式的べき級数を非負整数乗し, `x^degree` まで (含む) を切り詰めて返す
    /// (密な実装).
    ///
    /// # Args
    /// - `exponent`: 非負整数冪指数.
    /// - `degree`: 計算する最高次数 (含む).
    ///
    /// # Returns
    /// `Self`: `self(x)^exponent` を `degree` 次まで切り詰めた形式的べき級数.
    ///
    /// # Constraints
    /// - `exponent > 0` のとき `degree + 1 < 998244353`.
    ///
    /// # Panics
    /// - `exponent > 0` かつ `degree + 1 >= 998244353` のとき.
    ///
    /// # Complexity
    /// - Time complexity: O(N log N). ここで N は `degree + 1`.
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::modulo998244353::fps;
    ///
    /// let f = fps::FPS::new(vec![0, 1, 1]);
    /// let result = f.pow_dense(3, 4);
    /// assert_eq!(1, result.get(3));
    /// ```
    pub fn pow_dense(&self, exponent: usize, degree: usize) -> Self {
        let target_len = degree + 1;
        if exponent == 0 {
            return Self { coeffs: vec![1] };
        }
        assert!(
            target_len < modulo::M as usize,
            "pow_dense requires degree + 1 < modulus"
        );

        if self.is_zero() {
            return Self { coeffs: Vec::new() };
        }

        let (k, m) = self
            .non_zero_terms_iter()
            .next()
            .expect("pow_dense requires a non-zero series");

        let shift = exponent.saturating_mul(k);
        if shift >= target_len {
            return Self { coeffs: Vec::new() };
        }

        let inner_len = target_len - shift;
        let m_inv = modulo::inv(m);

        // f(x) = x^k * m * g(x) (g(0) = 1) となるように正規化する.
        let mut g_coeffs = Vec::with_capacity(inner_len);
        for i in 0..inner_len {
            let coeff = self.get(k + i);
            g_coeffs.push(modulo::mul(coeff, m_inv));
        }
        let mut g = Self { coeffs: g_coeffs };
        g.trim();

        // f(x)^n = x^{kn} * m^n * exp(n * log(g(x))) を用いる.
        let exponent_mod = modulo::modulo(exponent as u64);
        let mut log_g = g
            .log_dense(inner_len - 1)
            .expect("pow_dense: log_dense should succeed");
        for c in log_g.coeffs.iter_mut() {
            *c = modulo::mul(*c, exponent_mod);
        }

        let mut res = log_g
            .exp_dense(inner_len - 1)
            .expect("pow_dense: exp_dense should succeed");
        let m_pow = modulo::pow(m, exponent);
        for c in res.coeffs.iter_mut() {
            *c = modulo::mul(*c, m_pow);
        }

        let mut coeffs = vec![0_u32; target_len];
        for i in 0..inner_len {
            coeffs[shift + i] = res.get(i);
        }
        let mut out = Self { coeffs };
        out.trim();
        out
    }

    /// 疎な形式的べき級数を非負整数乗し, `x^degree` まで (含む) を切り詰めて返す.
    ///
    /// 非ゼロ係数が少ないことを利用し, `F(x) = f(x)^M` が満たす
    /// `f(x) F'(x) = M F(x) f'(x)` に基づく漸化式で次数の低い順に計算する.
    ///
    /// # Args
    /// - `exponent`: 非負整数冪指数 `M`.
    /// - `degree`: 計算する最高次数 (含む).
    ///
    /// # Returns
    /// `Self`: `self(x)^exponent` を `degree` 次まで切り詰めた形式的べき級数.
    ///
    /// # Constraints
    /// - `exponent > 0` のとき `degree + 1 < 998244353`.
    /// - `degree` より高い非ゼロ係数は結果に影響しないものとして扱う.
    ///
    /// # Panics
    /// - `exponent > 0` かつ `degree + 1 >= 998244353` のとき.
    ///
    /// # Complexity
    /// - Time complexity: O((N - M t) * K). ここで
    ///   N = degree + 1, t は最小の非ゼロ係数の添字, K は非ゼロ係数の個数である.
    /// - Space complexity: O(N + K).
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::modulo998244353::fps;
    ///
    /// // f(x) = x + x^2, M = 3
    /// let f = fps::FPS::new(vec![0, 1, 1]);
    /// let result = f.pow_sparse(3, 4);
    ///
    /// // (x + x^2)^3 = x^3 + 3x^4 + 3x^5 + x^6
    /// assert_eq!(0, result.get(0));
    /// assert_eq!(0, result.get(1));
    /// assert_eq!(0, result.get(2));
    /// assert_eq!(1, result.get(3));
    /// assert_eq!(3, result.get(4));
    /// ```
    pub fn pow_sparse(&self, exponent: usize, degree: usize) -> Self {
        let target_len = degree + 1;
        // 0 乗は常に 1 になるため, ここで確定させる.
        if exponent == 0 {
            return Self { coeffs: vec![1] };
        }
        assert!(
            target_len < modulo::M as usize,
            "pow_sparse requires degree + 1 < modulus"
        );

        // 対象範囲に非ゼロ項が無ければ, 結果はゼロ多項式になる.
        if self.len() == 0 {
            return Self { coeffs: Vec::new() };
        }

        // 非ゼロ項だけを疎な表現で保持し, 以降の計算を簡略化する.
        let mut sparse_terms = Vec::new();
        for (i, &c) in self.coeffs.iter().enumerate() {
            if c != 0 {
                assert!(c < modulo::M);
                sparse_terms.push((i, c));
            }
        }

        // 最初の非ゼロ項の次数を t とする.
        let min_non_zero_degree = sparse_terms[0].0;
        let first_non_zero_coeff = sparse_terms[0].1;

        // M * t >= N なら要求次数までの係数は全て 0 になる.
        let total_degree_shift = exponent.saturating_mul(min_non_zero_degree);

        // L = N - M t は x^t を外に出した後に必要な項数である.
        if total_degree_shift >= target_len {
            return Self::new(vec![]);
        }
        let normalized_len = target_len - total_degree_shift;

        // c^M を別に保持し, 定数項を 1 にした系列の結果へ掛け戻す.
        let scaled_leading_power = modulo::pow(first_non_zero_coeff, exponent);
        let inv_first_coeff = modulo::inv(first_non_zero_coeff);

        // f(x) = c * g(x), g(0) = 1 となるように正規化し, 必要な次数だけ残す.
        let mut normalized_sparse_terms = Vec::new();
        for (idx, coef) in sparse_terms.into_iter() {
            if idx < min_non_zero_degree {
                continue;
            }
            let shifted_degree = idx - min_non_zero_degree;
            if shifted_degree >= normalized_len {
                break;
            }
            let scaled = modulo::mul(coef, inv_first_coeff);
            normalized_sparse_terms.push((shifted_degree, scaled));
        }

        // 0 次項 (1) を除き, 非ゼロ項だけを抽出する.
        let mut base_series_terms = Vec::new();
        for (degree, coeff) in normalized_sparse_terms.into_iter() {
            if degree == 0 {
                continue;
            }
            base_series_terms.push((degree, coeff));
        }

        // F[n] は g(x)^M の x^n 係数で, F[0] = 1 とする.
        let exponent_mod = modulo::modulo(exponent as u64);

        // 次の漸化式を用いて次数の低い順に求める.
        //
        // $$
        // F_{n+1}
        //
        //
        // \frac{1}{n+1}
        // \sum_{i=0}^{n}
        // \bigl(M(i+1) - (n-i)\bigr), f_{i+1} F_{n-i}
        // $$
        let offset = target_len - normalized_len;
        let inv_indices = modulo::build_inv_indices(normalized_len);
        let mut ans = vec![0_u32; target_len];
        ans[offset] = 1;
        for k in 0..(normalized_len - 1) {
            let mut next = 0_u32;
            for &(i, f_i) in &base_series_terms {
                if i <= k + 1 {
                    let rhs = (k + 1 - i) as u32;
                    let ci = modulo::sub(modulo::mul(exponent_mod, i as u32), rhs);
                    let cont = modulo::mul(f_i, ans[offset + k + 1 - i]);
                    next = modulo::add(next, modulo::mul(ci, cont));
                } else {
                    break;
                }
            }
            next = modulo::mul(next, inv_indices[k + 1]);
            ans[offset + k + 1] = next;
        }

        // 元の正規化に戻すため, c^M を掛け戻す.
        for idx in offset..target_len {
            ans[idx] = modulo::mul(scaled_leading_power, ans[idx]);
        }
        let mut res = Self { coeffs: ans };
        res.trim();
        res
    }
}
