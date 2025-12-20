//! 998244353 を法として number theoretic transform を用いた畳み込みを提供する.

use super::modulo;

/// Montgomery 表現を用いた高速化実装を提供する.

/// この畳み込み実装で用いる法.
pub const MOD: u32 = 998244353;

/// 変換がサポートする最大長.
pub const MAX_NTT_LEN: usize = 1 << 19;

/// 小さい長さの NTT/INTT に対して愚直計算へ切り替える閾値である.
///
/// 実測により 8, 16, 32 などへ調整することを想定する.
const NTT_NAIVE_THRESHOLD: usize = 32;

/// `2^lg <= NTT_NAIVE_THRESHOLD` を満たす最大の `lg` の値である.
const NTT_NAIVE_LG_MAX: usize =
    (usize::BITS as usize - 1) - (NTT_NAIVE_THRESHOLD.leading_zeros() as usize);

/// 愚直 NTT で用いる `ω^i` のテーブルである.
///
/// `table[lg][i] = ω^i` を表し, `ω` は長さ `2^lg` の原始根である.
/// `i >= 2^lg` の領域は未使用で 0 のままとする.
const NAIVE_NTT_OMEGA_POWS: [[u32; NTT_NAIVE_THRESHOLD]; NTT_NAIVE_LG_MAX + 1] =
    build_naive_ntt_omega_pows();

/// 愚直 INTT で用いる `ω^{-1}` のテーブルである.
///
/// `omega_inv[lg]` は長さ `2^lg` に対応する `ω^{-1}` を表す.
const NAIVE_INTT_OMEGA_INV: [u32; NTT_NAIVE_LG_MAX + 1] = build_naive_intt_omega_inv();

/// 愚直 NTT で用いる `(ω^k)^j` のテーブルである.
///
/// `table[lg][k][j]` は `(ω^k)^j` を表し, `ω` は長さ `2^lg` の原始根である.
/// `k >= 2^lg` または `j >= 2^lg` の領域は未使用で 0 のままとする.
const NAIVE_NTT_BASE_POWS: [[[u32; NTT_NAIVE_THRESHOLD]; NTT_NAIVE_THRESHOLD];
    NTT_NAIVE_LG_MAX + 1] = build_naive_ntt_base_pows();

/// 愚直 INTT で用いる `(ω^{-j})^k` のテーブルである.
///
/// `table[lg][j][k]` は `(ω^{-j})^k` を表し, `ω` は長さ `2^lg` の原始根である.
/// `j >= 2^lg` または `k >= 2^lg` の領域は未使用で 0 のままとする.
const NAIVE_INTT_BASE_POWS: [[[u32; NTT_NAIVE_THRESHOLD]; NTT_NAIVE_THRESHOLD];
    NTT_NAIVE_LG_MAX + 1] = build_naive_intt_base_pows();

/// 愚直 NTT/INTT で用いるビット反転のテーブルである.
///
/// `table[lg][i]` は, `i` の下位 `lg` bit を反転した値を表す.
/// `i >= 2^lg` の領域は未使用で 0 のままとする.
const NAIVE_BIT_REVERSE: [[usize; NTT_NAIVE_THRESHOLD]; NTT_NAIVE_LG_MAX + 1] =
    build_naive_bit_reverse();

/// 998244353 における原始根である.
pub const PRIMITIVE_ROOT: u32 = 3;

/// NTT の各段で用いる回転因子のテーブルである.
pub const NTT_RATE: [u32; 22] = [
    0x3656d65b, 0x1e5ea9e6, 0x16038782, 0x13caac90, 0x3a9a4cfa, 0x761af21, 0xe372007, 0x3a2be7d4,
    0x23fe18b2, 0x330f5b68, 0x7d37cf9, 0x3239edef, 0x2b8ea5c3, 0x382d2452, 0x300e9be2, 0x908b3f5,
    0x1e726cd9, 0x1e02c2f0, 0x2c49629c, 0x2c2b7c93, 0x35a5081, 0x33b69d8b,
];

