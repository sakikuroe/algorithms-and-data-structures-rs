//! mod 998244353 における階乗、階乗逆元、逆元、二項係数を提供する。
//!
//! このモジュールは、`Combinatorics` を通じて次を提供する。
//! - `fact(n) = n!`
//! - `inv_fact(n) = (n!)^-1`
//! - `inv(n) = n^-1` (ただし `n % 998244353 == 0` の場合は存在しない)
//! - `comb(n, k) = C(n, k)`
//! - `perm(n, k) = P(n, k)`
//!
//! テーブルは遅延構築する。`n <= MAX_TABLE_N` の範囲では、
//! `BLOCK_SIZE` ごとに切り上げた `0..=m` の範囲を一括で構築する。
//!
//! `0..=m` のテーブルを伸ばすときは、`m!` を順に計算し、
//! `inv(m!)` を 1 回だけ計算する。その後、
//! `inv((m-1)!)..inv(0!)` を逆向きに埋めることで、
//! 各要素ごとに `pow` を呼ばずに `inv_fact` を構築できる。
//!
//! `n > MAX_TABLE_N` の場合はメモリ節約のためテーブルを拡張せず、愚直に計算する。

use super::modint;
use super::modulo;

/// 事前計算テーブルで扱う最大の `n` です。
///
/// `n > MAX_TABLE_N` のクエリーは、テーブルを拡張せずに逐次計算へフォールバックする。
/// この定数を大きくすると最悪時のメモリ使用量が増えるため、初期値は控えめにしている。
const MAX_TABLE_N: usize = 1 << 25;

/// テーブルを拡張する際のブロックサイズです。
///
/// `n` を含むブロックの末尾 `(k+1) * BLOCK_SIZE - 1` までを一括で構築する。
/// この値を小さくすると拡張回数が増えるが、1 回の拡張で確保するメモリ量を抑えられる。
const BLOCK_SIZE: usize = 1 << 10;

/// mod 998244353 における階乗、階乗逆元、逆元、二項係数を提供する。
#[derive(Debug, Clone)]
pub struct Combinatorics {
    /// `fact[i] = i!` のテーブル。
    fact: Vec<modint::ModInt998244353>,
    /// `inv_fact[i] = (i!)^-1` のテーブル。
    inv_fact: Vec<modint::ModInt998244353>,
    /// `inv[i] = i^-1` のテーブル (`inv[0]` は未定義のため 0)。
    inv: Vec<modint::ModInt998244353>,
}

/// `Combinatorics` の各種計算メソッドを提供する。
impl Combinatorics {
    /// 最小限のテーブルを持つ `Combinatorics` を生成する。
    ///
    /// # Args
    /// 引数はない。
    ///
    /// # Returns
    /// `0!` と `inv_fact(0)` のみを持つ `Self` を返す。
    ///
    /// # Constraints
    /// 制約はない。
    ///
    /// # Panics
    /// パニックしない。
    ///
    /// # Complexity
    /// - 時間計算量: O(1)。
    /// - 空間計算量: O(1)。
    ///
    /// # Examples
    /// ```
    /// use anmitsu::modulo998244353;
    ///
    /// let mut comb = modulo998244353::combinatorics::Combinatorics::new();
    /// assert_eq!(1, comb.fact(0).val());
    /// ```
    pub fn new() -> Self {
        // 初期値として `0! = 1` と `inv_fact(0) = 1` を持たせる。
        let one = modint::ModInt998244353::new_raw(1);

        Combinatorics {
            fact: vec![one],
            inv_fact: vec![one],
            inv: vec![modint::ModInt998244353::default()],
        }
    }

