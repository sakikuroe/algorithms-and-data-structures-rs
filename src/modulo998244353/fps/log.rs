//! 法 998244353 上の形式的べき級数の対数 (log) を実装するモジュールである。

use super::super::modulo;

impl super::FPS {
    /// `x^degree` まで (含む) の形式的対数を計算する (疎密を自動選択する)。
    ///
    /// # Args
    /// - `degree`: 計算する最高次数 (含む)
    ///
    /// # Returns
    /// `Option<Self>`: 制約を満たすときの切り詰められた対数系列
    ///
    /// # Constraints
    /// - 定数項は 1 でなければならない。
    ///
    /// # Panics
    /// - この関数はパニックしない。
    ///
    /// # Complexity
    /// - 時間計算量: 実行時に `log_dense` または `log_sparse` を選択する。
    /// - 空間計算量: 実行時に選択される実装に依存する。
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
        // 疎な系列に対しては疎実装を選択し、それ以外は密実装を用いる。
        if self.should_use_sparse_log(degree) {
            self.log_sparse(degree)
        } else {
            self.log_dense(degree)
        }
    }

    /// `log` が疎実装を選択すべきかどうかを判定する。
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
    /// // `pub fn log` から呼び出される。
    /// ```
    fn should_use_sparse_log(&self, degree: usize) -> bool {
        let len = degree + 1;
        if len <= 1 {
            return false;
        }

        // 疎実装は O(K * len) と見積もる。ここで K は `1..=degree` の非ゼロ項数である。
        // 密実装は NTT 長 `t` に対して O(t log2 t) と見積もる。定数倍は無視し、f32 で比較する。
        //
        // 密実装は `inverse_dense` と畳み込みをそれぞれ 1 回ずつ使うため、コストは 2 倍と見積もる。
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

    /// `x^degree` まで (含む) の形式的対数を計算する (密な実装)。
    ///
    /// # Args
    /// - `degree`: 計算する最高次数 (含む)
    ///
    /// # Returns
    /// `Option<Self>`: 制約を満たすときの切り詰められた対数系列
    ///
    /// # Constraints
    /// - 定数項は 1 でなければならない。
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
    /// let fps = fps::FPS::new(vec![1, 1]);
    /// let log = fps.log_dense(3).unwrap();
    /// assert_eq!(1, log.get(1));
    /// ```
    pub fn log_dense(&self, degree: usize) -> Option<Self> {
        // 定数項が 1 でなければ、形式的対数は定義されない。
        if self.get(0) != 1 {
            return None;
        }

        // 以降は `x^degree` までを扱うため、項数を `degree + 1` にそろえる。
        let target_len = degree + 1;
        if target_len == 1 {
            return Some(Self { coeffs: vec![0] });
        }

        // f を `x^degree` まで切り詰めた上で、(log f)' = f'/f を計算する。
        let mut f = Self {
            coeffs: self.coeffs.iter().cloned().take(target_len).collect(),
        };
        let inv = f.inverse_dense(degree - 1)?;
        f.derivative();

        // f'/f を求め、積分して定数項 0 の対数を得る。
        let mut res = f * inv;
        res.truncate(degree);
        res.integral();
        res.truncate(target_len);
        Some(res)
    }

    /// 疎な形式的べき級数の形式的対数を `x^degree` まで (含む) 計算する。
    ///
    /// `(log f)' = f'/f` を用い、`inverse_sparse` と同様の疎な漸化式で商を求めてから積分する。
    ///
    /// # Args
    /// - `degree`: 計算する最高次数 (含む)
    ///
    /// # Returns
    /// `Option<Self>`: 制約を満たすときの切り詰められた対数系列
    ///
    /// # Constraints
    /// - 定数項は 1 でなければならない。
    ///
    /// # Panics
    /// - この関数はパニックしない。
    ///
    /// # Complexity
    /// - 時間計算量: O(K * degree)。K は非ゼロ係数の個数である。
    /// - 空間計算量: O(K + degree)
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

        // 非ゼロ項を疎な表現にまとめ、計算対象を `degree` 以下に制限する。
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

        // f(x) * h(x) = f'(x) を満たす h(x) = (log f)' を次数の昇順で解く。
        // f_0 = 1 で固定し、低い次数から順に求める。
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

#[cfg(test)]
mod tests {
    /// Background: 定数項が 1 で非ゼロ項が少ない疎な形式的べき級数。
    fn create_fps_with_constant_one() -> super::super::FPS {
        super::super::FPS::new(vec![1, 2, 0, 3, 0, 0, 4])
    }

    /// Background: 1 + x を表す、定数項が 1 の形式的べき級数。
    fn create_one_plus_x_fps() -> super::super::FPS {
        super::super::FPS::new(vec![1, 1])
    }

    /// Background: 定数項が 1 ではない形式的べき級数。log が定義できない境界値である。
    fn create_fps_with_non_one_constant() -> super::super::FPS {
        super::super::FPS::new(vec![2, 1])
    }

    // log のテスト: 戻り値そのものを検証する
    mod log {
        use super::*;

        /// Scenario: 1 + x の形式的対数の 1 次係数は 1 である
        /// - Given: 定数項が 1 で 1 次係数が 1 の形式的べき級数 (1 + x) がある
        /// - When: log を計算する
        /// - Then: 定数項が 0、1 次係数が 1 になる
        #[test]
        fn one_plus_x_has_first_order_coefficient_one() {
            // Given
            let sut = create_one_plus_x_fps();
            let degree = 6;

            // When
            let log = sut.log(degree);

            // Then
            let log = log.unwrap();
            assert_eq!(0, log.get(0));
            assert_eq!(1, log.get(1));
        }

        /// Scenario: 定数項が 1 でない級数には形式的対数が定義できない
        /// - Given: 定数項が 1 ではない形式的べき級数がある
        /// - When: log を計算する
        /// - Then: None が返る
        #[test]
        fn returns_none_when_constant_term_is_not_one() {
            // Given
            let sut = create_fps_with_non_one_constant();

            // When
            let result = sut.log(4);

            // Then
            assert!(result.is_none());
        }

        /// Scenario: degree が 0 のとき、log の結果は定数項 0 のみになる
        /// - Given: 定数項が 1 の形式的べき級数がある
        /// - When: degree = 0 で log を計算する
        /// - Then: 結果の定数項が 0 になる
        #[test]
        fn returns_constant_zero_when_degree_is_zero() {
            // Given
            let sut = create_fps_with_constant_one();

            // When
            let result = sut.log(0);

            // Then
            assert_eq!(0, result.unwrap().get(0));
        }
    }

    // log_dense のテスト: 戻り値そのものを検証する
    mod log_dense {
        use super::*;

        /// Scenario: 1 + x の形式的対数の 1 次係数は 1 である
        /// - Given: 定数項が 1 で 1 次係数が 1 の形式的べき級数 (1 + x) がある
        /// - When: log_dense を計算する
        /// - Then: 定数項が 0、1 次係数が 1 になる
        #[test]
        fn one_plus_x_has_first_order_coefficient_one() {
            // Given
            let sut = create_one_plus_x_fps();
            let degree = 3;

            // When
            let log = sut.log_dense(degree);

            // Then
            let log = log.unwrap();
            assert_eq!(0, log.get(0));
            assert_eq!(1, log.get(1));
        }

        /// Scenario: 定数項が 1 でない級数には形式的対数が定義できない
        /// - Given: 定数項が 1 ではない形式的べき級数がある
        /// - When: log_dense を計算する
        /// - Then: None が返る
        #[test]
        fn returns_none_when_constant_term_is_not_one() {
            // Given
            let sut = create_fps_with_non_one_constant();

            // When
            let result = sut.log_dense(4);

            // Then
            assert!(result.is_none());
        }
    }

    // log_sparse のテスト: 戻り値そのものを検証する
    mod log_sparse {
        use super::*;

        /// Scenario: 疎な級数に対して、log_sparse は log_dense と同じ結果になる
        /// - Given: 定数項が 1 で非ゼロ項が少ない疎な形式的べき級数がある
        /// - When: log_sparse と log_dense をそれぞれ計算する
        /// - Then: 両者の結果は等しい
        #[test]
        fn matches_log_dense_for_sparse_series() {
            // Given
            let sut = create_fps_with_constant_one();
            let degree = 6;

            // When
            let log_sparse = sut.log_sparse(degree).unwrap();

            // Then
            let log_dense = sut.log_dense(degree).unwrap();
            assert_eq!(log_dense, log_sparse);
        }

        /// Scenario: 定数項が 1 でない級数には形式的対数が定義できない
        /// - Given: 定数項が 1 ではない形式的べき級数がある
        /// - When: log_sparse を計算する
        /// - Then: None が返る
        #[test]
        fn returns_none_when_constant_term_is_not_one() {
            // Given
            let sut = create_fps_with_non_one_constant();

            // When
            let result = sut.log_sparse(4);

            // Then
            assert!(result.is_none());
        }
    }
}
