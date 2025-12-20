//! 998244353 を法とした Montgomery 表現 + AVX2 による NTT などの実装である.

#![cfg(target_arch = "x86_64")]

use super::super::modulo998244353::modulo;
use super::convolution;
use std::arch::x86_64;
use std::sync;

/// Montgomery reduction で用いる `-MOD^{-1} mod 2^32` である.
const N_INV: u32 = 998244351;
/// Montgomery 表現における `R = 2^32 mod MOD`.
pub const R: u32 = 301989884;
/// Montgomery 表現における `R^2 = 2^64 mod MOD`.
const R2: u32 = 932051910;

/// NTT の各段で用いる回転因子のテーブル (モンゴメリ表現).
pub const NTT_RATE_MONT: [u32; 22] = [
    0x2934548a, 0x125558a6, 0x21c90447, 0x34588745, 0x165c4943, 0x83d9830, 0x1dd6967a, 0x4b74372,
    0x2f24280c, 0x3a503634, 0x26d3f337, 0x12667d13, 0x2b181adb, 0x1c4cd5c4, 0x28bbc449, 0x2a18c05,
    0x2000526a, 0x3860c1e5, 0xa74a97e, 0x1ff54d24, 0x31931580, 0x2b009445,
];
/// 逆 NTT の各段で用いる回転因子のテーブル (モンゴメリ表現).
pub const INTT_RATE_MONT: [u32; 22] = [
    0x124bab77, 0x34f7035f, 0x844bfb0, 0x3ea0705, 0x323893d2, 0x38d16113, 0xba20d91, 0x7137c51,
    0x2f35c41b, 0x316125c4, 0x362a09f8, 0xd06f7b0, 0x25764555, 0xecb65ec, 0x21c524da, 0x5fe919,
    0x4ebf1f8, 0x2fab632, 0xd6f87e4, 0x14cfdeae, 0x3aaa342, 0x2d7dadf0,
];
/// `2^k` (k = 0..=22) の `MOD` における逆元のテーブル (モンゴメリ表現).
const INVS_MONT: [u32; 23] = [
    0x11fffffc, 0x08fffffe, 0x047fffff, 0x20000000, 0x10000000, 0x08000000, 0x04000000, 0x02000000,
    0x01000000, 0x00800000, 0x00400000, 0x00200000, 0x00100000, 0x00080000, 0x00040000, 0x00020000,
    0x00010000, 0x00008000, 0x00004000, 0x00002000, 0x00001000, 0x00000800, 0x00000400,
];

/// `0..=MAX_NTT_LEN` の整数列を Montgomery 表現で保持する.
static NUM_MONT_TABLE: sync::OnceLock<Box<[u32]>> = sync::OnceLock::new();

/// `0..=MAX_NTT_LEN` の逆元列を Montgomery 表現で保持する.
static INV_NUM_MONT_TABLE: sync::OnceLock<Box<[u32]>> = sync::OnceLock::new();

/// `0..=MAX_NTT_LEN` の整数列を Montgomery 表現で返す.
///
/// # Args
/// 引数はない.
///
/// # Returns
/// `&'static [u32]`: `i` 番目が `i` (Montgomery 表現) となるテーブル.
///
/// # Constraints
/// - `convolution::MAX_NTT_LEN < 998244353` を満たす.
///
/// # Panics
/// - この関数はパニックしない.
///
/// # Complexity
/// - Time complexity: 初回のみ O(MAX_NTT_LEN), 2 回目以降は O(1).
/// - Space complexity: O(MAX_NTT_LEN).
///
/// # Examples
/// ```rust
/// use anmitsu::modulo998244353::convolution_mont;
///
/// let table = convolution_mont::num_mont_table();
/// assert_eq!(0, convolution_mont::mont_to_standard_scalar(table[0]));
/// assert_eq!(5, convolution_mont::mont_to_standard_scalar(table[5]));
/// ```
pub fn num_mont_table() -> &'static [u32] {
    NUM_MONT_TABLE
        .get_or_init(|| {
            let max_len = convolution::MAX_NTT_LEN;
            let mut table = vec![0_u32; max_len + 1];
            for i in 0..=max_len {
                table[i] = standard_to_mont_scalar(i as u32);
            }
            table.into_boxed_slice()
        })
        .as_ref()
}

/// `0..=MAX_NTT_LEN` の逆元列を Montgomery 表現で返す.
///
/// # Args
/// 引数はない.
///
/// # Returns
/// `&'static [u32]`: `i` 番目が `inv(i)` (Montgomery 表現) となるテーブルである.
/// ただし `inv(0)` は 0 とする.
///
/// # Constraints
/// - `convolution::MAX_NTT_LEN + 1 < 998244353` を満たす.
///
/// # Panics
/// - この関数はパニックしない.
///
/// # Complexity
/// - Time complexity: 初回のみ O(MAX_NTT_LEN), 2 回目以降は O(1).
/// - Space complexity: O(MAX_NTT_LEN).
///
/// # Examples
/// ```rust
/// use anmitsu::modulo998244353::convolution_mont;
/// use anmitsu::modulo998244353::modulo;
///
/// let table = convolution_mont::inv_num_mont_table();
/// let inv_5 = convolution_mont::mont_to_standard_scalar(table[5]);
/// assert_eq!(modulo::inv(5), inv_5);
/// ```
pub fn inv_num_mont_table() -> &'static [u32] {
    INV_NUM_MONT_TABLE
        .get_or_init(|| {
            let max_len = convolution::MAX_NTT_LEN;
            let inv_indices = modulo::build_inv_indices(max_len + 1);
            let mut table = vec![0_u32; max_len + 1];
            for i in 0..=max_len {
                table[i] = standard_to_mont_scalar(inv_indices[i]);
            }
            table.into_boxed_slice()
        })
        .as_ref()
}

/// AVX2 の 32-bit lane 数である.
const AVX2_U32_LANES: usize = 8;

/// Montgomery reduction を行う.
#[inline(always)]
pub fn reduce_mont(val: u64) -> u32 {
    let t = (val as u32).wrapping_mul(N_INV);
    let res = ((val + t as u64 * convolution::MOD as u64) >> 32) as u32;
    if res >= convolution::MOD {
        res - convolution::MOD
    } else {
        res
    }
}

/// Montgomery 乗算を行う.
#[inline(always)]
pub fn mul_mont(a: u32, b: u32) -> u32 {
    reduce_mont(a as u64 * b as u64)
}

/// スカラー値を Montgomery 表現へ変換する.
#[inline(always)]
pub fn standard_to_mont_scalar(x: u32) -> u32 {
    mul_mont(x, R2)
}

