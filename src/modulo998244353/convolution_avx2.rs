//! `convolution` における AVX2 による高速化実装を提供する。

#[cfg(target_arch = "x86_64")]
use super::convolution_mont;
use super::{convolution, modulo};

/// AVX2 + Montgomery により NTT を実行する。
///
/// # Args
/// - `a`: 係数列。`a.len()` は 0 ではない 2 の冪である。
///
/// # Returns
/// `()`: `a` をインプレースで更新する。
///
/// # Constraints
/// - この関数は AVX2 が利用可能な環境でのみ呼び出す。
///
/// # Panics
/// - この関数はパニックしない (debug assert のみ)。
///
/// # Complexity
/// - Time complexity: O(n log n)、ここで n は `a.len()` である。
/// - Space complexity: O(1)。
///
/// # Examples
/// ```rust,ignore
/// // `pub fn ntt` から呼び出される。
/// ```
#[cfg(target_arch = "x86_64")]
pub(super) unsafe fn ntt_avx2(a: &mut [u32]) {
    unsafe {
        debug_assert!(std::is_x86_feature_detected!("avx2"));
        debug_assert!(!a.is_empty());
        debug_assert!(a.len().is_power_of_two());

        // Montgomery 乗算は通常表現の値をそのままでは扱えないため、変換してから
        // NTT を実行し、呼び出し側には通常表現のまま結果を返す。
        convolution_mont::standard_to_mont(a);
        convolution_mont::ntt_mont(a);
        convolution_mont::mont_to_standard(a);
    }
}

/// AVX2 + Montgomery により逆 NTT を実行する (正規化なし)。
///
/// # Args
/// - `a`: NTT 値。`a.len()` は 0 ではない 2 の冪である。
///
/// # Returns
/// `()`: `a` をインプレースで更新する。
///
/// # Constraints
/// - この関数は AVX2 が利用可能な環境でのみ呼び出す。
///
/// # Panics
/// - この関数はパニックしない (debug assert のみ)。
///
/// # Complexity
/// - Time complexity: O(n log n)、ここで n は `a.len()` である。
/// - Space complexity: O(1)。
///
/// # Examples
/// ```rust,ignore
/// // `pub fn intt` から呼び出される。
/// ```
#[cfg(target_arch = "x86_64")]
pub(super) unsafe fn intt_avx2(a: &mut [u32]) {
    unsafe {
        debug_assert!(std::is_x86_feature_detected!("avx2"));
        debug_assert!(!a.is_empty());
        debug_assert!(a.len().is_power_of_two());

        // ntt_avx2 と同様に、Montgomery 表現へ変換してから逆 NTT を実行し、
        // 結果を通常表現へ戻して返す。
        convolution_mont::standard_to_mont(a);
        convolution_mont::intt_mont(a);
        convolution_mont::mont_to_standard(a);
    }
}

