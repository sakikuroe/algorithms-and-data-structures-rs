//! 法 998244353 上の形式的べき級数を扱うモジュールである.
mod add;
pub mod bostan_mori;
mod div;
pub mod exp;
pub mod inv;
pub mod log;
mod mul;
pub mod partition;
pub mod pow;
mod sub;

use super::convolution;
use super::modulo;
use std::fmt;
use std::sync;

/// `FPS::integral` で用いる階乗と階乗逆元のテーブルである.
struct FactorialTables {
    fact: Vec<u32>,
    ifact: Vec<u32>,
}

/// `FPS::integral` で用いる階乗テーブルを 1 度だけ構築して保持する.
static FACTORIAL_TABLES: sync::OnceLock<FactorialTables> = sync::OnceLock::new();

/// 階乗と階乗逆元のテーブルへの参照を返す.
///
/// # Args
/// 引数はない.
///
/// # Returns
/// `&'static FactorialTables`: `0..=MAX_NTT_LEN` の階乗テーブルと逆階乗テーブル.
///
/// # Constraints
/// - `convolution::MAX_NTT_LEN < 998244353` を満たす.
///
/// # Panics
/// - この関数はパニックしない.
///
/// # Complexity
/// - Time complexity: 初回のみ O(MAX_NTT_LEN), 2 回目以降は O(1).
/// - Space complexity: O(MAX_NTT_LEN).
///
/// # Examples
/// ```rust,ignore
/// // `FPS::integral` の内部で利用する.
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

/// 法 998244353 上の形式的べき級数を表す. 末尾のゼロ係数は正規形として削除する.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FPS {
    coeffs: Vec<u32>,
}

impl FPS {
    /// 形式的べき級数が保持できる項数の上限を検証する.
    ///
    /// # Args
    /// - `len`: 検証する項数.
    ///
    /// # Returns
    /// `()`: 検証するだけで値は返さない.
    ///
    /// # Constraints
    /// - 末尾のゼロ係数を除去した後の項数は `MAX_NTT_LEN` 以下でなければならない.
    ///
    /// # Panics
    /// - `len > MAX_NTT_LEN` のときパニックする.
    ///
    /// # Complexity
    /// - Time complexity: O(1).
    /// - Space complexity: O(1).
    ///
    /// # Examples
    /// ```rust,ignore
    /// // `FPS::new` などから呼び出される.
    /// ```
    fn assert_len_within_max_ntt_len(len: usize) {
        assert!(
            len <= convolution::MAX_NTT_LEN,
            "FPS length must not exceed MAX_NTT_LEN"
        );
    }

    /// 係数列から形式的べき級数を生成する.
    ///
    /// # Args
    /// - `coefficients`: 次数が小さい順に並んだ係数列.
    ///
    /// # Returns
    /// `Self`: 末尾のゼロ項を除去した形式的べき級数.
    ///
    /// # Constraints
    /// 制約はない.
    ///
    /// # Panics
    /// - 係数が法 998244353 以上のとき.
    /// - 末尾のゼロ係数を除去した後の項数が `MAX_NTT_LEN` を超えるとき.
    ///
    /// # Complexity
    /// - Time complexity: O(N). ここで N は `coefficients.len()`.
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

    /// 保持している項数 (最高次 + 1) を返す.
    ///
    /// # Args
    /// 引数はない.
    ///
    /// # Returns
    /// `usize`: ゼロを取り除いた後の項数 (最高次数 + 1).
    ///
    /// # Constraints
    /// - `value != 0` のとき, `index + 1` は `MAX_NTT_LEN` 以下でなければならない.
    ///
    /// # Panics
    /// - この関数はパニックしない.
    ///
    /// # Complexity
    /// - Time complexity: O(1).
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

    /// 非ゼロ多項式なら最高次数を返す.
    ///
    /// # Args
    /// 引数はない.
    ///
    /// # Returns
    /// `Option<usize>`: 非ゼロ多項式では `Some(degree)`, ゼロでは `None`.
    ///
    /// # Constraints
    /// 制約はない.
    ///
    /// # Panics
    /// - この関数はパニックしない.
    ///
    /// # Complexity
    /// - Time complexity: O(1).
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

    /// ゼロ多項式のときに `true` を返す.
    ///
    /// # Args
    /// 引数はない.
    ///
    /// # Returns
    /// `bool`: すべての係数がゼロかどうか.
    ///
    /// # Constraints
    /// 制約はない.
    ///
    /// # Panics
    /// - この関数はパニックしない.
    ///
    /// # Complexity
    /// - Time complexity: O(1).
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

    /// `x^index` の係数を返す. 対応する項が無い場合は 0 を返す.
    ///
    /// # Args
    /// - `index`: 取り出す次数.
    ///
    /// # Returns
    /// `u32`: 指定した次数の係数 (法 998244353 での値).
    ///
    /// # Constraints
    /// 制約はない.
    ///
    /// # Panics
    /// - この関数はパニックしない.
    ///
    /// # Complexity
    /// - Time complexity: O(1).
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

