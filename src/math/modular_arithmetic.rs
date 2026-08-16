//! 任意の法に対するモジュラー演算を提供するモジュールである。
//!
//! `mod_inv` を除く関数は `u64` の全域を法として扱える。`mod_inv` は内部で
//! `number_theory::extended_gcd` (符号付き整数を扱う) を利用するため、
//! 法は `i64` に収まる範囲に制限される。

use crate::math::number_theory;

/// 法 `m` の下での加算 `(a + b) mod m` を計算する。
///
/// # Args
/// - `a` - `[0, m)` の範囲の非負整数
/// - `b` - `[0, m)` の範囲の非負整数
/// - `m` - 法であり、`0` より大きい必要がある
///
/// # Returns
/// `(a + b) mod m`
///
/// # Complexity
/// - 時間計算量: $O(1)$
///
/// # Examples
/// ```
/// use anmitsu::math::modular_arithmetic;
///
/// assert_eq!(2, modular_arithmetic::add_mod(4, 5, 7));
/// assert_eq!(0, modular_arithmetic::add_mod(3, 4, 7));
/// ```
pub fn add_mod(a: u64, b: u64, m: u64) -> u64 {
    debug_assert!(m != 0);
    debug_assert!(a < m);
    debug_assert!(b < m);

    // a + b が u64 の範囲を超える場合でも、2^{64} ≡ 2^{64} - m (mod m) を
    // 利用した wrapping 演算で正しい剰余が得られる。
    let (t, overflowed) = a.overflowing_add(b);
    if overflowed || t >= m {
        t.wrapping_sub(m)
    } else {
        t
    }
}

/// 法 `m` の下での減算 `(a - b) mod m` を計算する。
///
/// # Args
/// - `a` - `[0, m)` の範囲の非負整数
/// - `b` - `[0, m)` の範囲の非負整数
/// - `m` - 法であり、`0` より大きい必要がある
///
/// # Returns
/// `(a - b) mod m` (`[0, m)` の範囲に正規化された値)
///
/// # Complexity
/// - 時間計算量: $O(1)$
///
/// # Examples
/// ```
/// use anmitsu::math::modular_arithmetic;
///
/// assert_eq!(2, modular_arithmetic::sub_mod(5, 3, 7));
/// assert_eq!(5, modular_arithmetic::sub_mod(3, 5, 7));
/// ```
pub fn sub_mod(a: u64, b: u64, m: u64) -> u64 {
    debug_assert!(m != 0);
    debug_assert!(a < m);
    debug_assert!(b < m);

    let (t, overflowed) = a.overflowing_sub(b);
    if overflowed { t.wrapping_add(m) } else { t }
}

/// 法 `m` の下での乗算 `(a * b) mod m` を計算する。
///
/// # Args
/// - `a` - `[0, m)` の範囲の非負整数
/// - `b` - `[0, m)` の範囲の非負整数
/// - `m` - 法であり、`0` より大きい必要がある
///
/// # Returns
/// `(a * b) mod m`
///
/// # Complexity
/// - 時間計算量: $O(1)$
///
/// # Examples
/// ```
/// use anmitsu::math::modular_arithmetic;
///
/// assert_eq!(1, modular_arithmetic::mul_mod(3, 5, 7));
/// assert_eq!(6, modular_arithmetic::mul_mod(2, 3, 7));
/// ```
pub fn mul_mod(a: u64, b: u64, m: u64) -> u64 {
    debug_assert!(m != 0);
    debug_assert!(a < m);
    debug_assert!(b < m);

    // a * b は u64 の範囲を超える可能性があるため、u128 に拡張してから剰余を取る。
    ((a as u128 * b as u128) % m as u128) as u64
}

