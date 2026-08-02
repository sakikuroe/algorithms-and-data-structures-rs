//! Bostan-Mori アルゴリズムにより、有理形式的べき級数の係数を計算するモジュールである。

use super::super::modulo;

/// `P(x) / Q(x)` の `x^k` 係数を Bostan-Mori アルゴリズムで求める。
///
/// # Args
/// - `p`: 分子 `P(x)` を表す形式的べき級数。
/// - `q`: 分母 `Q(x)` を表す形式的べき級数。
/// - `k`: 求めたい係数の次数。
///
/// # Returns
/// `u32`: `P(x) / Q(x)` の `x^k` 係数 (mod 998244353)。
///
/// # Complexity
/// - 時間計算量: O(d log d log k)。d は `Q(x)` の次数である。
/// - 空間計算量: O(d)。
///
/// # Examples
/// ```rust
/// use anmitsu::modulo998244353::fps;
/// use anmitsu::modulo998244353::fps::bostan_mori;
/// use anmitsu::modulo998244353::modulo;
///
/// // 1 / (1 - x) = 1 + x + x^2 + ...
/// let p = fps::FPS::new(vec![1]);
/// let q = fps::FPS::new(vec![1, modulo::neg(1)]);
/// assert_eq!(1, bostan_mori::bostan_mori(&p, &q, 10));
/// ```
pub fn bostan_mori(p: &super::FPS, q: &super::FPS, k: usize) -> u32 {
    // 実行環境が AVX2 をサポートする場合は、より高速な実装を選択する。
    #[cfg(target_arch = "x86_64")]
    {
        use std::arch;

        if arch::is_x86_feature_detected!("avx2") {
            return bostan_mori_avx2(p, q, k);
        }
    }

    bostan_mori_scalar(p, q, k)
}

/// `bostan_mori` の非 SIMD 実装である。
///
/// # Args
/// - `p`: 分子 `P(x)` を表す形式的べき級数。
/// - `q`: 分母 `Q(x)` を表す形式的べき級数。
/// - `k`: 求めたい係数の次数。この関数は `k` をインプレースで更新する。
///
/// # Returns
/// `u32`: `P(x) / Q(x)` の `x^k` 係数 (mod 998244353)。
///
/// # Constraints
/// - 制約はない。
///
/// # Panics
/// - この関数はパニックし得る (内部の畳み込み実装に依存する)。
///
/// # Complexity
/// - 時間計算量: O(d log d log k)。d は `Q(x)` の次数である。
/// - 空間計算量: O(d)。
///
/// # Examples
/// ```rust,ignore
/// // `pub fn bostan_mori` から呼び出される。
/// ```
fn bostan_mori_scalar(p: &super::FPS, q: &super::FPS, mut k: usize) -> u32 {
    // 分子・分母を破壊的に更新するため、ここで複製する。
    let mut p = p.clone();
    let mut q = q.clone();

    while k > 0 {
        // Q(-x) を構築するために、奇数次の係数だけ符号反転する。
        let q_neg_x = {
            let mut res = q.clone();
            res.coeffs
                .iter_mut()
                .skip(1)
                .step_by(2)
                .for_each(|x| *x = modulo::neg(*x));
            res
        };

        // P <- P * Q(-x)、Q <- Q * Q(-x) により、偶数次・奇数次の項を分離する。
        p *= q_neg_x.clone();
        q *= q_neg_x;

        // k の偶奇に応じて、分子は偶数次または奇数次の係数だけを抽出する。
        p.coeffs = p.coeffs.into_iter().skip(k % 2).step_by(2).collect();

        // 分母は常に偶数次の係数だけを抽出する。
        q.coeffs = q.coeffs.into_iter().step_by(2).collect();
        p.trim();
        q.trim();

        // 次数を半分に縮約する。
        k /= 2;

        // 係数が十分そろった場合、逆元で商を求めて打ち切る。
        if p.len() > k && q.len() > k {
            return (p * q.inverse(k).unwrap()).get(k);
        }
    }

    // k = 0 まで縮約した場合、定数項の比が答えになる。
    modulo::mul(p.get(0), modulo::inv(q.get(0)))
}

