//! 法 998244353 の下での基本的なモジュラー演算を提供するモジュールである。

/// 演算で用いる法である。
pub const M: u32 = 998244353;

/// `a` を `[0, M)` の範囲に還元する。
///
/// # Args
/// - `a`: 還元する値。
///
/// # Returns
/// `u32`: 還元後の値。
///
/// # Constraints
/// 制約はない。
///
/// # Panics
/// この関数はパニックしない。
///
/// # Complexity
/// - 時間計算量: O(1)。
///
/// # Examples
/// ```rust
/// use anmitsu::modulo998244353::modulo;
///
/// assert_eq!(42, modulo::modulo(42));
/// assert_eq!(1, modulo::modulo(998244354));
/// ```
pub const fn modulo(a: u64) -> u32 {
    // a を M で割った余りを [0, M) の範囲の値として u32 に収める。
    (a % M as u64) as u32
}

/// 法の下で 2 つの値を加算する。
///
/// # Args
/// - `a`: 左辺の被演算子 (M 未満)。
/// - `b`: 右辺の被演算子 (M 未満)。
///
/// # Returns
/// `u32`: `(a + b) mod M` の値。
///
/// # Constraints
/// - `a` と `b` はいずれも M 未満でなければならない。
///
/// # Panics
/// 被演算子が M 以上の場合にパニックする。
///
/// # Complexity
/// - 時間計算量: O(1)。
///
/// # Examples
/// ```rust
/// use anmitsu::modulo998244353::modulo;
///
/// assert_eq!(3, modulo::add(1, 2));
/// ```
pub const fn add(a: u32, b: u32) -> u32 {
    debug_assert!(a < M);
    debug_assert!(b < M);

    // a, b はいずれも M 未満なので、和 t は 2M 未満に収まり u32 でオーバーフローしない。
    let t = a + b;
    // t が M 以上になった場合のみ M を引き、[0, M) の範囲に戻す。
    if t < M { t } else { t.wrapping_sub(M) }
}

/// 法の下で 2 つの値を減算する。
///
/// # Args
/// - `a`: 左辺の被演算子 (M 未満)。
/// - `b`: 右辺の被演算子 (M 未満)。
///
/// # Returns
/// `u32`: `(a - b) mod M` の値。
///
/// # Constraints
/// - `a` と `b` はいずれも M 未満でなければならない。
///
/// # Panics
/// 被演算子が M 以上の場合にパニックする。
///
/// # Complexity
/// - 時間計算量: O(1)。
///
/// # Examples
/// ```rust
/// use anmitsu::modulo998244353::modulo;
///
/// assert_eq!(1, modulo::sub(3, 2));
/// ```
pub const fn sub(a: u32, b: u32) -> u32 {
    debug_assert!(a < M);
    debug_assert!(b < M);

    // a - b を試み、アンダーフローが発生したかどうかをフラグ f として受け取る。
    let (t, f) = a.overflowing_sub(b);
    // アンダーフローが発生した場合 (a < b の場合) は、M を加えて [0, M) の範囲に戻す。
    if !f { t } else { t.wrapping_add(M) }
}

/// 法の下で 2 つの値を乗算する。
///
/// # Args
/// - `a`: 左辺の被演算子 (M 未満)。
/// - `b`: 右辺の被演算子 (M 未満)。
///
/// # Returns
/// `u32`: `(a * b) mod M` の値。
///
/// # Constraints
/// - `a` と `b` はいずれも M 未満でなければならない。
///
/// # Panics
/// 被演算子が M 以上の場合にパニックする。
///
/// # Complexity
/// - 時間計算量: O(1)。
///
/// # Examples
/// ```rust
/// use anmitsu::modulo998244353::modulo;
///
/// assert_eq!(6, modulo::mul(2, 3));
/// ```
pub const fn mul(a: u32, b: u32) -> u32 {
    debug_assert!(a < M);
    debug_assert!(b < M);

    // a * b は最大で (M - 1)^2 程度になり u32 の範囲を超えるため、u64 に拡張してから乗算する。
    modulo(a as u64 * b as u64)
}

/// `a` の加法逆元を計算する。
///
/// # Args
/// - `a`: 被演算子 (M 未満)。
///
/// # Returns
/// `u32`: `(-a) mod M` の値。
///
/// # Constraints
/// - `a` は M 未満でなければならない。
///
/// # Panics
/// `a` が M 以上の場合にパニックする。
///
/// # Complexity
/// - 時間計算量: O(1)。
///
/// # Examples
/// ```rust
/// use anmitsu::modulo998244353::modulo;
///
/// assert_eq!(0, modulo::neg(0));
/// assert_eq!(998244352, modulo::neg(1));
/// ```
pub const fn neg(a: u32) -> u32 {
    debug_assert!(a < M);
    // 0 の加法逆元は 0 自身であり、それ以外は M から a を引いた値が加法逆元となる。
    if a == 0 { 0 } else { M - a }
}