/// 逆 NTT の各段で用いる回転因子のテーブルである.
pub const INTT_RATE: [u32; 22] = [
    0x52929a6, 0x163456b8, 0x16400573, 0x267c5b5f, 0x6b059a5, 0x294c15f1, 0x94415d9, 0x2f83389c,
    0x569c0ec, 0x3346ebba, 0x37473ab0, 0x1524e16f, 0x68442e3, 0x117ab9d0, 0x1fe52df0, 0x1263f553,
    0x7392943, 0x24433aa8, 0x1a2993eb, 0x156d2fbf, 0x311e570f, 0x6294a13,
];

/// `2^k` (k = 0..=22) の `MOD` における逆元のテーブルである.
pub const INVS: [u32; 23] = [
    1, 499122177, 748683265, 873463809, 935854081, 967049217, 982646785, 990445569, 994344961,
    996294657, 997269505, 997756929, 998000641, 998122497, 998183425, 998213889, 998229121,
    998236737, 998240545, 998242449, 998243401, 998243877, 998244115,
];

/// 愚直 NTT の `ω^i` テーブルを構築する.
///
/// # Args
/// - `()`: 引数はない.
///
/// # Returns
/// `[[u32; NTT_NAIVE_THRESHOLD]; NTT_NAIVE_LG_MAX + 1]`: `ω^i` テーブル.
///
/// # Constraints
/// - `NTT_NAIVE_THRESHOLD > 0` である.
///
/// # Panics
/// - この関数はパニックしない.
///
/// # Complexity
/// - Time complexity: O(NTT_NAIVE_THRESHOLD).
/// - Space complexity: O(1).
///
/// # Examples
/// ```rust,ignore
/// // 内部関数のため, 直接の使用例は省略する.
/// ```
const fn build_naive_ntt_omega_pows() -> [[u32; NTT_NAIVE_THRESHOLD]; NTT_NAIVE_LG_MAX + 1] {
    let mut table = [[0_u32; NTT_NAIVE_THRESHOLD]; NTT_NAIVE_LG_MAX + 1];
    let mut lg = 0usize;
    while lg <= NTT_NAIVE_LG_MAX {
        let n = 1usize << lg;
        let exp = (MOD as usize - 1) / n;
        let omega = modulo::pow(PRIMITIVE_ROOT, exp);
        let mut p = 1_u32;
        let mut i = 0usize;
        while i < n {
            table[lg][i] = p;
            p = modulo::mul(p, omega);
            i += 1;
        }
        lg += 1;
    }
    table
}

/// 愚直 INTT の `ω^{-1}` テーブルを構築する.
///
/// # Args
/// - `()`: 引数はない.
///
/// # Returns
/// `[u32; NTT_NAIVE_LG_MAX + 1]`: `ω^{-1}` テーブル.
///
/// # Constraints
/// - `NTT_NAIVE_THRESHOLD > 0` である.
///
/// # Panics
/// - この関数はパニックしない.
///
/// # Complexity
/// - Time complexity: O(NTT_NAIVE_LG_MAX).
/// - Space complexity: O(1).
///
/// # Examples
/// ```rust,ignore
/// // 内部関数のため, 直接の使用例は省略する.
/// ```
const fn build_naive_intt_omega_inv() -> [u32; NTT_NAIVE_LG_MAX + 1] {
    let mut table = [0_u32; NTT_NAIVE_LG_MAX + 1];
    let mut lg = 0usize;
    while lg <= NTT_NAIVE_LG_MAX {
        let n = 1usize << lg;
        let exp = (MOD as usize - 1) / n;
        let omega = modulo::pow(PRIMITIVE_ROOT, exp);
        table[lg] = modulo::inv(omega);
        lg += 1;
    }
    table
}

