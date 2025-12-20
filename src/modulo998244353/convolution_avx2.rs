//! `convolution` における AVX2 による高速化実装を提供する.

use super::convolution;
#[cfg(target_arch = "x86_64")]
use super::convolution_mont;

/// AVX2 + Montgomery により NTT を実行する.
///
/// # Args
/// - `a`: 係数列. `a.len()` は 0 ではない 2 の冪である.
///
/// # Returns
/// `()`: `a` をインプレースで更新する.
///
/// # Constraints
/// - この関数は AVX2 が利用可能な環境でのみ呼び出す.
///
/// # Panics
/// - この関数はパニックしない (debug assert のみ).
///
/// # Complexity
/// - Time complexity: O(n log n), ここで n は `a.len()` である.
/// - Space complexity: O(1).
///
/// # Examples
/// ```rust,ignore
/// // `pub fn ntt` から呼び出される.
/// ```
#[cfg(target_arch = "x86_64")]
pub(super) unsafe fn ntt_avx2(a: &mut [u32]) {
    unsafe {
        debug_assert!(std::is_x86_feature_detected!("avx2"));
        debug_assert!(!a.is_empty());
        debug_assert!(a.len().is_power_of_two());

        convolution_mont::standard_to_mont(a);
        convolution_mont::ntt_mont(a);
        convolution_mont::mont_to_standard(a);
    }
}

/// AVX2 + Montgomery により逆 NTT を実行する (正規化なし).
///
/// # Args
/// - `a`: NTT 値. `a.len()` は 0 ではない 2 の冪である.
///
/// # Returns
/// `()`: `a` をインプレースで更新する.
///
/// # Constraints
/// - この関数は AVX2 が利用可能な環境でのみ呼び出す.
///
/// # Panics
/// - この関数はパニックしない (debug assert のみ).
///
/// # Complexity
/// - Time complexity: O(n log n), ここで n は `a.len()` である.
/// - Space complexity: O(1).
///
/// # Examples
/// ```rust,ignore
/// // `pub fn intt` から呼び出される.
/// ```
#[cfg(target_arch = "x86_64")]
pub(super) unsafe fn intt_avx2(a: &mut [u32]) {
    unsafe {
        debug_assert!(std::is_x86_feature_detected!("avx2"));
        debug_assert!(!a.is_empty());
        debug_assert!(a.len().is_power_of_two());

        convolution_mont::standard_to_mont(a);
        convolution_mont::intt_mont(a);
        convolution_mont::mont_to_standard(a);
    }
}

/// AVX2 を用いて NTT による畳み込みを実行する.
#[cfg(target_arch = "x86_64")]
pub unsafe fn convolution_avx2(mut a: Vec<u32>, mut b: Vec<u32>) -> Vec<u32> {
    if a.is_empty() || b.is_empty() {
        return Vec::new();
    }

    debug_assert!(a.iter().all(|&x| x < convolution::MOD));
    debug_assert!(b.iter().all(|&x| x < convolution::MOD));

    let s = a.len() + b.len() - 1;
    let t = s.next_power_of_two();
    assert!(
        t <= convolution::MAX_NTT_LEN,
        "Convolution length {} exceeds supported maximum {}",
        t,
        convolution::MAX_NTT_LEN
    );

    unsafe {
        if a.len().min(b.len()) <= 32 {
            let mut res = vec![0_u32; s];
            for (i, &ai) in a.iter().enumerate() {
                for (j, &bj) in b.iter().enumerate() {
                    res[i + j] = super::modulo::add(res[i + j], super::modulo::mul(ai, bj));
                }
            }
            return res;
        }

        a.resize(t, 0);
        b.resize(t, 0);

        convolution_mont::standard_to_mont(&mut a);
        convolution_mont::standard_to_mont(&mut b);
        convolution_mont::ntt_mont(&mut a);
        convolution_mont::ntt_mont(&mut b);
        convolution_mont::mul_pointwise_mont(&mut a, &mut b);
        convolution_mont::intt_mont(&mut a);

        let inv_len_mont = convolution_mont::inv_len_mont(t.trailing_zeros() as usize);
        convolution_mont::mul_scalar_mont(&mut a, inv_len_mont);
        convolution_mont::mont_to_standard(&mut a);

        let mut res = Vec::with_capacity(s);
        res.extend_from_slice(&a.as_mut_slice()[..s]);
        res
    }
}