    /// `n` を含むブロックまでテーブルを拡張する。
    ///
    /// # Args
    /// - `n` - テーブルで参照したい最大の添字。
    ///
    /// # Returns
    /// 戻り値はない。
    ///
    /// # Constraints
    /// - `n < modulo::M`。
    /// - `n <= MAX_TABLE_N`。
    /// - 上記を満たさない場合、この関数は何もしない。
    ///
    /// # Panics
    /// パニックしない。
    ///
    /// # Complexity
    /// - 時間計算量: O(m - t + log MOD)。
    ///   - `t` は拡張前の最大添字、`m` は拡張後の最大添字を表す。
    /// - 空間計算量: O(m)。
    ///
    /// # Examples
    /// ```
    /// use anmitsu::modulo998244353;
    ///
    /// let mut comb = modulo998244353::combinatorics::Combinatorics::new();
    /// let _ = comb.fact(1024);
    /// ```
    fn ensure_table(&mut self, n: usize) {
        if n >= modulo::M as usize {
            return;
        }
        // メモリ使用量を制御するため、ある閾値より大きい場合はテーブルを拡張しない。
        if n > MAX_TABLE_N {
            return;
        }

        // `modulo::M` は素数なので、`i < modulo::M` の範囲の `i!` は 0 にならない。
        // よって `inv_fact` を保持しても逆元が存在し続ける。
        let max_len = (MAX_TABLE_N + 1).min(modulo::M as usize);

        // `n` を含むブロックまでを一括で確保し、同一ブロック内の逐次呼び出しで
        // 再拡張が発生しないようにする。
        let mut target_len = (n / BLOCK_SIZE + 1) * BLOCK_SIZE;
        target_len = target_len.min(max_len);

        // すでに必要な範囲が構築済みなら何もしない。
        if self.fact.len() >= target_len {
            return;
        }

        // 既存の末尾から必要な範囲までだけを伸ばす。
        let old_len = self.fact.len();

        self.fact
            .resize(target_len, modint::ModInt998244353::default());
        for i in old_len..target_len {
            self.fact[i] = self.fact[i - 1] * i as u32;
        }

        self.inv_fact
            .resize(target_len, modint::ModInt998244353::default());
        let last = target_len - 1;
        // `inv_fact[last]` を 1 回だけ計算し、そこから逆向きに連鎖させる。
        self.inv_fact[last] = self.fact[last].inv().unwrap();

        // 新しく伸ばした区間だけ `inv_fact` を埋める。
        for i in (old_len + 1..=last).rev() {
            self.inv_fact[i - 1] = self.inv_fact[i] * i as u32;
        }

        self.inv
            .resize(target_len, modint::ModInt998244353::default());

        // `inv` は `fact` と `inv_fact` から計算できるため、伸長区間だけを更新する。
        for i in old_len..target_len {
            self.inv[i] = self.inv_fact[i] * self.fact[i - 1];
        }
    }

    /// テーブルを拡張せずに `n!` を計算する。
    ///
    /// # Args
    /// - `n` - 階乗の引数。
    ///
    /// # Returns
    /// `n! (mod 998244353)` を返す。
    ///
    /// # Constraints
    /// - `n < modulo::M`。
    ///
    /// # Panics
    /// パニックしない。
    ///
    /// # Complexity
    /// - 時間計算量: O(n - t)。
    ///   - `t` は `self.fact` が保持する最大添字を表す。
    /// - 空間計算量: O(1)。
    ///
    /// # Examples
    /// ```
    /// use anmitsu::modulo998244353;
    ///
    /// let mut comb = modulo998244353::combinatorics::Combinatorics::new();
    /// assert_eq!(120, comb.fact(5).val());
    /// ```
    fn fact_without_extending(&self, n: usize) -> modint::ModInt998244353 {
        if n == 0 {
            return modint::ModInt998244353::new_raw(1);
        }

        // テーブルを拡張しない場合でも、既存の末尾から乗算を再開する。
        let start = self.fact.len() - 1;
        let mut res = self.fact[start];

        for i in (start + 1)..=n {
            res *= i as u32;
        }
        res
    }

