//! 素数判定・素因数分解に関連する機能を提供するモジュールである。

use crate::math::modular_arithmetic;

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
}
