//! 法 998244353 上の形式的べき級数を扱うモジュールである。

/// `FPS` の加算 (`Add`, `AddAssign`) を実装するモジュールである。
mod add;
/// 線形漸化式の第 `k` 項を求める Bostan-Mori 法を実装するモジュールである。
pub mod bostan_mori;
/// `FPS` の `x^k` による除算 (`div_xk`) を実装するモジュールである。
mod div;
/// `FPS` の指数関数 (`exp`) を実装するモジュールである。
pub mod exp;
/// `FPS` の逆元 (`inverse`) を実装するモジュールである。
pub mod inv;
/// `FPS` の対数関数 (`log`) を実装するモジュールである。
pub mod log;
/// `FPS` の乗算 (`Mul`, `MulAssign`, `mul_xk`, `product`) を実装するモジュールである。
mod mul;
/// 分割数関連の母関数を計算するモジュールである。
pub mod partition;
/// `FPS` の冪乗 (`pow`) を実装するモジュールである。
pub mod pow;
/// `FPS` の減算・符号反転 (`Sub`, `SubAssign`, `Neg`) を実装するモジュールである。
mod sub;

use super::{convolution, modulo};
use std::{fmt, sync};

/// `FPS::integral` で用いる階乗と階乗逆元のテーブルである。
struct FactorialTables {
    /// 階乗テーブル。`fact[i]` は `i!` を法 998244353 で表した値である。
    fact: Vec<u32>,

    /// 階乗逆元テーブル。`ifact[i]` は `i!` の法 998244353 上での逆元である。
    ifact: Vec<u32>,
}

/// `FPS::integral` で用いる階乗テーブルを 1 度だけ構築して保持する。
static FACTORIAL_TABLES: sync::OnceLock<FactorialTables> = sync::OnceLock::new();

/// 階乗と階乗逆元のテーブルへの参照を返す。
///
/// # Args
/// 引数はない。
///
/// # Returns
/// `&'static FactorialTables`: `0..=MAX_NTT_LEN` の階乗テーブルと逆階乗テーブル。
///
/// # Constraints
/// - `convolution::MAX_NTT_LEN < 998244353` を満たす。
///
/// # Panics
/// - この関数はパニックしない。
///
/// # Complexity
/// - Time complexity: 初回のみ O(MAX_NTT_LEN)、2 回目以降は O(1)。
/// - Space complexity: O(MAX_NTT_LEN)。
///
/// # Examples
/// ```rust,ignore
/// // `FPS::integral` の内部で利用する。
/// ```
fn factorial_tables() -> &'static FactorialTables {
    FACTORIAL_TABLES.get_or_init(|| {
        let max_len = convolution::MAX_NTT_LEN;

        let mut fact = vec![0_u32; max_len + 1];
        fact[0] = 1;
        for i in 1..=max_len {
            fact[i] = modulo::mul(fact[i - 1], i as u32);
        }

        let mut ifact = vec![0_u32; max_len + 1];
        ifact[max_len] = modulo::inv(fact[max_len]);
        for i in (1..=max_len).rev() {
            ifact[i - 1] = modulo::mul(ifact[i], i as u32);
        }

        FactorialTables { fact, ifact }
    })
}

/// 法 998244353 上の形式的べき級数を表す。末尾のゼロ係数は正規形として削除する。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FPS {
    /// 次数の昇順に並んだ係数列。`coeffs[i]` は `x^i` の係数を法 998244353 で表す。
    coeffs: Vec<u32>,
}

impl FPS {
    /// 形式的べき級数が保持できる項数の上限を検証する。
    ///
    /// # Args
    /// - `len`: 検証する項数。
    ///
    /// # Returns
    /// `()`: 検証するだけで値は返さない。
    ///
    /// # Constraints
    /// - 末尾のゼロ係数を除去した後の項数は `MAX_NTT_LEN` 以下でなければならない。
    ///
    /// # Panics
    /// - `len > MAX_NTT_LEN` のときパニックする。
    ///
    /// # Complexity
    /// - Time complexity: O(1)。
    /// - Space complexity: O(1)。
    ///
    /// # Examples
    /// ```rust,ignore
    /// // `FPS::new` などから呼び出される。
    /// ```
    fn assert_len_within_max_ntt_len(len: usize) {
        assert!(
            len <= convolution::MAX_NTT_LEN,
            "FPS length must not exceed MAX_NTT_LEN"
        );
    }

