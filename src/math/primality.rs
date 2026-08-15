//! 素数判定・素因数分解に関連する機能を提供するモジュールである。

use crate::math::modular_arithmetic;
use crate::math::number_theory;
use std::collections::HashMap;

/// 決定的 Miller-Rabin 法により、`n` が素数かどうかを判定する。
///
/// # Args
/// - `n` - 判定対象の非負整数
///
/// # Returns
/// `n` が素数であれば `true`、そうでなければ `false`。
///
/// # Complexity
/// - 時間計算量: $O(\log^2 n)$
///   - 基底 `[2, 325, 9375, 28178, 450775, 9780504, 1795265022]` の 7 個それぞれについて
///     $O(\log n)$ 回のモジュラー乗算を行う。この基底の組み合わせは `u64` の全域で
///     決定的に正しい結果を返すことが知られている。
///
/// # Examples
/// ```
/// use anmitsu::math::primality;
///
/// assert!(!primality::is_prime(0));
/// assert!(!primality::is_prime(1));
/// assert!(primality::is_prime(2));
/// assert!(!primality::is_prime(4));
/// assert!(primality::is_prime(998244353));
/// ```
#[must_use]
pub fn is_prime(n: u64) -> bool {
    if n == 0 || n == 1 {
        return false;
    }
    if n == 2 {
        return true;
    }
    if n % 2 == 0 {
        return false;
    }

    // n - 1 = 2^s * d (d は奇数) と分解する。
    let s = (n - 1).trailing_zeros();
    let d = (n - 1) >> s;

    // 基底 a が n の合成数性の証拠にならない (= n が合成数であってもこの a だけでは
    // 見抜けない) 場合に true を返す。a^d ≡ 1 または a^(2^r * d) ≡ -1 (mod n) と
    // なる 0 <= r < s が存在するなら、a はフェルマーテストを ("合成数の証拠なし" の
    // 意味で) 通過する。
    let passes_test = |a: u64| -> bool {
        let a = a % n;
        if a == 0 {
            return true;
        }

        let mut x = modular_arithmetic::pow_mod(a, d, n);
        if x == 1 || x == n - 1 {
            return true;
        }

        for _ in 1..s {
            x = modular_arithmetic::mul_mod(x, x, n);
            if x == n - 1 {
                return true;
            }
        }

        false
    };

    [2, 325, 9375, 28178, 450775, 9780504, 1795265022]
        .into_iter()
        .all(passes_test)
}

/// 試し割り法によって、閾値未満の合成数 `n` を素因数分解する。
fn factorize_by_trial_division(mut n: u64) -> HashMap<u64, usize> {
    let mut result = HashMap::new();

    let mut p = 2;
    while p * p <= n {
        while n % p == 0 {
            *result.entry(p).or_insert(0) += 1;
            n /= p;
        }
        p += 1;
    }
    if n > 1 {
        *result.entry(n).or_insert(0) += 1;
    }

    result
}

/// Pollard's rho 法 (Floyd の閉路検出) により、合成数 `n` の非自明な約数を 1 つ見つける。
fn find_divisor(n: u64) -> u64 {
    if n % 2 == 0 {
        return 2;
    }

    // c を変えながら繰り返す。1 つの c で閉路検出が n 自身に退化した (d == n) 場合は
    // 別の c で仕切り直す。
    for c in 1.. {
        let f = |x: u64| -> u64 {
            modular_arithmetic::add_mod(modular_arithmetic::mul_mod(x, x, n), c % n, n)
        };

        let mut x = 2;
        let mut y = 2;
        let mut d = 1;

        // x は f を 1 回、y は f を 2 回適用しながら進める (Floyd の閉路検出)。
        // gcd(|x - y|, n) が 1 でない非自明な値になった時点で約数が見つかる。
        while d == 1 {
            x = f(x);
            y = f(f(y));
            d = number_theory::gcd(x.abs_diff(y) as u128, n as u128) as u64;
        }

        if d != n {
            return d;
        }
    }

    unreachable!()
}

