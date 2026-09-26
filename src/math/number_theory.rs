/// 2 つの非負整数の最大公約数（GCD）を計算する。
///
/// # Args
///
/// a: 最初の非負整数
/// b: ２つ目の非負整数
///
/// # Returns
///
/// `a` と `b` の最大公約数。
/// 具体的には、非負整数 `a` と `b` に対して:
/// - `a = 0` かつ `b = 0` の場合、`0` を返す。
/// - `a = 0` かつ `b > 0` の場合、`b` を返す。
/// - `a > 0` かつ `b = 0` の場合、`a` を返す。
/// - `a > 0` かつ `b > 0` の場合、`a` と `b` の両方を割り切る最大の正の整数を返す。
///
/// # Complexity
///
/// 時間計算量: O(log(min(a, b)))
///
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

/// 2 つの非負整数の最小公倍数（LCM）を計算する。
///
/// # Args
///
/// a: 最初の非負整数
/// b: 2 つ目の非負整数
///
/// # Returns
///
/// `a` と `b` の最小公倍数 (`a` と `b` の非負な公倍数のうち、最小のもの) を `Option<u128>` で返す。
/// 具体的には:
/// - `a` と b の最小公倍数が `u128` の範囲を超過する場合、`None` を返す。
/// - それ以外の場合、最小公倍数である `lcm_value` を `Some(lcm_value)` で返す。
///
/// # Complexity
///
/// 時間計算量: O(log(min(a, b)))
///
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