/// `inv[i] = 1/i (mod M)` を満たす逆元テーブルを `i = 0..len-1` について構築する。
///
/// # Args
/// - `len`: テーブル長。
///
/// # Returns
/// `Vec<u32>`: `i >= 1` について `inv[i] = 1/i (mod M)` を満たすベクターを返す。
///
/// # Constraints
/// - `len` は `M` 未満でなければならない。
///
/// # Panics
/// `len >= M` のときパニックする。
///
/// # Complexity
/// - 時間計算量: O(len)。
/// - 空間計算量: O(len)。
///
/// # Examples
/// ```rust
/// use anmitsu::modulo998244353::modulo;
///
/// let inv = modulo::build_inv_indices(5);
/// assert_eq!(1, inv[1]);
/// assert_eq!(499122177, inv[2]);
/// ```
pub fn build_inv_indices(len: usize) -> Vec<u32> {
    assert!(len < M as usize, "build_inv_indices requires len < modulus");

    if len == 0 {
        return Vec::new();
    }

    // len 個の逆元を格納するテーブルを事前に確保する。inv[0] は未定義のため 0 のままとする。
    let mut inv = vec![0_u32; len];
    if len > 1 {
        // 1 の逆元は 1 自身である。
        inv[1] = 1;
        for i in 2..len {
            // M = q * i + r と表すと、q * i + r ≡ 0 (mod M) が成り立つ。
            // この式の両辺を i * r で割ると q * inv[r] + inv[i] ≡ 0 (mod M) となるため、
            // 既に求めた inv[r] (r < i) を用いて inv[i] = -q * inv[r] (mod M) として計算できる。
            let iu = i as u32;
            let q = M / iu;
            let r = (M % iu) as usize;
            let t = mul(q, inv[r]);
            // -t (mod M) を計算する。t が 0 の場合は M - t が M に等しくなってしまうため、0 に補正する。
            let val = M - t;
            inv[i] = if val == M { 0 } else { val };
        }
    }

    inv
}

/// `a` の `n` 乗を法の下で計算する。
///
/// # Args
/// - `a`: 底 (M 未満)。
/// - `n`: 指数。
///
/// # Returns
/// `u32`: `a^n mod M` の値。
///
/// # Constraints
/// - `a` は M 未満でなければならない。
///
/// # Panics
/// `a` が M 以上の場合にパニックする。
///
/// # Complexity
/// - 時間計算量: O(log n)。
///
/// # Examples
/// ```rust
/// use anmitsu::modulo998244353::modulo;
///
/// assert_eq!(8, modulo::pow(2, 3));
/// ```
pub const fn pow(a: u32, mut n: usize) -> u32 {
    debug_assert!(a < M);

    // 累積する結果。初期値は乗法単位元の 1 である。
    let mut res = 1;
    // 2 乗を繰り返していく途中経過の底。
    let mut x = a;

    // 二分累乗法 (繰り返し二乗法) により、O(log n) 回の乗算で a^n を計算する。
    while n > 0 {
        // n の最下位ビットが 1 の場合、現在の底を結果に掛け合わせる。
        if n % 2 == 1 {
            res = mul(res, x);
        }
        // 底を 2 乗し、次の桁に備える。
        x = mul(x, x);
        n /= 2;
    }

    res
}

/// `a` の乗法逆元を計算する。
///
/// # Args
/// - `a`: 被演算子 (M 未満)。
///
/// # Returns
/// `u32`: `a^-1 mod M` の値。
///
/// # Constraints
/// - `a` は M 未満で、かつゼロであってはならない。
///
/// # Panics
/// `a` が 0 または M 以上の場合にパニックする。
///
/// # Complexity
/// - 時間計算量: O(log M)。
///
/// # Examples
/// ```rust
/// use anmitsu::modulo998244353::modulo;
///
/// let inv_two = modulo::inv(2);
/// assert_eq!(1, modulo::mul(2, inv_two));
/// ```
pub const fn inv(a: u32) -> u32 {
    debug_assert!(a < M);
    debug_assert!(a != 0);
    // フェルマーの小定理より、M が素数のとき a^(M-2) mod M が乗法逆元となる。
    pow(a, M as usize - 2)
}

#[cfg(test)]
mod tests {
    use super::*;

    // add のテスト: 戻り値を検証する。
    mod add {
        use super::*;