    /// 係数列から形式的べき級数を生成する。
    ///
    /// # Args
    /// - `coefficients`: 次数が小さい順に並んだ係数列。
    ///
    /// # Returns
    /// `Self`: 末尾のゼロ項を除去した形式的べき級数。
    ///
    /// # Constraints
    /// 制約はない。
    ///
    /// # Panics
    /// - 係数が法 998244353 以上のとき。
    /// - 末尾のゼロ係数を除去した後の項数が `MAX_NTT_LEN` を超えるとき。
    ///
    /// # Complexity
    /// - Time complexity: O(N)。ここで N は `coefficients.len()`。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::modulo998244353::fps;
    ///
    /// let fps = fps::FPS::new(vec![1, 2, 0]);
    /// assert_eq!(2, fps.len());
    /// ```
    pub fn new(mut coefficients: Vec<u32>) -> Self {
        coefficients
            .iter_mut()
            .for_each(|c| assert!(*c < modulo::M));
        let mut fps = FPS {
            coeffs: coefficients,
        };
        fps.trim();
        Self::assert_len_within_max_ntt_len(fps.len());
        fps
    }

    /// 保持している項数 (最高次 + 1) を返す。
    ///
    /// # Args
    /// 引数はない。
    ///
    /// # Returns
    /// `usize`: ゼロを取り除いた後の項数 (最高次数 + 1)。
    ///
    /// # Constraints
    /// - `value != 0` のとき、`index + 1` は `MAX_NTT_LEN` 以下でなければならない。
    ///
    /// # Panics
    /// - この関数はパニックしない。
    ///
    /// # Complexity
    /// - Time complexity: O(1)。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::modulo998244353::fps;
    ///
    /// let fps = fps::FPS::new(vec![3]);
    /// assert_eq!(1, fps.len());
    /// ```
    pub fn len(&self) -> usize {
        self.coeffs.len()
    }

    /// 非ゼロ多項式なら最高次数を返す。
    ///
    /// # Args
    /// 引数はない。
    ///
    /// # Returns
    /// `Option<usize>`: 非ゼロ多項式では `Some(degree)`、ゼロでは `None`。
    ///
    /// # Constraints
    /// 制約はない。
    ///
    /// # Panics
    /// - この関数はパニックしない。
    ///
    /// # Complexity
    /// - Time complexity: O(1)。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::modulo998244353::fps;
    ///
    /// let fps = fps::FPS::new(vec![0, 0, 5]);
    /// assert_eq!(Some(2), fps.degree());
    /// ```
    pub fn degree(&self) -> Option<usize> {
        if self.is_zero() {
            None
        } else {
            Some(self.len() - 1)
        }
    }

    /// ゼロ多項式のときに `true` を返す。
    ///
    /// # Args
    /// 引数はない。
    ///
    /// # Returns
    /// `bool`: すべての係数がゼロかどうか。
    ///
    /// # Constraints
    /// 制約はない。
    ///
    /// # Panics
    /// - この関数はパニックしない。
    ///
    /// # Complexity
    /// - Time complexity: O(1)。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::modulo998244353::fps;
    ///
    /// let zero = fps::FPS::new(Vec::new());
    /// assert!(zero.is_zero());
    /// ```
    pub fn is_zero(&self) -> bool {
        self.coeffs.is_empty()
    }

    /// `x^index` の係数を返す。対応する項が無い場合は 0 を返す。
    ///
    /// # Args
    /// - `index`: 取り出す次数。
    ///
    /// # Returns
    /// `u32`: 指定した次数の係数 (法 998244353 での値)。
    ///
    /// # Constraints
    /// 制約はない。
    ///
    /// # Panics
    /// - この関数はパニックしない。
    ///
    /// # Complexity
    /// - Time complexity: O(1)。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::modulo998244353::fps;
    ///
    /// let fps = fps::FPS::new(vec![5]);
    /// assert_eq!(5, fps.get(0));
    /// assert_eq!(0, fps.get(3));
    /// ```
    pub fn get(&self, index: usize) -> u32 {
        *self.coeffs.get(index).unwrap_or(&0)
    }

    /// 必要に応じて拡張し、`x^index` の係数を設定する。
    ///
    /// # Args
    /// - `index`: 上書きする次数。
    /// - `value`: 設定する新しい係数。
    ///
    /// # Returns
    /// `()`: インプレースで更新する。
    ///
    /// # Constraints
    /// 制約はない。
    ///
    /// # Panics
    /// - `value >= 998244353` のとき。
    /// - `value != 0` かつ `index + 1 > MAX_NTT_LEN` のとき。
    ///
    /// # Complexity
    /// - Time complexity: O(N)。N は `index` になりうる。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::modulo998244353::fps;
    ///
    /// let mut fps = fps::FPS::new(vec![1]);
    /// fps.set(2, 4);
    /// assert_eq!(4, fps.get(2));
    /// ```
    pub fn set(&mut self, index: usize, value: u32) {
        assert!(value < modulo::M);
        if value == 0 && index >= self.len() {
            return;
        }
        let required_len = index.checked_add(1).expect("index overflow");
        Self::assert_len_within_max_ntt_len(required_len);
        if self.len() <= index {
            self.coeffs.resize(index + 1, 0);
        }
        self.coeffs[index] = value;
        if value == 0 {
            self.trim();
        }
    }

