/// Calculates the greatest common divisor (GCD) of two non-negative integers.
/// 2 つの非負整数の最大公約数（GCD）を計算する。
///
/// # Args
///
/// a: The first non-negative integer.
///     最初の非負整数
/// b: The second non-negative integer.
///     ２つ目の非負整数
///
/// # Returns
///
/// The greatest common divisor of `a` and `b`.
/// Specifically, for non-negative integers `a` and `b`:
/// - If `a = 0` and `b = 0`, returns `0`.
/// - If `a = 0` and `b > 0`, returns `b`.
/// - If `a > 0` and `b = 0`, returns `a`.
/// - If `a > 0` and `b > 0`, returns the largest positive integer that divides both `a` and `b`.
/// `a` と `b` の最大公約数。
/// 具体的には、非負整数 `a` と `b` に対して:
/// - `a = 0` かつ `b = 0` の場合、`0` を返す。
/// - `a = 0` かつ `b > 0` の場合、`b` を返す。
/// - `a > 0` かつ `b = 0` の場合、`a` を返す。
/// - `a > 0` かつ `b > 0` の場合、`a` と `b` の両方を割り切る最大の正の整数を返す。
///
/// # Complexity
///
/// Time: O(log(min(a, b)))
/// 時間計算量: O(log(min(a, b)))
///
/// Space: O(1)
/// 空間計算量: O(1)
///
/// # Examples
///
/// ```
/// use anmitsu::math::number_theory;
///
/// assert_eq!(6, number_theory::gcd(12, 18));
/// assert_eq!(12, number_theory::gcd(12, 12));
/// assert_eq!(5, number_theory::gcd(0, 5));
/// assert_eq!(0, number_theory::gcd(0, 0));
/// ```
pub fn gcd(mut a: u128, mut b: u128) -> u128 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

/// Calculates the least common multiple (LCM) of two non-negative integers.
/// 2 つの非負整数の最小公倍数（LCM）を計算する。
///
/// # Args
///
/// a: The first non-negative integer.
///     最初の非負整数
/// b: The second non-negative integer.
///     2 つ目の非負整数
///
/// # Returns
///
/// Returns the least common multiple (LCM) of `a` and `b` (the smallest non-negative common multiple of `a` and `b`) as an `Option<u128>`.
/// Specifically:
/// - Returns `None` if the LCM of `a` and `b` overflows the `u128` range.
/// - Otherwise, returns `Some(lcm_value)`, where `lcm_value` is the calculated LCM.
/// `a` と `b` の最小公倍数 (`a` と `b` の非負な公倍数のうち、最小のもの) を `Option<u128>` で返す。
/// 具体的には:
/// - `a` と b の最小公倍数が `u128` の範囲を超過する場合、`None` を返す。
/// - それ以外の場合、最小公倍数である `lcm_value` を `Some(lcm_value)` で返す。
///
/// # Complexity
///
/// Time: O(log(min(a, b)))
/// 時間計算量: O(log(min(a, b)))
///
/// Space: O(1)
/// 空間計算量: O(1)
///
/// # Examples
///
/// ```
/// use anmitsu::math::number_theory;
///
/// assert_eq!(Some(36), number_theory::lcm(12, 18));
/// assert_eq!(Some(12), number_theory::lcm(12, 12));
/// assert_eq!(Some(0), number_theory::lcm(0, 5));
/// assert_eq!(Some(0), number_theory::lcm(0, 0));
/// assert_eq!(Some(72), number_theory::lcm(8, 9));
/// assert_eq!(Some(100), number_theory::lcm(20, 25));
/// ```
pub fn lcm(a: u128, b: u128) -> Option<u128> {
    if a == 0 && b == 0 {
        return Some(0);
    }

    (a / gcd(a, b)).checked_mul(b)
}

#[cfg(test)]
mod tests {
    use super::*;

    // gcd のテスト: 戻り値を検証する。
    mod gcd {
        use super::*;