        /// Scenario: 通常の値同士を加算すると、単純な和が返る。
        /// - Given: いずれも M 未満の 2 つの値がある。
        /// - When: 加算する。
        /// - Then: 算術的な和が返る。
        #[test]
        fn sums_two_values_without_wrapping() {
            // Given
            let a = 1;
            let b = 2;
            // When
            let result = add(a, b);
            // Then
            assert_eq!(3, result);
        }

        /// Scenario: 和が M 以上になる場合、M を引いた値が返る。
        /// - Given: 和が M を超える 2 つの値がある。
        /// - When: 加算する。
        /// - Then: M で還元された値が返る。
        #[test]
        fn wraps_around_when_sum_reaches_modulus() {
            // Given
            let a = M - 1;
            let b = 2;
            // When
            let result = add(a, b);
            // Then
            assert_eq!(1, result);
        }

        /// Scenario: 境界値 (0 および M - 1) を加算しても正しく計算できる。
        /// - Given: 複数の境界的な入力の組がある。
        /// - When: それぞれ加算する。
        /// - Then: 各ケースで期待した値が返る。
        #[test]
        fn handles_boundary_values() {
            // Given
            let cases = [(0, 0, 0), (0, M - 1, M - 1), (M - 1, M - 1, M - 2)];

            for (a, b, expected) in cases {
                // When
                let result = add(a, b);
                // Then
                assert_eq!(expected, result);
            }
        }

        /// Scenario: 被演算子が M 以上の場合はパニックする。
        /// - Given: M と等しい (M 以上の) 値がある。
        /// - When: 加算する。
        /// - Then: パニックする。
        #[test]
        #[should_panic]
        fn panics_when_operand_is_out_of_range() {
            // Given
            let a = M;
            let b = 0;
            // When, Then (パニックする)
            add(a, b);
        }
    }

    // sub のテスト: 戻り値を検証する。
    mod sub {
        use super::*;

        /// Scenario: 被減数が減数以上の場合、単純な差が返る。
        /// - Given: `a >= b` を満たす 2 つの値がある。
        /// - When: 減算する。
        /// - Then: 算術的な差が返る。
        #[test]
        fn subtracts_two_values_without_wrapping() {
            // Given
            let a = 3;
            let b = 2;
            // When
            let result = sub(a, b);
            // Then
            assert_eq!(1, result);
        }

        /// Scenario: 被減数が減数より小さい場合、M を加えた値が返る。
        /// - Given: `a < b` を満たす 2 つの値がある。
        /// - When: 減算する。
        /// - Then: M で還元された値が返る。
        #[test]
        fn wraps_around_when_result_would_be_negative() {
            // Given
            let a = 1;
            let b = 2;
            // When
            let result = sub(a, b);
            // Then
            assert_eq!(M - 1, result);
        }

        /// Scenario: 境界値 (0 および M - 1) を減算しても正しく計算できる。
        /// - Given: 複数の境界的な入力の組がある。
        /// - When: それぞれ減算する。
        /// - Then: 各ケースで期待した値が返る。
        #[test]
        fn handles_boundary_values() {
            // Given
            let cases = [(0, 0, 0), (M - 1, M - 1, 0), (0, M - 1, 1)];

            for (a, b, expected) in cases {
                // When
                let result = sub(a, b);
                // Then
                assert_eq!(expected, result);
            }
        }

        /// Scenario: 被演算子が M 以上の場合はパニックする。
        /// - Given: M と等しい (M 以上の) 値がある。
        /// - When: 減算する。
        /// - Then: パニックする。
        #[test]
        #[should_panic]
        fn panics_when_operand_is_out_of_range() {
            // Given
            let a = 0;
            let b = M;
            // When, Then (パニックする)
            sub(a, b);
        }
    }

    // mul のテスト: 戻り値を検証する。
    mod mul {
        use super::*;

        /// Scenario: 通常の値同士を乗算すると、法で還元された積が返る。
        /// - Given: いずれも M 未満の 2 つの値がある。
        /// - When: 乗算する。
        /// - Then: `(a * b) mod M` が返る。
        #[test]
        fn multiplies_two_values() {
            // Given
            let a = 2;
            let b = 3;
            // When
            let result = mul(a, b);
            // Then
            assert_eq!(6, result);
        }

        /// Scenario: 0 との乗算は常に 0 になる。
        /// - Given: 0 と M - 1 がある。
        /// - When: 乗算する。
        /// - Then: 0 が返る。
        #[test]
        fn returns_zero_when_operand_is_zero() {
            // Given
            let a = 0;
            let b = M - 1;
            // When
            let result = mul(a, b);
            // Then
            assert_eq!(0, result);
        }

