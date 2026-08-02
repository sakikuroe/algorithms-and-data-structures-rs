//! 998244353 を法として number theoretic transform を用いた畳み込みを提供する。
//!
//! AVX2 が利用可能な環境では、Montgomery 表現を用いた高速化実装
//! (`convolution_avx2` モジュール) へ自動的に切り替える。

#[cfg(target_arch = "x86_64")]
use super::convolution_avx2;
use super::modulo;

/// この畳み込み実装で用いる法。
pub const MOD: u32 = 998244353;

/// 変換がサポートする最大長。
pub const MAX_NTT_LEN: usize = 1 << 19;

/// 小さい長さの NTT/INTT に対して愚直計算へ切り替える閾値である。
///
/// 実測により 8、16、32 などへ調整することを想定する。
const NTT_NAIVE_THRESHOLD: usize = 32;

/// `2^lg <= NTT_NAIVE_THRESHOLD` を満たす最大の `lg` の値である。
const NTT_NAIVE_LG_MAX: usize =
    (usize::BITS as usize - 1) - (NTT_NAIVE_THRESHOLD.leading_zeros() as usize);

/// 愚直 NTT で用いる `ω^i` のテーブルである。
///
/// `table[lg][i] = ω^i` を表し、`ω` は長さ `2^lg` の原始根である。
/// `i >= 2^lg` の領域は未使用で 0 のままとする。
const NAIVE_NTT_OMEGA_POWS: [[u32; NTT_NAIVE_THRESHOLD]; NTT_NAIVE_LG_MAX + 1] =
    build_naive_ntt_omega_pows();

/// 愚直 INTT で用いる `ω^{-1}` のテーブルである。
///
/// `omega_inv[lg]` は長さ `2^lg` に対応する `ω^{-1}` を表す。
const NAIVE_INTT_OMEGA_INV: [u32; NTT_NAIVE_LG_MAX + 1] = build_naive_intt_omega_inv();

/// 愚直 NTT で用いる `(ω^k)^j` のテーブルである。
///
/// `table[lg][k][j]` は `(ω^k)^j` を表し、`ω` は長さ `2^lg` の原始根である。
/// `k >= 2^lg` または `j >= 2^lg` の領域は未使用で 0 のままとする。
const NAIVE_NTT_BASE_POWS: [[[u32; NTT_NAIVE_THRESHOLD]; NTT_NAIVE_THRESHOLD];
    NTT_NAIVE_LG_MAX + 1] = build_naive_ntt_base_pows();

/// 愚直 INTT で用いる `(ω^{-j})^k` のテーブルである。
///
/// `table[lg][j][k]` は `(ω^{-j})^k` を表し、`ω` は長さ `2^lg` の原始根である。
/// `j >= 2^lg` または `k >= 2^lg` の領域は未使用で 0 のままとする。
const NAIVE_INTT_BASE_POWS: [[[u32; NTT_NAIVE_THRESHOLD]; NTT_NAIVE_THRESHOLD];
    NTT_NAIVE_LG_MAX + 1] = build_naive_intt_base_pows();

/// 愚直 NTT/INTT で用いるビット反転のテーブルである。
///
/// `table[lg][i]` は、`i` の下位 `lg` bit を反転した値を表す。
/// `i >= 2^lg` の領域は未使用で 0 のままとする。
const NAIVE_BIT_REVERSE: [[usize; NTT_NAIVE_THRESHOLD]; NTT_NAIVE_LG_MAX + 1] =
    build_naive_bit_reverse();

/// 998244353 における原始根である。
pub const PRIMITIVE_ROOT: u32 = 3;

/// NTT の各段で用いる回転因子のテーブルである。
pub const NTT_RATE: [u32; 22] = [
    0x3656d65b, 0x1e5ea9e6, 0x16038782, 0x13caac90, 0x3a9a4cfa, 0x761af21, 0xe372007, 0x3a2be7d4,
    0x23fe18b2, 0x330f5b68, 0x7d37cf9, 0x3239edef, 0x2b8ea5c3, 0x382d2452, 0x300e9be2, 0x908b3f5,
    0x1e726cd9, 0x1e02c2f0, 0x2c49629c, 0x2c2b7c93, 0x35a5081, 0x33b69d8b,
];

/// 逆 NTT の各段で用いる回転因子のテーブルである。
pub const INTT_RATE: [u32; 22] = [
    0x52929a6, 0x163456b8, 0x16400573, 0x267c5b5f, 0x6b059a5, 0x294c15f1, 0x94415d9, 0x2f83389c,
    0x569c0ec, 0x3346ebba, 0x37473ab0, 0x1524e16f, 0x68442e3, 0x117ab9d0, 0x1fe52df0, 0x1263f553,
    0x7392943, 0x24433aa8, 0x1a2993eb, 0x156d2fbf, 0x311e570f, 0x6294a13,
];