/// 愚直 NTT の基底べき乗テーブルを構築する.
///
/// # Args
/// - `()`: 引数はない.
///
/// # Returns
/// `[[[u32; NTT_NAIVE_THRESHOLD]; NTT_NAIVE_THRESHOLD]; NTT_NAIVE_LG_MAX + 1]`:
/// `(ω^k)^j` テーブル.
///
/// # Constraints
/// - `NTT_NAIVE_THRESHOLD > 0` である.
///
/// # Panics
/// - この関数はパニックしない.
///
/// # Complexity
/// - Time complexity: O(NTT_NAIVE_THRESHOLD^2).
/// - Space complexity: O(1).
///
/// # Examples
/// ```rust,ignore
/// // 内部関数のため, 直接の使用例は省略する.
/// ```
const fn build_naive_ntt_base_pows()
-> [[[u32; NTT_NAIVE_THRESHOLD]; NTT_NAIVE_THRESHOLD]; NTT_NAIVE_LG_MAX + 1] {
    let mut table = [[[0_u32; NTT_NAIVE_THRESHOLD]; NTT_NAIVE_THRESHOLD]; NTT_NAIVE_LG_MAX + 1];
    let mut lg = 0usize;
    while lg <= NTT_NAIVE_LG_MAX {
        let n = 1usize << lg;
        let mut k = 0usize;
        while k < n {
            let base = NAIVE_NTT_OMEGA_POWS[lg][k];
            let mut p = 1_u32;
            let mut j = 0usize;
            while j < n {
                table[lg][k][j] = p;
                p = modulo::mul(p, base);
                j += 1;
            }
            k += 1;
        }
        lg += 1;
    }
    table
}

/// 愚直 INTT の基底べき乗テーブルを構築する.
///
/// # Args
/// - `()`: 引数はない.
///
/// # Returns
/// `[[[u32; NTT_NAIVE_THRESHOLD]; NTT_NAIVE_THRESHOLD]; NTT_NAIVE_LG_MAX + 1]`:
/// `(ω^{-j})^k` テーブル.
///
/// # Constraints
/// - `NTT_NAIVE_THRESHOLD > 0` である.
///
/// # Panics
/// - この関数はパニックしない.
///
/// # Complexity
/// - Time complexity: O(NTT_NAIVE_THRESHOLD^2).
/// - Space complexity: O(1).
///
/// # Examples
/// ```rust,ignore
/// // 内部関数のため, 直接の使用例は省略する.
/// ```
const fn build_naive_intt_base_pows()
-> [[[u32; NTT_NAIVE_THRESHOLD]; NTT_NAIVE_THRESHOLD]; NTT_NAIVE_LG_MAX + 1] {
    let mut table = [[[0_u32; NTT_NAIVE_THRESHOLD]; NTT_NAIVE_THRESHOLD]; NTT_NAIVE_LG_MAX + 1];
    let mut lg = 0usize;
    while lg <= NTT_NAIVE_LG_MAX {
        let n = 1usize << lg;
        let omega_inv = NAIVE_INTT_OMEGA_INV[lg];

        let mut omega_inv_pows = [0_u32; NTT_NAIVE_THRESHOLD];
        let mut p = 1_u32;
        let mut j = 0usize;
        while j < n {
            omega_inv_pows[j] = p;
            p = modulo::mul(p, omega_inv);
            j += 1;
        }

        j = 0usize;
        while j < n {
            let base = omega_inv_pows[j];
            let mut p = 1_u32;
            let mut k = 0usize;
            while k < n {
                table[lg][j][k] = p;
                p = modulo::mul(p, base);
                k += 1;
            }
            j += 1;
        }
        lg += 1;
    }
    table
}

