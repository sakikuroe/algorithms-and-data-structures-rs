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
    /// - 時間計算量: AVX2 が利用可能な場合 O(L log L), それ以外は
    ///   `convolution::convolution` の呼び出し回数に依存する. ここで
    ///   L = min(degree + 1, 総積の項数) である.
    /// - 空間計算量: O(L). ここで L は戻り値の項数である.
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
        // 実行環境が AVX2 をサポートする場合は, より高速な実装を選択する.
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
    /// - 結果の項数が `convolution::MAX_NTT_LEN` を超えるとき.
    ///
    /// # Complexity
    /// - 時間計算量: O(N + k). N は元の項数である.
    /// - 空間計算量: O(N + k). N は元の項数である.
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
        // 0 系列に対するシフトは, 常に 0 系列である.
        if self.is_zero() {
            return Self { coeffs: Vec::new() };
        }

        // 新しい項数は (元の項数 + k) であり, オーバーフローは許容しない.
        let new_len = k.checked_add(self.len()).expect("mul_xk length overflow");

        // 実装都合 (NTT 長) の制約により, 最大長を超える結果は許容しない.
        assert!(
            new_len <= convolution::MAX_NTT_LEN,
            "mul_xk requires len <= MAX_NTT_LEN"
        );

        // 先頭に k 個の 0 を追加し, 残りへ元の係数をコピーする.
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
    /// - 時間計算量: `convolution::convolution` の呼び出し回数に依存する.
    /// - 空間計算量: O(L). ここで L は戻り値の項数である.
    ///
    /// # Examples
    /// ```rust,ignore
    /// // `pub fn product` から呼び出される.
    /// ```
    fn product_scalar(polynomials: Vec<Self>, degree: usize) -> Self {
        // 切り詰め先の長さは `degree + 1` である.
        let limit = degree.checked_add(1).expect("degree + 1 overflow");
        assert!(
            limit <= convolution::MAX_NTT_LEN,
            "degree + 1 must not exceed MAX_NTT_LEN"
        );

        // 空の積は 1 と定義する.
        if polynomials.is_empty() {
            return Self { coeffs: vec![1] };
        }

        // 要素が 1 個だけであれば, 切り詰めのみでよい.
        if polynomials.len() == 1 {
            let mut poly = polynomials.into_iter().next().unwrap();
            poly.truncate(limit);
            return poly;
        }

        // 常に短い系列同士から畳み込むため, VecDeque に積み上げて順に消していく.
        let mut deque = collections::VecDeque::new();
        deque.push_back(vec![1]);
        for poly in polynomials {
            let mut coeffs = poly.coeffs;
            coeffs.truncate(limit);
            Self::trim_coeffs(&mut coeffs);
            deque.push_back(coeffs);
        }

        // 途中に 0 系列が含まれる場合, 総積も 0 系列になる.
        if deque.iter().any(|v| v.is_empty()) {
            return Self { coeffs: Vec::new() };
        }

        // 2 つずつ畳み込み, 末尾の 0 を削除しつつ再びキューへ戻す.
        for _ in 0..(deque.len() - 1) {
            let x = deque.pop_front().unwrap();
            let y = deque.pop_front().unwrap();
            let mut z = convolution::convolution(x, y);
            z.truncate(limit);
            Self::trim_coeffs(&mut z);
            deque.push_back(z);
        }

        // 最後に残った係数列を系列として返す.
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
    /// - 時間計算量: O(K). K は末尾の連続するゼロ係数の個数である.
    /// - 空間計算量: O(1).
    ///
    /// # Examples
    /// ```rust,ignore
    /// // 内部関数のため, 直接の使用例は省略する.
    /// ```
    fn trim_coeffs(coeffs: &mut Vec<u32>) {
        // 末尾から 0 を削除し, 最高次の係数が非 0 となるように正規化する.
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
    /// - 時間計算量: `convolution_avx2::convolution_avx2` の呼び出し回数に依存する.
    /// - 空間計算量: O(L). ここで L は戻り値の項数である.
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

        // 係数列の長さを制限し, 末尾の 0 を削除して正規化したものをキューに積む.
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

        for _ in 0..(deque.len() - 1) {
            let x = deque.pop_front().unwrap();
            let y = deque.pop_front().unwrap();

            // AVX2 を用いた畳み込みは, ターゲット機能の前提を満たす必要がある.
            //
            // この関数は `is_x86_feature_detected!("avx2")` を満たすときにのみ
            // 呼び出されるため, ここでの呼び出しは安全である.
            let mut z = unsafe { convolution_avx2::convolution_avx2(x, y) };
            z.truncate(limit);
            Self::trim_coeffs(&mut z);
            deque.push_back(z);
        }

        let coeffs = deque.pop_front().unwrap();
        Self::new(coeffs)
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
    /// - 時間計算量: O(L log L). L は結果の項数である.
    /// - 空間計算量: O(L). L は結果の項数である.
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
        // 係数列を畳み込みで乗算し, 末尾の 0 を削除して正規化する.
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
    /// - 時間計算量: O(L log L). L は結果の項数である.
    /// - 空間計算量: O(L). L は結果の項数である.
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
        // `self.coeffs` をムーブし, 余計な複製を避けて畳み込みを計算する.
        let a = std::mem::take(&mut self.coeffs);
        self.coeffs = convolution::convolution(a, rhs.coeffs);
        self.trim();
    }
}