/// 法 `m` の下でのべき乗 `a^n mod m` を、二分累乗法により計算する。
///
/// # Args
/// - `a` - `[0, m)` の範囲の非負整数
/// - `n` - 指数であり、非負整数
/// - `m` - 法であり、`0` より大きい必要がある
///
/// # Returns
/// `a^n mod m`
///
/// # Complexity
/// - 時間計算量: $O(\log n)$
///
/// # Examples
/// ```
/// use anmitsu::math::modular_arithmetic;
///
/// assert_eq!(1, modular_arithmetic::pow_mod(2, 0, 7));
/// assert_eq!(2, modular_arithmetic::pow_mod(2, 10, 7));
/// ```
pub fn pow_mod(a: u64, mut n: u64, m: u64) -> u64 {
    debug_assert!(m != 0);
    debug_assert!(a < m);

    let mut result = 1 % m;
    let mut base = a;

    while n > 0 {
        if n & 1 == 1 {
            result = mul_mod(result, base, m);
        }
        base = mul_mod(base, base, m);
        n >>= 1;
    }

    result
}

/// 法 `m` の下での `a` の逆元 `a^{-1} mod m` を、拡張ユークリッドの互除法により計算する。
///
/// # Args
/// - `a` - `[0, m)` の範囲の非負整数であり、`gcd(a, m) == 1` を満たす必要がある
/// - `m` - 法であり、`0 < m <= i64::MAX` を満たす必要がある
///
/// # Returns
/// `a * x ≡ 1 (mod m)` を満たす `x` を `[0, m)` の範囲で返す。
///
/// # Complexity
/// - 時間計算量: $O(\log m)$
///
/// # Examples
/// ```
/// use anmitsu::math::modular_arithmetic;
///
/// assert_eq!(4, modular_arithmetic::mod_inv(2, 7));
/// assert_eq!(1, modular_arithmetic::mul_mod(2, modular_arithmetic::mod_inv(2, 7), 7));
/// ```
pub fn mod_inv(a: u64, m: u64) -> u64 {
    debug_assert!(m != 0);
    debug_assert!(m <= i64::MAX as u64);
    debug_assert!(a < m);
    debug_assert!(number_theory::gcd(a as u128, m as u128) == 1);

    let (x, _) = number_theory::extended_gcd(a as i64, m as i64);
    x.rem_euclid(m as i64) as u64
}

/// 中国剰余定理 (CRT) により、連立合同式 `x ≡ remainders[i] (mod moduli[i])`
/// (`i = 0, ..., remainders.len() - 1`) を満たす `x` を求める。`moduli` が互いに
/// 素であることを要求しない、一般の法に対応する。
///
/// # Args
/// - `remainders` - 各合同式の右辺の列
/// - `moduli` - 各合同式の法の列であり、`remainders` と同じ長さでなければならない。
///   各要素は `0` より大きく `i64::MAX` 以下である必要がある。
///
/// # Returns
/// 連立合同式を満たす `x` が存在する場合、`Some((r, m))` を返す。`m` は `moduli`
/// 全体の最小公倍数であり、`r` は `[0, m)` の範囲の解で、`x ≡ r (mod m)` が元の
/// 連立合同式全体と同値になる。連立合同式が矛盾し解が存在しない場合は `None` を
/// 返す。`remainders` と `moduli` がともに空の場合、法 `1` の下では任意の整数が
/// 合同であることから、「制約なし」を表す単位元として `Some((0, 1))` を返す。
///
/// # Complexity
/// - 時間計算量: $O(n \log(\max(\text{moduli})))$
///   - `n = remainders.len()` であり、隣接する 2 つの合同式を 1 つへまとめる処理を
///     `n - 1` 回繰り返す。各マージは [`number_theory::gcd`]・[`number_theory::extended_gcd`]
///     の計算が支配的である。
///
/// # Examples
/// ```
/// use anmitsu::math::modular_arithmetic;
///
/// // 法が互いに素な場合
/// assert_eq!(Some((23, 105)), modular_arithmetic::crt(&[2, 3, 2], &[3, 5, 7]));
///
/// // 法が互いに素でなくても解が存在する場合
/// assert_eq!(Some((4, 6)), modular_arithmetic::crt(&[1, 4], &[3, 6]));
///
/// // 矛盾して解が存在しない場合
/// assert_eq!(None, modular_arithmetic::crt(&[0, 1], &[2, 4]));
/// ```
#[must_use]
pub fn crt(remainders: &[u64], moduli: &[u64]) -> Option<(u64, u64)> {
    debug_assert_eq!(remainders.len(), moduli.len());
    debug_assert!(moduli.iter().all(|&m| m > 0 && m <= i64::MAX as u64));

    if remainders.is_empty() {
        return Some((0, 1));
    }

    // (r, m) は、これまでに読んだ合同式をすべてまとめた結果を x ≡ r (mod m) の
    // 形で保持する。最初の合同式は単独では矛盾しえないため、[0, m) に正規化した
    // 上でそのまま初期値として採用する。
    let mut r = (remainders[0] as i64).rem_euclid(moduli[0] as i64);
    let mut m = moduli[0] as i64;

    for i in 1..remainders.len() {
        let (r1, m1) = (remainders[i] as i64, moduli[i] as i64);

        // x ≡ r (mod m) と x ≡ r1 (mod m1) が両立するためには、(r1 - r) が
        // g = gcd(m, m1) で割り切れる必要がある (CRT の一般形における可解条件)。
        let g = number_theory::gcd(m as u128, m1 as u128) as i64;
        if (r1 - r) % g != 0 {
            return None;
        }

        // m * p + m1 * q == g を満たす p を用いて、2 つの合同式を単一の合同式
        // x ≡ new_r (mod lcm) へまとめる。
        let (p, _) = number_theory::extended_gcd(m, m1);
        let lcm = m / g * m1;
        let diff = (r1 - r) / g;
        let new_r = r + m * (diff * p).rem_euclid(m1 / g);

        // 得られた解を [0, lcm) の範囲に正規化し、次の合同式とのマージに備える。
        r = new_r.rem_euclid(lcm);
        m = lcm;
    }

    Some((r as u64, m as u64))
}