    /// 係数スライスへのイミュータブル参照を返す。
    ///
    /// # Args
    /// 引数はない。
    ///
    /// # Returns
    /// `&[u32]`: 係数への参照。
    ///
    /// # Constraints
    /// 制約はない。
    ///
    /// # Panics
    /// - この関数はパニックしない。
    ///
    /// # Complexity
    /// - Time complexity: O(1)。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::modulo998244353::fps;
    ///
    /// let fps = fps::FPS::new(vec![7]);
    /// assert_eq!(1, fps.coefficients().len());
    /// ```
    pub fn coefficients(&self) -> &[u32] {
        &self.coeffs
    }

    /// 非ゼロ係数を持つ項を列挙するイテレーターを返す。
    ///
    /// # Args
    /// 引数はない。
    ///
    /// # Returns
    /// `impl Iterator<Item = (usize, u32)> + '_`: `(次数, 係数)` のタプル列を返す。
    ///
    /// # Constraints
    /// 制約はない。
    ///
    /// # Panics
    /// - この関数はパニックしない。
    ///
    /// # Complexity
    /// - Time complexity: O(N)。ここで N は `self.len()`。
    /// - Space complexity: O(1)。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::modulo998244353::fps;
    ///
    /// let f = fps::FPS::new(vec![2, 0, 3]);
    /// let terms = f.non_zero_terms_iter().collect::<Vec<(usize, u32)>>();
    /// assert_eq!(vec![(0, 2), (2, 3)], terms);
    /// ```
    pub fn non_zero_terms_iter(&self) -> impl Iterator<Item = (usize, u32)> + '_ {
        self.coeffs
            .iter()
            .enumerate()
            .filter_map(|(i, &c)| if c == 0 { None } else { Some((i, c)) })
    }

    /// 非ゼロ係数を持つ項を列挙する。
    ///
    /// # Args
    /// 引数はない。
    ///
    /// # Returns
    /// `Vec<(usize, u32)>`: `(次数, 係数)` のタプル列を返す。
    ///
    /// # Constraints
    /// 制約はない。
    ///
    /// # Panics
    /// - この関数はパニックしない。
    ///
    /// # Complexity
    /// - Time complexity: O(N)。ここで N は `self.len()`。
    /// - Space complexity: O(K)。ここで K は非ゼロ項の個数。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::modulo998244353::fps;
    ///
    /// let f = fps::FPS::new(vec![2, 0, 3]);
    /// assert_eq!(vec![(0, 2), (2, 3)], f.non_zero_terms());
    /// ```
    pub fn non_zero_terms(&self) -> Vec<(usize, u32)> {
        self.non_zero_terms_iter().collect()
    }

    /// 指定した項数に切り詰める。
    ///
    /// # Args
    /// - `len`: 新しい項数 (最高次数 + 1)。
    ///
    /// # Returns
    /// `()`: インプレースで更新する。
    ///
    /// # Constraints
    /// 制約はない。
    ///
    /// # Panics
    /// - この関数はパニックしない。
    ///
    /// # Complexity
    /// - Time complexity: O(N)。N は `self.len()`。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::modulo998244353::fps;
    ///
    /// let mut fps = fps::FPS::new(vec![1; 4]);
    /// fps.truncate(2);
    /// assert_eq!(2, fps.len());
    /// ```
    pub fn truncate(&mut self, len: usize) {
        self.coeffs.truncate(len);
        self.trim();
    }

    // `mul_xk` と `div_xk` はそれぞれ `mul.rs` と `div.rs` に定義する。

    /// 形式的微分をインプレースで計算する。
    ///
    /// # Args
    /// 引数はない。
    ///
    /// # Returns
    /// `&mut Self`: 微分後の系列への参照。
    ///
    /// # Constraints
    /// 制約はない。
    ///
    /// # Panics
    /// - この関数はパニックしない。
    ///
    /// # Complexity
    /// - Time complexity: O(N)。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::modulo998244353::fps;
    ///
    /// let mut fps = fps::FPS::new(vec![3, 4]);
    /// fps.derivative();
    /// assert_eq!(4, fps.get(0));
    /// ```
    pub fn derivative(&mut self) -> &mut Self {
        let len = self.len();
        if len <= 1 {
            self.coeffs.clear();
            return self;
        }
        for i in 0..(len - 1) {
            self.coeffs[i] = modulo::mul((i + 1) as u32, self.coeffs[i + 1]);
        }
        self.coeffs.truncate(len - 1);
        self.trim();
        self
    }

    /// 定数項を 0 とする形式的積分をインプレースで計算する。
    ///
    /// # Args
    /// 引数はない。
    ///
    /// # Returns
    /// `&mut Self`: 定数項を 0 とした積分系列への参照。
    ///
    /// # Constraints
    /// - 各分母 (1、2、...) が 998244353 上で逆元を持つ。
    /// - `self.len() + 1` は法より小さくなければならない。
    /// - `self.len() + 1` は `MAX_NTT_LEN` 以下でなければならない。
    ///
    /// # Panics
    /// - `self.len() + 1 >= 998244353` のときパニックする。
    /// - `self.len() + 1 > MAX_NTT_LEN` のときパニックする。
    ///
    /// # Complexity
    /// - Time complexity: O(N)。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::modulo998244353::fps;
    ///
    /// let mut fps = fps::FPS::new(vec![2]);
    /// fps.integral();
    /// assert_eq!(0, fps.get(0));
    /// assert_eq!(2, fps.get(1));
    /// ```
    pub fn integral(&mut self) -> &mut Self {
        assert!(
            self.len() + 1 < modulo::M as usize,
            "Integral requires degree + 1 < modulus"
        );
        assert!(
            self.len() + 1 <= convolution::MAX_NTT_LEN,
            "Integral requires len + 1 <= MAX_NTT_LEN"
        );
        if self.is_zero() {
            return self;
        }

        let len = self.len();
        let tables = factorial_tables();
        let fact = &tables.fact;
        let ifact = &tables.ifact;

        self.coeffs.push(0);
        for i in (0..len).rev() {
            let scaled = modulo::mul(self.coeffs[i], ifact[i + 1]);
            let integrated = modulo::mul(scaled, fact[i]);
            self.coeffs[i + 1] = integrated;
        }
        self.coeffs[0] = 0;
        self.trim();
        self
    }

    /// 末尾のゼロ係数を除去して正規形にする。
    ///
    /// # Args
    /// - `self`: 正規化対象となる系列。
    ///
    /// # Returns
    /// `()`: 自身をインプレースで更新する。
    ///
    /// # Constraints
    /// 制約はない。
    ///
    /// # Panics
    /// - 項数が `MAX_NTT_LEN` を超えるとき。
    ///
    /// # Complexity
    /// - Time complexity: O(N)。N は末尾のゼロの個数である。
    /// - Space complexity: O(1)。
    ///
    /// # Examples
    /// ```rust,ignore
    /// // `FPS::new` などから呼び出される内部関数である。
    /// use anmitsu::modulo998244353::fps;
    ///
    /// let mut f = fps::FPS::new(vec![1, 0, 0]);
    /// f.trim();
    /// assert_eq!(fps::FPS::new(vec![1]), f);
    /// ```
    fn trim(&mut self) {
        while self.coeffs.last().map_or(false, |c| *c == 0) {
            self.coeffs.pop();
        }
        Self::assert_len_within_max_ntt_len(self.len());
    }
}

