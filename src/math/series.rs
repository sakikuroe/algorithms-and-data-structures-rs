//! 等差数列・等比数列の和を、任意の法の下で計算する機能を提供するモジュールである。
//!
//! `modulo` は `u32` 型で受け取るが、内部では `crate::math::modular_arithmetic` の
//! 関数群 (`u64` を扱う) を用いて計算し、最後に `u32` へキャストして返す。

use crate::math::modular_arithmetic;

/// 初項 `a`、公差 `d` の等差数列の、最初の `n` 項の和
/// `Σ_{i=0}^{n-1} (a + i*d)` を法 `modulo` の下で計算する。
///
/// # Args
/// - `a` - 等差数列の初項
/// - `d` - 等差数列の公差
/// - `n` - 項数
/// - `modulo` - 法であり、`0` より大きい必要がある
///
/// # Returns
/// `Σ_{i=0}^{n-1} (a + i*d) mod modulo`
///
/// # Complexity
/// - 時間計算量: $O(1)$
///
/// # Examples
/// ```
/// use anmitsu::math::series;
///
/// // 1 + 2 + 3 + 4 + 5 = 15
/// assert_eq!(15, series::sum_arithmetic(1, 1, 5, 1_000_000_007));
///
/// // 項数が 0 の場合は 0 を返す。
/// assert_eq!(0, series::sum_arithmetic(3, 4, 0, 1_000_000_007));
/// ```
#[must_use]
pub fn sum_arithmetic(a: u64, d: u64, n: u64, modulo: u32) -> u32 {
    let m = modulo as u64;
    debug_assert!(m != 0);

    if n == 0 {
        return 0;
    }

    // 閉じた式 S = n*a + d*n*(n-1)/2 における n*(n-1) は必ず偶数になる。
    // n の偶奇に応じて割り切れる方を選び、mod 演算に入れる前に整数のまま
    // 2 で割ることで、剰余を取る前の値を正確に保つ。
    let (half, other) = if n % 2 == 0 {
        (n / 2, n - 1)
    } else {
        ((n - 1) / 2, n)
    };

    let term1 = modular_arithmetic::mul_mod(n % m, a % m, m);
    let term2 = modular_arithmetic::mul_mod(
        modular_arithmetic::mul_mod(d % m, half % m, m),
        other % m,
        m,
    );

    modular_arithmetic::add_mod(term1, term2, m) as u32
}

/// 初項 `a`、公比 `r` の等比数列の、最初の `n` 項の和
/// `Σ_{i=0}^{n-1} a*r^i` を法 `modulo` の下で計算する。
///
/// # Args
/// - `a` - 等比数列の初項
/// - `r` - 等比数列の公比
/// - `n` - 項数
/// - `modulo` - 法であり、`0` より大きい必要がある
///
/// # Returns
/// `Σ_{i=0}^{n-1} a*r^i mod modulo`
///
/// # Complexity
/// - 時間計算量: $O(\log n)$
///
/// # Examples
/// ```
/// use anmitsu::math::series;
///
/// // 1 + 2 + 4 + 8 + 16 = 31
/// assert_eq!(31, series::sum_geometric(1, 2, 5, 1_000_000_007));
///
/// // 公比が 1 の場合は a*n に退化する。
/// assert_eq!(5, series::sum_geometric(3, 1, 4, 7));
/// ```
#[must_use]
pub fn sum_geometric(a: u64, r: u64, mut n: u64, modulo: u32) -> u32 {
    let m = modulo as u64;
    debug_assert!(m != 0);

    // ブロック長 2^k の区間の和を倍加させながら計算する (doubling)。
    // t: 現在のブロックの和 Σ_{i=0}^{2^k - 1} a*r^i。
    // r0: 現在のブロックにおける公比の累乗 r^{2^k}。
    // r1: 確定済みの項数だけ後ろにずらすためのオフセット倍率 r^{確定済み項数}。
    let mut t = a % m;
    let mut r0 = r % m;
    let mut r1 = 1 % m;
    let mut res = 0 % m;

    while n > 0 {
        // n の最下位ビットが立っている場合、現在のブロックにオフセットを
        // 掛けて結果へ加算し、オフセットをブロック長ぶん進める。
        if n & 1 == 1 {
            res = modular_arithmetic::add_mod(res, modular_arithmetic::mul_mod(t, r1, m), m);
            r1 = modular_arithmetic::mul_mod(r1, r0, m);
        }

        // ブロック長を 2 倍にする。和は t*(1 + r0)、公比の累乗は r0^2 になる。
        t = modular_arithmetic::add_mod(t, modular_arithmetic::mul_mod(r0, t, m), m);
        r0 = modular_arithmetic::mul_mod(r0, r0, m);
        n >>= 1;
    }

    res as u32
}

