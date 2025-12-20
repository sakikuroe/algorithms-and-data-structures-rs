//! 法 998244353 上の形式的べき級数の乗算を実装するモジュールである.

use super::super::convolution;
use std::collections;
use std::ops;

impl super::FPS {
    /// 形式的べき級数の列の総積を返す.
    ///
    /// AVX2 が利用可能な環境では Montgomery 表現 + NTT doubling を用いた
    /// 実装を選択し, それ以外の環境では通常表現のままの実装を選択する.
    ///
    /// # Args
    /// - `polynomials`: 総積を求める形式的べき級数の列.
    /// - `degree`: 計算する最高次数 (含む).
    ///
    /// # Returns
    /// `Self`: `polynomials` の総積のうち, `x^degree` まで (含む) の係数列.
    ///
    /// # Constraints
    /// - `degree + 1` は `convolution::MAX_NTT_LEN` 以下でなければならない.
    ///
    /// # Panics
    /// - `degree + 1` が `convolution::MAX_NTT_LEN` を超えるとき.
    ///
    /// # Complexity
    /// - Time complexity: AVX2 が利用可能な場合 O(L log L), それ以外は
    ///   `convolution::convolution` の呼び出し回数に依存する. ここで L は
    ///   `min(degree + 1, product_len)` である.
    /// - Space complexity: O(L). ここで L は戻り値の項数である.
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::modulo998244353::fps;
    ///
    /// let polynomials = vec![
    ///     fps::FPS::new(vec![1, 1]),
    ///     fps::FPS::new(vec![1, 2]),
    ///     fps::FPS::new(vec![3]),
    /// ];
    /// let product = fps::FPS::product(polynomials, 2);
    /// assert_eq!(3, product.get(0));
    /// assert_eq!(9, product.get(1));
    /// assert_eq!(6, product.get(2));
    /// ```
    pub fn product(polynomials: Vec<Self>, degree: usize) -> Self {
        #[cfg(target_arch = "x86_64")]
        {
            if std::is_x86_feature_detected!("avx2") {
                return Self::product_avx2(polynomials, degree);
            }
        }

        Self::product_scalar(polynomials, degree)
    }

    /// `x^k` を掛ける.
    ///
    /// # Args
    /// - `k`: 掛け合わせる `x` の指数.
    ///
    /// # Returns
    /// `Self`: シフト後の系列.
    ///
    /// # Constraints
    /// 制約はない.
    ///
    /// # Panics
    /// - 結果の項数が `MAX_NTT_LEN` を超えるとき.
    ///
    /// # Complexity
    /// - Time complexity: O(N + k).
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::modulo998244353::fps;
    ///
    /// let fps = fps::FPS::new(vec![1]);
    /// let shifted = fps.mul_xk(2);
    /// assert_eq!(0, shifted.get(0));
    /// assert_eq!(1, shifted.get(2));
    /// ```
    pub fn mul_xk(&self, k: usize) -> Self {
        if self.is_zero() {
            return Self { coeffs: Vec::new() };
        }
        let new_len = k.checked_add(self.len()).expect("mul_xk length overflow");
        assert!(
            new_len <= convolution::MAX_NTT_LEN,
            "mul_xk requires len <= MAX_NTT_LEN"
        );
        let mut coeffs = vec![0; new_len];
        coeffs[k..].copy_from_slice(&self.coeffs);
        Self { coeffs }
    }

    /// 形式的べき級数の列の総積を計算する (非 SIMD 実装).
    ///
    /// # Args
    /// - `polynomials`: 総積を求める形式的べき級数の列.
    /// - `degree`: 計算する最高次数 (含む).
    ///
    /// # Returns
    /// `Self`: `polynomials` の総積のうち, `x^degree` まで (含む) の係数列.
    ///
    /// # Constraints
    /// - `degree + 1` は `convolution::MAX_NTT_LEN` 以下でなければならない.
    ///
    /// # Panics
    /// - `degree + 1` が `convolution::MAX_NTT_LEN` を超えるとき.
    ///
    /// # Complexity
    /// - Time complexity: `convolution::convolution` の呼び出し回数に依存する.
    /// - Space complexity: O(L). ここで L は戻り値の項数である.
    ///
    /// # Examples
    /// ```rust,ignore
    /// // `pub fn product` から呼び出される.
    /// ```
    fn product_scalar(polynomials: Vec<Self>, degree: usize) -> Self {
        let limit = degree.checked_add(1).expect("degree + 1 overflow");
        assert!(
            limit <= convolution::MAX_NTT_LEN,
            "degree + 1 must not exceed MAX_NTT_LEN"
        );

        if polynomials.is_empty() {
            return Self { coeffs: vec![1] };
        }

        if polynomials.len() == 1 {
            let mut poly = polynomials.into_iter().next().unwrap();
            poly.truncate(limit);
            return poly;
        }

        let mut deque = collections::VecDeque::new();
        deque.push_back(vec![1]);
        for poly in polynomials {
            let mut coeffs = poly.coeffs;
            coeffs.truncate(limit);
            Self::trim_coeffs(&mut coeffs);
            deque.push_back(coeffs);
        }
        if deque.iter().any(|v| v.is_empty()) {
            return Self { coeffs: Vec::new() };
        }
        for _ in 0..(deque.len() - 1) {
            let x = deque.pop_front().unwrap();
            let y = deque.pop_front().unwrap();
            let mut z = convolution::convolution(x, y);
            z.truncate(limit);
            Self::trim_coeffs(&mut z);
            deque.push_back(z);
        }

        let coeffs = deque.pop_front().unwrap();
        Self::new(coeffs)
    }