impl fmt::Display for FPS {
    /// 表示用の文字列表現を生成する。
    ///
    /// # Args
    /// - `self`: 出力対象の系列。
    /// - `f`: 出力先フォーマッタ。
    ///
    /// # Returns
    /// `fmt::Result`: フォーマット結果。
    ///
    /// # Constraints
    /// 制約はない。
    ///
    /// # Panics
    /// - フォーマット先がエラーを返したとき。
    ///
    /// # Complexity
    /// - Time complexity: O(N)。N は項数。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::modulo998244353::fps;
    ///
    /// let fps = fps::FPS::new(vec![1, 0, 2]);
    /// assert_eq!("1x^0 + 2x^2", format!("{}", fps));
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_zero() {
            return write!(f, "0");
        }
        let terms = self
            .coeffs
            .iter()
            .enumerate()
            .filter(|(_, c)| **c != 0)
            .map(|(i, c)| format!("{}x^{}", c, i))
            .collect::<Vec<String>>();
        write!(f, "{}", terms.join(" + "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::panic;

    // new のテスト: 戻り値そのものを検証する
    mod new {
        use super::*;

        /// Scenario: 末尾にゼロ係数を含む係数列から生成すると、 正規化されて末尾のゼロが除去される
        /// - Given: 末尾にゼロ係数を含む係数列 [1, 2, 0] がある
        /// - When: FPS::new で生成する
        /// - Then: 項数は 2 になる
        #[test]
        fn trims_trailing_zero_coefficients() {
            // Given
            let coeffs = vec![1, 2, 0];
            // When
            let sut = FPS::new(coeffs);
            // Then
            assert_eq!(2, sut.len());
            assert_eq!(1, sut.get(0));
            assert_eq!(2, sut.get(1));
        }

        /// Scenario: 境界となる係数列からも、 正しく正規化された系列が生成される
        /// - Given: 空、 単項、 全てがゼロなど、 境界となる係数列がある
        /// - When: FPS::new で生成する
        /// - Then: 各ケースで期待通りの項数と係数になる
        #[test]
        fn normalizes_boundary_value_coefficient_lists() {
            // Given
            let cases = [
                // 空の係数列はそのままゼロ多項式になる
                (vec![], 0),
                // 全てがゼロの係数列は、 全て除去されてゼロ多項式になる
                (vec![0, 0, 0], 0),
                // 単項の係数列は、 そのままの項数になる
                (vec![5], 1),
            ];

            for (coeffs, expected_len) in cases {
                // When
                let sut = FPS::new(coeffs);
                // Then
                assert_eq!(expected_len, sut.len());
                assert!(sut.is_zero() == (expected_len == 0));
            }
        }

        /// Scenario: 法 998244353 以上の係数を渡すとパニックする
        /// - Given: 法と等しい係数を含む係数列がある
        /// - When: FPS::new で生成しようとする
        /// - Then: パニックする
        #[test]
        fn panics_when_coefficient_is_not_less_than_modulus() {
            // Given
            let coeffs = vec![modulo::M];
            // When
            let result = panic::catch_unwind(|| FPS::new(coeffs));
            // Then
            assert!(
                result.is_err(),
                "new should panic when a coefficient is not less than the modulus"
            );
        }

        /// Scenario: 正規化後の項数が MAX_NTT_LEN を超えるとパニックする
        /// - Given: MAX_NTT_LEN を 1 超える項数を持つ係数列がある
        /// - When: FPS::new で生成しようとする
        /// - Then: パニックする
        #[test]
        fn panics_when_length_exceeds_max_ntt_len() {
            // Given
            let coeffs = vec![1_u32; convolution::MAX_NTT_LEN + 1];
            // When
            let result = panic::catch_unwind(|| FPS::new(coeffs));
            // Then
            assert!(
                result.is_err(),
                "new should panic when length exceeds MAX_NTT_LEN"
            );
        }
    }

    // len のテスト: 戻り値そのものを検証する
    mod len {
        use super::*;

        /// Scenario: 境界となる項数の系列で、 len が正しい項数を返す
        /// - Given: 項数が 0, 1 の系列がある
        /// - When: len を呼び出す
        /// - Then: 各ケースで期待通りの項数が返る
        #[test]
        fn returns_number_of_terms_at_boundary_sizes() {
            // Given
            let cases = [(vec![], 0), (vec![9], 1)];

            for (coeffs, expected) in cases {
                let sut = FPS::new(coeffs);
                // When
                let result = sut.len();
                // Then
                assert_eq!(expected, result);
            }
        }
    }

    // degree のテスト: 戻り値そのものを検証する
    mod degree {
        use super::*;

        /// Scenario: 非ゼロ多項式では、 最高次数が Some で返る
        /// - Given: 係数 [0, 0, 5] を持つ系列がある
        /// - When: degree を呼び出す
        /// - Then: Some(2) が返る
        #[test]
        fn returns_highest_degree_for_non_zero_polynomial() {
            // Given
            let sut = FPS::new(vec![0, 0, 5]);
            // When
            let result = sut.degree();
            // Then
            assert_eq!(Some(2), result);
        }

        /// Scenario: ゼロ多項式では、 None が返る
        /// - Given: ゼロ多項式がある
        /// - When: degree を呼び出す
        /// - Then: None が返る
        #[test]
        fn returns_none_for_zero_polynomial() {
            // Given
            let sut = FPS::new(Vec::new());
            // When
            let result = sut.degree();
            // Then
            assert_eq!(None, result);
        }
    }

    // is_zero のテスト: 戻り値そのものを検証する
    mod is_zero {
        use super::*;

        /// Scenario: 境界となる系列で、 is_zero が正しく判定する
        /// - Given: ゼロ多項式と非ゼロ多項式がある
        /// - When: is_zero を呼び出す
        /// - Then: 各ケースで期待通りの真偽値が返る
        #[test]
        fn distinguishes_zero_from_non_zero_polynomial() {
            // Given
            let cases = [(vec![], true), (vec![1], false)];

            for (coeffs, expected) in cases {
                let sut = FPS::new(coeffs);
                // When
                let result = sut.is_zero();
                // Then
                assert_eq!(expected, result);
            }
        }
    }

    // get のテスト: 戻り値そのものを検証する
    mod get {
        use super::*;

        /// Scenario: 保持している次数の係数を取得できる
        /// - Given: 係数 [5] を持つ系列がある
        /// - When: 次数 0 の係数を取得する
        /// - Then: 5 が返る
        #[test]
        fn returns_coefficient_at_held_index() {
            // Given
            let sut = FPS::new(vec![5]);
            // When
            let result = sut.get(0);
            // Then
            assert_eq!(5, result);
        }

        /// Scenario: 保持していない次数を取得すると 0 が返る
        /// - Given: 係数 [5] を持つ系列と、 ゼロ多項式がある
        /// - When: 項数を超える次数の係数を取得する
        /// - Then: 各ケースで 0 が返る
        #[test]
        fn returns_zero_for_index_beyond_length() {
            // Given
            let cases = [(vec![5], 3), (Vec::new(), 0)];

            for (coeffs, index) in cases {
                let sut = FPS::new(coeffs);
                // When
                let result = sut.get(index);
                // Then
                assert_eq!(0, result);
            }
        }
    }

    // set のテスト: 呼び出し後の状態変化を検証する
    mod set {
        use super::*;

        /// Scenario: 保持している次数へ値を設定すると、 その次数の係数が更新される
        /// - Given: 係数 [1] を持つ系列がある
        /// - When: 次数 0 へ 9 を設定する
        /// - Then: 次数 0 の係数が 9 になる
        #[test]
        fn overwrites_coefficient_at_held_index() {
            // Given
            let mut sut = FPS::new(vec![1]);
            // When
            sut.set(0, 9);
            // Then
            assert_eq!(9, sut.get(0));
        }

        /// Scenario: 保持していない次数へ非ゼロ値を設定すると、 その次数まで拡張される
        /// - Given: 係数 [1] を持つ系列がある
        /// - When: 次数 4 へ 3 を設定する
        /// - Then: 項数が 5 になり、 次数 4 の係数が 3 になる
        #[test]
        fn extends_length_when_setting_non_zero_value_beyond_length() {
            // Given
            let mut sut = FPS::new(vec![1]);
            // When
            sut.set(4, 3);
            // Then
            assert_eq!(5, sut.len());
            assert_eq!(3, sut.get(4));
        }

        /// Scenario: 保持していない次数へ 0 を設定しても、 系列は拡張されない
        /// - Given: ゼロ多項式がある
        /// - When: 項数を超える複数の次数へ 0 を設定する
        /// - Then: 系列はゼロ多項式のまま拡張されない
        #[test]
        fn does_not_extend_when_setting_zero_beyond_length() {
            // Given
            let mut sut = FPS::new(Vec::new());
            // When
            for i in 0..16 {
                sut.set(i, 0);
            }
            // Then
            assert!(sut.is_zero());
            assert_eq!(0, sut.len());
        }

        /// Scenario: 最高次の係数へ 0 を設定すると、 末尾のゼロが除去されて正規化される
        /// - Given: 係数 [1, 2] を持つ系列がある
        /// - When: 最高次 (次数 1) へ 0 を設定する
        /// - Then: 項数が 1 に縮む
        #[test]
        fn trims_when_setting_zero_at_highest_degree() {
            // Given
            let mut sut = FPS::new(vec![1, 2]);
            // When
            sut.set(1, 0);
            // Then
            assert_eq!(1, sut.len());
            assert_eq!(1, sut.get(0));
        }

        /// Scenario: 法 998244353 以上の値を設定しようとするとパニックする
        /// - Given: 係数 [1] を持つ系列がある
        /// - When: 法と等しい値を設定しようとする
        /// - Then: パニックする
        #[test]
        fn panics_when_value_is_not_less_than_modulus() {
            // Given
            let mut sut = FPS::new(vec![1]);
            // When
            let result = panic::catch_unwind(panic::AssertUnwindSafe(|| sut.set(0, modulo::M)));
            // Then
            assert!(
                result.is_err(),
                "set should panic when the value is not less than the modulus"
            );
        }

        /// Scenario: 拡張後の項数が MAX_NTT_LEN を超えるとパニックする
        /// - Given: 係数 [1] を持つ系列と、 MAX_NTT_LEN と等しい次数がある
        /// - When: その次数へ非ゼロ値を設定しようとする
        /// - Then: パニックする
        #[test]
        fn panics_when_extended_length_exceeds_max_ntt_len() {
            // Given
            let mut sut = FPS::new(vec![1]);
            let index = convolution::MAX_NTT_LEN;
            // When
            let result = panic::catch_unwind(panic::AssertUnwindSafe(|| sut.set(index, 1)));
            // Then
            assert!(
                result.is_err(),
                "set should panic when exceeding MAX_NTT_LEN"
            );
        }
    }

    // coefficients のテスト: 戻り値そのものを検証する
    mod coefficients {
        use super::*;

        /// Scenario: 保持している係数列と同じ内容のスライスが返る
        /// - Given: 係数 [1, 0, 2] を持つ系列がある
        /// - When: coefficients を呼び出す
        /// - Then: [1, 0, 2] が返る
        #[test]
        fn returns_slice_matching_stored_coefficients() {
            // Given
            let sut = FPS::new(vec![1, 0, 2]);
            // When
            let result = sut.coefficients();
            // Then
            assert_eq!(&[1, 0, 2], result);
        }
    }

    // non_zero_terms と non_zero_terms_iter のテスト: 戻り値そのものを検証する
    mod non_zero_terms {
        use super::*;

        /// Scenario: 非ゼロ係数のみが (次数、 係数) の組として列挙される
        /// - Given: 係数 [2, 0, 3] を持つ系列がある
        /// - When: non_zero_terms を呼び出す
        /// - Then: [(0, 2), (2, 3)] が返る
        #[test]
        fn enumerates_only_non_zero_terms() {
            // Given
            let sut = FPS::new(vec![2, 0, 3]);
            // When
            let result = sut.non_zero_terms();
            // Then
            assert_eq!(vec![(0, 2), (2, 3)], result);
        }

        /// Scenario: non_zero_terms_iter も、 non_zero_terms と同じ列を返す
        /// - Given: 係数 [2, 0, 3] を持つ系列がある
        /// - When: non_zero_terms_iter を呼び出す
        /// - Then: [(0, 2), (2, 3)] が返る
        #[test]
        fn iterator_matches_non_zero_terms() {
            // Given
            let sut = FPS::new(vec![2, 0, 3]);
            // When
            let result = sut.non_zero_terms_iter().collect::<Vec<(usize, u32)>>();
            // Then
            assert_eq!(vec![(0, 2), (2, 3)], result);
        }

        /// Scenario: ゼロ多項式では、 非ゼロ項の列は空になる
        /// - Given: ゼロ多項式がある
        /// - When: non_zero_terms を呼び出す
        /// - Then: 空の列が返る
        #[test]
        fn returns_empty_for_zero_polynomial() {
            // Given
            let sut = FPS::new(Vec::new());
            // When
            let result = sut.non_zero_terms();
            // Then
            assert_eq!(Vec::<(usize, u32)>::new(), result);
        }
    }

    // truncate のテスト: 呼び出し後の状態変化を検証する
    mod truncate {
        use super::*;

        /// Scenario: 現在の項数より小さい長さへ切り詰めると、 項数がその長さになる
        /// - Given: 項数 4 の系列がある
        /// - When: 長さ 2 へ切り詰める
        /// - Then: 項数が 2 になる
        #[test]
        fn shrinks_to_specified_length() {
            // Given
            let mut sut = FPS::new(vec![1; 4]);
            // When
            sut.truncate(2);
            // Then
            assert_eq!(2, sut.len());
        }

        /// Scenario: 境界となる長さへ切り詰めても、 正しく処理される
        /// - Given: 項数 3 の系列がある
        /// - When: 長さ 0, および現在の項数を超える長さへ切り詰める
        /// - Then: それぞれ期待通りの項数になる
        #[test]
        fn truncates_for_boundary_lengths() {
            // Given
            let cases = [
                // 長さ 0 への切り詰めはゼロ多項式になる
                (0, 0),
                // 現在の項数を超える長さを指定しても、 項数は変化しない
                (10, 3),
            ];

            for (len, expected) in cases {
                let mut sut = FPS::new(vec![1, 2, 3]);
                // When
                sut.truncate(len);
                // Then
                assert_eq!(expected, sut.len());
            }
        }

        /// Scenario: 最高次の係数が切り詰めで失われると、 さらに末尾のゼロが除去される
        /// - Given: 係数 [1, 0, 3] を持つ系列がある
        /// - When: 長さ 2 へ切り詰める
        /// - Then: 最高次のゼロ係数が除去され、 項数が 1 になる
        #[test]
        fn trims_trailing_zero_after_truncation() {
            // Given
            let mut sut = FPS::new(vec![1, 0, 3]);
            // When
            sut.truncate(2);
            // Then
            assert_eq!(1, sut.len());
            assert_eq!(1, sut.get(0));
        }
    }

    // derivative のテスト: 呼び出し後の状態変化を検証する
    mod derivative {
        use super::*;

        /// Scenario: 形式的微分をすると、 各係数が次数倍された上で 1 次シフトする
        /// - Given: 係数 [3, 4] を持つ系列がある
        /// - When: derivative を呼び出す
        /// - Then: 係数 [4] を持つ系列になる
        #[test]
        fn differentiates_each_term() {
            // Given
            let mut sut = FPS::new(vec![3, 4]);
            // When
            sut.derivative();
            // Then
            assert_eq!(FPS::new(vec![4]), sut);
        }

        /// Scenario: 境界となる項数の系列でも、 微分が正しく行われる
        /// - Given: 項数 0 (ゼロ多項式) および項数 1 (定数項のみ) の系列がある
        /// - When: derivative を呼び出す
        /// - Then: いずれもゼロ多項式になる
        #[test]
        fn differentiates_boundary_length_series_to_zero() {
            // Given
            let cases = [Vec::new(), vec![7]];

            for coeffs in cases {
                let mut sut = FPS::new(coeffs);
                // When
                sut.derivative();
                // Then
                assert!(sut.is_zero());
            }
        }
    }

    // integral のテスト: 呼び出し後の状態変化を検証する
    mod integral {
        use super::*;

        /// Scenario: 定数項を 0 とする形式的積分をすると、 各係数が 1 次シフトして次数で割られる
        /// - Given: 係数 [2] を持つ系列がある
        /// - When: integral を呼び出す
        /// - Then: 係数 [0, 2] を持つ系列になる
        #[test]
        fn integrates_with_zero_constant_term() {
            // Given
            let mut sut = FPS::new(vec![2]);
            // When
            sut.integral();
            // Then
            assert_eq!(FPS::new(vec![0, 2]), sut);
        }

        /// Scenario: 定数項が 0 の系列は、 微分してから積分すると元に戻る
        /// - Given: 定数項が 0 の系列 [0, 2, 3] がある
        /// - When: derivative してから integral を呼び出す
        /// - Then: 元の系列と一致する
        #[test]
        fn restores_original_series_after_derivative_when_constant_term_is_zero() {
            // Given
            let sut = FPS::new(vec![0, 2, 3]);
            let mut restored = sut.clone();
            // When
            restored.derivative();
            restored.integral();
            // Then
            assert_eq!(sut, restored);
        }

        /// Scenario: ゼロ多項式を積分しても、 ゼロ多項式のままである
        /// - Given: ゼロ多項式がある
        /// - When: integral を呼び出す
        /// - Then: ゼロ多項式のままである
        #[test]
        fn keeps_zero_polynomial_zero() {
            // Given
            let mut sut = FPS::new(Vec::new());
            // When
            sut.integral();
            // Then
            assert!(sut.is_zero());
        }

        /// Scenario: 積分後の項数が MAX_NTT_LEN を超えるとパニックする
        /// - Given: 項数が MAX_NTT_LEN と等しい系列がある
        /// - When: integral を呼び出そうとする
        /// - Then: パニックする
        #[test]
        fn panics_when_result_length_exceeds_max_ntt_len() {
            // Given
            let mut sut = FPS::new(vec![1_u32; convolution::MAX_NTT_LEN]);
            // When
            let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                sut.integral();
            }));
            // Then
            assert!(
                result.is_err(),
                "integral should panic when the resulting length exceeds MAX_NTT_LEN"
            );
        }
    }

    // trim のテスト: 呼び出し後の状態変化を検証する
    mod trim {
        use super::*;

        /// Scenario: 末尾のゼロ係数を正規化して除去する
        /// - Given: 末尾にゼロ係数を含む係数列を直接保持する系列がある
        /// - When: trim を呼び出す
        /// - Then: 末尾のゼロが除去された系列と一致する
        #[test]
        fn removes_trailing_zero_coefficients() {
            // Given
            let mut sut = FPS {
                coeffs: vec![1, 0, 0],
            };
            // When
            sut.trim();
            // Then
            assert_eq!(FPS::new(vec![1]), sut);
        }
    }

    // fmt::Display のテスト: 戻り値そのものを検証する
    mod fmt {
        use super::*;

        /// Scenario: 非ゼロ項を "係数x^次数" の形で " + " 区切りに整形する
        /// - Given: 係数 [1, 0, 2] を持つ系列がある
        /// - When: 表示用の文字列に変換する
        /// - Then: "1x^0 + 2x^2" になる
        #[test]
        fn formats_non_zero_terms_joined_by_plus() {
            // Given
            let sut = FPS::new(vec![1, 0, 2]);
            // When
            let result = format!("{}", sut);
            // Then
            assert_eq!("1x^0 + 2x^2", result);
        }

        /// Scenario: ゼロ多項式は "0" と表示される
        /// - Given: ゼロ多項式がある
        /// - When: 表示用の文字列に変換する
        /// - Then: "0" になる
        #[test]
        fn formats_zero_polynomial_as_zero() {
            // Given
            let sut = FPS::new(Vec::new());
            // When
            let result = format!("{}", sut);
            // Then
            assert_eq!("0", result);
        }
    }

    /// Scenario: log, exp, inverse は FPS の公開 API 全体の契約として、 制約を満たさない入力を拒否する
    /// - Given: 定数項が 1 でない系列と、 定数項が 0 でない系列がある
    /// - When: それぞれの制約に違反する呼び出しを行う
    /// - Then: log と exp は None を返し、 inverse は制約を満たす呼び出しに対しては Some を返す
    #[test]
    fn error_is_raised_when_constraints_violate() {
        // Given
        let not_one = FPS::new(vec![2, 1]);
        let not_zero = FPS::new(vec![1]);

        // When
        let log_result = not_one.log(3);
        let exp_result = not_zero.exp(2);
        let inverse_result = not_zero.inverse(0);

        // Then
        assert!(log_result.is_none(), "log should reject constant != 1");
        assert!(exp_result.is_none(), "exp should reject constant != 0");
        assert!(
            inverse_result.is_some(),
            "inverse should allow degree 0 when constant is non-zero"
        );
    }
}