/// 愚直 NTT/INTT のビット反転テーブルを構築する.
///
/// # Args
/// - `()`: 引数はない.
///
/// # Returns
/// `[[usize; NTT_NAIVE_THRESHOLD]; NTT_NAIVE_LG_MAX + 1]`: ビット反転テーブル.
///
/// # Constraints
/// - `NTT_NAIVE_THRESHOLD > 0` である.
///
/// # Panics
/// - この関数はパニックしない.
///
/// # Complexity
/// - Time complexity: O(NTT_NAIVE_THRESHOLD).
/// - Space complexity: O(1).
///
/// # Examples
/// ```rust,ignore
/// // 内部関数のため, 直接の使用例は省略する.
/// ```
const fn build_naive_bit_reverse() -> [[usize; NTT_NAIVE_THRESHOLD]; NTT_NAIVE_LG_MAX + 1] {
    let mut table = [[0usize; NTT_NAIVE_THRESHOLD]; NTT_NAIVE_LG_MAX + 1];
    let mut lg = 0usize;
    while lg <= NTT_NAIVE_LG_MAX {
        let n = 1usize << lg;
        let mut i = 0usize;
        while i < n {
            let mut x = i;
            let mut res = 0usize;
            let mut j = 0usize;
            while j < lg {
                res = (res << 1) | (x & 1);
                x >>= 1;
                j += 1;
            }
            table[lg][i] = res;
            i += 1;
        }
        lg += 1;
    }
    table
}

/// 下位 `lg` bit のビット反転を行う.
///
/// このモジュールの NTT 実装は, 結果がビット反転順 (bit-reversed order) に並ぶ.
/// そのため, 愚直 NTT でも同じ並びを再現するために用いる.
///
/// # Args
/// - `x`: 変換対象の値.
/// - `lg`: 反転する bit 数.
///
/// # Returns
/// `usize`: 下位 `lg` bit を反転した値.
///
/// # Constraints
/// - `lg` は `usize::BITS` 以下である.
///
/// # Panics
/// - この関数はパニックしない.
///
/// # Complexity
/// - Time complexity: O(lg).
/// - Space complexity: O(1).
///
/// # Examples
/// ```rust,ignore
/// // 内部関数のため, 直接の使用例は省略する.
/// ```
pub fn bit_reverse(mut x: usize, lg: usize) -> usize {
    let mut res = 0usize;
    for _ in 0..lg {
        res = (res << 1) | (x & 1);
        x >>= 1;
    }
    res
}

/// NTT を愚直に実行する.
///
/// この関数は, バタフライによらず `f(ω^i)` を 2 重ループで計算し,
/// 既存の NTT 実装と同じビット反転順で出力する.
///
/// # Args
/// - `a`: 係数列. `a.len()` は 0 ではない 2 の冪である.
///
/// # Returns
/// `()`: `a` をインプレースで更新する.
///
/// # Constraints
/// - `a.len()` は 0 ではない 2 の冪である.
/// - 全ての要素は `MOD` 未満である.
///
/// # Panics
/// - この関数はパニックしない (debug assert のみ).
///
/// # Complexity
/// - Time complexity: O(n^2), ここで n は `a.len()` である.
/// - Space complexity: O(n).
///
/// # Examples
/// ```rust,ignore
/// // 内部関数のため, 直接の使用例は省略する.
/// ```
fn ntt_naive(a: &mut [u32]) {
    let n = a.len();
    debug_assert!(n.is_power_of_two());
    debug_assert!(n > 0);
    debug_assert!(n <= NTT_NAIVE_THRESHOLD);

    if n == 1 {
        return;
    }

    let lg = n.trailing_zeros() as usize;
    let bit_reverse = &NAIVE_BIT_REVERSE[lg];
    let mut buf = [0_u32; NTT_NAIVE_THRESHOLD];
    for i in 0..n {
        let k = bit_reverse[i];
        let pow_table = &NAIVE_NTT_BASE_POWS[lg][k];
        let mut sum = 0_u32;
        for j in 0..n {
            sum = modulo::add(sum, modulo::mul(a[j], pow_table[j]));
        }
        buf[i] = sum;
    }

    a.copy_from_slice(&buf[..n]);
}