        /// Scenario: u32 の範囲を超える積になる大きな値同士でも、オーバーフローせずに計算できる。
        /// - Given: いずれも M - 1 に近い、積が u32 を超える 2 つの値がある。
        /// - When: 乗算する。
        /// - Then: u64 での計算結果を M で還元した値が返る。
        #[test]
        fn does_not_overflow_for_large_operands() {
            // Given
            let a = M - 1;
            let b = M - 1;
            // When
            let result = mul(a, b);
            // Then
            assert_eq!(1, result);
        }

        /// Scenario: 被演算子が M 以上の場合はパニックする。
        /// - Given: M と等しい (M 以上の) 値がある。
        /// - When: 乗算する。
        /// - Then: パニックする。
        #[test]
        #[should_panic]
        fn panics_when_operand_is_out_of_range() {
            // Given
            let a = M;
            let b = 1;
            // When, Then (パニックする)
            mul(a, b);
        }
    }

    // neg のテスト: 戻り値を検証する。
    mod neg {
        use super::*;

        /// Scenario: 0 の加法逆元は 0 自身になる。
        /// - Given: 0 がある。
        /// - When: 加法逆元を計算する。
        /// - Then: 0 が返る。
        #[test]
        fn returns_zero_for_zero() {
            // Given
            let a = 0;
            // When
            let result = neg(a);
            // Then
            assert_eq!(0, result);
        }

        /// Scenario: 非ゼロの値の加法逆元は、元の値と加算すると 0 になる。
        /// - Given: 1 と M - 1 がある。
        /// - When: それぞれ加法逆元を計算する。
        /// - Then: `M - a` が返り、元の値との和は 0 になる。
        #[test]
        fn returns_additive_inverse_for_nonzero_values() {
            // Given
            let cases = [1, M - 1];

            for a in cases {
                // When
                let result = neg(a);
                // Then
                assert_eq!(M - a, result);
                assert_eq!(0, add(a, result));
            }
        }

        /// Scenario: 被演算子が M 以上の場合はパニックする。
        /// - Given: M と等しい値がある。
        /// - When: 加法逆元を計算する。
        /// - Then: パニックする。
        #[test]
        #[should_panic]
        fn panics_when_operand_is_out_of_range() {
            // Given, When, Then (パニックする)
            neg(M);
        }
    }

    // pow のテスト: 戻り値を検証する。
    mod pow {
        use super::*;

        /// Scenario: 通常の底と指数に対して、べき乗が計算できる。
        /// - Given: 底 2、指数 3 がある。
        /// - When: べき乗を計算する。
        /// - Then: 8 が返る。
        #[test]
        fn computes_power_for_typical_values() {
            // Given
            let a = 2;
            let n = 3;
            // When
            let result = pow(a, n);
            // Then
            assert_eq!(8, result);
        }

        /// Scenario: 指数が 0 の場合は、底に関わらず 1 になる。
        /// - Given: 底 12345、指数 0 がある。
        /// - When: べき乗を計算する。
        /// - Then: 1 が返る。
        #[test]
        fn returns_one_when_exponent_is_zero() {
            // Given
            let a = 12345;
            let n = 0;
            // When
            let result = pow(a, n);
            // Then
            assert_eq!(1, result);
        }

        /// Scenario: 指数が 1 の場合は、底そのものが返る。
        /// - Given: 底 12345、指数 1 がある。
        /// - When: べき乗を計算する。
        /// - Then: 底と同じ値が返る。
        #[test]
        fn returns_base_when_exponent_is_one() {
            // Given
            let a = 12345;
            let n = 1;
            // When
            let result = pow(a, n);
            // Then
            assert_eq!(a, result);
        }

        /// Scenario: 被演算子が M 以上の場合はパニックする。
        /// - Given: M と等しい値がある。
        /// - When: べき乗を計算する。
        /// - Then: パニックする。
        #[test]
        #[should_panic]
        fn panics_when_operand_is_out_of_range() {
            // Given, When, Then (パニックする)
            pow(M, 1);
        }
    }

    // inv のテスト: 戻り値を検証する。
    mod inv {
        use super::*;

        /// Scenario: 値の逆元は、元の値と乗算すると乗法単位元 1 になる。
        /// - Given: 7 がある。
        /// - When: 逆元を計算し、元の値と乗算する。
        /// - Then: 積は 1 になる。
        #[test]
        fn multiplies_to_one_with_original_value() {
            // Given
            let value = 7;
            // When
            let result = inv(value);
            // Then
            assert_eq!(1, mul(value, result));
        }