/// `n` を素因数分解する。
///
/// # Args
/// - `n` - 素因数分解の対象となる非負整数
///
/// # Returns
/// `n` の素因数分解を、素因数からその指数への `HashMap<u64, usize>` として返す。
/// 具体的には:
/// - `n = 0` または `n = 1` の場合、空の `HashMap` を返す。
/// - `n >= 2` の場合、`n == product(p.pow(e) for (p, e) in result)` を満たす。
///
/// # Complexity
/// - 時間計算量: 期待値 $O(n^{1/4} \log n)$
///   - `n` が試し割りで素早く分解できる程度に小さい場合は $O(\sqrt{n})$ の試し割り法を、
///     そうでない場合は [`is_prime`] と Pollard's rho 法を組み合わせて分解する。
///
/// # Examples
/// ```
/// use anmitsu::math::primality;
/// use std::collections::HashMap;
///
/// assert_eq!(HashMap::new(), primality::factorize(1));
/// assert_eq!(HashMap::from([(2, 2), (3, 1)]), primality::factorize(12));
/// assert_eq!(HashMap::from([(17, 1)]), primality::factorize(17));
/// ```
#[must_use]
pub fn factorize(n: u64) -> HashMap<u64, usize> {
    // 試し割り法が O(sqrt(n)) の時間で十分実用的に収まる閾値である。
    const TRIAL_DIVISION_THRESHOLD: u64 = 1_000_000_000;

    if n == 0 || n == 1 {
        return HashMap::new();
    }
    if is_prime(n) {
        return HashMap::from([(n, 1)]);
    }
    if n < TRIAL_DIVISION_THRESHOLD {
        return factorize_by_trial_division(n);
    }

    let mut result = HashMap::new();
    let mut composites = vec![n];

    while let Some(m) = composites.pop() {
        let d = find_divisor(m);
        for divisor in [d, m / d] {
            if is_prime(divisor) {
                *result.entry(divisor).or_insert(0) += 1;
            } else {
                composites.push(divisor);
            }
        }
    }

    result
}

/// `n` の正の約数を昇順に列挙する。
///
/// # Args
/// - `n` - 約数を求める対象の整数であり、`0` より大きい必要がある。
///
/// # Returns
/// `n` の約数を昇順に格納した `Vec<u64>`。
///
/// # Complexity
/// - 時間計算量: 期待値 $O(n^{1/4} \log n + d(n))$
///   - 内部で呼び出す [`factorize`] の計算量が支配的であり、素因数分解の結果から
///     約数を列挙する処理は約数の個数 $d(n)$ に比例した時間で行える。
/// - 空間計算量: $O(d(n))$
///   - 列挙された約数を保持するために、約数の個数に比例したメモリを使用する。
///
/// # Examples
/// ```
/// use anmitsu::math::primality;
///
/// assert_eq!(vec![1], primality::divisors(1));
/// assert_eq!(vec![1, 7], primality::divisors(7));
/// assert_eq!(vec![1, 2, 3, 4, 6, 12], primality::divisors(12));
/// ```
#[must_use]
pub fn divisors(n: u64) -> Vec<u64> {
    debug_assert!(n >= 1);

    // 約数の列挙結果を蓄積するためのベクタ。初期状態では、どんな n に対しても
    // 約数である 1 のみを持つ。
    let mut result = vec![1_u64];

    // 素因数 p とその指数 e ごとに、それまでに求めた約数それぞれに
    // p^0, p^1, ..., p^e を掛け合わせた値を、新たな約数集合として構築していく。
    for (p, e) in factorize(n) {
        let mut next = Vec::with_capacity(result.len() * (e + 1));
        let mut power = 1_u64;
        for _ in 0..=e {
            for &d in &result {
                next.push(d * power);
            }
            power *= p;
        }
        result = next;
    }

    result.sort_unstable();
    result
}

/// オイラーの $\varphi$ 関数の値を計算する。
///
/// # Args
/// - `n` - 計算対象の整数であり、`0` より大きい必要がある。
///
/// # Returns
/// `1` から `n` までの整数のうち、`n` と互いに素であるものの個数。
///
/// # Complexity
/// - 時間計算量: 期待値 $O(n^{1/4} \log n)$
///   - 内部で呼び出す [`factorize`] の計算量が支配的である。
/// - 空間計算量: $O(\log n)$
///   - `n` が持つ相異なる素因数の個数に比例したメモリを使用する。
///
/// # Examples
/// ```
/// use anmitsu::math::primality;
///
/// assert_eq!(1, primality::euler_phi(1));
/// assert_eq!(6, primality::euler_phi(9));
/// assert_eq!(16, primality::euler_phi(17));
/// ```
#[must_use]
pub fn euler_phi(n: u64) -> u64 {
    debug_assert!(n >= 1);

    let mut result = n;
    for p in factorize(n).into_keys() {
        // 先に n を p で割ってから (p - 1) を掛けることで、掛け算を先に行う場合に
        // 生じうる u64 のオーバーフローを避ける。result が p で割り切れることは、
        // p が factorize(n) の素因数であることから数学的に保証されている。
        result = result / p * (p - 1);
    }
    result
}