    /// `n! (mod 998244353)` を返す。
    ///
    /// # Args
    /// - `n` - 階乗の引数。
    ///
    /// # Returns
    /// `ModInt998244353` として `n!` を返す。
    ///
    /// # Constraints
    /// - `n >= modulo::M` の場合、`n! = 0` となる。
    /// - `n <= MAX_TABLE_N` の場合、テーブルを必要に応じて拡張する。
    /// - `n > MAX_TABLE_N` の場合、テーブルは拡張せず愚直に計算する。
    ///
    /// # Panics
    /// パニックしない。
    ///
    /// # Complexity
    /// - 時間計算量: O(1) (`n` がテーブル範囲内の場合)、それ以外は O(n - t)。
    ///   - `t` は現在のテーブル最大添字を表す。
    /// - 空間計算量: O(1) (計算部分のみ)、ただし拡張時は O(m)。
    ///
    /// # Examples
    /// ```
    /// use anmitsu::modulo998244353;
    ///
    /// let mut comb = modulo998244353::combinatorics::Combinatorics::new();
    /// assert_eq!(120, comb.fact(5).val());
    /// ```
    pub fn fact(&mut self, n: usize) -> modint::ModInt998244353 {
        // `n >= M` のとき `n!` は `M` を因数に持つため、常に 0 になる。
        if n >= modulo::M as usize {
            return modint::ModInt998244353::new_raw(0);
        }

        if n <= MAX_TABLE_N {
            // 参照頻度が高い範囲はテーブル化し、同一ブロック内での連続クエリーを高速化する。
            self.ensure_table(n);
            return self.fact[n];
        }

        // メモリ使用量を抑えるため、閾値より大きい場合はテーブル化しない。
        self.fact_without_extending(n)
    }

    /// `1 / n! (mod 998244353)` を返す。
    ///
    /// # Args
    /// - `n` - 階乗逆元の引数。
    ///
    /// # Returns
    /// `ModInt998244353` として `1 / n!` を返す。
    ///
    /// # Constraints
    /// - `n >= modulo::M` の場合、逆元が存在しないため 0 を返す。
    /// - `n <= MAX_TABLE_N` の場合、テーブルを必要に応じて拡張する。
    /// - `n > MAX_TABLE_N` の場合、`inv(fact(n))` を用いて愚直に計算する。
    ///
    /// # Panics
    /// パニックしない。
    ///
    /// # Complexity
    /// - 時間計算量: O(1) (`n` がテーブル範囲内の場合)、
    ///   それ以外は O(n - t + log MOD)。
    /// - 空間計算量: O(1) (計算部分のみ)、ただし拡張時は O(m)。
    ///
    /// # Examples
    /// ```
    /// use anmitsu::modulo998244353;
    ///
    /// let mut comb = modulo998244353::combinatorics::Combinatorics::new();
    /// let inv_fact_7 = comb.inv_fact(7);
    /// assert_eq!(1, (comb.fact(7) * inv_fact_7).val());
    /// ```
    pub fn inv_fact(&mut self, n: usize) -> modint::ModInt998244353 {
        // `n >= M` のとき `n! = 0` なので逆元は存在しない。
        // この実装では 0 を返して呼び出し側に委ねる。
        if n >= modulo::M as usize {
            return modint::ModInt998244353::new_raw(0);
        }

        if n <= MAX_TABLE_N {
            // `inv_fact` は `inv(m!)` の 1 回の計算から逆向きに埋められるため、
            // 連続クエリーで `pow` が多発しないようにテーブルを伸ばす。
            self.ensure_table(n);
            return self.inv_fact[n];
        }

        // 閾値より大きい場合は、`fact(n)` を愚直に計算してから 1 回だけ `inv` を呼ぶ。
        self.fact_without_extending(n).inv().unwrap()
    }