/// 逆 NTT を愚直に実行する (正規化なし).
///
/// # Args
/// - `a`: NTT 値 (ビット反転順). `a.len()` は 0 ではない 2 の冪である.
///
/// # Returns
/// `()`: `a` をインプレースで更新する.
///
/// # Constraints
/// - `a.len()` は 0 ではない 2 の冪である.
/// - 全ての要素は `MOD` 未満である.
///
/// # Panics
/// - この関数はパニックしない (debug assert のみ).
///
/// # Complexity
/// - Time complexity: O(n^2), ここで n は `a.len()` である.
/// - Space complexity: O(n).
///
/// # Examples
/// ```rust,ignore
/// // 内部関数のため, 直接の使用例は省略する.
/// ```
fn intt_naive(a: &mut [u32]) {
    let n = a.len();
    debug_assert!(n.is_power_of_two());
    debug_assert!(n > 0);
    debug_assert!(n <= NTT_NAIVE_THRESHOLD);

    if n == 1 {
        return;
    }

    let lg = n.trailing_zeros() as usize;
    let bit_reverse = &NAIVE_BIT_REVERSE[lg];

    let mut buf = [0_u32; NTT_NAIVE_THRESHOLD];
    for j in 0..n {
        let pow_table = &NAIVE_INTT_BASE_POWS[lg][j];
        let mut sum = 0_u32;
        for k in 0..n {
            sum = modulo::add(sum, modulo::mul(a[bit_reverse[k]], pow_table[k]));
        }
        buf[j] = sum;
    }

    a.copy_from_slice(&buf[..n]);
}

/// NTT をバタフライで実行する.
///
/// # Args
/// - `a`: 係数列. `a.len()` は 0 ではない 2 の冪である.
///
/// # Returns
/// `()`: `a` をインプレースで更新する.
///
/// # Constraints
/// - `a.len()` は 0 ではない 2 の冪である.
/// - 全ての要素は `MOD` 未満である.
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
fn ntt_butterfly(a: &mut [u32]) {
    let n = a.len();
    debug_assert!(n.is_power_of_two());
    debug_assert!(n > 0);
    debug_assert!(a.iter().all(|&x| x < MOD));

    let h = n.trailing_zeros();

    for len in 0..h {
        let p = 1 << (h - len - 1);
        let mut rot = 1;
        let step = 1 << (h - len);
        for (s, chunk) in a.chunks_mut(step).enumerate() {
            let ptr = chunk.as_mut_ptr();
            for i in 0..p {
                unsafe {
                    let l = *ptr.add(i);
                    let r = modulo::mul(*ptr.add(i + p), rot);
                    *ptr.add(i) = modulo::add(l, r);
                    *ptr.add(i + p) = modulo::sub(l, r);
                }
            }
            rot = modulo::mul(rot, NTT_RATE[s.trailing_ones() as usize]);
        }
    }
}

/// 逆 NTT をバタフライで実行する (正規化なし).
///
/// # Args
/// - `a`: NTT 値. `a.len()` は 0 ではない 2 の冪である.
///
/// # Returns
/// `()`: `a` をインプレースで更新する.
///
/// # Constraints
/// - `a.len()` は 0 ではない 2 の冪である.
/// - 全ての要素は `MOD` 未満である.
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
fn intt_butterfly(a: &mut [u32]) {
    let n = a.len();
    debug_assert!(n.is_power_of_two());
    debug_assert!(n > 0);
    debug_assert!(a.iter().all(|&x| x < MOD));

    let h = n.trailing_zeros();

    for len in (1..=h).rev() {
        let mut irot = 1;
        let p = 1 << (h - len);
        let step = 1 << (h - len + 1);
        for (s, chunk) in a.chunks_mut(step).enumerate() {
            let ptr = chunk.as_mut_ptr();
            for i in 0..p {
                unsafe {
                    let l = *ptr.add(i);
                    let r = *ptr.add(i + p);
                    *ptr.add(i) = modulo::add(l, r);
                    *ptr.add(i + p) = modulo::mul(modulo::sub(l, r), irot);
                }
            }
            irot = modulo::mul(irot, INTT_RATE[s.trailing_ones() as usize]);
        }
    }
}