        /// Scenario: 境界値 (1 および M - 1) についても逆元が正しく計算できる。
        /// - Given: 1 と M - 1 がある。
        /// - When: それぞれ逆元を計算する。
        /// - Then: いずれも元の値との積が 1 になる。
        #[test]
        fn handles_boundary_values() {
            // Given
            let cases = [1, M - 1];

            for value in cases {
                // When
                let result = inv(value);
                // Then
                assert_eq!(1, mul(value, result));
            }
        }

        /// Scenario: 0 に対しては逆元が存在しないため、パニックする。
        /// - Given: 0 がある。
        /// - When: 逆元を計算する。
        /// - Then: パニックする。
        #[test]
        #[should_panic]
        fn panics_for_zero() {
            // Given, When, Then (パニックする)
            inv(0);
        }

        /// Scenario: 被演算子が M 以上の場合はパニックする。
        /// - Given: M と等しい値がある。
        /// - When: 逆元を計算する。
        /// - Then: パニックする。
        #[test]
        #[should_panic]
        fn panics_when_operand_is_out_of_range() {
            // Given, When, Then (パニックする)
            inv(M);
        }
    }

    // build_inv_indices のテスト: 戻り値を検証する。
    mod build_inv_indices {
        use super::*;

        /// Background: 長さ 5 の逆元テーブル
        fn create_table() -> Vec<u32> {
            build_inv_indices(5)
        }

        /// Scenario: `len == 0` の場合は空のベクターが返る。
        /// - Given: 長さ 0 が指定されている。
        /// - When: テーブルを構築する。
        /// - Then: 空のベクターが返る。
        #[test]
        fn returns_empty_vec_for_zero_length() {
            // Given
            let len = 0;
            // When
            let result = build_inv_indices(len);
            // Then
            assert!(result.is_empty());
        }

        /// Scenario: `len == 1` の場合は要素数 1 のベクターが返る。
        /// - Given: 長さ 1 が指定されている。
        /// - When: テーブルを構築する。
        /// - Then: 未定義の `inv[0]` のみを持つベクターが返る。
        #[test]
        fn returns_single_element_vec_for_length_one() {
            // Given
            let len = 1;
            // When
            let result = build_inv_indices(len);
            // Then
            assert_eq!(1, result.len());
        }

        /// Scenario: テーブルの各要素は、対応する添字の乗法逆元になっている。
        /// - Given: 長さ 5 の逆元テーブルがある。
        /// - When: 添字 `i` (`i >= 1`) の要素を取り出す。
        /// - Then: `i` との積が 1 になる。
        #[test]
        fn each_entry_is_multiplicative_inverse_of_its_index() {
            // Given
            let sut = create_table();
            // When, Then
            for i in 1..sut.len() {
                assert_eq!(1, mul(i as u32, sut[i]));
            }
        }

        /// Scenario: `len` が M 以上の場合はパニックする。
        /// - Given: M と等しい長さが指定されている。
        /// - When: テーブルを構築する。
        /// - Then: パニックする。
        #[test]
        #[should_panic]
        fn panics_when_length_reaches_modulus() {
            // Given
            let len = M as usize;
            // When, Then (パニックする)
            build_inv_indices(len);
        }
    }

    // modulo のテスト: 戻り値を検証する。
    mod modulo_fn {
        use super::*;

        /// Scenario: M 未満の値はそのまま返る。
        /// - Given: M 未満の値がある。
        /// - When: 還元する。
        /// - Then: 入力と同じ値が返る。
        #[test]
        fn returns_value_unchanged_when_already_reduced() {
            // Given
            let a = 42;
            // When
            let result = modulo(a);
            // Then
            assert_eq!(42, result);
        }

        /// Scenario: M 以上の値は、M で割った余りに還元される。
        /// - Given: M より大きい値がある。
        /// - When: 還元する。
        /// - Then: `a mod M` が返る。
        #[test]
        fn reduces_value_greater_than_modulus() {
            // Given
            let a = 998244354;
            // When
            let result = modulo(a);
            // Then
            assert_eq!(1, result);
        }

        /// Scenario: 境界値 (0 および u64::MAX) についても正しく還元できる。
        /// - Given: 0 と u64::MAX がある。
        /// - When: それぞれ還元する。
        /// - Then: 各ケースで `a mod M` が返る。
        #[test]
        fn handles_boundary_values() {
            // Given
            let cases = [(0_u64, 0_u32), (u64::MAX, (u64::MAX % M as u64) as u32)];

            for (a, expected) in cases {
                // When
                let result = modulo(a);
                // Then
                assert_eq!(expected, result);
            }
        }
    }
}