    /// `1 / n (mod 998244353)` を返す。
    ///
    /// # Args
    /// - `n` - 逆元の引数。
    ///
    /// # Returns
    /// - `n % 998244353 == 0` の場合は `None`。
    /// - それ以外は `Some(n^-1)`。
    ///
    /// # Constraints
    /// - `n % 998244353 != 0` のときに限り逆元が存在する。
    /// - `n % 998244353 <= MAX_TABLE_N` の場合、テーブルを必要に応じて拡張する。
    ///
    /// # Panics
    /// パニックしない。
    ///
    /// # Complexity
    /// - 時間計算量: O(1) (`n % 998244353` がテーブル範囲内の場合)、
    ///   それ以外は O(log MOD)。
    /// - 空間計算量: O(1) (計算部分のみ)、ただし拡張時は O(m)。
    ///
    /// # Examples
    /// ```
    /// use anmitsu::modulo998244353;
    ///
    /// let mut comb = modulo998244353::combinatorics::Combinatorics::new();
    /// let inv_two = comb.inv(2).unwrap();
    /// assert_eq!(1, (inv_two * 2_u32).val());
    /// assert!(comb.inv(0).is_none());
    /// ```
    pub fn inv(&mut self, n: usize) -> Option<modint::ModInt998244353> {
        // `n` の逆元は `n mod M` だけを見れば良い。
        let r = n % modulo::M as usize;
        if r == 0 {
            // `0` の逆元は存在しない。
            return None;
        }

        if r <= MAX_TABLE_N {
            // 小さい範囲はテーブルを用いて O(1) で返す。
            self.ensure_table(r);
            return Some(self.inv[r]);
        }

        // 閾値より大きい場合は、`pow` ベースの逆元計算にフォールバックする。
        Some(modint::ModInt998244353::new_raw(modulo::inv(r as u32)))
    }

    /// `C(n, k) (mod 998244353)` を返す。
    ///
    /// # Args
    /// - `n` - 上側の値。
    /// - `k` - 下側の値。
    ///
    /// # Returns
    /// 二項係数 `C(n, k)` を `ModInt998244353` として返す。
    ///
    /// # Constraints
    /// - `k < modulo::M`。
    /// - `n0 = n % modulo::M` とする。
    /// - `k < modulo::M` のとき、`C(n, k) = C(n0, k) (mod modulo::M)` が成り立つ。
    /// - `n0 < k` の場合、0 を返す。
    /// - それ以外は `fact(n0) * inv_fact(k) * inv_fact(n0 - k)` で計算する。
    ///
    /// # Panics
    /// - `k >= modulo::M` のときパニックする。
    ///
    /// # Complexity
    /// - 時間計算量: O(1) (テーブル利用時)、それ以外は O(k + log MOD)。
    /// - 空間計算量: O(1)。
    ///
    /// # Examples
    /// ```
    /// use anmitsu::modulo998244353;
    ///
    /// let mut comb = modulo998244353::combinatorics::Combinatorics::new();
    /// assert_eq!(10, comb.comb(5, 2).val());
    /// ```
    pub fn comb(&mut self, n: u64, k: usize) -> modint::ModInt998244353 {
        if k as u64 > n {
            // 定義により `k > n` のとき `C(n, k) = 0`。
            return modint::ModInt998244353::new_raw(0);
        }

        assert!(
            k < modulo::M as usize,
            "k must be less than MOD for comb without Lucas"
        );

        let n0 = (n % modulo::M as u64) as usize;
        if n0 < k {
            return modint::ModInt998244353::new_raw(0);
        }

        self.fact(n0) * self.inv_fact(k) * self.inv_fact(n0 - k)
    }