        /// Scenario: 典型的な値に対して最大公約数を返す。
        /// - Given: 共通の約数を持つ 2 つの正整数がある。
        /// - When: `gcd` を呼ぶ。
        /// - Then: 期待した最大公約数が返る。
        #[test]
        fn returns_greatest_common_divisor_for_typical_values() {
            let cases = [(10_u128, 5_u128, 5_u128), (27, 18, 9), (100, 75, 25)];

            for (a, b, expected) in cases {
                // Given, When
                let result = gcd(a, b);
                // Then
                assert_eq!(expected, result);
            }
        }

        /// Scenario: 互いに素な数の最大公約数は `1` になる。
        /// - Given: 互いに素な 2 つの正整数がある。
        /// - When: `gcd` を呼ぶ。
        /// - Then: `1` が返る。
        #[test]
        fn returns_one_for_coprime_numbers() {
            let cases = [(7_u128, 5_u128), (13, 17)];

            for (a, b) in cases {
                // Given, When
                let result = gcd(a, b);
                // Then
                assert_eq!(1, result);
            }
        }

        /// Scenario: 片方がもう片方の倍数である場合、最大公約数は小さい方の値になる。
        /// - Given: 一方が他方の倍数となっている 2 つの正整数がある。
        /// - When: `gcd` を呼ぶ。
        /// - Then: 小さい方の値が最大公約数として返る。
        #[test]
        fn returns_smaller_value_when_one_is_multiple_of_other() {
            let cases = [(10_u128, 2_u128, 2_u128), (5, 20, 5)];

            for (a, b, expected) in cases {
                // Given, When
                let result = gcd(a, b);
                // Then
                assert_eq!(expected, result);
            }
        }

        /// Scenario: 片方が `0` の場合、もう片方の値が最大公約数になる (境界値)。
        /// - Given: 一方が `0`、他方が正整数である組み合わせがある。
        /// - When: `gcd` を呼ぶ。
        /// - Then: `0` でない方の値が返る。
        #[test]
        fn returns_other_value_when_one_is_zero() {
            let cases = [(0_u128, 5_u128, 5_u128), (10, 0, 10)];

            for (a, b, expected) in cases {
                // Given, When
                let result = gcd(a, b);
                // Then
                assert_eq!(expected, result);
            }
        }

        /// Scenario: 両方が `0` の場合、定義により `0` を返す (境界値)。
        /// - Given: `a`, `b` がともに `0` である。
        /// - When: `gcd` を呼ぶ。
        /// - Then: `0` が返る。
        #[test]
        fn returns_zero_when_both_are_zero() {
            // Given, When
            let result = gcd(0, 0);
            // Then
            assert_eq!(0, result);
        }

        /// Scenario: `u128` の範囲に収まる大きな値でも正しく計算できる (境界値)。
        /// - Given: `u128::MAX` を含む大きな値の組み合わせがある。
        /// - When: `gcd` を呼ぶ。
        /// - Then: 期待した最大公約数が返る。
        #[test]
        fn returns_correct_value_for_large_numbers() {
            // Given, When
            let result_with_max = gcd(u128::MAX, 1);
            let result_with_max_swapped = gcd(1, u128::MAX);
            let result_with_common_power_of_two = gcd(3 * (1 << 125), 5 * (1 << 125));

            // Then
            assert_eq!(1, result_with_max);
            assert_eq!(1, result_with_max_swapped);
            assert_eq!(1 << 125, result_with_common_power_of_two);
        }
    }

    // lcm のテスト: 戻り値を検証する。
    mod lcm {
        use super::*;

        /// Scenario: 典型的な値に対して最小公倍数を返す。
        /// - Given: 2 つの正整数がある。
        /// - When: `lcm` を呼ぶ。
        /// - Then: `Some(期待した最小公倍数)` が返る。
        #[test]
        fn returns_least_common_multiple_for_typical_values() {
            let cases = [(12_u128, 18_u128, 36_u128), (100, 75, 300)];

            for (a, b, expected) in cases {
                // Given, When
                let result = lcm(a, b);
                // Then
                assert_eq!(Some(expected), result);
            }
        }