#[cfg(test)]
mod tests {
    use super::*;

    // add_mod のテスト: 戻り値を検証する。
    mod add_mod {
        use super::*;

        /// Scenario: 桁上がりが発生しない典型的な値の和を返す。
        /// - Given: 和が法未満に収まる 2 つの値がある。
        /// - When: `add_mod` を呼ぶ。
        /// - Then: 通常の和がそのまま返る。
        #[test]
        fn returns_sum_when_no_overflow() {
            // Given, When
            let result = add_mod(2, 3, 7);
            // Then
            assert_eq!(5, result);
        }

        /// Scenario: 和が法以上になる場合、法で還元された値を返す (境界値)。
        /// - Given: 和がちょうど法に達する組み合わせと、法を超える組み合わせがある。
        /// - When: `add_mod` を呼ぶ。
        /// - Then: 法で還元された値が返る。
        #[test]
        fn returns_reduced_value_when_sum_reaches_modulus() {
            let cases = [(3_u64, 4_u64, 7_u64, 0_u64), (4, 5, 7, 2)];

            for (a, b, m, expected) in cases {
                // Given, When
                let result = add_mod(a, b, m);
                // Then
                assert_eq!(expected, result);
            }
        }

        /// Scenario: `u64` の加算がオーバーフローする組み合わせでも正しい値を返す (境界値)。
        /// - Given: `a`, `b` がともに `u64::MAX` に近い、法未満の大きな値である。
        /// - When: `add_mod` を呼ぶ。
        /// - Then: オーバーフローを考慮した正しい和が返る。
        #[test]
        fn returns_correct_value_when_u64_addition_overflows() {
            // Given
            let m = u64::MAX;
            let a = m - 1;
            let b = m - 1;

            // When
            let result = add_mod(a, b, m);

            // Then
            assert_eq!(m - 2, result);
        }
    }

    // sub_mod のテスト: 戻り値を検証する。
    mod sub_mod {
        use super::*;

        /// Scenario: 差が非負になる典型的な値の差を返す。
        /// - Given: `a >= b` である 2 つの値がある。
        /// - When: `sub_mod` を呼ぶ。
        /// - Then: 通常の差がそのまま返る。
        #[test]
        fn returns_difference_when_non_negative() {
            // Given, When
            let result = sub_mod(5, 3, 7);
            // Then
            assert_eq!(2, result);
        }