    /// `P(n, k) (mod 998244353)` を返す。
    ///
    /// # Args
    /// - `n` - 上側の値。
    /// - `k` - 下側の値。
    ///
    /// # Returns
    /// 順列 `P(n, k)` を `ModInt998244353` として返す。
    ///
    /// # Constraints
    /// - `k < modulo::M`。
    /// - `n <= MAX_TABLE_N` かつ `n < modulo::M` の場合、テーブルを用いて O(1) で計算する。
    /// - それ以外は、`k` 個の積で愚直に計算する。
    ///
    /// # Panics
    /// - `k >= modulo::M` のときパニックする。
    ///
    /// # Complexity
    /// - 時間計算量: O(1) (テーブル利用時)、それ以外は O(k)。
    /// - 空間計算量: O(1)。
    ///
    /// # Examples
    /// ```
    /// use anmitsu::modulo998244353;
    ///
    /// let mut comb = modulo998244353::combinatorics::Combinatorics::new();
    /// assert_eq!(60, comb.perm(5, 3).val());
    /// ```
    pub fn perm(&mut self, n: u64, k: usize) -> modint::ModInt998244353 {
        if k as u64 > n {
            // 定義により `k > n` のとき `P(n, k) = 0`。
            return modint::ModInt998244353::new_raw(0);
        }

        assert!(
            k < modulo::M as usize,
            "k must be less than MOD for perm without Lucas"
        );

        if k == 0 {
            return modint::ModInt998244353::new_raw(1);
        }

        if n < modulo::M as u64 && n as usize <= MAX_TABLE_N {
            // テーブル範囲内では、`n! / (n-k)!` を O(1) で計算できる。
            let nu = n as usize;
            self.ensure_table(nu);
            return self.fact[nu] * self.inv_fact[nu - k];
        }

        // `n` がテーブル範囲外のときは、`n (n-1) .. (n-k+1)` の積で計算する。
        let mut res = modint::ModInt998244353::new_raw(1);
        for i in 0..k {
            res *= modint::ModInt998244353::new(n - i as u64);
        }
        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Background: 最小限のテーブルのみを持つ `Combinatorics`。
    fn create_combinatorics() -> Combinatorics {
        Combinatorics::new()
    }

    // fact のテスト: 戻り値を検証する。
    mod fact {
        use super::*;

        /// Scenario: 典型的な値に対して階乗を返す。
        /// - Given: 新規に生成した `Combinatorics` がある。
        /// - When: `fact(5)` を呼ぶ。
        /// - Then: `120` (`5!`) が返る。
        #[test]
        fn returns_factorial_for_typical_value() {
            // Given
            let mut sut = create_combinatorics();
            // When
            let result = sut.fact(5);
            // Then
            assert_eq!(120, result.val());
        }

        /// Scenario: `n = 0` の階乗は `1` になる (境界値)。
        /// - Given: 新規に生成した `Combinatorics` がある。
        /// - When: `fact(0)` を呼ぶ。
        /// - Then: `1` (`0!`) が返る。
        #[test]
        fn returns_one_for_zero() {
            // Given
            let mut sut = create_combinatorics();
            // When
            let result = sut.fact(0);
            // Then
            assert_eq!(1, result.val());
        }

        /// Scenario: `n >= modulo::M` のとき `n!` は `0` になる (境界値)。
        /// - Given: 新規に生成した `Combinatorics` がある。
        /// - When: `fact(modulo::M as usize)` を呼ぶ。
        /// - Then: `0` が返る。
        #[test]
        fn returns_zero_for_n_at_least_modulus() {
            // Given
            let mut sut = create_combinatorics();
            // When
            let result = sut.fact(modulo::M as usize);
            // Then
            assert_eq!(0, result.val());
        }
    }

    // inv_fact のテスト: 戻り値、 および fact との整合性を検証する。
    mod inv_fact {
        use super::*;

        /// Scenario: `inv_fact(n)` は `fact(n)` の乗法逆元になる。
        /// - Given: 新規に生成した `Combinatorics` がある。
        /// - When: `fact(5)` と `inv_fact(5)` を求める。
        /// - Then: それらの積は `1` になる。
        #[test]
        fn is_multiplicative_inverse_of_fact() {
            // Given
            let mut sut = create_combinatorics();
            // When
            let fact_5 = sut.fact(5);
            let inv_fact_5 = sut.inv_fact(5);
            // Then
            assert_eq!(1, (fact_5 * inv_fact_5).val());
        }

        /// Scenario: 連続したクエリーに対しても、 テーブルの遅延拡張を挟みながら
        /// `fact(n) * inv_fact(n) = 1` が成り立ち続ける。
        /// - Given: 新規に生成した `Combinatorics` がある。
        /// - When: `n = 0..=2000` の範囲で `fact(n)` と `inv_fact(n)` を順に求める。
        /// - Then: すべての `n` について積が `1` になる。
        #[test]
        fn holds_for_many_sequential_queries_without_repeated_pow() {
            // Given
            let mut sut = create_combinatorics();
            // When, Then
            for n in 0..=2000 {
                let fact_n = sut.fact(n);
                let inv_fact_n = sut.inv_fact(n);
                assert_eq!(1, (fact_n * inv_fact_n).val());
            }
        }

        /// Scenario: `n >= modulo::M` のとき `inv_fact(n)` は `0` になる (境界値)。
        /// - Given: 新規に生成した `Combinatorics` がある。
        /// - When: `inv_fact(modulo::M as usize)` を呼ぶ。
        /// - Then: `0` が返る。
        #[test]
        fn returns_zero_for_n_at_least_modulus() {
            // Given
            let mut sut = create_combinatorics();
            // When
            let result = sut.inv_fact(modulo::M as usize);
            // Then
            assert_eq!(0, result.val());
        }
    }

    // inv のテスト: 戻り値を検証する。
    mod inv {
        use super::*;

        /// Scenario: 典型的な値に対して乗法逆元を返す。
        /// - Given: 新規に生成した `Combinatorics` がある。
        /// - When: `inv(2)` を呼ぶ。
        /// - Then: `Some` が返り、 その値と `2` の積は `1` になる。
        #[test]
        fn returns_multiplicative_inverse_for_typical_value() {
            // Given
            let mut sut = create_combinatorics();
            // When
            let result = sut.inv(2);
            // Then
            assert!(result.is_some());
            assert_eq!(1, (result.unwrap() * 2_u32).val());
        }

        /// Scenario: `0` には逆元が存在しないため `None` を返す (境界値)。
        /// - Given: 新規に生成した `Combinatorics` がある。
        /// - When: `inv(0)` を呼ぶ。
        /// - Then: `None` が返る。
        #[test]
        fn returns_none_for_zero() {
            // Given
            let mut sut = create_combinatorics();
            // When
            let result = sut.inv(0);
            // Then
            assert!(result.is_none());
        }

        /// Scenario: `modulo::M` の倍数には逆元が存在しないため `None` を返す (境界値)。
        /// - Given: 新規に生成した `Combinatorics` がある。
        /// - When: `inv(modulo::M as usize)` を呼ぶ。
        /// - Then: `None` が返る。
        #[test]
        fn returns_none_for_multiple_of_modulus() {
            // Given
            let mut sut = create_combinatorics();
            // When
            let result = sut.inv(modulo::M as usize);
            // Then
            assert!(result.is_none());
        }
    }

    // comb のテスト: 戻り値を検証する。
    mod comb {
        use super::*;

        /// Scenario: 典型的な `(n, k)` の組み合わせに対して二項係数を返す。
        /// - Given: 新規に生成した `Combinatorics` がある。
        /// - When: 複数の `(n, k)` の組で `comb(n, k)` を求める。
        /// - Then: 各ケースで期待する二項係数が返る。
        #[test]
        fn matches_expected_values_for_typical_inputs() {
            // Given
            let mut sut = create_combinatorics();
            let cases = [
                (5_u64, 2_usize, 10_u32),
                (10, 0, 1),
                (10, 10, 1),
                (10, 3, 120),
                (10, 7, 120),
            ];
            // When, Then
            for (n, k, expected) in cases {
                let result = sut.comb(n, k);
                assert_eq!(expected, result.val());
            }
        }

        /// Scenario: `k > n` のとき、 定義により `0` を返す (境界値)。
        /// - Given: 新規に生成した `Combinatorics` がある。
        /// - When: `comb(3, 5)` を呼ぶ。
        /// - Then: `0` が返る。
        #[test]
        fn returns_zero_when_k_greater_than_n() {
            // Given
            let mut sut = create_combinatorics();
            // When
            let result = sut.comb(3, 5);
            // Then
            assert_eq!(0, result.val());
        }

        /// Scenario: `n` が `modulo::M` 以上でも、 `n mod modulo::M` を用いて計算する。
        /// - Given: 新規に生成した `Combinatorics` と、 `modulo::M` より少し大きい `n` がある。
        /// - When: `comb(n, k)` を求める。
        /// - Then: 定義通りに `n (n-1) .. (n-k+1) / k!` を計算した結果と一致する。
        #[test]
        fn matches_naive_computation_for_large_n_small_k() {
            // Given
            let mut sut = create_combinatorics();
            let n = modulo::M as u64 + 5;
            let k = 3_usize;
            let mut num = modint::ModInt998244353::new_raw(1);
            for i in 0..k {
                num *= modint::ModInt998244353::new(n - i as u64);
            }
            let mut den = modint::ModInt998244353::new_raw(1);
            for i in 1..=k {
                den *= i as u32;
            }
            let expected = num * den.inv().unwrap();
            // When
            let result = sut.comb(n, k);
            // Then
            assert_eq!(expected, result);
        }

        /// Scenario: `k >= modulo::M` のときはサポート対象外としてパニックする (異常系)。
        /// - Given: 新規に生成した `Combinatorics` と、 `k >= n` となる `n`, `k = modulo::M` がある。
        /// - When: `comb(n, k)` を呼ぶ。
        /// - Then: パニックする。
        #[test]
        #[should_panic(expected = "k must be less than MOD")]
        fn panics_when_k_at_least_modulus() {
            // Given
            let mut sut = create_combinatorics();
            let n = modulo::M as u64;
            let k = modulo::M as usize;
            // When, Then (panic)
            let _ = sut.comb(n, k);
        }
    }

    // perm のテスト: 戻り値を検証する。
    mod perm {
        use super::*;

        /// Scenario: 典型的な `(n, k)` の組み合わせに対して順列数を返す。
        /// - Given: 新規に生成した `Combinatorics` がある。
        /// - When: 複数の `(n, k)` の組で `perm(n, k)` を求める。
        /// - Then: 各ケースで期待する順列数が返る。
        #[test]
        fn matches_expected_values_for_typical_inputs() {
            // Given
            let mut sut = create_combinatorics();
            let cases = [(5_u64, 3_usize, 60_u32), (5, 0, 1)];
            // When, Then
            for (n, k, expected) in cases {
                let result = sut.perm(n, k);
                assert_eq!(expected, result.val());
            }
        }

        /// Scenario: `k > n` のとき、 定義により `0` を返す (境界値)。
        /// - Given: 新規に生成した `Combinatorics` がある。
        /// - When: `perm(5, 6)` を呼ぶ。
        /// - Then: `0` が返る。
        #[test]
        fn returns_zero_when_k_greater_than_n() {
            // Given
            let mut sut = create_combinatorics();
            // When
            let result = sut.perm(5, 6);
            // Then
            assert_eq!(0, result.val());
        }

        /// Scenario: `k >= modulo::M` のときはサポート対象外としてパニックする (異常系)。
        /// - Given: 新規に生成した `Combinatorics` と、 `k >= n` となる `n`, `k = modulo::M` がある。
        /// - When: `perm(n, k)` を呼ぶ。
        /// - Then: パニックする。
        #[test]
        #[should_panic(expected = "k must be less than MOD")]
        fn panics_when_k_at_least_modulus() {
            // Given
            let mut sut = create_combinatorics();
            let n = modulo::M as u64;
            let k = modulo::M as usize;
            // When, Then (panic)
            let _ = sut.perm(n, k);
        }
    }

    // テーブルの遅延拡張に関するテスト: ブロック境界をまたいでも整合性が保たれることを検証する。
    mod table_extension {
        use super::*;

        /// Scenario: `BLOCK_SIZE` の境界をまたいでテーブルを拡張しても、
        /// `fact`, `inv_fact`, `inv` の値の整合性が保たれる (境界値)。
        /// - Given: 新規に生成した `Combinatorics` がある。
        /// - When: ブロック境界の直前 (`1023`) と直後 (`1024`) の両方で
        ///   `fact`, `inv_fact`, `inv` を求める。
        /// - Then: 各値が定義通りの関係を満たす。
        #[test]
        fn keeps_consistency_across_block_boundary() {
            // Given
            let mut sut = create_combinatorics();
            // When
            let fact_1023 = sut.fact(1023);
            let inv_fact_1023 = sut.inv_fact(1023);
            let fact_1024 = sut.fact(1024);
            let inv_fact_1024 = sut.inv_fact(1024);
            let inv_1024 = sut.inv(1024).unwrap();
            // Then
            assert_eq!(1, (fact_1023 * inv_fact_1023).val());
            assert_eq!(fact_1024, fact_1023 * 1024_u32);
            assert_eq!(1, (fact_1024 * inv_fact_1024).val());
            assert_eq!(1, (inv_1024 * 1024_u32).val());
        }
    }
}
