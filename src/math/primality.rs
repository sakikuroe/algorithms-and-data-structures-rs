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
}