    /// 必要に応じて拡張し, `x^index` の係数を設定する.
    ///
    /// # Args
    /// - `index`: 上書きする次数.
    /// - `value`: 設定する新しい係数.
    ///
    /// # Returns
    /// `()`: インプレースで更新する.
    ///
    /// # Constraints
    /// 制約はない.
    ///
    /// # Panics
    /// - `value >= 998244353` のとき.
    /// - `value != 0` かつ `index + 1 > MAX_NTT_LEN` のとき.
    ///
    /// # Complexity
    /// - Time complexity: O(N). N は `index` になりうる.
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

    /// 係数スライスへのイミュータブル参照を返す.
    ///
    /// # Args
    /// 引数はない.
    ///
    /// # Returns
    /// `&[u32]`: 係数への参照.
    ///
    /// # Constraints
    /// 制約はない.
    ///
    /// # Panics
    /// - この関数はパニックしない.
    ///
    /// # Complexity
    /// - Time complexity: O(1).
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

    /// 非ゼロ係数を持つ項を列挙するイテレーターを返す.
    ///
    /// # Args
    /// 引数はない.
    ///
    /// # Returns
    /// `impl Iterator<Item = (usize, u32)> + '_`: `(次数, 係数)` のタプル列を返す.
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

    /// 非ゼロ係数を持つ項を列挙する.
    ///
    /// # Args
    /// 引数はない.
    ///
    /// # Returns
    /// `Vec<(usize, u32)>`: `(次数, 係数)` のタプル列を返す.
    ///
    /// # Constraints
    /// 制約はない.
    ///
    /// # Panics
    /// - この関数はパニックしない.
    ///
    /// # Complexity
    /// - Time complexity: O(N). ここで N は `self.len()`.
    /// - Space complexity: O(K). ここで K は非ゼロ項の個数.
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

    /// 指定した項数に切り詰める.
    ///
    /// # Args
    /// - `len`: 新しい項数 (最高次数 + 1).
    ///
    /// # Returns
    /// `()`: インプレースで更新する.
    ///
    /// # Constraints
    /// 制約はない.
    ///
    /// # Panics
    /// - この関数はパニックしない.
    ///
    /// # Complexity
    /// - Time complexity: O(N). N は `self.len()`.
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

    // `mul_xk` と `div_xk` はそれぞれ `mul.rs` と `div.rs` に定義する.

    /// 形式的微分をインプレースで計算する.
    ///
    /// # Args
    /// 引数はない.
    ///
    /// # Returns
    /// `&mut Self`: 微分後の系列への参照.
    ///
    /// # Constraints
    /// 制約はない.
    ///
    /// # Panics
    /// - この関数はパニックしない.
    ///
    /// # Complexity
    /// - Time complexity: O(N).
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

    /// 定数項を 0 とする形式的積分をインプレースで計算する.
    ///
    /// # Args
    /// 引数はない.
    ///
    /// # Returns
    /// `&mut Self`: 定数項を 0 とした積分系列への参照.
    ///
    /// # Constraints
    /// - 各分母 (1, 2, ...) が 998244353 上で逆元を持つ.
    /// - `self.len() + 1` は法より小さくなければならない.
    /// - `self.len() + 1` は `MAX_NTT_LEN` 以下でなければならない.
    ///
    /// # Panics
    /// - `self.len() + 1 >= 998244353` のときパニックする.
    /// - `self.len() + 1 > MAX_NTT_LEN` のときパニックする.
    ///
    /// # Complexity
    /// - Time complexity: O(N).
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

    /// 末尾のゼロ係数を除去して正規形にする.
    ///
    /// # Args
    /// - `self`: 正規化対象となる系列.
    ///
    /// # Returns
    /// `()`: 自身をインプレースで更新する.
    ///
    /// # Constraints
    /// 制約はない.
    ///
    /// # Panics
    /// - 項数が `MAX_NTT_LEN` を超えるとき.
    ///
    /// # Complexity
    /// - Time complexity: O(N). N は末尾のゼロの個数である.
    /// - Space complexity: O(1).
    ///
    /// # Examples
    /// ```rust,ignore
    /// // `FPS::new` などから呼び出される内部関数である.
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
    /// 表示用の文字列表現を生成する.
    ///
    /// # Args
    /// - `self`: 出力対象の系列.
    /// - `f`: 出力先フォーマッタ.
    ///
    /// # Returns
    /// `fmt::Result`: フォーマット結果.
    ///
    /// # Constraints
    /// 制約はない.
    ///
    /// # Panics
    /// - フォーマット先がエラーを返したとき.
    ///
    /// # Complexity
    /// - Time complexity: O(N). N は項数.
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