/// AVX2 + Montgomery 表現を用いて、998244353 を法とした 2 つの列の畳み込みを計算する。
///
/// # Args
/// - `a`: 法 `MOD` で還元された最初の入力列。この関数は `a` を消費する。
/// - `b`: 法 `MOD` で還元された 2 番目の入力列。この関数は `b` を消費する。
///
/// # Returns
/// `Vec<u32>`: 法 `MOD` での畳み込み結果。
///
/// # Constraints
/// - この関数は AVX2 が利用可能な環境でのみ呼び出す。
/// - いずれかのベクターが空の場合は空のベクターを返す。
/// - `a.len() + b.len() - 1` は `convolution::MAX_NTT_LEN` を超えてはならない。
///
/// # Panics
/// - 長さ制約に違反した場合にパニックする。
///
/// # Complexity
/// - Time complexity: O((N + M) log K)、ここで N、M は入力長で、K は
///   `N + M - 1` を超えない最小の 2 の冪である。
/// - Space complexity: O(K)。
///
/// # Examples
/// ```rust,ignore
/// // `pub fn convolution` から呼び出される。
/// ```
#[cfg(target_arch = "x86_64")]
pub unsafe fn convolution_avx2(mut a: Vec<u32>, mut b: Vec<u32>) -> Vec<u32> {
    // 定義により、一方でも空列との畳み込みは空列になる。
    if a.is_empty() || b.is_empty() {
        return Vec::new();
    }

    debug_assert!(a.iter().all(|&x| x < convolution::MOD));
    debug_assert!(b.iter().all(|&x| x < convolution::MOD));

    // 結果の長さ `s = |a| + |b| - 1` と、NTT のために必要な 2 の冪長 `t` を求める。
    let s = a.len() + b.len() - 1;
    let t = s.next_power_of_two();
    assert!(
        t <= convolution::MAX_NTT_LEN,
        "Convolution length {} exceeds supported maximum {}",
        t,
        convolution::MAX_NTT_LEN
    );

    unsafe {
        // 短い入力では、NTT の前処理コストが O(|a| |b|) の愚直な畳み込みを
        // 上回ってしまうため、小さい側の長さが閾値以下なら愚直計算で済ませる。
        if a.len().min(b.len()) <= 32 {
            let mut res = vec![0_u32; s];
            for (i, &ai) in a.iter().enumerate() {
                for (j, &bj) in b.iter().enumerate() {
                    res[i + j] = modulo::add(res[i + j], modulo::mul(ai, bj));
                }
            }
            return res;
        }

        // NTT は 2 の冪長でしか実行できないため、長さ `t` までゼロ埋めする。
        a.resize(t, 0);
        b.resize(t, 0);

        // 畳み込み定理により、点ごとの積を NTT 空間で計算してから逆変換すると
        // 元の畳み込みが得られる。全過程を Montgomery 表現のまま行うことで、
        // 通常表現との往復変換の回数を最小限に抑える。
        convolution_mont::standard_to_mont(&mut a);
        convolution_mont::standard_to_mont(&mut b);
        convolution_mont::ntt_mont(&mut a);
        convolution_mont::ntt_mont(&mut b);
        convolution_mont::mul_pointwise_mont(&mut a, &mut b);
        convolution_mont::intt_mont(&mut a);

        // `intt_mont` は正規化を行わないため、ここで `1/t` (Montgomery 表現) を
        // 掛けて辻褄を合わせてから、通常表現へ戻す。
        let inv_len_mont = convolution_mont::inv_len_mont(t.trailing_zeros() as usize);
        convolution_mont::mul_scalar_mont(&mut a, inv_len_mont);
        convolution_mont::mont_to_standard(&mut a);

        // パディング分を含まない、本来の結果長 `s` だけを取り出す。
        // 新しい `Vec` へコピーするのではなく、`a` 自体を切り詰めて返すことで、
        // 要素数分のコピーを丸ごと避ける。
        a.truncate(s);
        a
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Background: この環境で AVX2 が利用可能かどうかを判定する。
    ///
    /// 本モジュールの関数は AVX2 命令を直接発行するため、 AVX2 非対応の環境で
    /// 呼び出すと未定義動作になる。CI 環境によっては AVX2 が利用できない場合も
    /// あるため、 各テストの冒頭でこの関数を確認し、 利用不可能なら早期リターンで
    /// スキップする。
    fn avx2_available() -> bool {
        std::is_x86_feature_detected!("avx2")
    }

    // ntt_avx2 のテスト: 戻り値を検証する。
    mod ntt_avx2 {
        use super::*;

        /// Scenario: 定数項のみを持つ多項式 (デルタ関数) の NTT は、
        /// すべての評価点で `1` になる。
        /// - Given: AVX2 が利用可能な環境で、 `[1, 0, 0, 0, 0, 0, 0, 0]` がある。
        /// - When: `ntt_avx2` を適用する。
        /// - Then: すべての要素が `1` になる。
        #[test]
        fn transforms_impulse_to_all_ones() {
            if !avx2_available() {
                return;
            }
            // Given
            let mut actual = vec![1_u32, 0, 0, 0, 0, 0, 0, 0];

            // When
            unsafe {
                ntt_avx2(&mut actual);
            }

            // Then
            assert_eq!(vec![1_u32; 8], actual);
        }

        /// Scenario: NTT を実行してから逆 NTT を適用すると、 元の値の `n` 倍
        /// (正規化なし) に戻る。
        /// - Given: AVX2 が利用可能な環境で、 長さ 8 の係数列がある。
        /// - When: `ntt_avx2` を適用してから `intt_avx2` を適用し、 `1/n` で正規化する。
        /// - Then: 元の入力と一致する。
        #[test]
        fn round_trip_with_intt_avx2_matches_original_after_scaling() {
            if !avx2_available() {
                return;
            }
            // Given
            let input = vec![1_u32, 2, 3, 4, 5, 6, 7, 8];
            let mut actual = input.clone();

            // When
            unsafe {
                ntt_avx2(&mut actual);
                intt_avx2(&mut actual);
            }
            let inv_len = convolution::INVS[3]; // 998244353 における 8 の逆元
            actual
                .iter_mut()
                .for_each(|x| *x = modulo::mul(*x, inv_len));

            // Then
            assert_eq!(input, actual);
        }
    }

    // intt_avx2 のテスト: 戻り値を検証する。
    mod intt_avx2 {
        use super::*;

        /// Scenario: すべての評価点で `1` になる列の逆 NTT は、 デルタ関数を
        /// `n` 倍 (正規化なし) したものになる。
        /// - Given: AVX2 が利用可能な環境で、 `[1, 1, 1, 1, 1, 1, 1, 1]` がある。
        /// - When: `intt_avx2` を適用する。
        /// - Then: `[8, 0, 0, 0, 0, 0, 0, 0]` になる。
        #[test]
        fn transforms_all_ones_to_scaled_impulse() {
            if !avx2_available() {
                return;
            }
            // Given
            let mut actual = vec![1_u32; 8];

            // When
            unsafe {
                intt_avx2(&mut actual);
            }

            // Then
            let mut expected = vec![0_u32; 8];
            expected[0] = 8;
            assert_eq!(expected, actual);
        }
    }

    // convolution_avx2 のテスト: 戻り値を検証する。
    mod convolution_avx2_fn {
        use super::*;

        /// Scenario: 一方でも空列との畳み込みは空列になる (境界値)。
        /// - Given: AVX2 が利用可能な環境で、 空列と非空列がある。
        /// - When: `convolution_avx2` を実行する。
        /// - Then: 空列が返る。
        #[test]
        fn returns_empty_when_either_input_is_empty() {
            if !avx2_available() {
                return;
            }
            // Given
            let a = Vec::<u32>::new();
            let b = vec![1, 2, 3];

            // When
            let result = unsafe { convolution_avx2(a, b) };

            // Then
            assert!(result.is_empty());
        }

        /// Scenario: 小さい入力 (愚直計算の分岐) に対して期待通りの結果を返す。
        /// - Given: AVX2 が利用可能な環境で、 長さ 3 の係数列が 2 つある。
        /// - When: `convolution_avx2` を実行する。
        /// - Then: 手計算による期待値と一致する。
        #[test]
        fn matches_expected_for_small_inputs() {
            if !avx2_available() {
                return;
            }
            // Given
            let a = vec![1, 2, 3];
            let b = vec![4, 5, 6];
            let expected = vec![4, 13, 28, 27, 18];

            // When
            let result = unsafe { convolution_avx2(a, b) };

            // Then
            assert_eq!(expected, result);
        }

        /// Scenario: NTT を用いる分岐に切り替わる大きさの入力でも、
        /// 愚直に計算した結果と一致する。
        /// - Given: AVX2 が利用可能な環境で、 長さ 40 の係数列が 2 つある。
        /// - When: `convolution_avx2` を実行する。
        /// - Then: 二重ループで愚直に計算した期待値と一致する。
        #[test]
        fn matches_naive_result_when_ntt_branch_is_used() {
            if !avx2_available() {
                return;
            }
            // Given
            let a = (0..40_u32).collect::<Vec<u32>>();
            let b = (0..40_u32).map(|x| x + 1).collect::<Vec<u32>>();
            let expected_len = a.len() + b.len() - 1;
            let mut expected = vec![0_u32; expected_len];
            for (i, &x) in a.iter().enumerate() {
                for (j, &y) in b.iter().enumerate() {
                    expected[i + j] = modulo::add(expected[i + j], modulo::mul(x, y));
                }
            }

            // When
            let result = unsafe { convolution_avx2(a, b) };

            // Then
            assert_eq!(expected, result);
        }

        /// Scenario: 結果長が `MAX_NTT_LEN` を超える場合はパニックする (異常系)。
        /// - Given: パディング後の長さが `MAX_NTT_LEN` を超える 2 つの係数列がある。
        /// - When: `convolution_avx2` を実行する。
        /// - Then: パニックする。
        ///
        /// 長さの検証は AVX2 命令を発行するより前に行われるため、 このテストは
        /// AVX2 の利用可否によらず安全に実行できる。
        #[test]
        #[should_panic(expected = "Convolution length")]
        fn panics_when_length_exceeds_max_ntt_len() {
            // Given
            let len_a = convolution::MAX_NTT_LEN;
            let len_b = 64;
            let a = vec![1_u32; len_a];
            let b = vec![1_u32; len_b];

            // When, Then (panic)
            let _ = unsafe { convolution_avx2(a, b) };
        }
    }
}