/// `p` を法として `a` が原始根であるかどうかを判定する。
///
/// # Args
/// - `a` - 判定対象の整数であり、`0 <= a < p` を満たす必要がある。
/// - `p` - 法であり、素数である必要がある。この契約に違反する場合、
///   `debug_assert!` によりパニックする。
///
/// # Returns
/// `a` が `p` を法とする原始根であれば `true`、そうでなければ `false`。
///
/// # Complexity
/// - 時間計算量: 期待値 $O(p^{1/4} \log p)$
///   - 内部で呼び出す [`factorize`] の計算量が支配的であり、`p - 1` の相異なる
///     素因数それぞれについて [`modular_arithmetic::pow_mod`] を1回ずつ呼び出す。
///
/// # Examples
/// ```
/// use anmitsu::math::primality;
///
/// assert!(primality::is_primitive_root(3, 7));
/// assert!(!primality::is_primitive_root(2, 7));
/// ```
#[must_use]
pub fn is_primitive_root(a: u64, p: u64) -> bool {
    debug_assert!(is_prime(p));
    debug_assert!(a < p);

    // p = 2 の場合、乗法群 (Z/2Z)^* の位数は 1 であり、唯一の元である 1 が
    // 原始根となる。
    if p == 2 {
        return a == 1;
    }

    // a が原始根であることは、a の位数 (乗法群における周期) が p - 1 と一致する
    // ことと同値である。これはさらに、p - 1 の相異なる素因数 q それぞれについて
    // a^((p-1)/q) != 1 (mod p) が成り立つことと同値であるため、この条件を判定する。
    factorize(p - 1)
        .into_keys()
        .all(|q| modular_arithmetic::pow_mod(a, (p - 1) / q, p) != 1)
}

/// `p` を法とする原始根を1つ見つける。
///
/// # Args
/// - `p` - 法であり、素数である必要がある。この契約に違反する場合、
///   `debug_assert!` によりパニックする。
///
/// # Returns
/// `p` を法とする原始根の1つ。最小のものであるとは限らない。
///
/// # Complexity
/// - 時間計算量: 期待値 $O(p^{1/4} \log^2 p)$
///   - 最小の原始根は $O(\log \log p)$ 個程度の候補を試せば見つかることが経験的に
///     知られており、候補ごとに [`is_primitive_root`] による判定を行う。
///
/// # Examples
/// ```
/// use anmitsu::math::primality;
///
/// assert_eq!(1, primality::find_primitive_root(2));
/// assert!(primality::is_primitive_root(primality::find_primitive_root(7), 7));
/// ```
#[must_use]
pub fn find_primitive_root(p: u64) -> u64 {
    debug_assert!(is_prime(p));

    if p == 2 {
        return 1;
    }

    // 小さい候補から順に原始根であるかを判定し、最初に見つかったものを返す。
    for a in 2.. {
        if is_primitive_root(a, p) {
            return a;
        }
    }

    unreachable!()
}

#[cfg(test)]
mod tests {
    use super::*;

    // is_prime のテスト: 戻り値を検証する。
    mod is_prime {
        use super::*;

        /// Scenario: `0` と `1` は素数ではない (境界値)。
        /// - Given: `0` と `1` がある。
        /// - When: `is_prime` を呼ぶ。
        /// - Then: `false` が返る。
        #[test]
        fn returns_false_for_zero_and_one() {
            let cases = [0_u64, 1_u64];

            for n in cases {
                // Given, When
                let result = is_prime(n);
                // Then
                assert!(!result);
            }
        }

        /// Scenario: `2` は素数である (境界値)。
        /// - Given: `2` がある。
        /// - When: `is_prime` を呼ぶ。
        /// - Then: `true` が返る。
        #[test]
        fn returns_true_for_two() {
            // Given, When
            let result = is_prime(2);
            // Then
            assert!(result);
        }