/// Montgomery 表現をスカラー値へ変換する.
///
/// AVX2 が利用可能であっても, 長さが小さい場合は SIMD 化のオーバーヘッドが
/// 支配的になり得るため, スカラーでの変換関数も用意する.
#[inline(always)]
pub fn mont_to_standard_scalar(x_mont: u32) -> u32 {
    reduce_mont(x_mont as u64)
}

/// NTT 長が `2^lg` のときの逆元 (Montgomery 表現) を返す.
#[inline(always)]
pub fn inv_len_mont(lg: usize) -> u32 {
    INVS_MONT[lg]
}

/// NTT doubling で用いる `ζ` の最大 `lg` である.
///
/// `n = 2^lg` に対して `2n <= MAX_NTT_LEN` を満たす必要があるため, 最大 `lg` は
/// `log2(MAX_NTT_LEN) - 1` となる.
const NTT_DOUBLING_LG_MAX: usize = convolution::MAX_NTT_LEN.trailing_zeros() as usize - 1;

/// NTT doubling で用いる `ζ` のテーブル (通常表現) である.
///
/// `NTT_DOUBLING_ZETA[lg]` は `n = 2^lg` に対する `2n` の原始根 `ζ` を表す.
const NTT_DOUBLING_ZETA: [u32; NTT_DOUBLING_LG_MAX + 1] = build_ntt_doubling_zeta();

/// NTT doubling で用いる `ζ` テーブルを構築する.
///
/// # Args
/// - `()`: 引数はない.
///
/// # Returns
/// `[u32; NTT_DOUBLING_LG_MAX + 1]`: `ζ` テーブル.
///
/// # Constraints
/// - `convolution::MAX_NTT_LEN` は 2 の冪である.
///
/// # Panics
/// - この関数はパニックしない.
///
/// # Complexity
/// - Time complexity: O(log MAX_NTT_LEN).
/// - Space complexity: O(1).
///
/// # Examples
/// ```rust,ignore
/// // 内部定数のため, 直接の使用例は省略する.
/// ```
const fn build_ntt_doubling_zeta() -> [u32; NTT_DOUBLING_LG_MAX + 1] {
    let mut table = [0_u32; NTT_DOUBLING_LG_MAX + 1];
    let mut lg = 0usize;
    while lg <= NTT_DOUBLING_LG_MAX {
        let n = 1usize << lg;
        let exp = (modulo::M as usize - 1) / (2 * n);
        table[lg] = modulo::pow(convolution::PRIMITIVE_ROOT, exp);
        lg += 1;
    }
    table
}

/// NTT doubling で用いる回転因子列 (Montgomery 表現) をキャッシュする.
struct NttDoublingPowersMont {
    zeta_mont_by_lg: [u32; NTT_DOUBLING_LG_MAX + 1],
    pows_mont_by_lg: [sync::OnceLock<Box<[u32]>>; NTT_DOUBLING_LG_MAX + 1],
}

impl NttDoublingPowersMont {
    /// `NttDoublingPowersMont` を初期化する.
    fn new() -> Self {
        let zeta_mont_by_lg =
            std::array::from_fn(|lg| standard_to_mont_scalar(NTT_DOUBLING_ZETA[lg]));
        let pows_mont_by_lg = std::array::from_fn(|_| sync::OnceLock::new());
        Self {
            zeta_mont_by_lg,
            pows_mont_by_lg,
        }
    }

    /// `n = 2^lg` に対して `ζ^i` (i = 0..n-1) の列を返す.
    fn powers(&self, n: usize) -> &[u32] {
        debug_assert!(n.is_power_of_two());
        debug_assert!(n > 0);
        debug_assert!(2 * n <= convolution::MAX_NTT_LEN);

        let lg = n.trailing_zeros() as usize;
        debug_assert!(lg <= NTT_DOUBLING_LG_MAX);

        self.pows_mont_by_lg[lg]
            .get_or_init(|| {
                let zeta_mont = self.zeta_mont_by_lg[lg];
                let mut res = vec![0_u32; n];
                let mut p = standard_to_mont_scalar(1);
                for v in res.iter_mut() {
                    *v = p;
                    p = mul_mont(p, zeta_mont);
                }
                res.into_boxed_slice()
            })
            .as_ref()
    }
}

/// NTT doubling で用いる回転因子列のキャッシュである.
static NTT_DOUBLING_POWERS_MONT: sync::OnceLock<NttDoublingPowersMont> = sync::OnceLock::new();

/// 初期化済みの NTT 前計算を保持する.
static NTT: sync::OnceLock<Ntt> = sync::OnceLock::new();

/// NTT doubling で用いる `ζ^i` (i = 0..n-1) の列 (Montgomery 表現) を返す.
///
/// # Args
/// - `n`: `2^lg` となる NTT 長である.
///
/// # Returns
/// `&'static [u32]`: `ζ^i` の列 (Montgomery 表現) を返す.
///
/// # Constraints
/// - `n` は 0 ではない 2 の冪である.
/// - `2 * n <= convolution::MAX_NTT_LEN` を満たす.
///
/// # Panics
/// - この関数はパニックし得る (デバッグアサート).
///
/// # Complexity
/// - Time complexity: 初回のみ O(n), 2 回目以降は O(1).
/// - Space complexity: O(n).
///
/// # Examples
/// ```rust,ignore
/// // AVX2 依存の FPS 実装の内部で利用する.
/// ```
pub fn ntt_doubling_powers_mont(n: usize) -> &'static [u32] {
    debug_assert!(n.is_power_of_two());
    debug_assert!(n > 0);
    debug_assert!(2 * n <= convolution::MAX_NTT_LEN);

    NTT_DOUBLING_POWERS_MONT
        .get_or_init(|| NttDoublingPowersMont::new())
        .powers(n)
}

/// AVX2 を用いた Montgomery 演算の補助テーブル.
#[derive(Clone, Copy)]
struct MontgomerySimd {
    mod_v: x86_64::__m256i,
    n_inv_v: x86_64::__m256i,
    r2_v: x86_64::__m256i,
}

impl MontgomerySimd {
    /// `MontgomerySimd` を初期化する.
    #[target_feature(enable = "avx2")]
    unsafe fn new() -> Self {
        Self {
            mod_v: x86_64::_mm256_set1_epi32(convolution::MOD as i32),
            n_inv_v: x86_64::_mm256_set1_epi32(N_INV as i32),
            r2_v: x86_64::_mm256_set1_epi32(R2 as i32),
        }
    }