/// AVX2 + Montgomery + NTT doubling によって `P(x) / Q(x)` の `x^k` 係数を計算する。
///
/// # Args
/// - `p`: 分子 `P(x)` を表す形式的べき級数。
/// - `q`: 分母 `Q(x)` を表す形式的べき級数。
/// - `k`: 求めたい係数の次数。
///
/// # Returns
/// `u32`: `P(x) / Q(x)` の `x^k` 係数 (mod 998244353)。
///
/// # Constraints
/// - AVX2 が利用可能な環境でのみ呼び出す。
/// - `2 * n <= MAX_NTT_LEN` を満たさない場合、非 SIMD 実装へフォールバックする。
///
/// # Panics
/// - この関数はパニックし得る (デバッグアサート、および内部の畳み込み実装に依存する)。
///
/// # Complexity
/// - 時間計算量: O(n log n log(k / n))。n は `Q(x)` の長さを 2 冪へ切り上げた値である。
/// - 空間計算量: O(n)。
///
/// # Examples
/// ```rust,ignore
/// // AVX2 依存のため、直接の使用例は省略する。
/// ```
#[cfg(target_arch = "x86_64")]
fn bostan_mori_avx2(p: &super::FPS, q: &super::FPS, k: usize) -> u32 {
    use super::super::{convolution, convolution_mont};
    use std::arch;

    debug_assert!(arch::is_x86_feature_detected!("avx2"));

    // 定数項が 0 の場合、逆元が存在しないため係数は定義できないものとして 0 を返す。
    if q.get(0) == 0 {
        return 0;
    }

    let n = q.len().next_power_of_two().max(1);
    if 2 * n > convolution::MAX_NTT_LEN {
        return bostan_mori_scalar(p, q, k);
    }

    let mut k = k as usize;
    if k < n {
        let mut p = p.coeffs.clone();
        let mut q = q.coeffs.clone();
        p.resize(k + 1, 0);
        q.resize(k + 1, 0);
        return (super::FPS::new(p) * super::FPS::new(q).inverse(k).unwrap()).get(k);
    }

    let lg = n.trailing_zeros() as usize;
    let inv2 = convolution_mont::standard_to_mont_scalar(modulo::inv(2));
    let inv_n = convolution_mont::inv_len_mont(lg);
    let w = build_w_for_pairing(n);

    let mut p = p.coeffs.clone();
    let mut q = q.coeffs.clone();
    p.resize(2 * n, 0);
    q.resize(2 * n, 0);
    unsafe {
        convolution_mont::standard_to_mont(&mut p);
        convolution_mont::standard_to_mont(&mut q);
        convolution_mont::ntt_mont(&mut p);
        convolution_mont::ntt_mont(&mut q);
    }

    while k >= n {
        debug_assert_eq!(2 * n, p.len());
        debug_assert_eq!(2 * n, q.len());

        if k % 2 == 0 {
            for i in 0..n {
                let p0 = p[2 * i];
                let p1 = p[2 * i + 1];
                let q0 = q[2 * i];
                let q1 = q[2 * i + 1];
                let t = modulo::add(
                    convolution_mont::mul_mont(p0, q1),
                    convolution_mont::mul_mont(p1, q0),
                );
                p[i] = convolution_mont::mul_mont(t, inv2);
                q[i] = convolution_mont::mul_mont(q0, q1);
            }
        } else {
            for i in 0..n {
                let p0 = p[2 * i];
                let p1 = p[2 * i + 1];
                let q0 = q[2 * i];
                let q1 = q[2 * i + 1];
                let t = modulo::sub(
                    convolution_mont::mul_mont(p0, q1),
                    convolution_mont::mul_mont(p1, q0),
                );
                p[i] = convolution_mont::mul_mont(t, w[i]);
                q[i] = convolution_mont::mul_mont(q0, q1);
            }
        }

        p.truncate(n);
        q.truncate(n);
        k /= 2;
        if k < n {
            break;
        }
        unsafe {
            convolution_mont::ntt_doubling_mont(&mut p);
            convolution_mont::ntt_doubling_mont(&mut q);
        }
    }

    unsafe {
        convolution_mont::intt_mont(&mut p);
        convolution_mont::intt_mont(&mut q);
        convolution_mont::mul_scalar_mont(&mut p, inv_n);
        convolution_mont::mul_scalar_mont(&mut q, inv_n);
        convolution_mont::mont_to_standard(&mut p);
        convolution_mont::mont_to_standard(&mut q);
    }
    p.truncate(k + 1);
    q.truncate(k + 1);

    (super::FPS::new(p) * super::FPS::new(q).inverse(k).unwrap()).get(k)
}