        /// Scenario: 差が負になる場合、法を加えて正規化した値を返す (境界値)。
        /// - Given: `a < b` である 2 つの値がある。
        /// - When: `sub_mod` を呼ぶ。
        /// - Then: `[0, m)` の範囲に正規化された値が返る。
        #[test]
        fn returns_normalized_value_when_difference_is_negative() {
            // Given, When
            let result = sub_mod(3, 5, 7);
            // Then
            assert_eq!(5, result);
        }

        /// Scenario: 同じ値同士の差は `0` になる (境界値)。
        /// - Given: `a` と `b` が同じ値である。
        /// - When: `sub_mod` を呼ぶ。
        /// - Then: `0` が返る。
        #[test]
        fn returns_zero_for_equal_values() {
            // Given, When
            let result = sub_mod(4, 4, 7);
            // Then
            assert_eq!(0, result);
        }
    }

    // mul_mod のテスト: 戻り値を検証する。
    mod mul_mod {
        use super::*;

        /// Scenario: 積が法未満に収まる場合、そのままの積を返す。
        /// - Given: 積が法未満に収まる 2 つの値がある。
        /// - When: `mul_mod` を呼ぶ。
        /// - Then: 通常の積がそのまま返る。
        #[test]
        fn returns_product_when_no_reduction_needed() {
            // Given, When
            let result = mul_mod(2, 3, 7);
            // Then
            assert_eq!(6, result);
        }

        /// Scenario: 積が法以上になる場合、法で還元された値を返す。
        /// - Given: 積が法を超える 2 つの値がある。
        /// - When: `mul_mod` を呼ぶ。
        /// - Then: 法で還元された値が返る。
        #[test]
        fn returns_reduced_value_when_product_exceeds_modulus() {
            // Given, When
            let result = mul_mod(3, 5, 7);
            // Then
            assert_eq!(1, result);
        }

        /// Scenario: `u64` の乗算がオーバーフローする組み合わせでも正しい値を返す (境界値)。
        /// - Given: `a`, `b` がともに `u64::MAX` に近い、法未満の大きな値である。
        /// - When: `mul_mod` を呼ぶ。
        /// - Then: `u128` 経由で正しく還元された積が返る。
        #[test]
        fn returns_correct_value_when_u64_multiplication_overflows() {
            // Given
            let m = u64::MAX;
            let a = m - 1;
            let b = m - 1;

            // When
            let result = mul_mod(a, b, m);

            // Then
            let expected = ((a as u128 * b as u128) % m as u128) as u64;
            assert_eq!(expected, result);
        }
    }

    // pow_mod のテスト: 戻り値を検証する。
    mod pow_mod {
        use super::*;

        /// Scenario: 指数が `0` の場合、`1 mod m` を返す (境界値)。
        /// - Given: 指数が `0` である。
        /// - When: `pow_mod` を呼ぶ。
        /// - Then: `1 mod m` が返る。
        #[test]
        fn returns_one_when_exponent_is_zero() {
            // Given, When
            let result = pow_mod(5, 0, 7);
            // Then
            assert_eq!(1, result);
        }

        /// Scenario: 典型的な値に対して正しいべき乗の剰余を返す。
        /// - Given: 底、指数、法の典型的な組み合わせがある。
        /// - When: `pow_mod` を呼ぶ。
        /// - Then: 期待したべき乗の剰余が返る。
        #[test]
        fn returns_correct_power_for_typical_values() {
            let cases = [(2_u64, 10_u64, 7_u64, 2_u64), (3, 4, 5, 1)];

            for (a, n, m, expected) in cases {
                // Given, When
                let result = pow_mod(a, n, m);
                // Then
                assert_eq!(expected, result);
            }
        }

        /// Scenario: 法が `1` の場合、常に `0` を返す (境界値)。
        /// - Given: 法が `1` である。
        /// - When: `pow_mod` を呼ぶ。
        /// - Then: `0` が返る。
        #[test]
        fn returns_zero_when_modulus_is_one() {
            // Given, When
            let result = pow_mod(0, 5, 1);
            // Then
            assert_eq!(0, result);
        }
    }