    /// 値を `0..MOD` に正規化する.
    #[inline(always)]
    fn shrink(self, vec: x86_64::__m256i) -> x86_64::__m256i {
        unsafe { x86_64::_mm256_min_epu32(vec, x86_64::_mm256_sub_epi32(vec, self.mod_v)) }
    }

    /// `__m256i` 上で Montgomery reduction を行う.
    #[inline(always)]
    unsafe fn reduce(self, x0246: x86_64::__m256i, x1357: x86_64::__m256i) -> x86_64::__m256i {
        unsafe {
            let x0246_ninv = x86_64::_mm256_mul_epu32(x0246, self.n_inv_v);
            let x1357_ninv = x86_64::_mm256_mul_epu32(x1357, self.n_inv_v);

            let x0246_res =
                x86_64::_mm256_add_epi64(x0246, x86_64::_mm256_mul_epu32(x0246_ninv, self.mod_v));
            let x1357_res =
                x86_64::_mm256_add_epi64(x1357, x86_64::_mm256_mul_epu32(x1357_ninv, self.mod_v));

            let mut res =
                x86_64::_mm256_or_si256(x86_64::_mm256_bsrli_epi128(x0246_res, 4), x1357_res);
            res = self.shrink(res);
            res
        }
    }

    /// `u32x8` に対する Montgomery 乗算を行う.
    #[inline(always)]
    unsafe fn mul_u32x8<const B_USE_ONLY_EVEN: bool>(
        self,
        a: x86_64::__m256i,
        b: x86_64::__m256i,
    ) -> x86_64::__m256i {
        unsafe {
            let a_sh = x86_64::_mm256_bsrli_epi128(a, 4);
            let b_sh = if B_USE_ONLY_EVEN {
                b
            } else {
                x86_64::_mm256_bsrli_epi128(b, 4)
            };
            let x0246 = x86_64::_mm256_mul_epu32(a, b);
            let x1357 = x86_64::_mm256_mul_epu32(a_sh, b_sh);
            self.reduce(x0246, x1357)
        }
    }
}

/// AVX2 用 NTT の前計算テーブル.
struct Ntt {
    mts: MontgomerySimd,
    ntt_rots_by_chunks_lg_max: Box<[u32]>,
    intt_rots_by_chunks_lg_max: Box<[u32]>,
}

impl Ntt {
    /// NTT/INTT の回転因子列を構築する.
    ///
    /// # Args
    /// - `rate`: `trailing_ones` に応じて回転因子を更新するためのレート表
    ///   (Montgomery 表現) である.
    /// - `chunks_lg`: 生成する回転因子列の長さ `chunks = 2^chunks_lg` を表す.
    ///
    /// # Returns
    /// `Box<[u32]>`: 各 `s` に対する回転因子 `rot[s]` の列を返す.
    ///
    /// # Constraints
    /// - `chunks_lg < 22` を満たす必要がある.
    ///
    /// # Panics
    /// - `chunks_lg` が大きすぎて `1usize << chunks_lg` がオーバーフローする場合,
    ///   パニックし得る.
    ///
    /// # Complexity
    /// - Time complexity: O(2^chunks_lg), ここで `chunks_lg` は引数である.
    /// - Space complexity: O(2^chunks_lg), ここで `chunks_lg` は引数である.
    ///
    /// # Examples
    /// ```rust,ignore
    /// // 内部前計算用関数のため, 直接の使用例は省略する.
    /// ```
    fn build_rot_pows(rate: &[u32; 22], chunks_lg: usize) -> Box<[u32]> {
        // NTT/INTT の呼び出しごとの mul_mont を削減するため, 回転因子列を事前計算する.
        let chunks = 1usize << chunks_lg;
        let mut res = vec![0_u32; chunks];
        let mut rot = standard_to_mont_scalar(1);

        for (s, v) in res.iter_mut().enumerate() {
            *v = rot;

            let idx = (s as u32).trailing_ones() as usize;
            rot = mul_mont(rot, rate[idx]);
        }

        res.into_boxed_slice()
    }

    /// NTT の前計算テーブルを構築する.
    #[target_feature(enable = "avx2")]
    unsafe fn new() -> Self {
        unsafe {
            let mts = MontgomerySimd::new();
            let max_chunks_lg = convolution::MAX_NTT_LEN.trailing_zeros() as usize - 1;
            let ntt_rots_by_chunks_lg_max = Self::build_rot_pows(&NTT_RATE_MONT, max_chunks_lg);
            let intt_rots_by_chunks_lg_max = Self::build_rot_pows(&INTT_RATE_MONT, max_chunks_lg);

            Self {
                mts,
                ntt_rots_by_chunks_lg_max,
                intt_rots_by_chunks_lg_max,
            }
        }
    }

    /// 通常表現を Montgomery 表現へ変換する.
    #[target_feature(enable = "avx2")]
    unsafe fn to_mont(&self, data: *mut u32, len: usize) {
        unsafe {
            debug_assert!(len.is_power_of_two());
            debug_assert!(len >= AVX2_U32_LANES);
            debug_assert_eq!(0, len % AVX2_U32_LANES);

            let mts = self.mts;
            for i in (0..len).step_by(AVX2_U32_LANES) {
                let v = x86_64::_mm256_loadu_si256(data.add(i).cast());
                let v = mts.mul_u32x8::<true>(v, mts.r2_v);
                x86_64::_mm256_storeu_si256(data.add(i).cast(), v);
            }
        }
    }

    /// Montgomery 表現を通常表現へ変換する.
    #[target_feature(enable = "avx2")]
    unsafe fn to_standard(&self, data: *mut u32, len: usize) {
        unsafe {
            debug_assert!(len.is_power_of_two());
            debug_assert!(len >= AVX2_U32_LANES);
            debug_assert_eq!(0, len % AVX2_U32_LANES);

            let mts = self.mts;
            let one = x86_64::_mm256_set1_epi32(1);
            for i in (0..len).step_by(AVX2_U32_LANES) {
                let v = x86_64::_mm256_loadu_si256(data.add(i).cast());
                let v = mts.mul_u32x8::<true>(v, one);
                x86_64::_mm256_storeu_si256(data.add(i).cast(), v);
            }
        }
    }

    /// Montgomery 表現上で点ごとの積を計算する.
    #[target_feature(enable = "avx2")]
    unsafe fn mul_pointwise_mont(&self, a: *mut u32, b: *const u32, len: usize) {
        unsafe {
            debug_assert!(len.is_power_of_two());
            debug_assert!(len >= AVX2_U32_LANES);
            debug_assert_eq!(0, len % AVX2_U32_LANES);

            let mts = self.mts;
            for i in (0..len).step_by(AVX2_U32_LANES) {
                let va = x86_64::_mm256_loadu_si256(a.add(i).cast());
                let vb = x86_64::_mm256_loadu_si256(b.add(i).cast());
                let prod = mts.mul_u32x8::<false>(va, vb);
                x86_64::_mm256_storeu_si256(a.add(i).cast(), prod);
            }
        }
    }