        /// Scenario: 典型的な素数に対して `true` を返す。
        /// - Given: 典型的な素数がいくつかある。
        /// - When: `is_prime` を呼ぶ。
        /// - Then: `true` が返る。
        #[test]
        fn returns_true_for_typical_primes() {
            let cases = [3_u64, 7, 13, 97, 998244353];

            for n in cases {
                // Given, When
                let result = is_prime(n);
                // Then
                assert!(result);
            }
        }

        /// Scenario: 偶数の合成数に対して `false` を返す。
        /// - Given: `2` より大きい偶数がある。
        /// - When: `is_prime` を呼ぶ。
        /// - Then: `false` が返る。
        #[test]
        fn returns_false_for_even_composites() {
            let cases = [4_u64, 100, 998244352];

            for n in cases {
                // Given, When
                let result = is_prime(n);
                // Then
                assert!(!result);
            }
        }

        /// Scenario: フェルマーテストを誤って通過しやすいカーマイケル数に対して `false` を返す。
        /// - Given: カーマイケル数がいくつかある。
        /// - When: `is_prime` を呼ぶ。
        /// - Then: `false` が返る。
        #[test]
        fn returns_false_for_carmichael_numbers() {
            let cases = [561_u64, 1105, 1729, 2465, 41041];

            for n in cases {
                // Given, When
                let result = is_prime(n);
                // Then
                assert!(!result);
            }
        }

        /// Scenario: `u64` の範囲に収まる大きな素数・合成数でも正しく判定できる (境界値)。
        /// - Given: `u64` の範囲に収まる大きな素数と、その素数同士の積である合成数がある。
        /// - When: `is_prime` を呼ぶ。
        /// - Then: 期待した判定結果が返る。
        #[test]
        fn returns_correct_result_for_large_numbers() {
            // Given
            // 18446744073709551557 は u64::MAX 未満の最大の素数である。
            let large_prime = 18446744073709551557_u64;
            // 4295098369 = 65537 * 65537 は大きな平方数の合成数である。
            let large_composite = 4295098369_u64;

            // When
            let prime_result = is_prime(large_prime);
            let composite_result = is_prime(large_composite);

            // Then
            assert!(prime_result);
            assert!(!composite_result);
        }
    }

    // factorize のテスト: 戻り値を検証する。
    mod factorize {
        use super::*;

        /// Scenario: `0` と `1` は空の `HashMap` を返す (境界値)。
        /// - Given: `0` と `1` がある。
        /// - When: `factorize` を呼ぶ。
        /// - Then: 空の `HashMap` が返る。
        #[test]
        fn returns_empty_map_for_zero_and_one() {
            let cases = [0_u64, 1_u64];

            for n in cases {
                // Given, When
                let result = factorize(n);
                // Then
                assert_eq!(HashMap::new(), result);
            }
        }

        /// Scenario: 素数はそれ自身を指数 `1` として返す (境界値)。
        /// - Given: 素数がある。
        /// - When: `factorize` を呼ぶ。
        /// - Then: `{n: 1}` が返る。
        #[test]
        fn returns_itself_for_prime_numbers() {
            let cases = [2_u64, 17, 998244353];

            for n in cases {
                // Given, When
                let result = factorize(n);
                // Then
                assert_eq!(HashMap::from([(n, 1)]), result);
            }
        }

        /// Scenario: 典型的な合成数を試し割り法の範囲内で正しく素因数分解する。
        /// - Given: 複数の素因数からなる典型的な合成数がある。
        /// - When: `factorize` を呼ぶ。
        /// - Then: 期待した素因数分解が返る。
        #[test]
        fn returns_correct_factorization_for_typical_values() {
            let cases = [
                (12_u64, HashMap::from([(2, 2), (3, 1)])),
                (100, HashMap::from([(2, 2), (5, 2)])),
                (360, HashMap::from([(2, 3), (3, 2), (5, 1)])),
            ];

            for (n, expected) in cases {
                // Given, When
                let result = factorize(n);
                // Then
                assert_eq!(expected, result);
            }
        }