/// 与えられた列に対してインプレースで number theoretic transform (NTT) を実行する.
///
/// # Args
/// - `a`: 長さが 2 の冪となる係数列.
///
/// # Returns
/// `()`: この関数は `a` をインプレースで更新する.
///
/// # Constraints
/// - `a.len()` は 0 ではない 2 の冪でなければならない.
/// - `a.len()` は `MAX_NTT_LEN` を超えてはならない.
/// - 全ての要素は `MOD` 未満でなければならない.
///
/// # Panics
/// - 制約に違反した場合にパニックする.
///
/// # Complexity
/// - Time complexity: O(N^2), ただし `N <= NTT_NAIVE_THRESHOLD` の場合.
/// - Time complexity: O(N log N), ただし `N > NTT_NAIVE_THRESHOLD` の場合.
/// - Space complexity: O(N), ただし `N <= NTT_NAIVE_THRESHOLD` の場合.
/// - Space complexity: O(1), ただし `N > NTT_NAIVE_THRESHOLD` の場合.
///
/// # Examples
/// ```rust
/// use anmitsu::modulo998244353::convolution::{intt, ntt};
///
/// let mut values = vec![1, 2, 3, 4];
/// ntt(&mut values);
/// intt(&mut values);
/// let inv_len = 748683265; // 998244353 における 4 の逆元
/// values
///     .iter_mut()
///     .for_each(|v| *v = (*v as u64 * inv_len as u64 % 998244353) as u32);
/// assert_eq!(vec![1, 2, 3, 4], values);
/// ```
pub fn ntt(a: &mut [u32]) {
    if a.is_empty() {
        return;
    }

    let n = a.len();
    assert!(
        n.is_power_of_two(),
        "NTT length {} is not a power of two",
        n
    );
    assert!(
        n <= MAX_NTT_LEN,
        "NTT length {} exceeds supported maximum {}",
        n,
        MAX_NTT_LEN
    );
    debug_assert!(a.iter().all(|&x| x < MOD));

    #[cfg(target_arch = "x86_64")]
    {
        if std::is_x86_feature_detected!("avx2") {
            unsafe {
                super::convolution_avx2::ntt_avx2(a);
            }
            return;
        }
    }

    if n <= NTT_NAIVE_THRESHOLD {
        ntt_naive(a);
        return;
    }

    ntt_butterfly(a);
}

/// 与えられた列に対してインプレースで逆 number theoretic transform (INTT) を実行する.
///
/// # Args
/// - `a`: 長さが 2 の冪となる係数列.
///
/// # Returns
/// `()`: この関数は `a` をインプレースで更新する.
///
/// # Constraints
/// - `a.len()` は 0 ではない 2 の冪でなければならない.
/// - `a.len()` は `MAX_NTT_LEN` を超えてはならない.
/// - 全ての要素は `MOD` 未満でなければならない.
///
/// # Panics
/// - 制約に違反した場合にパニックする.
///
/// # Complexity
/// - Time complexity: O(N^2), ただし `N <= NTT_NAIVE_THRESHOLD` の場合.
/// - Time complexity: O(N log N), ただし `N > NTT_NAIVE_THRESHOLD` の場合.
/// - Space complexity: O(N), ただし `N <= NTT_NAIVE_THRESHOLD` の場合.
/// - Space complexity: O(1), ただし `N > NTT_NAIVE_THRESHOLD` の場合.
///
/// # Examples
/// ```rust
/// use anmitsu::modulo998244353::convolution::{intt, ntt};
///
/// let mut values = vec![5, 6, 7, 8];
/// ntt(&mut values);
/// intt(&mut values);
/// let inv_len = 748683265; // 998244353 における 4 の逆元
/// values
///     .iter_mut()
///     .for_each(|v| *v = (*v as u64 * inv_len as u64 % 998244353) as u32);
/// assert_eq!(vec![5, 6, 7, 8], values);
/// ```
pub fn intt(a: &mut [u32]) {
    if a.is_empty() {
        return;
    }

    let n = a.len();
    assert!(
        n.is_power_of_two(),
        "NTT length {} is not a power of two",
        n
    );
    assert!(
        n <= MAX_NTT_LEN,
        "NTT length {} exceeds supported maximum {}",
        n,
        MAX_NTT_LEN
    );
    debug_assert!(a.iter().all(|&x| x < MOD));

    #[cfg(target_arch = "x86_64")]
    {
        if std::is_x86_feature_detected!("avx2") {
            unsafe {
                super::convolution_avx2::intt_avx2(a);
            }
            return;
        }
    }

    if n <= NTT_NAIVE_THRESHOLD {
        intt_naive(a);
        return;
    }

    intt_butterfly(a);
}