    /// Montgomery 表現上でスカラー倍を行う.
    #[target_feature(enable = "avx2")]
    unsafe fn mul_scalar_mont(&self, data: *mut u32, len: usize, sc_mont: u32) {
        unsafe {
            debug_assert!(len.is_power_of_two());
            debug_assert!(len >= AVX2_U32_LANES);
            debug_assert_eq!(0, len % AVX2_U32_LANES);

            let mts = self.mts;
            let sc_v = x86_64::_mm256_set1_epi32(sc_mont as i32);
            for i in (0..len).step_by(AVX2_U32_LANES) {
                let v = x86_64::_mm256_loadu_si256(data.add(i).cast());
                let v = mts.mul_u32x8::<true>(v, sc_v);
                x86_64::_mm256_storeu_si256(data.add(i).cast(), v);
            }
        }
    }

    /// Montgomery 表現上で, 数論変換を実行する.
    #[target_feature(enable = "avx2")]
    unsafe fn ntt_mont(&self, data: *mut u32, len: usize) {
        unsafe {
            debug_assert!(len.is_power_of_two());
            debug_assert!(len >= AVX2_U32_LANES);
            let lg = len.trailing_zeros() as usize;
            let n = len;
            let mts = self.mts;

            for stage in 0..lg {
                let p = 1usize << (lg - stage - 1);
                let step = 1usize << (lg - stage);

                let chunks = n / step;
                let rots = self.ntt_rots_by_chunks_lg_max.as_ref();
                debug_assert!(rots.len() >= chunks);
                if p >= 8 {
                    for s in 0..chunks {
                        let ptr = data.add(s * step);

                        let rot = rots[s];
                        let rot_v = x86_64::_mm256_set1_epi32(rot as i32);
                        for i in (0..p).step_by(8) {
                            let l = x86_64::_mm256_loadu_si256(ptr.add(i).cast());
                            let r = x86_64::_mm256_loadu_si256(ptr.add(i + p).cast());
                            let r = mts.mul_u32x8::<true>(r, rot_v);
                            let sum = mts.shrink(x86_64::_mm256_add_epi32(l, r));
                            let diff = mts.shrink(x86_64::_mm256_sub_epi32(
                                x86_64::_mm256_add_epi32(l, mts.mod_v),
                                r,
                            ));
                            x86_64::_mm256_storeu_si256(ptr.add(i).cast(), sum);
                            x86_64::_mm256_storeu_si256(ptr.add(i + p).cast(), diff);
                        }
                    }
                } else {
                    if p == 4 {
                        for s in 0..chunks {
                            let ptr = data.add(8 * s);
                            let rot = rots[s];

                            let l0 = *ptr.add(0);
                            let l1 = *ptr.add(1);
                            let l2 = *ptr.add(2);
                            let l3 = *ptr.add(3);
                            let r0 = mul_mont(*ptr.add(4), rot);
                            let r1 = mul_mont(*ptr.add(5), rot);
                            let r2 = mul_mont(*ptr.add(6), rot);
                            let r3 = mul_mont(*ptr.add(7), rot);

                            *ptr.add(0) = super::modulo::add(l0, r0);
                            *ptr.add(1) = super::modulo::add(l1, r1);
                            *ptr.add(2) = super::modulo::add(l2, r2);
                            *ptr.add(3) = super::modulo::add(l3, r3);
                            *ptr.add(5) = super::modulo::sub(l1, r1);
                            *ptr.add(4) = super::modulo::sub(l0, r0);
                            *ptr.add(6) = super::modulo::sub(l2, r2);
                            *ptr.add(7) = super::modulo::sub(l3, r3);
                        }
                    } else if p == 2 {
                        debug_assert_eq!(0, chunks % 2);
                        let idx_l = x86_64::_mm256_setr_epi32(0, 1, 0, 1, 4, 5, 4, 5);
                        let idx_r = x86_64::_mm256_setr_epi32(2, 3, 2, 3, 6, 7, 6, 7);

                        for s in (0..chunks).step_by(2) {
                            let s1 = s + 1;
                            let rot0 = rots[s];
                            let rot1 = rots[s1];

                            let mul_v = x86_64::_mm256_setr_epi32(
                                R as i32,
                                R as i32,
                                rot0 as i32,
                                rot0 as i32,
                                R as i32,
                                R as i32,
                                rot1 as i32,
                                rot1 as i32,
                            );

                            let ptr = data.add(4 * s);
                            let v = x86_64::_mm256_loadu_si256(ptr.cast());
                            let v = mts.mul_u32x8::<true>(v, mul_v);

                            let l = x86_64::_mm256_permutevar8x32_epi32(v, idx_l);
                            let r = x86_64::_mm256_permutevar8x32_epi32(v, idx_r);

                            let sum = mts.shrink(x86_64::_mm256_add_epi32(l, r));
                            let diff = mts.shrink(x86_64::_mm256_sub_epi32(
                                x86_64::_mm256_add_epi32(l, mts.mod_v),
                                r,
                            ));
                            let out = x86_64::_mm256_blend_epi32(sum, diff, 0xCC);
                            x86_64::_mm256_storeu_si256(ptr.cast(), out);
                        }
                    } else if p == 1 {
                        debug_assert_eq!(0, chunks % 4);
                        let idx_l = x86_64::_mm256_setr_epi32(0, 0, 2, 2, 4, 4, 6, 6);
                        let idx_r = x86_64::_mm256_setr_epi32(1, 1, 3, 3, 5, 5, 7, 7);

                        for s in (0..chunks).step_by(4) {
                            let s1 = s + 1;
                            let s2 = s + 2;
                            let s3 = s + 3;
                            let rot0 = rots[s];
                            let rot1 = rots[s1];
                            let rot2 = rots[s2];
                            let rot3 = rots[s3];

                            let mul_v = x86_64::_mm256_setr_epi32(
                                R as i32,
                                rot0 as i32,
                                R as i32,
                                rot1 as i32,
                                R as i32,
                                rot2 as i32,
                                R as i32,
                                rot3 as i32,
                            );

                            let ptr = data.add(2 * s);
                            let v = x86_64::_mm256_loadu_si256(ptr.cast());
                            let v = mts.mul_u32x8::<false>(v, mul_v);

                            let l = x86_64::_mm256_permutevar8x32_epi32(v, idx_l);
                            let r = x86_64::_mm256_permutevar8x32_epi32(v, idx_r);

                            let sum = mts.shrink(x86_64::_mm256_add_epi32(l, r));
                            let diff = mts.shrink(x86_64::_mm256_sub_epi32(
                                x86_64::_mm256_add_epi32(l, mts.mod_v),
                                r,
                            ));
                            let out = x86_64::_mm256_blend_epi32(sum, diff, 0xAA);
                            x86_64::_mm256_storeu_si256(ptr.cast(), out);
                        }
                    } else {
                        unreachable!();
                    }
                }
            }
        }
    }