/// pairing で用いる `inv(2 * x)` の列を構築する。
///
/// # Args
/// - `n`: pairing 後の NTT 長。`n` は 0 ではない 2 の冪である。
///
/// # Returns
/// `Vec<u32>`: `w[i] = inv(2 * x_i)` を満たす列 (Montgomery 表現)、ここで
/// `x_i = ω^(bit_reverse(i, log2 n))` である。
///
/// # Constraints
/// - AVX2 が利用可能な環境でのみ呼び出す。
/// - `2 * n <= MAX_NTT_LEN` を満たす。
///
/// # Panics
/// - この関数はパニックし得る (デバッグアサート)。
///
/// # Complexity
/// - 時間計算量: O(n)。
/// - 空間計算量: O(n)。
///
/// # Examples
/// ```rust,ignore
/// // 内部関数のため、直接の使用例は省略する。
/// ```
#[cfg(target_arch = "x86_64")]
fn build_w_for_pairing(n: usize) -> Vec<u32> {
    use super::super::{convolution, convolution_mont};
    use std::arch;

    debug_assert!(arch::is_x86_feature_detected!("avx2"));
    debug_assert!(n.is_power_of_two());
    debug_assert!(n > 0);
    debug_assert!(2 * n <= convolution::MAX_NTT_LEN);

    let lg = n.trailing_zeros() as usize;
    let omega = modulo::pow(
        convolution::PRIMITIVE_ROOT,
        (modulo::M as usize - 1) / (2 * n),
    );
    let omega_inv = modulo::inv(omega);
    let inv2 = modulo::inv(2);

    let mut res = vec![0_u32; n];
    let mut x_inv = 1_u32;
    for exp in 0..n {
        let i = convolution::bit_reverse(exp, lg);
        res[i] = modulo::mul(inv2, x_inv);
        x_inv = modulo::mul(x_inv, omega_inv);
    }

    unsafe {
        convolution_mont::standard_to_mont(&mut res);
    }
    res
}