        /// Scenario: 完全平方数を正しく素因数分解する。
        /// - Given: 素数の平方である完全平方数がある。
        /// - When: `factorize` を呼ぶ。
        /// - Then: `{p: 2}` が返る。
        #[test]
        fn returns_correct_factorization_for_perfect_squares() {
            // Given
            // 1000003 は素数であり、r * r は試し割りの閾値を超える。
            let p = 1000003_u64;
            let n = p * p;

            // When
            let result = factorize(n);

            // Then
            assert_eq!(HashMap::from([(p, 2)]), result);
        }

        /// Scenario: 試し割りの閾値を超える大きな合成数を、Pollard's rho 法の経路で
        /// 正しく素因数分解する (境界値)。
        /// - Given: 2 つの大きな素数の積である合成数がある。
        /// - When: `factorize` を呼ぶ。
        /// - Then: 期待した素因数分解が返る。
        #[test]
        fn returns_correct_factorization_via_pollards_rho_for_large_semiprimes() {
            let cases = [
                // 1000000007, 1000000009 はともに大きな素数である。
                (
                    1000000007_u64 * 1000000009,
                    HashMap::from([(1000000007, 1), (1000000009, 1)]),
                ),
                // 999999999000000007 は 1e18 級の大きな素数である。
                (
                    999999999000000007_u64 * 2,
                    HashMap::from([(2, 1), (999999999000000007, 1)]),
                ),
            ];

            for (n, expected) in cases {
                // Given, When
                let result = factorize(n);
                // Then
                assert_eq!(expected, result);
            }
        }
    }

    // divisors のテスト: 戻り値を検証する。
    mod divisors {
        use super::*;

        /// Scenario: 典型的な合成数に対して、昇順に並んだ約数の一覧を返す。
        /// - Given: 複数の素因数からなる典型的な合成数がある。
        /// - When: `divisors` を呼ぶ。
        /// - Then: 期待した約数の一覧が昇順で返る。
        #[test]
        fn returns_sorted_divisors_for_typical_values() {
            let cases = [
                (12_u64, vec![1_u64, 2, 3, 4, 6, 12]),
                (
                    360,
                    vec![
                        1, 2, 3, 4, 5, 6, 8, 9, 10, 12, 15, 18, 20, 24, 30, 36, 40, 45, 60, 72, 90,
                        120, 180, 360,
                    ],
                ),
            ];

            for (n, expected) in cases {
                // Given, When
                let result = divisors(n);
                // Then
                assert_eq!(expected, result);
            }
        }

        /// Scenario: 素数の約数は `1` と自分自身のみになる。
        /// - Given: 素数がいくつかある。
        /// - When: `divisors` を呼ぶ。
        /// - Then: `[1, n]` が返る。
        #[test]
        fn returns_one_and_itself_for_prime_numbers() {
            let cases = [7_u64, 17, 998244353];

            for n in cases {
                // Given, When
                let result = divisors(n);
                // Then
                assert_eq!(vec![1, n], result);
            }
        }

        /// Scenario: `1` の約数は `1` のみになる (境界値)。
        /// - Given: `1` がある。
        /// - When: `divisors` を呼ぶ。
        /// - Then: `[1]` が返る。
        #[test]
        fn returns_singleton_for_one() {
            // Given, When
            let result = divisors(1);
            // Then
            assert_eq!(vec![1], result);
        }
    }

    // euler_phi のテスト: 戻り値を検証する。
    mod euler_phi {
        use super::*;

        /// Scenario: 典型的な合成数に対して正しい `φ` の値を返す。
        /// - Given: 複数の素因数からなる典型的な合成数がある。
        /// - When: `euler_phi` を呼ぶ。
        /// - Then: 期待した `φ` の値が返る。
        #[test]
        fn returns_correct_value_for_typical_values() {
            let cases = [(12_u64, 4_u64), (9, 6)];

            for (n, expected) in cases {
                // Given, When
                let result = euler_phi(n);
                // Then
                assert_eq!(expected, result);
            }
        }

        /// Scenario: 素数 `p` に対しては `p - 1` を返す。
        /// - Given: 素数がいくつかある。
        /// - When: `euler_phi` を呼ぶ。
        /// - Then: `p - 1` が返る。
        #[test]
        fn returns_predecessor_for_prime_numbers() {
            let cases = [17_u64, 998244353];

            for p in cases {
                // Given, When
                let result = euler_phi(p);
                // Then
                assert_eq!(p - 1, result);
            }
        }