    // mod_inv のテスト: 戻り値を検証する。
    mod mod_inv {
        use super::*;

        /// Scenario: 典型的な値に対して `a * mod_inv(a, m) ≡ 1 (mod m)` を満たす逆元を返す。
        /// - Given: 法が素数であり、`a` がその法と互いに素である。
        /// - When: `mod_inv` を呼ぶ。
        /// - Then: 返った値が `a` の逆元としての性質を満たす。
        #[test]
        fn satisfies_modular_inverse_property_for_typical_values() {
            let cases = [(2_u64, 7_u64), (3, 7), (1, 998244353), (5, 998244353)];

            for (a, m) in cases {
                // Given, When
                let inv = mod_inv(a, m);

                // Then
                assert_eq!(1, mul_mod(a, inv, m));
            }
        }

        /// Scenario: `a` が `1` の場合、逆元は `1` になる (境界値)。
        /// - Given: `a` が `1` である。
        /// - When: `mod_inv` を呼ぶ。
        /// - Then: `1` が返る。
        #[test]
        fn returns_one_when_a_is_one() {
            // Given, When
            let result = mod_inv(1, 7);
            // Then
            assert_eq!(1, result);
        }
    }

    // crt のテスト: 戻り値を検証する。
    mod crt {
        use super::*;

        /// Scenario: 法が互いに素な複数の合同式に対して、統合された合同式を返す。
        /// - Given: 互いに素な法を持つ 3 つの合同式がある。
        /// - When: `crt` を呼ぶ。
        /// - Then: `Some((r, lcm))` の形で、元の連立合同式と同値な解が返る。
        #[test]
        fn returns_merged_congruence_for_pairwise_coprime_moduli() {
            // Given, When
            let result = crt(&[2, 3, 2], &[3, 5, 7]);
            // Then
            assert_eq!(Some((23, 105)), result);
        }

        /// Scenario: 法が互いに素でなくても解が存在すれば、統合された合同式を返す。
        /// - Given: `gcd` が `1` より大きい法を持つ、矛盾しない合同式の組がある。
        /// - When: `crt` を呼ぶ。
        /// - Then: `Some((r, lcm))` の形で、元の連立合同式と同値な解が返る。
        #[test]
        fn returns_merged_congruence_for_non_coprime_moduli_with_solution() {
            let cases = [
                (vec![1_u64, 4], vec![3_u64, 6], Some((4_u64, 6_u64))),
                (vec![2, 5, 0], vec![4, 9, 10], Some((50, 180))),
            ];

            for (remainders, moduli, expected) in cases {
                // Given, When
                let result = crt(&remainders, &moduli);
                // Then
                assert_eq!(expected, result);
            }
        }

        /// Scenario: 合同式が矛盾する場合、`None` を返す (異常系)。
        /// - Given: `gcd` が `1` より大きい法を持ち、互いに矛盾する合同式の組がある。
        /// - When: `crt` を呼ぶ。
        /// - Then: `None` が返る。
        #[test]
        fn returns_none_for_contradictory_congruences() {
            // Given, When
            let result = crt(&[0, 1], &[2, 4]);
            // Then
            assert!(result.is_none());
        }

        /// Scenario: 合同式が 1 つだけの場合、その合同式自身を返す (境界値)。
        /// - Given: 合同式が 1 つだけある。
        /// - When: `crt` を呼ぶ。
        /// - Then: `Some((remainders[0], moduli[0]))` が返る。
        #[test]
        fn returns_single_congruence_as_is_for_single_element() {
            // Given, When
            let result = crt(&[5], &[11]);
            // Then
            assert_eq!(Some((5, 11)), result);
        }

        /// Scenario: 合同式が 1 つもない場合、制約なしを表す単位元を返す (境界値)。
        /// - Given: `remainders` と `moduli` がともに空である。
        /// - When: `crt` を呼ぶ。
        /// - Then: `Some((0, 1))` が返る。
        #[test]
        fn returns_identity_for_empty_input() {
            // Given, When
            let result = crt(&[], &[]);
            // Then
            assert_eq!(Some((0, 1)), result);
        }
    }
}