/// 線形漸化式で定義された数列の第 `k` 項を求める。
///
/// 次の線形漸化式で定義される数列 `a` を考える。
///
/// `a_i = sum_{j=1..d} c_j a_{i-j} (mod 998244353), i >= d`。
///
/// # Args
/// - `initial_terms`: `a_0..a_{d-1}` の初期項。
/// - `coefficients`: `c_1..c_d` の係数。
/// - `k`: 求めたい項番号。
///
/// # Returns
/// `u32`: `a_k (mod 998244353)`。
///
/// # Constraints
/// - `initial_terms.len() == coefficients.len()`。
/// - `initial_terms.len() >= 1`。
///
/// # Panics
/// - `initial_terms.len() != coefficients.len()` のとき。
/// - `initial_terms.is_empty()` のとき。
///
/// # Complexity
/// - 時間計算量: O(d log d log k)。d は漸化式の次数である。
/// - 空間計算量: O(d)。
///
/// # Examples
/// ```rust
/// use anmitsu::modulo998244353::fps::bostan_mori;
///
/// // Fibonacci: a0 = 1, a1 = 1, a_n = a_{n-1} + a_{n-2}
/// let initial = vec![1, 1];
/// let coeffs = vec![1, 1];
/// assert_eq!(8, bostan_mori::linear_recurrence_kth_term(&initial, &coeffs, 5));
/// ```
pub fn linear_recurrence_kth_term(initial_terms: &[u32], coefficients: &[u32], k: usize) -> u32 {
    assert!(!initial_terms.is_empty());
    assert_eq!(initial_terms.len(), coefficients.len());

    let degree = initial_terms.len();
    if k < degree {
        return initial_terms[k];
    }

    let mut q_coeffs = Vec::with_capacity(degree + 1);
    q_coeffs.push(1);
    for &c in coefficients {
        q_coeffs.push(modulo::neg(c));
    }

    let a = super::FPS::new(initial_terms.to_vec());
    let q = super::FPS::new(q_coeffs);

    let mut p = a * q.clone();
    p.truncate(degree);

    bostan_mori(&p, &q, k)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Background: 1 / (1 - x) = 1 + x + x^2 + ... を表す分子 `p` と分母 `q` の組。
    fn create_geometric_series() -> (super::super::FPS, super::super::FPS) {
        let p = super::super::FPS::new(vec![1]);
        let q = super::super::FPS::new(vec![1, super::modulo::neg(1)]);
        (p, q)
    }

    // bostan_mori のテスト: 戻り値そのものを検証する
    mod bostan_mori {
        use super::*;

        /// Scenario: 幾何級数 1 / (1 - x) の係数は、次数によらず常に 1 である
        /// - Given: 1 / (1 - x) を表す分子と分母の組がある
        /// - When: 次数 0、1、63 のそれぞれについて係数を求める
        /// - Then: いずれの次数でも結果は 1 になる
        #[test]
        fn extracts_geometric_series_coefficients_for_various_degrees() {
            // Given
            let (p, q) = create_geometric_series();

            for k in [0_usize, 1, 63] {
                // When
                let result = bostan_mori(&p, &q, k);

                // Then
                assert_eq!(1, result, "coefficient at degree {k} should be 1");
            }
        }
    }

    // linear_recurrence_kth_term のテスト: 戻り値そのものを検証する
    mod linear_recurrence_kth_term {
        use super::*;

        /// Scenario: フィボナッチ数列の第 k 項が既知の値と一致する
        /// - Given: a0 = 1、a1 = 1、漸化式 a_n = a_{n-1} + a_{n-2} がある
        /// - When: 第 5 項を求める
        /// - Then: 結果は 8 になる
        #[test]
        fn matches_known_fibonacci_value() {
            // Given
            let initial = vec![1, 1];
            let coeffs = vec![1, 1];

            // When
            let result = linear_recurrence_kth_term(&initial, &coeffs, 5);

            // Then
            assert_eq!(8, result);
        }

        /// Scenario: 項番号が初期項の個数未満のとき、初期項をそのまま返す
        /// - Given: 3 項の初期値を持つ数列がある
        /// - When: 項番号 0 と 2 について項を求める
        /// - Then: それぞれ初期項の値がそのまま返る
        #[test]
        fn returns_initial_term_when_k_is_smaller_than_degree() {
            // Given
            let initial = vec![7, 11, 13];
            let coeffs = vec![2, 3, 5];

            for (k, expected) in [(0_usize, 7_u32), (2, 13)] {
                // When
                let result = linear_recurrence_kth_term(&initial, &coeffs, k);

                // Then
                assert_eq!(expected, result);
            }
        }

        /// Scenario: 大きな項番号でも、素朴な漸化式の逐次計算と一致する
        /// - Given: 5 項の初期値と係数を持つ線形漸化式がある
        /// - When: 逐次計算した第 40 項と、`linear_recurrence_kth_term` の結果を比較する
        /// - Then: 両者は一致する
        #[test]
        fn matches_naive_recurrence_for_large_k() {
            // Given
            let degree = 5;
            let initial = vec![1, 2, 3, 4, 5];
            let coeffs = vec![6, 7, 8, 9, 10];
            let k = 40_usize;

            let mut naive = initial.clone();
            for i in degree..=k {
                let mut acc = 0_u32;
                for j in 1..=degree {
                    let term = naive[i - j];
                    let coef = coeffs[j - 1];
                    acc = super::modulo::add(acc, super::modulo::mul(coef, term));
                }
                naive.push(acc);
            }

            // When
            let result = linear_recurrence_kth_term(&initial, &coeffs, k);

            // Then
            assert_eq!(naive[k], result);
        }

        /// Scenario: 初期項と係数の個数が異なる場合はパニックする
        /// - Given: 初期項が 2 個、係数が 1 個の不整合な入力がある
        /// - When: 第 k 項を求める
        /// - Then: パニックする
        #[test]
        #[should_panic]
        fn panics_when_lengths_do_not_match() {
            // Given
            let initial = vec![1, 1];
            let coeffs = vec![1];

            // When, Then
            linear_recurrence_kth_term(&initial, &coeffs, 5);
        }

        /// Scenario: 初期項が空の場合はパニックする
        /// - Given: 初期項も係数も空の入力がある
        /// - When: 第 k 項を求める
        /// - Then: パニックする
        #[test]
        #[should_panic]
        fn panics_when_initial_terms_is_empty() {
            // Given
            let initial = Vec::new();
            let coeffs = Vec::new();

            // When, Then
            linear_recurrence_kth_term(&initial, &coeffs, 0);
        }
    }
}