/// `2^k` (k = 0..=22) の `MOD` における逆元のテーブルである。
pub const INVS: [u32; 23] = [
    1, 499122177, 748683265, 873463809, 935854081, 967049217, 982646785, 990445569, 994344961,
    996294657, 997269505, 997756929, 998000641, 998122497, 998183425, 998213889, 998229121,
    998236737, 998240545, 998242449, 998243401, 998243877, 998244115,
];

/// 愚直 NTT の `ω^i` テーブルを構築する。
///
/// # Args
/// - `()`: 引数はない。
///
/// # Returns
/// `[[u32; NTT_NAIVE_THRESHOLD]; NTT_NAIVE_LG_MAX + 1]`: `ω^i` テーブル。
///
/// # Constraints
/// - `NTT_NAIVE_THRESHOLD > 0` である。
///
/// # Panics
/// - この関数はパニックしない。
///
/// # Complexity
/// - Time complexity: O(NTT_NAIVE_THRESHOLD)。
/// - Space complexity: O(1)。
///
/// # Examples
/// ```rust,ignore
/// // 内部関数のため、直接の使用例は省略する。
/// ```
const fn build_naive_ntt_omega_pows() -> [[u32; NTT_NAIVE_THRESHOLD]; NTT_NAIVE_LG_MAX + 1] {
    // const fn ではヒープを使えないため、固定長配列を 0 埋めで確保しておく。
    let mut table = [[0_u32; NTT_NAIVE_THRESHOLD]; NTT_NAIVE_LG_MAX + 1];
    let mut lg = 0usize;
    // 長さ `2^lg` (lg = 0..=NTT_NAIVE_LG_MAX) ごとに、専用の原始根テーブルを構築する。
    while lg <= NTT_NAIVE_LG_MAX {
        let n = 1usize << lg;
        // `MOD - 1` は乗法群 `(Z/MOD Z)^*` の位数である。位数を `n` で割った指数で
        // `PRIMITIVE_ROOT` を累乗すると、1 の原始 n 乗根 `ω` が得られる。
        let exp = (MOD as usize - 1) / n;
        let omega = modulo::pow(PRIMITIVE_ROOT, exp);
        // `ω^0, ω^1, ..., ω^{n-1}` を、直前の値に `ω` を掛けながら順に埋める。
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

/// 愚直 INTT の `ω^{-1}` テーブルを構築する。
///
/// # Args
/// - `()`: 引数はない。
///
/// # Returns
/// `[u32; NTT_NAIVE_LG_MAX + 1]`: `ω^{-1}` テーブル。
///
/// # Constraints
/// - `NTT_NAIVE_THRESHOLD > 0` である。
///
/// # Panics
/// - この関数はパニックしない。
///
/// # Complexity
/// - Time complexity: O(NTT_NAIVE_LG_MAX)。
/// - Space complexity: O(1)。
///
/// # Examples
/// ```rust,ignore
/// // 内部関数のため、直接の使用例は省略する。
/// ```
const fn build_naive_intt_omega_inv() -> [u32; NTT_NAIVE_LG_MAX + 1] {
    let mut table = [0_u32; NTT_NAIVE_LG_MAX + 1];
    let mut lg = 0usize;
    // 長さ `2^lg` ごとに、原始 n 乗根 `ω` を求めてからその逆元を保存する。
    while lg <= NTT_NAIVE_LG_MAX {
        let n = 1usize << lg;
        let exp = (MOD as usize - 1) / n;
        let omega = modulo::pow(PRIMITIVE_ROOT, exp);
        table[lg] = modulo::inv(omega);
        lg += 1;
    }
    table
}

/// 愚直 NTT の基底べき乗テーブルを構築する。
///
/// # Args
/// - `()`: 引数はない。
///
/// # Returns
/// `[[[u32; NTT_NAIVE_THRESHOLD]; NTT_NAIVE_THRESHOLD]; NTT_NAIVE_LG_MAX + 1]`:
/// `(ω^k)^j` テーブル。
///
/// # Constraints
/// - `NTT_NAIVE_THRESHOLD > 0` である。
///
/// # Panics
/// - この関数はパニックしない。
///
/// # Complexity
/// - Time complexity: O(NTT_NAIVE_THRESHOLD^2)。
/// - Space complexity: O(1)。
///
/// # Examples
/// ```rust,ignore
/// // 内部関数のため、直接の使用例は省略する。
/// ```
const fn build_naive_ntt_base_pows()
-> [[[u32; NTT_NAIVE_THRESHOLD]; NTT_NAIVE_THRESHOLD]; NTT_NAIVE_LG_MAX + 1] {
    let mut table = [[[0_u32; NTT_NAIVE_THRESHOLD]; NTT_NAIVE_THRESHOLD]; NTT_NAIVE_LG_MAX + 1];
    let mut lg = 0usize;
    while lg <= NTT_NAIVE_LG_MAX {
        let n = 1usize << lg;
        // `f(ω^k) = Σ_j a[j] (ω^k)^j` を愚直に評価するため、出力添字 `k` ごとに
        // 底 `base = ω^k` を求め、その 0 乗から n-1 乗までを事前に列挙しておく。
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

/// 愚直 INTT の基底べき乗テーブルを構築する。
///
/// # Args
/// - `()`: 引数はない。
///
/// # Returns
/// `[[[u32; NTT_NAIVE_THRESHOLD]; NTT_NAIVE_THRESHOLD]; NTT_NAIVE_LG_MAX + 1]`:
/// `(ω^{-j})^k` テーブル。
///
/// # Constraints
/// - `NTT_NAIVE_THRESHOLD > 0` である。
///
/// # Panics
/// - この関数はパニックしない。
///
/// # Complexity
/// - Time complexity: O(NTT_NAIVE_THRESHOLD^2)。
/// - Space complexity: O(1)。
///
/// # Examples
/// ```rust,ignore
/// // 内部関数のため、直接の使用例は省略する。
/// ```
const fn build_naive_intt_base_pows()
-> [[[u32; NTT_NAIVE_THRESHOLD]; NTT_NAIVE_THRESHOLD]; NTT_NAIVE_LG_MAX + 1] {
    let mut table = [[[0_u32; NTT_NAIVE_THRESHOLD]; NTT_NAIVE_THRESHOLD]; NTT_NAIVE_LG_MAX + 1];
    let mut lg = 0usize;
    while lg <= NTT_NAIVE_LG_MAX {
        let n = 1usize << lg;
        let omega_inv = NAIVE_INTT_OMEGA_INV[lg];

        // まず `(ω^{-1})^j` (j = 0..n-1) を求めておく。逆変換の底 `ω^{-j}` は
        // これらの値そのものである。
        let mut omega_inv_pows = [0_u32; NTT_NAIVE_THRESHOLD];
        let mut p = 1_u32;
        let mut j = 0usize;
        while j < n {
            omega_inv_pows[j] = p;
            p = modulo::mul(p, omega_inv);
            j += 1;
        }

        // `f(ω^{-j}) = Σ_k A[k] (ω^{-j})^k` を愚直に評価するため、各 `j` について
        // 底 `base = ω^{-j}` の 0 乗から n-1 乗までを列挙する。
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

/// 愚直 NTT/INTT のビット反転テーブルを構築する。
///
/// # Args
/// - `()`: 引数はない。
///
/// # Returns
/// `[[usize; NTT_NAIVE_THRESHOLD]; NTT_NAIVE_LG_MAX + 1]`: ビット反転テーブル。
///
/// # Constraints
/// - `NTT_NAIVE_THRESHOLD > 0` である。
///
/// # Panics
/// - この関数はパニックしない。
///
/// # Complexity
/// - Time complexity: O(NTT_NAIVE_THRESHOLD)。
/// - Space complexity: O(1)。
///
/// # Examples
/// ```rust,ignore
/// // 内部関数のため、直接の使用例は省略する。
/// ```
const fn build_naive_bit_reverse() -> [[usize; NTT_NAIVE_THRESHOLD]; NTT_NAIVE_LG_MAX + 1] {
    let mut table = [[0usize; NTT_NAIVE_THRESHOLD]; NTT_NAIVE_LG_MAX + 1];
    let mut lg = 0usize;
    while lg <= NTT_NAIVE_LG_MAX {
        let n = 1usize << lg;
        let mut i = 0usize;
        while i < n {
            // `i` の下位 `lg` bit を 1 bit ずつ取り出し、反転させながら `res` に積む。
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

/// 下位 `lg` bit のビット反転を行う。
///
/// このモジュールの NTT 実装は、結果がビット反転順 (bit-reversed order) に並ぶ。
/// そのため、愚直 NTT でも同じ並びを再現するために用いる。
///
/// # Args
/// - `x`: 変換対象の値。
/// - `lg`: 反転する bit 数。
///
/// # Returns
/// `usize`: 下位 `lg` bit を反転した値。
///
/// # Constraints
/// - `lg` は `usize::BITS` 以下である。
///
/// # Panics
/// - この関数はパニックしない。
///
/// # Complexity
/// - Time complexity: O(lg)。
/// - Space complexity: O(1)。
///
/// # Examples
/// ```rust,ignore
/// // 内部関数のため、直接の使用例は省略する。
/// ```
pub fn bit_reverse(mut x: usize, lg: usize) -> usize {
    let mut res = 0usize;
    // `x` の最下位 bit を 1 bit ずつ取り出して `res` へ積み直すことで、下位
    // `lg` bit の並びを反転させる。
    for _ in 0..lg {
        res = (res << 1) | (x & 1);
        x >>= 1;
    }
    res
}

/// NTT を愚直に実行する。
///
/// この関数は、バタフライによらず `f(ω^i)` を 2 重ループで計算し、
/// 既存の NTT 実装と同じビット反転順で出力する。
///
/// # Args
/// - `a`: 係数列。`a.len()` は 0 ではない 2 の冪である。
///
/// # Returns
/// `()`: `a` をインプレースで更新する。
///
/// # Constraints
/// - `a.len()` は 0 ではない 2 の冪である。
/// - 全ての要素は `MOD` 未満である。
///
/// # Panics
/// - この関数はパニックしない (debug assert のみ)。
///
/// # Complexity
/// - Time complexity: O(n^2)、ここで n は `a.len()` である。
/// - Space complexity: O(n)。
///
/// # Examples
/// ```rust,ignore
/// // 内部関数のため、直接の使用例は省略する。
/// ```
fn ntt_naive(a: &mut [u32]) {
    let n = a.len();
    debug_assert!(n.is_power_of_two());
    debug_assert!(n > 0);
    debug_assert!(n <= NTT_NAIVE_THRESHOLD);

    // n = 1 のとき DFT は恒等変換であり、変更すべき要素がない。
    if n == 1 {
        return;
    }

    let lg = n.trailing_zeros() as usize;
    // バタフライ実装と出力順を揃えるため、出力先の添字はビット反転させる。
    let bit_reverse = &NAIVE_BIT_REVERSE[lg];
    let mut buf = [0_u32; NTT_NAIVE_THRESHOLD];
    // ビット反転順の出力位置 `i` ごとに、対応する `k = bit_reverse[i]` を用いて
    // `f(ω^k) = Σ_j a[j] (ω^k)^j` を素朴な内積として計算する。
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

/// 逆 NTT を愚直に実行する (正規化なし)。
///
/// # Args
/// - `a`: NTT 値 (ビット反転順)。`a.len()` は 0 ではない 2 の冪である。
///
/// # Returns
/// `()`: `a` をインプレースで更新する。
///
/// # Constraints
/// - `a.len()` は 0 ではない 2 の冪である。
/// - 全ての要素は `MOD` 未満である。
///
/// # Panics
/// - この関数はパニックしない (debug assert のみ)。
///
/// # Complexity
/// - Time complexity: O(n^2)、ここで n は `a.len()` である。
/// - Space complexity: O(n)。
///
/// # Examples
/// ```rust,ignore
/// // 内部関数のため、直接の使用例は省略する。
/// ```
fn intt_naive(a: &mut [u32]) {
    let n = a.len();
    debug_assert!(n.is_power_of_two());
    debug_assert!(n > 0);
    debug_assert!(n <= NTT_NAIVE_THRESHOLD);

    // n = 1 のとき逆 DFT は恒等変換であり、変更すべき要素がない。
    if n == 1 {
        return;
    }

    let lg = n.trailing_zeros() as usize;
    // 入力 `a` はビット反転順で渡されるため、参照時に元の添字へ戻す。
    let bit_reverse = &NAIVE_BIT_REVERSE[lg];

    let mut buf = [0_u32; NTT_NAIVE_THRESHOLD];
    // 出力位置 `j` ごとに、`f(ω^{-j}) = Σ_k A[k] (ω^{-j})^k` を素朴な内積として
    // 計算する。`A[k]` はビット反転順で格納されているため `bit_reverse[k]` 経由で読む。
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

/// NTT をバタフライで実行する。
///
/// # Args
/// - `a`: 係数列。`a.len()` は 0 ではない 2 の冪である。
///
/// # Returns
/// `()`: `a` をインプレースで更新する。
///
/// # Constraints
/// - `a.len()` は 0 ではない 2 の冪である。
/// - 全ての要素は `MOD` 未満である。
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
fn ntt_butterfly(a: &mut [u32]) {
    let n = a.len();
    debug_assert!(n.is_power_of_two());
    debug_assert!(n > 0);
    debug_assert!(a.iter().all(|&x| x < MOD));

    let h = n.trailing_zeros();

    // decimation-in-frequency (DIF) 方式で、ブロック幅を `n` から 1 まで
    // 半分ずつ縮めながら段階的にバタフライ演算を適用する。
    for len in 0..h {
        // この段で扱うブロックの半幅 `p` と全幅 `step = 2p` である。
        let p = 1 << (h - len - 1);
        let mut rot = 1;
        let step = 1 << (h - len);
        // 幅 `step` のブロックごとに、前半 `[0, p)` と後半 `[p, 2p)` を
        // `(l + r, l - r)` の形でまとめてバタフライ演算する。
        // `rot` は現在のブロックに対応する回転因子であり、後半の値へ乗じる。
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
            // 次のブロックの回転因子は、ブロック番号 `s` の下位から連続する 1 の
            // 個数 (`trailing_ones`) に応じた段数分だけ `NTT_RATE` を掛けることで
            // 得られる。これにより、毎回 `pow` を呼ばずに O(1) 償却で更新できる。
            rot = modulo::mul(rot, NTT_RATE[s.trailing_ones() as usize]);
        }
    }
}

/// 逆 NTT をバタフライで実行する (正規化なし)。
///
/// # Args
/// - `a`: NTT 値。`a.len()` は 0 ではない 2 の冪である。
///
/// # Returns
/// `()`: `a` をインプレースで更新する。
///
/// # Constraints
/// - `a.len()` は 0 ではない 2 の冪である。
/// - 全ての要素は `MOD` 未満である。
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
fn intt_butterfly(a: &mut [u32]) {
    let n = a.len();
    debug_assert!(n.is_power_of_two());
    debug_assert!(n > 0);
    debug_assert!(a.iter().all(|&x| x < MOD));

    let h = n.trailing_zeros();

    // decimation-in-time (DIT) 方式で、ブロック幅を 1 段目 (幅 2) から `n` まで
    // 倍々に広げながらバタフライ演算を適用する。これは `ntt_butterfly` の
    // 各ステップを逆順にたどる形になる。
    for len in (1..=h).rev() {
        let mut irot = 1;
        let p = 1 << (h - len);
        let step = 1 << (h - len + 1);
        // 幅 `step` のブロックごとに、前半 `[0, p)` と後半 `[p, 2p)` から
        // `(l + r, (l - r) * irot)` を計算する。和は正規化因子を後段でまとめて
        // 掛けるため、ここではまだ乗じない。
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
            // 逆回転因子も `trailing_ones` を用いた同じ償却更新で求める。
            irot = modulo::mul(irot, INTT_RATE[s.trailing_ones() as usize]);
        }
    }
}

/// 与えられた列に対してインプレースで number theoretic transform (NTT) を実行する。
///
/// # Args
/// - `a`: 長さが 2 の冪となる係数列。
///
/// # Returns
/// `()`: この関数は `a` をインプレースで更新する。
///
/// # Constraints
/// - `a.len()` は 0 ではない 2 の冪でなければならない。
/// - `a.len()` は `MAX_NTT_LEN` を超えてはならない。
/// - 全ての要素は `MOD` 未満でなければならない。
///
/// # Panics
/// - 制約に違反した場合にパニックする。
///
/// # Complexity
/// - Time complexity: O(N^2)、ただし `N <= NTT_NAIVE_THRESHOLD` の場合。
/// - Time complexity: O(N log N)、ただし `N > NTT_NAIVE_THRESHOLD` の場合。
/// - Space complexity: O(N)、ただし `N <= NTT_NAIVE_THRESHOLD` の場合。
/// - Space complexity: O(1)、ただし `N > NTT_NAIVE_THRESHOLD` の場合。
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
    // 空列の DFT は空列であり、変換すべき要素がない。
    if a.is_empty() {
        return;
    }

    let n = a.len();
    // バタフライ演算は 2 の冪長を前提とするため、事前に長さを検証する。
    assert!(
        n.is_power_of_two(),
        "NTT length {} is not a power of two",
        n
    );
    // 事前計算テーブル (回転因子など) は `MAX_NTT_LEN` までしか用意していない。
    assert!(
        n <= MAX_NTT_LEN,
        "NTT length {} exceeds supported maximum {}",
        n,
        MAX_NTT_LEN
    );
    debug_assert!(a.iter().all(|&x| x < MOD));

    // AVX2 が利用可能な環境では、Montgomery 表現による SIMD 実装の方が高速なため、
    // そちらへ処理を委譲する。
    #[cfg(target_arch = "x86_64")]
    {
        if std::is_x86_feature_detected!("avx2") {
            unsafe {
                convolution_avx2::ntt_avx2(a);
            }
            return;
        }
    }

    // 長さが小さい場合、バタフライ演算のオーバーヘッド (段数分のループや
    // 関数呼び出し) が支配的になり得るため、2 重ループの愚直計算に切り替える。
    if n <= NTT_NAIVE_THRESHOLD {
        ntt_naive(a);
        return;
    }

    ntt_butterfly(a);
}

/// 与えられた列に対してインプレースで逆 number theoretic transform (INTT) を実行する。
///
/// # Args
/// - `a`: 長さが 2 の冪となる係数列。
///
/// # Returns
/// `()`: この関数は `a` をインプレースで更新する。
///
/// # Constraints
/// - `a.len()` は 0 ではない 2 の冪でなければならない。
/// - `a.len()` は `MAX_NTT_LEN` を超えてはならない。
/// - 全ての要素は `MOD` 未満でなければならない。
///
/// # Panics
/// - 制約に違反した場合にパニックする。
///
/// # Complexity
/// - Time complexity: O(N^2)、ただし `N <= NTT_NAIVE_THRESHOLD` の場合。
/// - Time complexity: O(N log N)、ただし `N > NTT_NAIVE_THRESHOLD` の場合。
/// - Space complexity: O(N)、ただし `N <= NTT_NAIVE_THRESHOLD` の場合。
/// - Space complexity: O(1)、ただし `N > NTT_NAIVE_THRESHOLD` の場合。
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
    // 空列の逆 DFT は空列であり、変換すべき要素がない。
    if a.is_empty() {
        return;
    }

    let n = a.len();
    // バタフライ演算は 2 の冪長を前提とするため、事前に長さを検証する。
    assert!(
        n.is_power_of_two(),
        "NTT length {} is not a power of two",
        n
    );
    // 事前計算テーブル (回転因子など) は `MAX_NTT_LEN` までしか用意していない。
    assert!(
        n <= MAX_NTT_LEN,
        "NTT length {} exceeds supported maximum {}",
        n,
        MAX_NTT_LEN
    );
    debug_assert!(a.iter().all(|&x| x < MOD));

    // AVX2 が利用可能な環境では、Montgomery 表現による SIMD 実装の方が高速なため、
    // そちらへ処理を委譲する。
    #[cfg(target_arch = "x86_64")]
    {
        if std::is_x86_feature_detected!("avx2") {
            unsafe {
                convolution_avx2::intt_avx2(a);
            }
            return;
        }
    }

    // 長さが小さい場合、バタフライ演算のオーバーヘッドが支配的になり得るため、
    // 2 重ループの愚直計算に切り替える。
    if n <= NTT_NAIVE_THRESHOLD {
        intt_naive(a);
        return;
    }

    intt_butterfly(a);
}

/// 998244353 を法として 2 つの列の畳み込みを計算する。
///
/// # Args
/// - `a`: 法 `MOD` で還元された最初の入力列。この関数は `a` を消費する。
/// - `b`: 法 `MOD` で還元された 2 番目の入力列。この関数は `b` を消費する。
///
/// # Returns
/// `Vec<u32>`: 法 `MOD` での畳み込み結果。
///
/// # Constraints
/// - いずれかのベクターが空の場合は空のベクターを返す。
/// - `a.len() + b.len() - 1` は `MAX_NTT_LEN` を超えてはならない。
/// - すべての係数は `MOD` 未満でなければならない。
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
/// ```rust
/// use anmitsu::modulo998244353::convolution::convolution;
///
/// let a = vec![1, 2, 3];
/// let b = vec![4, 5, 6];
/// let result = convolution(a, b);
/// assert_eq!(vec![4, 13, 28, 27, 18], result);
/// ```
pub fn convolution(mut a: Vec<u32>, mut b: Vec<u32>) -> Vec<u32> {
    // 定義により、一方でも空列との畳み込みは空列になる。
    if a.is_empty() || b.is_empty() {
        return Vec::new();
    }

    debug_assert!(a.iter().all(|&x| x < MOD));
    debug_assert!(b.iter().all(|&x| x < MOD));

    // AVX2 が利用可能な環境では、Montgomery 表現による SIMD 実装の方が高速なため、
    // そちらへ丸ごと処理を委譲する。
    #[cfg(target_arch = "x86_64")]
    {
        if std::is_x86_feature_detected!("avx2") {
            unsafe {
                return convolution_avx2::convolution_avx2(a, b);
            }
        }
    }

    // 結果の長さ `s = |a| + |b| - 1` である。
    let s = a.len() + b.len() - 1;
    // 短い入力では、NTT の前処理 (パディングやビット反転順への変換) の
    // オーバーヘッドが O(|a| |b|) の愚直な畳み込みを上回ってしまうため、
    // 小さい側の長さが閾値以下なら愚直計算で済ませる。
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

    // NTT は 2 の冪長でしか実行できないため、結果長 `s` を超えない最小の
    // 2 の冪 `t` までパディングする。
    let t = s.next_power_of_two();
    assert!(
        t <= MAX_NTT_LEN,
        "Convolution length {} exceeds supported maximum {}",
        t,
        MAX_NTT_LEN
    );

    a.resize(t, 0);
    b.resize(t, 0);

    // 畳み込み定理により、点ごとの積を NTT 空間で計算してから逆変換すると
    // 元の畳み込みが得られる。
    ntt(&mut a);
    ntt(&mut b);
    a.iter_mut()
        .zip(b.iter())
        .for_each(|(x, y)| *x = modulo::mul(*x, *y));
    intt(&mut a);
    // `intt` は正規化を行わないため、ここで `1/t` を掛けて辻褄を合わせる。
    let t_inv = INVS[t.trailing_zeros() as usize];
    a.iter_mut()
        .take(s)
        .for_each(|x| *x = modulo::mul(*x, t_inv));
    // パディング分の余分な項を切り落とし、本来の結果長 `s` に揃える。
    a.truncate(s);
    a
}

#[cfg(test)]
mod tests {
    use super::*;

    // 以下は、 このモジュールの実装を検証するための参照実装 (オラクル) である。
    // 本体側の実装と独立に同じアルゴリズムを再実装しておくことで、
    // 実装のリグレッションを検出できるようにする。

    /// `NTT_RATE` を用いた、 テスト専用の愚直な NTT 参照実装である。
    fn ntt_ref(a: &mut [u32]) {
        if a.is_empty() {
            return;
        }

        let n = a.len();
        assert!(n.is_power_of_two(), "NTT length must be a power of two");
        assert!(n <= MAX_NTT_LEN, "NTT length exceeds MAX_NTT_LEN");

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

    /// `INTT_RATE` を用いた、 テスト専用の愚直な INTT 参照実装である (正規化なし)。
    fn intt_ref(a: &mut [u32]) {
        if a.is_empty() {
            return;
        }

        let n = a.len();
        assert!(n.is_power_of_two(), "NTT length must be a power of two");
        assert!(n <= MAX_NTT_LEN, "NTT length exceeds MAX_NTT_LEN");

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

    /// `ntt_ref` / `intt_ref` を用いた、 テスト専用の畳み込み参照実装である。
    fn convolution_ref(a: &[u32], b: &[u32]) -> Vec<u32> {
        if a.is_empty() || b.is_empty() {
            return Vec::new();
        }

        let s = a.len() + b.len() - 1;
        let t = s.next_power_of_two();
        assert!(t <= MAX_NTT_LEN, "Convolution length exceeds MAX_NTT_LEN");

        let mut fa = Vec::with_capacity(t);
        fa.extend_from_slice(a);
        fa.resize(t, 0);
        let mut fb = Vec::with_capacity(t);
        fb.extend_from_slice(b);
        fb.resize(t, 0);

        ntt_ref(&mut fa);
        ntt_ref(&mut fb);
        fa.iter_mut()
            .zip(fb.iter())
            .for_each(|(x, y)| *x = modulo::mul(*x, *y));
        intt_ref(&mut fa);

        let inv_len = INVS[t.trailing_zeros() as usize];
        fa.iter_mut()
            .take(s)
            .for_each(|x| *x = modulo::mul(*x, inv_len));
        fa.truncate(s);
        fa
    }

    /// Background: 線形合同法による、 再現可能な擬似乱数列 (`0..MOD` の範囲)。
    fn gen_values(len: usize, mut seed: u64) -> Vec<u32> {
        let mut values = Vec::with_capacity(len);
        for _ in 0..len {
            seed = seed
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            values.push((seed % MOD as u64) as u32);
        }
        values
    }

    // ntt のテスト: 戻り値を参照実装と比較して検証する。
    mod ntt {
        use super::*;

        /// Scenario: 様々な長さの係数列に対して、 参照実装と同じ結果を返す。
        /// - Given: 長さ `2^lg` (lg = 0..=16) の擬似乱数列がある。
        /// - When: `ntt` を実行する。
        /// - Then: 参照実装 `ntt_ref` の結果と一致する。
        #[test]
        fn matches_reference_for_various_lengths() {
            for lg in 0..=16 {
                // Given
                let len = 1usize << lg;
                let input = gen_values(len, 1 + lg as u64);
                let mut expected = input.clone();
                let mut actual = input;

                // When
                ntt_ref(&mut expected);
                ntt(&mut actual);

                // Then
                assert_eq!(expected, actual, "lg mismatch: {lg}");
            }
        }
    }

    // intt のテスト: 戻り値を参照実装と比較し、 ntt との往復も検証する。
    mod intt {
        use super::*;

        /// Scenario: 様々な長さの NTT 値に対して、 参照実装と同じ結果を返す。
        /// - Given: 長さ `2^lg` (lg = 0..=16) の擬似乱数列がある。
        /// - When: `intt` を実行する。
        /// - Then: 参照実装 `intt_ref` の結果と一致する。
        #[test]
        fn matches_reference_for_various_lengths() {
            for lg in 0..=16 {
                // Given
                let len = 1usize << lg;
                let input = gen_values(len, 100 + lg as u64);
                let mut expected = input.clone();
                let mut actual = input;

                // When
                intt_ref(&mut expected);
                intt(&mut actual);

                // Then
                assert_eq!(expected, actual, "lg mismatch: {lg}");
            }
        }

        /// Scenario: `ntt` に続けて `intt` を適用すると、 元の値の `n` 倍
        /// (正規化なし) に戻る。
        /// - Given: 長さ `2^lg` (lg = 0..=16) の擬似乱数列がある。
        /// - When: `ntt` を適用してから `intt` を適用し、 `1/n` を掛けて正規化する。
        /// - Then: 元の入力と一致する。
        #[test]
        fn round_trip_with_ntt_matches_original_after_scaling() {
            for lg in 0..=16 {
                // Given
                let len = 1usize << lg;
                let input = gen_values(len, 999 + lg as u64);
                let mut actual = input.clone();

                // When
                ntt(&mut actual);
                intt(&mut actual);
                let inv_len = INVS[lg];
                actual
                    .iter_mut()
                    .for_each(|x| *x = modulo::mul(*x, inv_len));

                // Then
                assert_eq!(input, actual, "lg mismatch: {lg}");
            }
        }
    }

    // convolution のテスト: 戻り値を検証する。
    mod convolution {
        use super::*;

        /// Scenario: 一方でも空列との畳み込みは空列になる (境界値)。
        /// - Given: 空列と非空列がある。
        /// - When: `convolution` を実行する。
        /// - Then: 空列が返る。
        #[test]
        fn returns_empty_when_either_input_is_empty() {
            // Given
            let a = Vec::<u32>::new();
            let b = vec![1, 2, 3];

            // When
            let result = convolution(a, b);

            // Then
            assert!(result.is_empty());
        }

        /// Scenario: 典型的な小さい入力に対して期待通りの畳み込み結果を返す。
        /// - Given: 長さ 3 の係数列が 2 つある。
        /// - When: `convolution` を実行する。
        /// - Then: 手計算による期待値と一致する。
        #[test]
        fn matches_expected_for_typical_small_inputs() {
            // Given
            let a = vec![1, 2, 3];
            let b = vec![4, 5, 6];
            let expected = vec![4, 13, 28, 27, 18];

            // When
            let result = convolution(a, b);

            // Then
            assert_eq!(expected.len(), result.len(), "length mismatch");
            assert_eq!(expected, result);
        }

        /// Scenario: 愚直計算の閾値を超える入力でも、 愚直に計算した結果と一致する。
        /// - Given: 長さ 64 の、 全要素が `1` の係数列が 2 つある。
        /// - When: `convolution` を実行する。
        /// - Then: 二重ループで愚直に計算した期待値と一致する。
        #[test]
        fn matches_naive_result_for_larger_inputs() {
            // Given
            let a = vec![1u32; 64];
            let b = vec![1u32; 64];
            let expected_len = a.len() + b.len() - 1;
            let mut expected = vec![0u32; expected_len];
            for (i, &x) in a.iter().enumerate() {
                for (j, &y) in b.iter().enumerate() {
                    expected[i + j] += x * y;
                }
            }

            // When
            let result = convolution(a, b);

            // Then
            assert_eq!(expected.len(), result.len(), "length mismatch");
            assert_eq!(expected, result);
        }

        /// Scenario: `MOD` 付近の値同士でも、 法を考慮した期待値と一致する (境界値)。
        /// - Given: `MOD` 未満かつ `MOD` に近い値を含む係数列が 2 つある。
        /// - When: `convolution` を実行する。
        /// - Then: 法を取りながら愚直に計算した期待値と一致する。
        #[test]
        fn handles_values_near_modulus() {
            // Given
            let m = MOD;
            let a = vec![m - 1, m - 2];
            let b = vec![2, 3];
            let mut expected = vec![0u32; a.len() + b.len() - 1];
            for (i, &x) in a.iter().enumerate() {
                for (j, &y) in b.iter().enumerate() {
                    let prod = ((x as u64 * y as u64) % m as u64) as u32;
                    expected[i + j] = ((expected[i + j] as u64 + prod as u64) % m as u64) as u32;
                }
            }

            // When
            let result = convolution(a, b);

            // Then
            assert_eq!(expected, result);
        }

        /// Scenario: 大きな乱数列に対しても、 参照実装と同じ結果を返す。
        /// - Given: 長さ 100 と 120 の擬似乱数列がある。
        /// - When: `convolution` を実行する。
        /// - Then: 参照実装 `convolution_ref` の結果と一致する。
        #[test]
        fn matches_reference_for_large_random_inputs() {
            // Given
            let a = gen_values(100, 12345);
            let b = gen_values(120, 54321);
            let expected = convolution_ref(&a, &b);

            // When
            let result = convolution(a, b);

            // Then
            assert_eq!(expected, result);
        }

        /// Scenario: 結果長が `MAX_NTT_LEN` を超える場合はパニックする (異常系)。
        /// - Given: パディング後の長さが `MAX_NTT_LEN` を超える 2 つの係数列がある。
        /// - When: `convolution` を実行する。
        /// - Then: パニックする。
        #[test]
        #[should_panic(expected = "Convolution length")]
        fn panics_when_length_exceeds_max_ntt_len() {
            // Given
            let len_a = MAX_NTT_LEN;
            let len_b = 64;
            let a = vec![1u32; len_a];
            let b = vec![1u32; len_b];

            // When, Then (panic)
            let _ = convolution(a, b);
        }
    }
}