    /// 末尾のゼロ係数を削除する.
    ///
    /// # Args
    /// - `coeffs`: 末尾のゼロ係数を削除する係数列.
    ///
    /// # Returns
    /// `()`: `coeffs` をインプレースで更新する.
    ///
    /// # Constraints
    /// 制約はない.
    ///
    /// # Panics
    /// - この関数はパニックしない.
    ///
    /// # Complexity
    /// - Time complexity: O(K). K は末尾の連続するゼロ係数の個数.
    /// - Space complexity: O(1).
    ///
    /// # Examples
    /// ```rust,ignore
    /// // 内部関数のため, 直接の使用例は省略する.
    /// ```
    fn trim_coeffs(coeffs: &mut Vec<u32>) {
        while coeffs.last().map_or(false, |c| *c == 0) {
            coeffs.pop();
        }
    }

    /// 形式的べき級数の列の総積を計算する (AVX2 + Montgomery 実装).
    ///
    /// # Args
    /// - `polynomials`: 総積を求める形式的べき級数の列.
    /// - `degree`: 計算する最高次数 (含む).
    ///
    /// # Returns
    /// `Self`: `polynomials` の総積のうち, `x^degree` まで (含む) の係数列.
    ///
    /// # Constraints
    /// - AVX2 が利用可能な環境でのみ呼び出す.
    /// - `degree + 1` は `convolution::MAX_NTT_LEN` 以下でなければならない.
    ///
    /// # Panics
    /// - `degree + 1` が `convolution::MAX_NTT_LEN` を超えるとき.
    ///
    /// # Complexity
    /// - Time complexity: `convolution_avx2::convolution_avx2` の呼び出し回数に依存する.
    /// - Space complexity: O(L). ここで L は戻り値の項数である.
    ///
    /// # Examples
    /// ```rust,ignore
    /// // `pub fn product` から呼び出される.
    /// ```
    #[cfg(target_arch = "x86_64")]
    fn product_avx2(polynomials: Vec<Self>, degree: usize) -> Self {
        use super::super::convolution_avx2;

        debug_assert!(std::is_x86_feature_detected!("avx2"));

        let limit = degree.checked_add(1).expect("degree + 1 overflow");
        assert!(
            limit <= convolution::MAX_NTT_LEN,
            "degree + 1 must not exceed MAX_NTT_LEN"
        );

        if polynomials.is_empty() {
            return Self { coeffs: vec![1] };
        }

        if polynomials.len() == 1 {
            let mut poly = polynomials.into_iter().next().unwrap();
            poly.truncate(limit);
            return poly;
        }

        let mut deque = collections::VecDeque::new();
        deque.push_back(vec![1]);
        for fps in polynomials {
            let mut coeffs = fps.coeffs;
            coeffs.truncate(limit);
            Self::trim_coeffs(&mut coeffs);
            deque.push_back(coeffs);
        }
        if deque.iter().any(|v| v.is_empty()) {
            return Self { coeffs: Vec::new() };
        }

        unsafe {
            for _ in 0..(deque.len() - 1) {
                let x = deque.pop_front().unwrap();
                let y = deque.pop_front().unwrap();
                let mut z = convolution_avx2::convolution_avx2(x, y);
                z.truncate(limit);
                Self::trim_coeffs(&mut z);
                deque.push_back(z);
            }

            let coeffs = deque.pop_front().unwrap();
            Self::new(coeffs)
        }
    }
}

impl ops::Mul for super::FPS {
    type Output = super::FPS;

    /// 2 つの形式的べき級数を畳み込みで乗算する.
    ///
    /// # Args
    /// - `self`: 左辺の系列.
    /// - `rhs`: 右辺の系列.
    ///
    /// # Returns
    /// `Self::Output`: 法 998244353 上での積を表す形式的べき級数.
    ///
    /// # Constraints
    /// 制約はない.
    ///
    /// # Panics
    /// - この関数はパニックしない.
    ///
    /// # Complexity
    /// - Time complexity: O(L log L). L は結果の項数.
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::modulo998244353::fps;
    ///
    /// let a = fps::FPS::new(vec![1, 1]);
    /// let b = fps::FPS::new(vec![1, 2]);
    /// assert_eq!(fps::FPS::new(vec![1, 3, 2]), a * b);
    /// ```
    fn mul(self, rhs: Self) -> Self::Output {
        let mut res = Self {
            coeffs: convolution::convolution(self.coeffs, rhs.coeffs),
        };
        res.trim();
        res
    }
}

impl ops::MulAssign for super::FPS {
    /// 右辺の系列を掛け, 自身を更新する.
    ///
    /// # Args
    /// - `self`: 更新対象となる系列.
    /// - `rhs`: 乗算する系列.
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
    /// - Time complexity: O(L log L). L は結果の項数.
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::modulo998244353::fps;
    ///
    /// let mut a = fps::FPS::new(vec![1, 1]);
    /// let b = fps::FPS::new(vec![1, 2]);
    /// a *= b;
    /// assert_eq!(fps::FPS::new(vec![1, 3, 2]), a);
    /// ```
    fn mul_assign(&mut self, rhs: Self) {
        let a = std::mem::take(&mut self.coeffs);
        self.coeffs = convolution::convolution(a, rhs.coeffs);
        self.trim();
    }
}