    /// Montgomery 表現上で, 逆数論変換を実行する.
    #[target_feature(enable = "avx2")]
    unsafe fn intt_mont(&self, data: *mut u32, len: usize) {
        unsafe {
            debug_assert!(len.is_power_of_two());
            debug_assert!(len >= AVX2_U32_LANES);
            let lg = len.trailing_zeros() as usize;
            let n = len;
            let mts = self.mts;

            for stage in (1..=lg).rev() {
                let p = 1usize << (lg - stage);
                let step = 1usize << (lg - stage + 1);

                let chunks = n / step;
                let irots = self.intt_rots_by_chunks_lg_max.as_ref();
                debug_assert!(irots.len() >= chunks);
                if p >= 8 {
                    for s in 0..chunks {
                        let ptr = data.add(s * step);

                        let irot = irots[s];
                        let irot_v = x86_64::_mm256_set1_epi32(irot as i32);
                        for i in (0..p).step_by(8) {
                            let l = x86_64::_mm256_loadu_si256(ptr.add(i).cast());
                            let r = x86_64::_mm256_loadu_si256(ptr.add(i + p).cast());
                            let sum = mts.shrink(x86_64::_mm256_add_epi32(l, r));
                            let diff = mts.shrink(x86_64::_mm256_sub_epi32(
                                x86_64::_mm256_add_epi32(l, mts.mod_v),
                                r,
                            ));
                            let diff = mts.mul_u32x8::<true>(diff, irot_v);
                            x86_64::_mm256_storeu_si256(ptr.add(i).cast(), sum);
                            x86_64::_mm256_storeu_si256(ptr.add(i + p).cast(), diff);
                        }
                    }
                } else {
                    if p == 4 {
                        for s in 0..chunks {
                            let ptr = data.add(8 * s);
                            let irot = irots[s];

                            let l0 = *ptr.add(0);
                            let l1 = *ptr.add(1);
                            let l2 = *ptr.add(2);
                            let l3 = *ptr.add(3);
                            let r0 = *ptr.add(4);
                            let r1 = *ptr.add(5);
                            let r2 = *ptr.add(6);
                            let r3 = *ptr.add(7);

                            let sum0 = super::modulo::add(l0, r0);
                            let sum1 = super::modulo::add(l1, r1);
                            let sum2 = super::modulo::add(l2, r2);
                            let sum3 = super::modulo::add(l3, r3);
                            let diff0 = super::modulo::sub(l0, r0);
                            let diff1 = super::modulo::sub(l1, r1);
                            let diff2 = super::modulo::sub(l2, r2);
                            let diff3 = super::modulo::sub(l3, r3);

                            *ptr.add(0) = sum0;
                            *ptr.add(1) = sum1;
                            *ptr.add(2) = sum2;
                            *ptr.add(3) = sum3;
                            *ptr.add(4) = mul_mont(diff0, irot);
                            *ptr.add(5) = mul_mont(diff1, irot);
                            *ptr.add(6) = mul_mont(diff2, irot);
                            *ptr.add(7) = mul_mont(diff3, irot);
                        }
                    } else if p == 2 {
                        debug_assert_eq!(0, chunks % 2);
                        let idx_l = x86_64::_mm256_setr_epi32(0, 1, 0, 1, 4, 5, 4, 5);
                        let idx_r = x86_64::_mm256_setr_epi32(2, 3, 2, 3, 6, 7, 6, 7);

                        for s in (0..chunks).step_by(2) {
                            let s1 = s + 1;
                            let irot0 = irots[s];
                            let irot1 = irots[s1];

                            let mul_v = x86_64::_mm256_setr_epi32(
                                R as i32,
                                R as i32,
                                irot0 as i32,
                                irot0 as i32,
                                R as i32,
                                R as i32,
                                irot1 as i32,
                                irot1 as i32,
                            );

                            let ptr = data.add(4 * s);
                            let v = x86_64::_mm256_loadu_si256(ptr.cast());

                            let l = x86_64::_mm256_permutevar8x32_epi32(v, idx_l);
                            let r = x86_64::_mm256_permutevar8x32_epi32(v, idx_r);

                            let sum = mts.shrink(x86_64::_mm256_add_epi32(l, r));
                            let diff = mts.shrink(x86_64::_mm256_sub_epi32(
                                x86_64::_mm256_add_epi32(l, mts.mod_v),
                                r,
                            ));
                            let out = x86_64::_mm256_blend_epi32(sum, diff, 0xCC);
                            let out = mts.mul_u32x8::<true>(out, mul_v);
                            x86_64::_mm256_storeu_si256(ptr.cast(), out);
                        }
                    } else if p == 1 {
                        debug_assert_eq!(0, chunks % 4);
                        let idx_l = x86_64::_mm256_setr_epi32(0, 0, 2, 2, 4, 4, 6, 6);
                        let idx_r = x86_64::_mm256_setr_epi32(1, 1, 3, 3, 5, 5, 7, 7);

                        for s in (0..chunks).step_by(4) {
                            let s1 = s + 1;
                            let s2 = s + 2;
                            let s3 = s + 3;
                            let irot0 = irots[s];
                            let irot1 = irots[s1];
                            let irot2 = irots[s2];
                            let irot3 = irots[s3];

                            let mul_v = x86_64::_mm256_setr_epi32(
                                R as i32,
                                irot0 as i32,
                                R as i32,
                                irot1 as i32,
                                R as i32,
                                irot2 as i32,
                                R as i32,
                                irot3 as i32,
                            );

                            let ptr = data.add(2 * s);
                            let v = x86_64::_mm256_loadu_si256(ptr.cast());

                            let l = x86_64::_mm256_permutevar8x32_epi32(v, idx_l);
                            let r = x86_64::_mm256_permutevar8x32_epi32(v, idx_r);

                            let sum = mts.shrink(x86_64::_mm256_add_epi32(l, r));
                            let diff = mts.shrink(x86_64::_mm256_sub_epi32(
                                x86_64::_mm256_add_epi32(l, mts.mod_v),
                                r,
                            ));
                            let out = x86_64::_mm256_blend_epi32(sum, diff, 0xAA);
                            let out = mts.mul_u32x8::<false>(out, mul_v);
                            x86_64::_mm256_storeu_si256(ptr.cast(), out);
                        }
                    } else {
                        unreachable!();
                    }
                }
            }
        }
    }
}