        /// Scenario: 互いに素な数の最小公倍数は積になる。
        /// - Given: 互いに素な 2 つの正整数がある。
        /// - When: `lcm` を呼ぶ。
        /// - Then: `a` と `b` の積が `Some` で返る。
        #[test]
        fn returns_product_for_coprime_numbers() {
            let cases = [(3_u128, 5_u128, 15_u128), (7, 11, 77)];

            for (a, b, expected) in cases {
                // Given, When
                let result = lcm(a, b);
                // Then
                assert_eq!(Some(expected), result);
            }
        }

        /// Scenario: 片方がもう片方の倍数である場合、最小公倍数は大きい方の値になる。
        /// - Given: 一方が他方の倍数となっている 2 つの正整数がある。
        /// - When: `lcm` を呼ぶ。
        /// - Then: 大きい方の値が `Some` で返る。
        #[test]
        fn returns_larger_value_when_one_is_multiple_of_other() {
            let cases = [(5_u128, 10_u128, 10_u128), (8, 2, 8), (10, 1, 10)];

            for (a, b, expected) in cases {
                // Given, When
                let result = lcm(a, b);
                // Then
                assert_eq!(Some(expected), result);
            }
        }

        /// Scenario: 同じ数同士の最小公倍数は、その数自身になる (境界値)。
        /// - Given: `a` と `b` が同じ値である組み合わせがある。
        /// - When: `lcm` を呼ぶ。
        /// - Then: その値自身が `Some` で返る。
        #[test]
        fn returns_same_value_for_equal_numbers() {
            let cases = [(1_u128, 1_u128), (7, 7), (100, 100)];

            for (a, b) in cases {
                // Given, When
                let result = lcm(a, b);
                // Then
                assert_eq!(Some(a), result);
            }
        }

        /// Scenario: 片方が `0` の場合、最小公倍数は `0` になる (境界値)。
        /// - Given: 一方が `0`、他方が正整数である組み合わせがある。
        /// - When: `lcm` を呼ぶ。
        /// - Then: `Some(0)` が返る。
        #[test]
        fn returns_zero_when_one_is_zero() {
            let cases = [(0_u128, 5_u128), (5, 0)];

            for (a, b) in cases {
                // Given, When
                let result = lcm(a, b);
                // Then
                assert_eq!(Some(0), result);
            }
        }

        /// Scenario: 両方が `0` の場合、最小公倍数は `0` になる (境界値)。
        /// - Given: `a`, `b` がともに `0` である。
        /// - When: `lcm` を呼ぶ。
        /// - Then: `Some(0)` が返る。
        #[test]
        fn returns_zero_when_both_are_zero() {
            // Given, When
            let result = lcm(0, 0);
            // Then
            assert_eq!(Some(0), result);
        }

        /// Scenario: `u128` の範囲に収まる大きな値でもオーバーフローせずに計算できる (境界値)。
        /// - Given: `2^64` 未満、および `2^128` 未満の最大の整数を含む組み合わせがある。
        /// - When: `lcm` を呼ぶ。
        /// - Then: 期待した最小公倍数が `Some` で返る。
        #[test]
        fn returns_correct_value_for_large_numbers_within_range() {
            // Given
            // `2^{64}` 未満の最大の整数
            let p = 18446744073709551557_u128;
            // `2^{128}` 未満の最大の整数
            let q = 340282366920938463463374607431768211297_u128;

            // When
            let result_below_2_pow_64 = lcm(p - 1, p);
            let result_below_2_pow_128 = lcm(q, q);

            // Then
            assert_eq!(Some((p - 1) * p), result_below_2_pow_64);
            assert_eq!(Some(q), result_below_2_pow_128);
        }

        /// Scenario: 最小公倍数が `u128` の範囲を超過する場合、`None` を返す (異常系)。
        /// - Given: `2^128` 未満の最大の整数と、その 1 つ小さい整数がある。
        /// - When: `lcm` を呼ぶ。
        /// - Then: `None` が返る。
        #[test]
        fn returns_none_when_result_overflows() {
            // Given
            // `2^{128}` 未満の最大の整数
            let p = 340282366920938463463374607431768211297_u128;

            // When
            let result = lcm(p - 1, p);

            // Then
            assert!(result.is_none());
        }
    }
}
