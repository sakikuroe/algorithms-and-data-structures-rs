//! 法 998244353 上の形式的べき級数の対数 (log) を実装するモジュールである.

use super::super::modulo;

impl super::FPS {
    /// `x^degree` まで (含む) の形式的対数を計算する (疎密を自動選択する).
    ///
    /// # Args
    /// - `degree`: 計算する最高次数 (含む).
    ///
    /// # Returns
    /// `Option<Self>`: 制約を満たすときの切り詰められた対数系列.
    ///
    /// # Constraints
    /// - 定数項は 1 でなければならない.
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
    /// let fps = fps::FPS::new(vec![1, 1]);
    /// let log = fps.log(3).unwrap();
    /// assert_eq!(1, log.get(1));
    /// ```
    pub fn log(&self, degree: usize) -> Option<Self> {
        if self.should_use_sparse_log(degree) {
            self.log_sparse(degree)
        } else {
            self.log_dense(degree)
        }
    }

    /// `log` が疎実装を選択すべきかどうかを判定する.
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
    /// // `pub fn log` から呼び出される.
    /// ```
    fn should_use_sparse_log(&self, degree: usize) -> bool {
        let len = degree + 1;
        if len <= 1 {
            return false;
        }

        // 疎実装は O(K * len) と見積もる. ここで K は `1..=degree` の非ゼロ項数である.
        // 密実装は NTT 長 `t` に対して O(t log2 t) と見積もる. 定数倍は無視し, f32 で比較する.
        //
        // 密実装は `inverse_dense` と畳み込みをそれぞれ 1 回ずつ使うため, コストは 2 倍と見積もる.
        let non_zero_count = self
            .non_zero_terms_iter()
            .take_while(|(i, _)| *i <= degree)
            .filter(|(i, _)| *i != 0)
            .count();
        let t = (2 * len).next_power_of_two();
        let sparse_cost = (len as f32) * (non_zero_count as f32);
        let dense_cost = 2.0 * (t as f32) * (t as f32).log2();

        sparse_cost < dense_cost
    }

    /// `x^degree` まで (含む) の形式的対数を計算する (密な実装).
    ///
    /// # Args
    /// - `degree`: 計算する最高次数 (含む).
    ///
    /// # Returns
    /// `Option<Self>`: 制約を満たすときの切り詰められた対数系列.
    ///
    /// # Constraints
    /// - 定数項は 1 でなければならない.
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
    /// let fps = fps::FPS::new(vec![1, 1]);
    /// let log = fps.log_dense(3).unwrap();
    /// assert_eq!(1, log.get(1));
    /// ```
    pub fn log_dense(&self, degree: usize) -> Option<Self> {
        if self.get(0) != 1 {
            return None;
        }
        let target_len = degree + 1;
        if target_len == 1 {
            return Some(Self { coeffs: vec![0] });
        }
        let mut f = Self {
            coeffs: self.coeffs.iter().cloned().take(target_len).collect(),
        };
        let inv = f.inverse_dense(degree - 1)?;
        f.derivative();
        let mut res = f * inv;
        res.truncate(degree);
        res.integral();
        res.truncate(target_len);
        Some(res)
    }

    /// 疎な形式的べき級数の形式的対数を `x^degree` まで (含む) 計算する.
    ///
    /// `(log f)' = f'/f` を用い, `inverse_sparse` と同様の疎な漸化式で商を求めてから積分する.
    ///
    /// # Args
    /// - `degree`: 計算する最高次数 (含む).
    ///
    /// # Returns
    /// `Option<Self>`: 制約を満たすときの切り詰められた対数系列.
    ///
    /// # Constraints
    /// - 定数項は 1 でなければならない.
    ///
    /// # Panics
    /// - この関数はパニックしない.
    ///
    /// # Complexity
    /// - Time complexity: O(K * degree). ここで K は非ゼロ係数の個数.
    /// - Space complexity: O(K + degree).
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::modulo998244353::fps;
    ///
    /// let fps = fps::FPS::new(vec![1, 1, 0, 1]);
    /// let log_sparse = fps.log_sparse(4).unwrap();
    /// let log_dense = fps.log(4).unwrap();
    /// assert_eq!(log_dense, log_sparse);
    /// ```
    pub fn log_sparse(&self, degree: usize) -> Option<Self> {
        if self.get(0) != 1 {
            return None;
        }
        let target_len = degree + 1;
        if target_len == 1 {
            return Some(Self { coeffs: vec![0] });
        }

        // 非ゼロ項を疎な表現にまとめ, 計算対象を `degree` 以下に制限する.
        let mut sparse_terms = Vec::new();
        for (i, &c) in self.coeffs.iter().enumerate().skip(1) {
            if i > degree {
                break;
            }
            if c != 0 {
                sparse_terms.push((i, c));
            }
        }

        let deriv_len = target_len - 1;
        let mut quotient = vec![0_u32; deriv_len];

        // f(x) * h(x) = f'(x) を満たす h(x) = (log f)' を次数の昇順で解く.
        // f_0 = 1 で固定し, 低い次数から順に求める.
        for n in 0..deriv_len {
            let mut acc = 0_u32;
            for &(i, c) in &sparse_terms {
                if i > n {
                    break;
                }
                acc = modulo::add(acc, modulo::mul(c, quotient[n - i]));
            }
            let deriv_n = modulo::mul(n as u32 + 1, self.get(n + 1));
            quotient[n] = modulo::sub(deriv_n, acc);
        }

        let mut res = Self { coeffs: quotient };
        res.integral();
        res.truncate(target_len);
        Some(res)
    }
}