/// AVX2 を用いて, Montgomery 表現へ変換する.
#[target_feature(enable = "avx2")]
pub unsafe fn standard_to_mont(a: &mut [u32]) {
    unsafe {
        debug_assert!(!a.is_empty());
        debug_assert!(a.len().is_power_of_two());

        if a.len() == 1 {
            a[0] = standard_to_mont_scalar(a[0]);
            return;
        } else if a.len() == 2 {
            a[0] = standard_to_mont_scalar(a[0]);
            a[1] = standard_to_mont_scalar(a[1]);
            return;
        } else if a.len() == 4 {
            a[0] = standard_to_mont_scalar(a[0]);
            a[1] = standard_to_mont_scalar(a[1]);
            a[2] = standard_to_mont_scalar(a[2]);
            a[3] = standard_to_mont_scalar(a[3]);
            return;
        }

        debug_assert!(a.len() >= AVX2_U32_LANES);
        debug_assert_eq!(0, a.len() % AVX2_U32_LANES);
        let ntt = NTT.get_or_init(|| Ntt::new());
        ntt.to_mont(a.as_mut_ptr(), a.len());
    }
}

/// AVX2 を用いて, Montgomery 表現から通常表現へ戻す.
#[target_feature(enable = "avx2")]
pub unsafe fn mont_to_standard(a: &mut [u32]) {
    unsafe {
        debug_assert!(!a.is_empty());
        debug_assert!(a.len().is_power_of_two());

        if a.len() == 1 {
            a[0] = mont_to_standard_scalar(a[0]);
            return;
        } else if a.len() == 2 {
            a[0] = mont_to_standard_scalar(a[0]);
            a[1] = mont_to_standard_scalar(a[1]);
            return;
        } else if a.len() == 4 {
            a[0] = mont_to_standard_scalar(a[0]);
            a[1] = mont_to_standard_scalar(a[1]);
            a[2] = mont_to_standard_scalar(a[2]);
            a[3] = mont_to_standard_scalar(a[3]);
            return;
        }

        debug_assert!(a.len() >= AVX2_U32_LANES);
        debug_assert_eq!(0, a.len() % AVX2_U32_LANES);
        let ntt = NTT.get_or_init(|| Ntt::new());
        ntt.to_standard(a.as_mut_ptr(), a.len());
    }
}

/// AVX2 を用いて, Montgomery 表現上の点ごとの積を計算する.
#[target_feature(enable = "avx2")]
pub unsafe fn mul_pointwise_mont(a: &mut [u32], b: &[u32]) {
    unsafe {
        debug_assert_eq!(a.len(), b.len());
        debug_assert!(!a.is_empty());
        debug_assert!(a.len().is_power_of_two());

        if a.len() == 1 {
            a[0] = mul_mont(a[0], b[0]);
            return;
        } else if a.len() == 2 {
            a[0] = mul_mont(a[0], b[0]);
            a[1] = mul_mont(a[1], b[1]);
            return;
        } else if a.len() == 4 {
            a[0] = mul_mont(a[0], b[0]);
            a[1] = mul_mont(a[1], b[1]);
            a[2] = mul_mont(a[2], b[2]);
            a[3] = mul_mont(a[3], b[3]);
            return;
        }

        let ntt = NTT.get_or_init(|| Ntt::new());
        ntt.mul_pointwise_mont(a.as_mut_ptr(), b.as_ptr(), a.len());
    }
}

/// AVX2 を用いて, Montgomery 表現上でスカラー倍を行う.
#[target_feature(enable = "avx2")]
pub unsafe fn mul_scalar_mont(a: &mut [u32], sc_mont: u32) {
    unsafe {
        let n = a.len();
        debug_assert!(n.is_power_of_two());
        debug_assert!(n > 0);

        if n < AVX2_U32_LANES {
            if n == 1 {
                a[0] = mul_mont(a[0], sc_mont);
                return;
            } else if n == 2 {
                a[0] = mul_mont(a[0], sc_mont);
                a[1] = mul_mont(a[1], sc_mont);
                return;
            } else if n == 4 {
                a[0] = mul_mont(a[0], sc_mont);
                a[1] = mul_mont(a[1], sc_mont);
                a[2] = mul_mont(a[2], sc_mont);
                a[3] = mul_mont(a[3], sc_mont);
                return;
            }
        } else {
            let ntt = NTT.get_or_init(|| Ntt::new());
            ntt.mul_scalar_mont(a.as_mut_ptr(), a.len(), sc_mont);
        }
    }
}

/// AVX2 を用いて, Montgomery 表現上のベクター同士の加算を行う (`a += b`).
///
/// # Args
/// - `a`: 加算対象の配列 (Montgomery 表現). `a[i]` は `MOD` 未満である必要がある.
/// - `b`: 加算する配列 (Montgomery 表現). `b[i]` は `MOD` 未満である必要がある.
///
/// # Returns
/// `()`: `a` を in-place に更新する.
///
/// # Constraints
/// - `a.len() == b.len()` を満たす.
/// - `a` と `b` の各要素は `convolution::MOD` 未満である.
///
/// # Panics
/// - この関数はパニックし得る (デバッグアサート).
///
/// # Complexity
/// - Time complexity: O(n), ここで n は配列長である.
/// - Space complexity: O(1).
///
/// # Examples
/// ```rust
/// use anmitsu::modulo998244353::convolution_mont;
///
/// let mut a = vec![convolution_mont::standard_to_mont_scalar(1); 8];
/// let b = vec![convolution_mont::standard_to_mont_scalar(2); 8];
/// unsafe { convolution_mont::add_assign_mont(&mut a, &b); }
/// assert_eq!(3, convolution_mont::mont_to_standard_scalar(a[0]));
/// ```
#[target_feature(enable = "avx2")]
pub unsafe fn add_assign_mont(a: &mut [u32], b: &[u32]) {
    unsafe {
        debug_assert_eq!(a.len(), b.len());
        let n = a.len();

        let mts = MontgomerySimd::new();

        let end = n / AVX2_U32_LANES * AVX2_U32_LANES;
        for i in (0..end).step_by(AVX2_U32_LANES) {
            let va = x86_64::_mm256_loadu_si256(a.as_ptr().add(i).cast());
            let vb = x86_64::_mm256_loadu_si256(b.as_ptr().add(i).cast());
            let sum = mts.shrink(x86_64::_mm256_add_epi32(va, vb));
            x86_64::_mm256_storeu_si256(a.as_mut_ptr().add(i).cast(), sum);
        }

        for i in end..n {
            a[i] = modulo::add(a[i], b[i]);
        }
    }
}