/// (等差数列)×(等比数列) の積の和
/// `Σ_{i=0}^{n-1} (a + i*d) * (b*r^i)` を法 `modulo` の下で計算する。
///
/// # Args
/// - `a` - 等差数列の初項
/// - `d` - 等差数列の公差
/// - `b` - 等比数列の初項
/// - `r` - 等比数列の公比
/// - `n` - 項数
/// - `modulo` - 法であり、`0` より大きい必要がある
///
/// # Returns
/// `Σ_{i=0}^{n-1} (a + i*d) * (b*r^i) mod modulo`
///
/// # Complexity
/// - 時間計算量: $O(\log n)$
///
/// # Examples
/// ```
/// use anmitsu::math::series;
///
/// // (1+0)*1 + (1+1)*2 + (1+2)*4 + (1+3)*8 = 1 + 4 + 12 + 32 = 49
/// assert_eq!(49, series::sum_arithmetic_geometric(1, 1, 1, 2, 4, 1_000_000_007));
/// ```
#[must_use]
pub fn sum_arithmetic_geometric(a: u64, d: u64, b: u64, r: u64, mut n: u64, modulo: u32) -> u32 {
    let m = modulo as u64;
    debug_assert!(m != 0);

    let a = a % m;
    let d = d % m;
    let b = b % m;

    // sum_geometric と同様にブロック長 2^k の区間を倍加させながら計算する。
    // t: 現在のブロックにおける Σ_{i=0}^{2^k-1} (a+i*d)*(b*r^i) の値。
    // g: 現在のブロックの等比部分の値 b*r^i を、ブロックの倍加に合わせて
    //    引き継ぐための補助変数。
    // s: 現在のブロックにおける等比部分の単純和 Σ_{i=0}^{2^k-1} r^i。
    // r0: 現在のブロックにおける公比の累乗 r^{2^k}。
    // p: 現在のブロック長 2^k。
    // r1: 確定済みの項数だけ後ろにずらすためのオフセット倍率 r^{確定済み項数}。
    // cnt: これまでに確定した項数。
    let mut t = modular_arithmetic::mul_mod(a, b, m);
    let mut s = 1 % m;
    let mut res = 0 % m;
    let mut cnt = 0 % m;
    let mut r0 = r % m;
    let mut r1 = 1 % m;
    let mut g = b;
    let mut p = 1 % m;

    while n > 0 {
        // n の最下位ビットが立っている場合、現在のブロックを結果に加算する。
        // 等差部分は確定済みの項数 cnt の分だけ b*d*s を底上げしてから加える。
        if n & 1 == 1 {
            let offset = modular_arithmetic::mul_mod(
                modular_arithmetic::mul_mod(cnt, b, m),
                modular_arithmetic::mul_mod(d, s, m),
                m,
            );
            let block_sum = modular_arithmetic::add_mod(t, offset, m);
            res =
                modular_arithmetic::add_mod(res, modular_arithmetic::mul_mod(r1, block_sum, m), m);
            r1 = modular_arithmetic::mul_mod(r1, r0, m);
            cnt = modular_arithmetic::add_mod(cnt, p, m);
        }

        // ブロック長を 2 倍にする。
        let r0_p_d_g = modular_arithmetic::mul_mod(
            modular_arithmetic::mul_mod(r0, p, m),
            modular_arithmetic::mul_mod(d, g, m),
            m,
        );
        t = modular_arithmetic::add_mod(
            modular_arithmetic::add_mod(t, modular_arithmetic::mul_mod(r0, t, m), m),
            r0_p_d_g,
            m,
        );
        g = modular_arithmetic::add_mod(g, modular_arithmetic::mul_mod(r0, g, m), m);
        s = modular_arithmetic::add_mod(s, modular_arithmetic::mul_mod(r0, s, m), m);
        r0 = modular_arithmetic::mul_mod(r0, r0, m);
        p = modular_arithmetic::add_mod(p, p, m);
        n >>= 1;
    }

    res as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    // sum_arithmetic のテスト: 戻り値を検証する。
    mod sum_arithmetic {
        use super::*;

        /// Scenario: 項数が `0` の場合、和は `0` になる (境界値)。
        /// - Given: 項数 `n` が `0` である。
        /// - When: `sum_arithmetic` を呼ぶ。
        /// - Then: `0` が返る。
        #[test]
        fn returns_zero_when_n_is_zero() {
            // Given, When
            let result = sum_arithmetic(3, 4, 0, 1_000_000_007);
            // Then
            assert_eq!(0, result);
        }

        /// Scenario: 典型的な値に対して等差数列の和を返す (`n` の偶奇双方を含む)。
        /// - Given: 初項・公差・項数・法の典型的な組み合わせがあり、項数には偶数・奇数の
        ///   両方が含まれる。
        /// - When: `sum_arithmetic` を呼ぶ。
        /// - Then: 期待した和が返る。
        #[test]
        fn returns_correct_sum_for_typical_values() {
            let cases = [
                // n = 5 (奇数): 1+2+3+4+5 = 15
                (1_u64, 1_u64, 5_u64, 1_000_000_007_u32, 15_u32),
                // n = 4 (偶数): 2+5+8+11 = 26
                (2, 3, 4, 1_000_000_007, 26),
                // 法による還元が発生する場合
                (5, 5, 100, 7, 1),
            ];

            for (a, d, n, modulo, expected) in cases {
                // Given, When
                let result = sum_arithmetic(a, d, n, modulo);
                // Then
                assert_eq!(expected, result);
            }
        }
    }

    // sum_geometric のテスト: 戻り値を検証する。
    mod sum_geometric {
        use super::*;

        /// Scenario: 項数が `0` の場合、和は `0` になる (境界値)。
        /// - Given: 項数 `n` が `0` である。
        /// - When: `sum_geometric` を呼ぶ。
        /// - Then: `0` が返る。
        #[test]
        fn returns_zero_when_n_is_zero() {
            // Given, When
            let result = sum_geometric(3, 5, 0, 1_000_000_007);
            // Then
            assert_eq!(0, result);
        }

        /// Scenario: 公比が `1` の場合、和は `a*n` に退化する (境界値)。
        /// - Given: 公比 `r` が `1` である。
        /// - When: `sum_geometric` を呼ぶ。
        /// - Then: `a*n mod modulo` が返る。
        #[test]
        fn returns_a_times_n_when_ratio_is_one() {
            // Given, When
            let result = sum_geometric(3, 1, 4, 7);
            // Then
            assert_eq!(5, result);
        }

        /// Scenario: 典型的な値に対して等比数列の和を返す。
        /// - Given: 初項・公比・項数・法の典型的な組み合わせがある。
        /// - When: `sum_geometric` を呼ぶ。
        /// - Then: 期待した和が返る。
        #[test]
        fn returns_correct_sum_for_typical_values() {
            let cases = [
                // 1+2+4+8+16 = 31
                (1_u64, 2_u64, 5_u64, 1_000_000_007_u32, 31_u32),
                // 3+15+75 = 93
                (3, 5, 3, 1_000_000_007, 93),
                // 法による還元が発生する場合
                (2, 3, 10, 13, 2),
            ];

            for (a, r, n, modulo, expected) in cases {
                // Given, When
                let result = sum_geometric(a, r, n, modulo);
                // Then
                assert_eq!(expected, result);
            }
        }
    }

    // sum_arithmetic_geometric のテスト: 戻り値を検証する。
    mod sum_arithmetic_geometric {
        use super::*;

        /// Scenario: 項数が `0` の場合、和は `0` になる (境界値)。
        /// - Given: 項数 `n` が `0` である。
        /// - When: `sum_arithmetic_geometric` を呼ぶ。
        /// - Then: `0` が返る。
        #[test]
        fn returns_zero_when_n_is_zero() {
            // Given, When
            let result = sum_arithmetic_geometric(1, 2, 3, 4, 0, 1_000_000_007);
            // Then
            assert_eq!(0, result);
        }

        /// Scenario: 公差が `0` の場合、`sum_geometric` の `a*b` 倍に退化する (性質検証)。
        /// - Given: 公差 `d` が `0` である。
        /// - When: `sum_arithmetic_geometric` と、同じ `n`, `r`, `modulo` で
        ///   `sum_geometric(1, r, n, modulo)` をそれぞれ呼ぶ。
        /// - Then: 前者が後者の `a*b` 倍 (mod `modulo`) に等しくなる。
        #[test]
        fn returns_a_times_b_times_geometric_sum_when_d_is_zero() {
            // Given
            let (a, b, r, n, modulo) = (6_u64, 7_u64, 4_u64, 5_u64, 1_000_000_007_u32);
            let m = modulo as u64;

            // When
            let result = sum_arithmetic_geometric(a, 0, b, r, n, modulo);
            let geometric_sum = sum_geometric(1, r, n, modulo);

            // Then
            let expected = modular_arithmetic::mul_mod(
                modular_arithmetic::mul_mod(a % m, b % m, m),
                geometric_sum as u64 % m,
                m,
            ) as u32;
            assert_eq!(expected, result);
        }

        /// Scenario: 典型的な値に対して (等差数列)×(等比数列) の積の和を返す。
        /// - Given: 等差数列・等比数列・項数・法の典型的な組み合わせがある。
        /// - When: `sum_arithmetic_geometric` を呼ぶ。
        /// - Then: 期待した和が返る。
        #[test]
        fn returns_correct_sum_for_typical_values() {
            let cases = [
                // (1+0)*1 + (1+1)*2 + (1+2)*4 + (1+3)*8 = 49
                (1_u64, 1_u64, 1_u64, 2_u64, 4_u64, 1_000_000_007_u32, 49_u32),
                (2, 1, 3, 2, 5, 1_000_000_007, 480),
                // 法による還元が発生する場合
                (1, 2, 1, 3, 6, 17, 8),
            ];

            for (a, d, b, r, n, modulo, expected) in cases {
                // Given, When
                let result = sum_arithmetic_geometric(a, d, b, r, n, modulo);
                // Then
                assert_eq!(expected, result);
            }
        }
    }
}