/// 拡張ユークリッドの互除法により、ベズー等式を満たす係数を求める。
///
/// # Args
/// - `a` - 整数
/// - `b` - 整数
///
/// # Returns
/// `(x, y)`: `a * x + b * y == gcd(a, b)` を満たす整数の組。
/// 具体的には:
/// - `a = 0` かつ `b = 0` の場合、`(0, 0)` を返す。
/// - それ以外の場合、`gcd(a, b)` は非負整数の最大公約数として計算され、これを満たす `(x, y)` のうち
///   `max(|x|, |y|) <= max(|a|, |b|)` を満たすものを返す。
///
/// # Complexity
/// - 時間計算量: $O(\log(\min(|a|, |b|)))$
/// - 空間計算量: $O(1)$
///
/// # Examples
/// ```
/// use anmitsu::math::number_theory;
///
/// // gcd(10, 4) = 2 = 10 * 1 + 4 * (-2)
/// assert_eq!((1, -2), number_theory::extended_gcd(10, 4));
///
/// // gcd(0, 4) = 4 = 0 * 0 + 4 * 1
/// assert_eq!((0, 1), number_theory::extended_gcd(0, 4));
///
/// // gcd(0, 0) = 0 = 0 * 0 + 0 * 0
/// assert_eq!((0, 0), number_theory::extended_gcd(0, 0));
///
/// // gcd(-12, 7) = 1 = -12 * (-3) + 7 * (-5)
/// assert_eq!((-3, -5), number_theory::extended_gcd(-12, 7));
/// ```
pub fn extended_gcd(a: i64, b: i64) -> (i64, i64) {
    if a == 0 && b == 0 {
        return (0, 0);
    }
    if a == 0 {
        return (0, if b > 0 { 1 } else { -1 });
    }
    if b == 0 {
        return (if a > 0 { 1 } else { -1 }, 0);
    }

    // (s, t) はユークリッドの互除法の途中経過であり、(xs, ys), (xt, yt) はそれぞれ
    // s == a * xs + b * ys, t == a * xt + b * yt を満たす係数の組である。
    // s を t で割った商 q に対して次の組を作ると、この不変条件を保ったまま
    // (s, t) を (t, s % t) へ進めることができる。
    // i64::MIN / -1 や、係数を更新する途中の積が i64 を超える場合も扱えるよう、
    // ユークリッドの互除法は i128 で計算する。
    let (mut xs, mut ys, mut s) = (1_i128, 0_i128, a as i128);
    let (mut xt, mut yt, mut t) = (0_i128, 1_i128, b as i128);

    while s % t != 0 {
        let q = s / t;
        let (u, xu, yu) = (s - q * t, xs - q * xt, ys - q * yt);
        (xs, ys, xt, yt) = (xt, yt, xu, yu);
        (s, t) = (t, u);
    }

    // t が gcd(a, b) の符号付きの値であり、非負整数の gcd に揃えるため
    // 負の場合は係数の符号を反転させる。
    let (x, y) = if t < 0 { (-xt, -yt) } else { (xt, yt) };
    (
        i64::try_from(x).expect("Bezout coefficient x fits in i64"),
        i64::try_from(y).expect("Bezout coefficient y fits in i64"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // gcd のテスト: 戻り値を検証する。
    mod gcd {
        use super::*;
        use rstest::rstest;

        /// Scenario: 典型的な値に対して最大公約数を返す。
        /// - Given: 共通の約数を持つ 2 つの正整数がある。
        /// - When: `gcd` を呼ぶ。
        /// - Then: 期待した最大公約数が返る。
        #[rstest]
        #[case::ten_and_five(10_u128, 5_u128, 5_u128)]
        #[case::twenty_seven_and_eighteen(27, 18, 9)]
        #[case::one_hundred_and_seventy_five(100, 75, 25)]
        fn returns_greatest_common_divisor_for_typical_values(
            #[case] a: u128,
            #[case] b: u128,
            #[case] expected: u128,
        ) {
            // Given, When
            let result = gcd(a, b);
            // Then
            assert_eq!(expected, result);
        }

        /// Scenario: 互いに素な数の最大公約数は `1` になる。
        /// - Given: 互いに素な 2 つの正整数がある。
        /// - When: `gcd` を呼ぶ。
        /// - Then: `1` が返る。
        #[rstest]
        #[case::seven_and_five(7_u128, 5_u128)]
        #[case::thirteen_and_seventeen(13, 17)]
        fn returns_one_for_coprime_numbers(#[case] a: u128, #[case] b: u128) {
            // Given, When
            let result = gcd(a, b);
            // Then
            assert_eq!(1, result);
        }

        /// Scenario: 片方がもう片方の倍数である場合、最大公約数は小さい方の値になる。
        /// - Given: 一方が他方の倍数となっている 2 つの正整数がある。
        /// - When: `gcd` を呼ぶ。
        /// - Then: 小さい方の値が最大公約数として返る。
        #[rstest]
        #[case::left_is_multiple(10_u128, 2_u128, 2_u128)]
        #[case::right_is_multiple(5, 20, 5)]
        fn returns_smaller_value_when_one_is_multiple_of_other(
            #[case] a: u128,
            #[case] b: u128,
            #[case] expected: u128,
        ) {
            // Given, When
            let result = gcd(a, b);
            // Then
            assert_eq!(expected, result);
        }

        /// Scenario: 片方が `0` の場合、もう片方の値が最大公約数になる (境界値)。
        /// - Given: 一方が `0`、他方が正整数である組み合わせがある。
        /// - When: `gcd` を呼ぶ。
        /// - Then: `0` でない方の値が返る。
        #[rstest]
        #[case::left_is_zero(0_u128, 5_u128, 5_u128)]
        #[case::right_is_zero(10, 0, 10)]
        fn returns_other_value_when_one_is_zero(
            #[case] a: u128,
            #[case] b: u128,
            #[case] expected: u128,
        ) {
            // Given, When
            let result = gcd(a, b);
            // Then
            assert_eq!(expected, result);
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

        /// Scenario: 大きな値の組み合わせでも最大公約数を正しく計算できる (境界値)。
        /// - Given: `u128::MAX` や共通因子を含む大きな値の組み合わせがある。
        /// - When: `gcd` を呼ぶ。
        /// - Then: 各ケースで期待した最大公約数が返る。
        #[rstest]
        #[case::max_and_one(u128::MAX, 1, 1)]
        #[case::one_and_max(1, u128::MAX, 1)]
        #[case::common_power_of_two(3 * (1 << 125), 5 * (1 << 125), 1 << 125)]
        fn returns_correct_value_for_large_numbers(
            #[case] a: u128,
            #[case] b: u128,
            #[case] expected: u128,
        ) {
            // Given, When
            let result = gcd(a, b);
            // Then
            assert_eq!(expected, result);
        }
    }

    // lcm のテスト: 戻り値を検証する。
    mod lcm {
        use super::*;
        use rstest::rstest;

        /// Scenario: 典型的な値に対して最小公倍数を返す。
        /// - Given: 2 つの正整数がある。
        /// - When: `lcm` を呼ぶ。
        /// - Then: `Some(期待した最小公倍数)` が返る。
        #[rstest]
        #[case::twelve_and_eighteen(12_u128, 18_u128, 36_u128)]
        #[case::one_hundred_and_seventy_five(100, 75, 300)]
        fn returns_least_common_multiple_for_typical_values(
            #[case] a: u128,
            #[case] b: u128,
            #[case] expected: u128,
        ) {
            // Given, When
            let result = lcm(a, b);
            // Then
            assert_eq!(Some(expected), result);
        }

        /// Scenario: 互いに素な数の最小公倍数は積になる。
        /// - Given: 互いに素な 2 つの正整数がある。
        /// - When: `lcm` を呼ぶ。
        /// - Then: `a` と `b` の積が `Some` で返る。
        #[rstest]
        #[case::three_and_five(3_u128, 5_u128, 15_u128)]
        #[case::seven_and_eleven(7, 11, 77)]
        fn returns_product_for_coprime_numbers(
            #[case] a: u128,
            #[case] b: u128,
            #[case] expected: u128,
        ) {
            // Given, When
            let result = lcm(a, b);
            // Then
            assert_eq!(Some(expected), result);
        }

        /// Scenario: 片方がもう片方の倍数である場合、最小公倍数は大きい方の値になる。
        /// - Given: 一方が他方の倍数となっている 2 つの正整数がある。
        /// - When: `lcm` を呼ぶ。
        /// - Then: 大きい方の値が `Some` で返る。
        #[rstest]
        #[case::left_is_multiple(5_u128, 10_u128, 10_u128)]
        #[case::right_is_multiple(8, 2, 8)]
        #[case::divisible_by_one(10, 1, 10)]
        fn returns_larger_value_when_one_is_multiple_of_other(
            #[case] a: u128,
            #[case] b: u128,
            #[case] expected: u128,
        ) {
            // Given, When
            let result = lcm(a, b);
            // Then
            assert_eq!(Some(expected), result);
        }

        /// Scenario: 同じ数同士の最小公倍数は、その数自身になる (境界値)。
        /// - Given: `a` と `b` が同じ値である組み合わせがある。
        /// - When: `lcm` を呼ぶ。
        /// - Then: その値自身が `Some` で返る。
        #[rstest]
        #[case::one(1_u128, 1_u128)]
        #[case::seven(7, 7)]
        #[case::one_hundred(100, 100)]
        fn returns_same_value_for_equal_numbers(#[case] a: u128, #[case] b: u128) {
            // Given, When
            let result = lcm(a, b);
            // Then
            assert_eq!(Some(a), result);
        }

        /// Scenario: 片方が `0` の場合、最小公倍数は `0` になる (境界値)。
        /// - Given: 一方が `0`、他方が正整数である組み合わせがある。
        /// - When: `lcm` を呼ぶ。
        /// - Then: `Some(0)` が返る。
        #[rstest]
        #[case::left_is_zero(0_u128, 5_u128)]
        #[case::right_is_zero(5, 0)]
        fn returns_zero_when_one_is_zero(#[case] a: u128, #[case] b: u128) {
            // Given, When
            let result = lcm(a, b);
            // Then
            assert_eq!(Some(0), result);
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

        /// Scenario: 大きな値でもオーバーフローせずに最小公倍数を計算できる (境界値)。
        /// - Given: `2^64` 未満の近接値、または `2^128` 未満の大きな値がある。
        /// - When: `lcm` を呼ぶ。
        /// - Then: 各ケースで期待した最小公倍数が `Some` で返る。
        #[rstest]
        #[case::adjacent_large_values(
            18446744073709551556_u128,
            18446744073709551557,
            Some(18446744073709551556_u128 * 18446744073709551557),
        )]
        #[case::equal_values_near_u128_max(
            340282366920938463463374607431768211297_u128,
            340282366920938463463374607431768211297,
            Some(340282366920938463463374607431768211297)
        )]
        fn returns_correct_value_for_large_numbers_within_range(
            #[case] a: u128,
            #[case] b: u128,
            #[case] expected: Option<u128>,
        ) {
            // Given, When
            let result = lcm(a, b);
            // Then
            assert_eq!(expected, result);
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

    // extended_gcd のテスト: 戻り値を検証する。
    mod extended_gcd {
        use super::*;
        use rstest::rstest;

        /// Scenario: 典型的な正整数の組に対してベズー等式を満たす係数を返す。
        /// - Given: 共通の約数を持つ 2 つの正整数がある。
        /// - When: `extended_gcd` を呼ぶ。
        /// - Then: 期待した係数の組が返る。
        #[rstest]
        #[case::ten_and_four(10, 4, 1, -2)]
        #[case::twenty_seven_and_eighteen(27, 18, 1, -1)]
        #[case::one_hundred_and_seventy_five(100, 75, 1, -1)]
        fn returns_bezout_coefficients_for_typical_values(
            #[case] a: i64,
            #[case] b: i64,
            #[case] expected_x: i64,
            #[case] expected_y: i64,
        ) {
            // Given, When
            let result = extended_gcd(a, b);
            // Then
            assert_eq!((expected_x, expected_y), result);
        }

        /// Scenario: 互いに素な数の組に対してベズー等式を満たす係数を返す。
        /// - Given: 互いに素な 2 つの正整数がある。
        /// - When: `extended_gcd` を呼ぶ。
        /// - Then: 期待した係数の組が返る。
        #[rstest]
        #[case::seven_and_five(7, 5, -2, 3)]
        #[case::thirteen_and_seventeen(13, 17, 4, -3)]
        fn returns_bezout_coefficients_for_coprime_numbers(
            #[case] a: i64,
            #[case] b: i64,
            #[case] expected_x: i64,
            #[case] expected_y: i64,
        ) {
            // Given, When
            let result = extended_gcd(a, b);
            // Then
            assert_eq!((expected_x, expected_y), result);
        }

        /// Scenario: 片方がもう片方の倍数である場合、その関係を反映した係数を返す。
        /// - Given: 一方が他方の倍数となっている 2 つの正整数がある。
        /// - When: `extended_gcd` を呼ぶ。
        /// - Then: 期待した係数の組が返る。
        #[rstest]
        #[case::first_is_multiple(10, 2, 0, 1)]
        #[case::second_is_multiple(5, 20, 1, 0)]
        fn returns_bezout_coefficients_when_one_is_multiple_of_other(
            #[case] a: i64,
            #[case] b: i64,
            #[case] expected_x: i64,
            #[case] expected_y: i64,
        ) {
            // Given, When
            let result = extended_gcd(a, b);
            // Then
            assert_eq!((expected_x, expected_y), result);
        }

        /// Scenario: 片方が `0` の場合、`0` でない方の符号に応じた係数を返す (境界値)。
        /// - Given: 一方が `0`、他方が正または負の整数である組み合わせがある。
        /// - When: `extended_gcd` を呼ぶ。
        /// - Then: 期待した係数の組が返る。
        #[rstest]
        #[case::first_zero_second_positive(0, 5, 0, 1)]
        #[case::second_zero_first_positive(10, 0, 1, 0)]
        #[case::first_zero_second_negative(0, -5, 0, -1)]
        #[case::second_zero_first_negative(-10, 0, -1, 0)]
        fn returns_bezout_coefficients_when_one_is_zero(
            #[case] a: i64,
            #[case] b: i64,
            #[case] expected_x: i64,
            #[case] expected_y: i64,
        ) {
            // Given, When
            let result = extended_gcd(a, b);
            // Then
            assert_eq!((expected_x, expected_y), result);
        }

        /// Scenario: 両方が `0` の場合、`(0, 0)` を返す (境界値)。
        /// - Given: `a`, `b` がともに `0` である。
        /// - When: `extended_gcd` を呼ぶ。
        /// - Then: `(0, 0)` が返る。
        #[test]
        fn returns_zero_pair_when_both_are_zero() {
            // Given, When
            let result = extended_gcd(0, 0);
            // Then
            assert_eq!((0, 0), result);
        }

        /// Scenario: 負の整数を含む組み合わせでも、ベズー等式 `a * x + b * y == gcd(a, b)` を満たす。
        /// - Given: 負の整数を含む 2 つの整数の組み合わせがある。
        /// - When: `extended_gcd` を呼ぶ。
        /// - Then: 返った係数の組がベズー等式を満たす。
        #[rstest]
        #[case::negative_first(-12, 7)]
        #[case::negative_second(12, -7)]
        #[case::both_negative(-12, -7)]
        #[case::minimum_and_negative_one(i64::MIN, -1)]
        #[case::minimum_and_zero(i64::MIN, 0)]
        #[case::zero_and_minimum(0, i64::MIN)]
        #[case::minimum_and_maximum(i64::MIN, i64::MAX)]
        fn satisfies_bezout_identity_for_negative_numbers(#[case] a: i64, #[case] b: i64) {
            // Given, When
            let (x, y) = extended_gcd(a, b);
            let expected_gcd = gcd(a.unsigned_abs() as u128, b.unsigned_abs() as u128);
            // Then
            assert_eq!(
                expected_gcd as i128,
                a as i128 * x as i128 + b as i128 * y as i128
            );
        }
    }
}