/// AVX2 を用いて, Montgomery 表現上のベクター同士の減算を行う (`a -= b`).
///
/// # Args
/// - `a`: 減算対象の配列 (Montgomery 表現). `a[i]` は `MOD` 未満である必要がある.
/// - `b`: 減算する配列 (Montgomery 表現). `b[i]` は `MOD` 未満である必要がある.
///
/// # Returns
/// `()`: `a` を in-place に更新する.
///
/// # Constraints
/// - `a.len() == b.len()` を満たす.
/// - `a` と `b` の各要素は `convolution::MOD` 未満である.
///
/// # Panics
/// - この関数はパニックし得る (デバッグアサート).
///
/// # Complexity
/// - Time complexity: O(n), ここで n は配列長である.
/// - Space complexity: O(1).
///
/// # Examples
/// ```rust
/// use anmitsu::modulo998244353::convolution_mont;
///
/// let mut a = vec![convolution_mont::standard_to_mont_scalar(5); 8];
/// let b = vec![convolution_mont::standard_to_mont_scalar(3); 8];
/// unsafe { convolution_mont::sub_assign_mont(&mut a, &b); }
/// assert_eq!(2, convolution_mont::mont_to_standard_scalar(a[0]));
/// ```
#[target_feature(enable = "avx2")]
pub unsafe fn sub_assign_mont(a: &mut [u32], b: &[u32]) {
    unsafe {
        debug_assert_eq!(a.len(), b.len());
        let n = a.len();

        let mts = MontgomerySimd::new();

        let end = n / AVX2_U32_LANES * AVX2_U32_LANES;
        for i in (0..end).step_by(AVX2_U32_LANES) {
            let va = x86_64::_mm256_loadu_si256(a.as_ptr().add(i).cast());
            let vb = x86_64::_mm256_loadu_si256(b.as_ptr().add(i).cast());
            let diff = x86_64::_mm256_sub_epi32(x86_64::_mm256_add_epi32(va, mts.mod_v), vb);
            let diff = mts.shrink(diff);
            x86_64::_mm256_storeu_si256(a.as_mut_ptr().add(i).cast(), diff);
        }

        for i in end..n {
            a[i] = modulo::sub(a[i], b[i]);
        }
    }
}

/// AVX2 を用いて, Montgomery 表現上のベクターを 2 倍する (`a *= 2`).
///
/// # Args
/// - `a`: 倍化対象の配列 (Montgomery 表現). `a[i]` は `MOD` 未満である必要がある.
///
/// # Returns
/// `()`: `a` を in-place に更新する.
///
/// # Constraints
/// - `a` の各要素は `convolution::MOD` 未満である.
///
/// # Panics
/// - この関数はパニックしない.
///
/// # Complexity
/// - Time complexity: O(n), ここで n は配列長である.
/// - Space complexity: O(1).
///
/// # Examples
/// ```rust
/// use anmitsu::modulo998244353::convolution_mont;
///
/// let mut a = vec![convolution_mont::standard_to_mont_scalar(7); 8];
/// unsafe { convolution_mont::double_mont(&mut a); }
/// assert_eq!(14, convolution_mont::mont_to_standard_scalar(a[0]));
/// ```
#[target_feature(enable = "avx2")]
pub unsafe fn double_mont(a: &mut [u32]) {
    unsafe {
        let n = a.len();
        let mts = MontgomerySimd::new();

        let end = n / AVX2_U32_LANES * AVX2_U32_LANES;
        for i in (0..end).step_by(AVX2_U32_LANES) {
            let va = x86_64::_mm256_loadu_si256(a.as_ptr().add(i).cast());
            let sum = mts.shrink(x86_64::_mm256_add_epi32(va, va));
            x86_64::_mm256_storeu_si256(a.as_mut_ptr().add(i).cast(), sum);
        }

        for i in end..n {
            a[i] = modulo::add(a[i], a[i]);
        }
    }
}

/// AVX2 を用いて, Montgomery 表現上で `a = b - a` を計算する.
///
/// # Args
/// - `a`: 結果を格納する配列 (Montgomery 表現). `a[i]` は `MOD` 未満である必要がある.
/// - `b`: 左辺となる配列 (Montgomery 表現). `b[i]` は `MOD` 未満である必要がある.
///
/// # Returns
/// `()`: `a` を in-place に更新する.
///
/// # Constraints
/// - `a.len() == b.len()` を満たす.
/// - `a` と `b` の各要素は `convolution::MOD` 未満である.
///
/// # Panics
/// - この関数はパニックし得る (デバッグアサート).
///
/// # Complexity
/// - Time complexity: O(n), ここで n は配列長である.
/// - Space complexity: O(1).
///
/// # Examples
/// ```rust
/// use anmitsu::modulo998244353::convolution_mont;
///
/// let mut a = vec![convolution_mont::standard_to_mont_scalar(3); 8];
/// let b = vec![convolution_mont::standard_to_mont_scalar(10); 8];
/// unsafe { convolution_mont::rev_sub_assign_mont(&mut a, &b); }
/// assert_eq!(7, convolution_mont::mont_to_standard_scalar(a[0]));
/// ```
#[target_feature(enable = "avx2")]
pub unsafe fn rev_sub_assign_mont(a: &mut [u32], b: &[u32]) {
    unsafe {
        debug_assert_eq!(a.len(), b.len());
        let n = a.len();

        let mts = MontgomerySimd::new();

        let end = n / AVX2_U32_LANES * AVX2_U32_LANES;
        for i in (0..end).step_by(AVX2_U32_LANES) {
            let va = x86_64::_mm256_loadu_si256(a.as_ptr().add(i).cast());
            let vb = x86_64::_mm256_loadu_si256(b.as_ptr().add(i).cast());
            let diff = x86_64::_mm256_sub_epi32(x86_64::_mm256_add_epi32(vb, mts.mod_v), va);
            let diff = mts.shrink(diff);
            x86_64::_mm256_storeu_si256(a.as_mut_ptr().add(i).cast(), diff);
        }

        for i in end..n {
            a[i] = modulo::sub(b[i], a[i]);
        }
    }
}