        /// Scenario: `1` に対しては `1` を返す (境界値)。
        /// - Given: `1` がある。
        /// - When: `euler_phi` を呼ぶ。
        /// - Then: `1` が返る。
        #[test]
        fn returns_one_for_one() {
            // Given, When
            let result = euler_phi(1);
            // Then
            assert_eq!(1, result);
        }

        /// Scenario: 試し割りの閾値を超える大きな `n` でも、オーバーフローせずに
        /// 正しい `φ` の値を返す (境界値)。
        /// - Given: `factorize` が Pollard's rho 法の経路を通るような、大きな
        ///   素因数からなる合成数がある。
        /// - When: `euler_phi` を呼ぶ。
        /// - Then: 期待した `φ` の値が返る。
        #[test]
        fn returns_correct_value_without_overflow_for_large_numbers() {
            let cases = [
                // 1000000007, 1000000009 はともに大きな素数であり、
                // φ(p * q) = (p - 1) * (q - 1) である。
                (1000000007_u64 * 1000000009, 1000000006_u64 * 1000000008),
                // 999999999000000007 は 1e18 級の大きな素数であり、
                // φ(2 * p) = p - 1 である。
                (999999999000000007_u64 * 2, 999999999000000006_u64),
            ];

            for (n, expected) in cases {
                // Given, When
                let result = euler_phi(n);
                // Then
                assert_eq!(expected, result);
            }
        }
    }

    // is_primitive_root のテスト: 戻り値を検証する。
    mod is_primitive_root {
        use super::*;

        /// Scenario: 原始根に対して `true` を返す。
        /// - Given: 素数 `7` を法とする原始根 (位数が `6` となる元) がある。
        /// - When: `is_primitive_root` を呼ぶ。
        /// - Then: `true` が返る。
        #[test]
        fn returns_true_for_primitive_roots() {
            let cases = [3_u64, 5];

            for a in cases {
                // Given, When
                let result = is_primitive_root(a, 7);
                // Then
                assert!(result);
            }
        }

        /// Scenario: 原始根でない元に対して `false` を返す。
        /// - Given: 素数 `7` を法とする、原始根でない元 (位数が `6` 未満となる元) が
        ///   いくつかある。
        /// - When: `is_primitive_root` を呼ぶ。
        /// - Then: `false` が返る。
        #[test]
        fn returns_false_for_non_primitive_roots() {
            let cases = [1_u64, 2, 4, 6];

            for a in cases {
                // Given, When
                let result = is_primitive_root(a, 7);
                // Then
                assert!(!result);
            }
        }

        /// Scenario: 法が `2` の場合、`1` のみが原始根と判定される (境界値)。
        /// - Given: 法が `2` であり、`a` が `0` または `1` である。
        /// - When: `is_primitive_root` を呼ぶ。
        /// - Then: `a` が `1` のときのみ `true` が返る。
        #[test]
        fn returns_true_only_for_one_when_modulus_is_two() {
            let cases = [(0_u64, false), (1, true)];

            for (a, expected) in cases {
                // Given, When
                let result = is_primitive_root(a, 2);
                // Then
                assert_eq!(expected, result);
            }
        }
    }

    // find_primitive_root のテスト: 戻り値を検証する。
    mod find_primitive_root {
        use super::*;

        /// Scenario: 見つけた元が、実際に原始根としての性質を満たす。
        /// - Given: 素数がいくつかある。
        /// - When: `find_primitive_root` を呼ぶ。
        /// - Then: 返った元が `is_primitive_root` で `true` と判定される。
        #[test]
        fn returns_value_satisfying_is_primitive_root_property() {
            let cases = [7_u64, 13, 998244353];

            for p in cases {
                // Given, When
                let result = find_primitive_root(p);
                // Then
                assert!(is_primitive_root(result, p));
            }
        }

        /// Scenario: 法が `2` の場合、`1` を返す (境界値)。
        /// - Given: 法が `2` である。
        /// - When: `find_primitive_root` を呼ぶ。
        /// - Then: `1` が返る。
        #[test]
        fn returns_one_when_modulus_is_two() {
            // Given, When
            let result = find_primitive_root(2);
            // Then
            assert_eq!(1, result);
        }
    }
}