/// 998244353 を法として 2 つの列の畳み込みを計算する.
///
/// # Args
/// - `a`: 法 `MOD` で還元された最初の入力列. この関数は `a` を消費する.
/// - `b`: 法 `MOD` で還元された 2 番目の入力列. この関数は `b` を消費する.
///
/// # Returns
/// `Vec<u32>`: 法 `MOD` での畳み込み結果.
///
/// # Constraints
/// - いずれかのベクターが空の場合は空のベクターを返す.
/// - `a.len() + b.len() - 1` は `MAX_NTT_LEN` を超えてはならない.
/// - すべての係数は `MOD` 未満でなければならない.
///
/// # Panics
/// - 長さ制約に違反した場合にパニックする.
///
/// # Complexity
/// - Time complexity: O((N + M) log K), ここで N, M は入力長で, K は
///   `N + M - 1` を超えない最小の 2 の冪である.
/// - Space complexity: O(K).
///
/// # Examples
/// ```rust
/// use anmitsu::modulo998244353::convolution::convolution;
///
/// let a = vec![1, 2, 3];
/// let b = vec![4, 5, 6];
/// let result = convolution(a, b);
/// assert_eq!(vec![4, 13, 28, 27, 18], result);
/// ```
pub fn convolution(mut a: Vec<u32>, mut b: Vec<u32>) -> Vec<u32> {
    if a.is_empty() || b.is_empty() {
        return Vec::new();
    }

    debug_assert!(a.iter().all(|&x| x < MOD));
    debug_assert!(b.iter().all(|&x| x < MOD));

    #[cfg(target_arch = "x86_64")]
    {
        if std::is_x86_feature_detected!("avx2") {
            unsafe {
                return super::convolution_avx2::convolution_avx2(a, b);
            }
        }
    }

    let s = a.len() + b.len() - 1;
    if a.len().min(b.len()) <= 32 {
        let mut res = vec![0; s];
        for i in 0..a.len() {
            let ai = a[i];
            for j in 0..b.len() {
                res[i + j] = modulo::add(res[i + j], modulo::mul(ai, b[j]));
            }
        }
        return res;
    }

    let t = s.next_power_of_two();
    assert!(
        t <= MAX_NTT_LEN,
        "Convolution length {} exceeds supported maximum {}",
        t,
        MAX_NTT_LEN
    );

    a.resize(t, 0);
    b.resize(t, 0);

    ntt(&mut a);
    ntt(&mut b);
    a.iter_mut()
        .zip(b.iter())
        .for_each(|(x, y)| *x = modulo::mul(*x, *y));
    intt(&mut a);
    let t_inv = INVS[t.trailing_zeros() as usize];
    a.iter_mut()
        .take(s)
        .for_each(|x| *x = modulo::mul(*x, t_inv));
    a.truncate(s);
    a
}