/// AVX2 を用いて, Montgomery 表現上で NTT を実行する.
#[target_feature(enable = "avx2")]
pub unsafe fn ntt_mont(a: &mut [u32]) {
    unsafe {
        let n = a.len();
        debug_assert!(n.is_power_of_two());
        debug_assert!(n > 0);

        if n < AVX2_U32_LANES {
            if n == 1 {
                return;
            } else if n == 2 {
                let t0 = modulo::add(a[0], a[1]);
                let t1 = modulo::sub(a[0], a[1]);
                a[0] = t0;
                a[1] = t1;
            } else if n == 4 {
                const OMEGA_1_4_MONT: u32 = 691295370;
                let e0 = modulo::add(a[0], a[2]); // a0 + a2
                let e1 = modulo::sub(a[0], a[2]); // a0 - a2
                let o0 = modulo::add(a[1], a[3]); // a1 + a3
                let o1 = modulo::sub(a[1], a[3]); // a1 - a3
                let t1 = mul_mont(OMEGA_1_4_MONT, o1);
                a[0] = modulo::add(e0, o0);
                a[1] = modulo::sub(e0, o0);
                a[2] = modulo::add(e1, t1);
                a[3] = modulo::sub(e1, t1);
            } else {
                unreachable!()
            }
        } else {
            let ntt = NTT.get_or_init(|| Ntt::new());
            ntt.ntt_mont(a.as_mut_ptr(), n);
        }
    }
}

/// AVX2 を用いて, Montgomery 表現上で INTT を実行する.
#[target_feature(enable = "avx2")]
pub unsafe fn intt_mont(a: &mut [u32]) {
    unsafe {
        let n = a.len();
        debug_assert!(n.is_power_of_two());
        debug_assert!(n > 0);

        // small-n: インライン展開
        if n < AVX2_U32_LANES {
            if n == 1 {
                return;
            } else if n == 2 {
                // 入力: [A0, A1]（n=2 は bitrev が恒等）
                // 出力: [2*a0, 2*a1]（正規化なし）
                let t0 = modulo::add(a[0], a[1]);
                let t1 = modulo::sub(a[0], a[1]);
                a[0] = t0;
                a[1] = t1;
                return;
            } else if n == 4 {
                const OMEGA_1_4_INV_MONT: u32 = 998244353 - 691295370; // inv(ω^1) (Mont)

                let e0 = modulo::add(a[0], a[1]);
                let e1 = modulo::sub(a[0], a[1]);
                let o0 = modulo::add(a[2], a[3]);
                let o1 = modulo::sub(a[2], a[3]);

                let t1 = mul_mont(OMEGA_1_4_INV_MONT, o1);

                a[0] = modulo::add(e0, o0);
                a[2] = modulo::sub(e0, o0);
                a[3] = modulo::sub(e1, t1);
                a[1] = modulo::add(e1, t1);

                return;
            } else {
                unreachable!()
            }
        } else {
            let ntt = NTT.get_or_init(|| Ntt::new());
            ntt.intt_mont(a.as_mut_ptr(), n);
        }
    }
}

#[target_feature(enable = "avx2")]
/// NTT 結果を in-place で doubling する.
///
/// # Args
/// - `a_ntt`: 長さ `n` の NTT 結果 (通常表現). この関数は `a_ntt` を in-place で更新する.
///
/// # Returns
/// `()`: `a_ntt` の長さを `2n` に拡張し, 後半 `n` 要素を計算して格納する.
///
/// # Constraints
/// - `a_ntt.len()` は 0 ではない 2 の冪である.
/// - `2 * a_ntt.len()` は `convolution::MAX_NTT_LEN` を超えてはならない.
/// - `a_ntt` の全要素は `convolution::MOD` 未満である.
/// - 呼び出し側は AVX2 が利用可能であることを保証する.
///
/// # Panics
/// - この関数はパニックし得る (デバッグアサート).
///
/// # Complexity
/// - Time complexity: O(n log n), ここで n は入力長である.
/// - Space complexity: O(n).
///
/// # Examples
/// ```rust,ignore
/// // AVX2 依存のため, 直接の使用例は省略する.
/// ```
pub unsafe fn ntt_doubling(a_ntt: &mut Vec<u32>) {
    unsafe {
        let n = a_ntt.len();
        debug_assert!(n.is_power_of_two());
        debug_assert!(2 * n <= convolution::MAX_NTT_LEN);
        standard_to_mont(a_ntt);
        ntt_doubling_mont(a_ntt);
        mont_to_standard(a_ntt);
    }
}

#[target_feature(enable = "avx2")]
/// Montgomery 表現上で NTT 結果を in-place で doubling する.
///
/// # Args
/// - `a_ntt_mont`: 長さ `n` の NTT 結果 (Montgomery 表現). この関数は `a_ntt_mont` を
///   in-place で更新する.
///
/// # Returns
/// `()`: `a_ntt_mont` の長さを `2n` に拡張し, 後半 `n` 要素を計算して格納する.
///
/// # Constraints
/// - `a_ntt_mont.len()` は 0 ではない 2 の冪である.
/// - `2 * a_ntt_mont.len()` は `convolution::MAX_NTT_LEN` を超えてはならない.
/// - 呼び出し側は AVX2 が利用可能であることを保証する.
///
/// # Panics
/// - この関数はパニックし得る (デバッグアサート).
///
/// # Complexity
/// - Time complexity: O(n log n), ここで n は入力長である.
/// - Space complexity: O(n).
///
/// # Examples
/// ```rust,ignore
/// // AVX2 依存のため, 直接の使用例は省略する.
/// ```
pub unsafe fn ntt_doubling_mont(a_ntt_mont: &mut Vec<u32>) {
    unsafe {
        let n = a_ntt_mont.len();
        debug_assert!(n.is_power_of_two());
        debug_assert!(2 * n <= convolution::MAX_NTT_LEN);
        let mut coeffs = a_ntt_mont.clone();
        intt_mont(&mut coeffs);
        let lg = n.trailing_zeros() as usize;
        let inv_n_mont = inv_len_mont(lg);
        mul_scalar_mont(&mut coeffs, inv_n_mont);
        let powers = NTT_DOUBLING_POWERS_MONT
            .get_or_init(|| NttDoublingPowersMont::new())
            .powers(n);
        let mut twisted = coeffs;
        mul_pointwise_mont(&mut twisted, powers);
        ntt_mont(&mut twisted);
        a_ntt_mont.resize(2 * n, 0);
        a_ntt_mont[n..].copy_from_slice(&twisted);
    }
}
