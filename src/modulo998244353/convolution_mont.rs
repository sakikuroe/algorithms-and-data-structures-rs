//! 998244353 を法とした Montgomery 表現 + AVX2 による NTT などの実装である。

#![cfg(target_arch = "x86_64")]

use super::{convolution, modulo};
use std::{arch::x86_64, sync};

/// Montgomery reduction で用いる `-MOD^{-1} mod 2^32` である。
const N_INV: u32 = 998244351;
/// Montgomery 表現における `R = 2^32 mod MOD`。
pub const R: u32 = 301989884;
/// Montgomery 表現における `R^2 = 2^64 mod MOD`。
const R2: u32 = 932051910;

/// NTT の各段で用いる回転因子のテーブル (モンゴメリ表現)。
pub const NTT_RATE_MONT: [u32; 22] = [
    0x2934548a, 0x125558a6, 0x21c90447, 0x34588745, 0x165c4943, 0x83d9830, 0x1dd6967a, 0x4b74372,
    0x2f24280c, 0x3a503634, 0x26d3f337, 0x12667d13, 0x2b181adb, 0x1c4cd5c4, 0x28bbc449, 0x2a18c05,
    0x2000526a, 0x3860c1e5, 0xa74a97e, 0x1ff54d24, 0x31931580, 0x2b009445,
];
/// 逆 NTT の各段で用いる回転因子のテーブル (モンゴメリ表現)。
pub const INTT_RATE_MONT: [u32; 22] = [
    0x124bab77, 0x34f7035f, 0x844bfb0, 0x3ea0705, 0x323893d2, 0x38d16113, 0xba20d91, 0x7137c51,
    0x2f35c41b, 0x316125c4, 0x362a09f8, 0xd06f7b0, 0x25764555, 0xecb65ec, 0x21c524da, 0x5fe919,
    0x4ebf1f8, 0x2fab632, 0xd6f87e4, 0x14cfdeae, 0x3aaa342, 0x2d7dadf0,
];
/// `2^k` (k = 0..=22) の `MOD` における逆元のテーブル (モンゴメリ表現)。
const INVS_MONT: [u32; 23] = [
    0x11fffffc, 0x08fffffe, 0x047fffff, 0x20000000, 0x10000000, 0x08000000, 0x04000000, 0x02000000,
    0x01000000, 0x00800000, 0x00400000, 0x00200000, 0x00100000, 0x00080000, 0x00040000, 0x00020000,
    0x00010000, 0x00008000, 0x00004000, 0x00002000, 0x00001000, 0x00000800, 0x00000400,
];

/// `0..=MAX_NTT_LEN` の整数列を Montgomery 表現で保持する。
static NUM_MONT_TABLE: sync::OnceLock<Box<[u32]>> = sync::OnceLock::new();

/// `0..=MAX_NTT_LEN` の逆元列を Montgomery 表現で保持する。
static INV_NUM_MONT_TABLE: sync::OnceLock<Box<[u32]>> = sync::OnceLock::new();

/// `0..=MAX_NTT_LEN` の整数列を Montgomery 表現で返す。
///
/// # Args
/// 引数はない。
///
/// # Returns
/// `&'static [u32]`: `i` 番目が `i` (Montgomery 表現) となるテーブル。
///
/// # Constraints
/// - `convolution::MAX_NTT_LEN < 998244353` を満たす。
///
/// # Panics
/// - この関数はパニックしない。
///
/// # Complexity
/// - Time complexity: 初回のみ O(MAX_NTT_LEN)、2 回目以降は O(1)。
/// - Space complexity: O(MAX_NTT_LEN)。
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
    // プロセス全体で 1 度だけ構築し、以後はキャッシュを返す。
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

/// `0..=MAX_NTT_LEN` の逆元列を Montgomery 表現で返す。
///
/// # Args
/// 引数はない。
///
/// # Returns
/// `&'static [u32]`: `i` 番目が `inv(i)` (Montgomery 表現) となるテーブルである。
/// ただし `inv(0)` は 0 とする。
///
/// # Constraints
/// - `convolution::MAX_NTT_LEN + 1 < 998244353` を満たす。
///
/// # Panics
/// - この関数はパニックしない。
///
/// # Complexity
/// - Time complexity: 初回のみ O(MAX_NTT_LEN)、2 回目以降は O(1)。
/// - Space complexity: O(MAX_NTT_LEN)。
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
    // プロセス全体で 1 度だけ構築し、以後はキャッシュを返す。
    INV_NUM_MONT_TABLE
        .get_or_init(|| {
            let max_len = convolution::MAX_NTT_LEN;
            // 逆元は 1 つずつ `pow` で求めると O(MAX_NTT_LEN log MOD) かかるため、
            // まとめて逆元を計算できる `build_inv_indices` に委譲する。
            let inv_indices = modulo::build_inv_indices(max_len + 1);
            let mut table = vec![0_u32; max_len + 1];
            for i in 0..=max_len {
                table[i] = standard_to_mont_scalar(inv_indices[i]);
            }
            table.into_boxed_slice()
        })
        .as_ref()
}

/// AVX2 の 32-bit lane 数である。
const AVX2_U32_LANES: usize = 8;

/// Montgomery reduction を行う。
///
/// REDC アルゴリズムにより、`val * R^{-1} mod MOD` を除算なしで計算する。
///
/// # Args
/// - `val`: reduction 対象の値
///
/// # Returns
/// `u32`: `val * R^{-1} mod MOD` を返す。
///
/// # Constraints
/// - `val < MOD * R` を満たす必要がある (この実装での `R = 2^32`)。
///
/// # Panics
/// パニックしない。
///
/// # Complexity
/// - Time complexity: O(1)。
/// - Space complexity: O(1)。
///
/// # Examples
/// ```rust
/// use anmitsu::modulo998244353::convolution_mont;
///
/// let x_mont = convolution_mont::standard_to_mont_scalar(5);
/// assert_eq!(5, convolution_mont::reduce_mont(x_mont as u64));
/// ```
#[inline(always)]
pub fn reduce_mont(val: u64) -> u32 {
    // `t = val * (-MOD^{-1}) mod 2^32` を求める。下位 32 bit のみで十分なため
    // `val as u32` の乗算 (mod 2^32 の自動ラップアラウンド) で計算できる。
    let t = (val as u32).wrapping_mul(N_INV);
    // `val + t * MOD` は構成により下位 32 bit が必ず 0 になるため、上位
    // 32 bit (`>> 32`) を取り出すだけで `val * R^{-1} mod MOD` の候補が求まる。
    let res = ((val + t as u64 * convolution::MOD as u64) >> 32) as u32;
    // 候補値は高々 `2*MOD` 未満なので、`MOD` 以上なら 1 回だけ引いて正規化する。
    if res >= convolution::MOD {
        res - convolution::MOD
    } else {
        res
    }
}

/// Montgomery 乗算を行う。
///
/// # Args
/// - `a`: 乗数 (Montgomery 表現)
/// - `b`: 被乗数 (Montgomery 表現)
///
/// # Returns
/// `u32`: `a * b * R^{-1} mod MOD` (Montgomery 表現での積) を返す。
///
/// # Constraints
/// - `a`、`b` は `MOD` 未満である必要がある。
///
/// # Panics
/// パニックしない。
///
/// # Complexity
/// - Time complexity: O(1)。
/// - Space complexity: O(1)。
///
/// # Examples
/// ```rust
/// use anmitsu::modulo998244353::convolution_mont;
///
/// let a_mont = convolution_mont::standard_to_mont_scalar(2);
/// let b_mont = convolution_mont::standard_to_mont_scalar(3);
/// let c_mont = convolution_mont::mul_mont(a_mont, b_mont);
/// assert_eq!(6, convolution_mont::mont_to_standard_scalar(c_mont));
/// ```
#[inline(always)]
pub fn mul_mont(a: u32, b: u32) -> u32 {
    // 通常の積 `a * b` に対して reduction を掛けると、Montgomery 表現の定義
    // `x_mont = x * R mod MOD` に従って `a_mont * b_mont * R^{-1} = (a*b)_mont`
    // が得られる。
    reduce_mont(a as u64 * b as u64)
}

/// スカラー値を Montgomery 表現へ変換する。
///
/// # Args
/// - `x`: 変換対象の値
///
/// # Returns
/// `u32`: `x * R mod MOD` (Montgomery 表現) を返す。
///
/// # Constraints
/// - `x` は `MOD` 未満である必要がある。
///
/// # Panics
/// パニックしない。
///
/// # Complexity
/// - Time complexity: O(1)。
/// - Space complexity: O(1)。
///
/// # Examples
/// ```rust
/// use anmitsu::modulo998244353::convolution_mont;
///
/// let x_mont = convolution_mont::standard_to_mont_scalar(5);
/// assert_eq!(5, convolution_mont::mont_to_standard_scalar(x_mont));
/// ```
#[inline(always)]
pub fn standard_to_mont_scalar(x: u32) -> u32 {
    // `x * R^2 * R^{-1} = x * R (mod MOD)` が Montgomery 表現の定義そのもの
    // なので、事前計算した `R2` との Montgomery 乗算 1 回だけで変換できる。
    mul_mont(x, R2)
}

/// Montgomery 表現をスカラー値へ変換する。
///
/// AVX2 が利用可能であっても、長さが小さい場合は SIMD 化のオーバーヘッドが
/// 支配的になり得るため、スカラーでの変換関数も用意する。
///
/// # Args
/// - `x_mont`: 変換対象の値 (Montgomery 表現)
///
/// # Returns
/// `u32`: `x_mont * R^{-1} mod MOD` (通常表現) を返す。
///
/// # Constraints
/// - `x_mont` は `MOD` 未満である必要がある。
///
/// # Panics
/// パニックしない。
///
/// # Complexity
/// - Time complexity: O(1)。
/// - Space complexity: O(1)。
///
/// # Examples
/// ```rust
/// use anmitsu::modulo998244353::convolution_mont;
///
/// let x_mont = convolution_mont::standard_to_mont_scalar(5);
/// assert_eq!(5, convolution_mont::mont_to_standard_scalar(x_mont));
/// ```
#[inline(always)]
pub fn mont_to_standard_scalar(x_mont: u32) -> u32 {
    // reduction 1 回がそのまま `R^{-1}` を掛けることに相当するため、通常表現へ戻せる。
    reduce_mont(x_mont as u64)
}

/// NTT 長が `2^lg` のときの逆元 (Montgomery 表現) を返す。
///
/// # Args
/// - `lg`: NTT 長 `2^lg` の指数
///
/// # Returns
/// `u32`: `(2^lg)^{-1} mod MOD` (Montgomery 表現) を返す。
///
/// # Constraints
/// - `lg` は `INVS_MONT` の添字範囲 (0..=22) 内である必要がある。
///
/// # Panics
/// - `lg` が範囲外の場合にパニックする。
///
/// # Complexity
/// - Time complexity: O(1)。
/// - Space complexity: O(1)。
///
/// # Examples
/// ```rust
/// use anmitsu::modulo998244353::convolution_mont;
///
/// let inv_8_mont = convolution_mont::inv_len_mont(3);
/// assert_eq!(1, convolution_mont::mont_to_standard_scalar(inv_8_mont) as u64 * 8 % 998244353);
/// ```
#[inline(always)]
pub fn inv_len_mont(lg: usize) -> u32 {
    INVS_MONT[lg]
}

/// NTT doubling で用いる `ζ` の最大 `lg` である。
///
/// `n = 2^lg` に対して `2n <= MAX_NTT_LEN` を満たす必要があるため、最大 `lg` は
/// `log2(MAX_NTT_LEN) - 1` となる。
const NTT_DOUBLING_LG_MAX: usize = convolution::MAX_NTT_LEN.trailing_zeros() as usize - 1;

/// NTT doubling で用いる `ζ` のテーブル (通常表現) である。
///
/// `NTT_DOUBLING_ZETA[lg]` は `n = 2^lg` に対する `2n` の原始根 `ζ` を表す。
const NTT_DOUBLING_ZETA: [u32; NTT_DOUBLING_LG_MAX + 1] = build_ntt_doubling_zeta();

/// NTT doubling で用いる `ζ` テーブルを構築する。
///
/// # Args
/// - `()`: 引数はない。
///
/// # Returns
/// `[u32; NTT_DOUBLING_LG_MAX + 1]`: `ζ` テーブル。
///
/// # Constraints
/// - `convolution::MAX_NTT_LEN` は 2 の冪である。
///
/// # Panics
/// - この関数はパニックしない。
///
/// # Complexity
/// - Time complexity: O(log MAX_NTT_LEN)。
/// - Space complexity: O(1)。
///
/// # Examples
/// ```rust,ignore
/// // 内部定数のため、直接の使用例は省略する。
/// ```
const fn build_ntt_doubling_zeta() -> [u32; NTT_DOUBLING_LG_MAX + 1] {
    let mut table = [0_u32; NTT_DOUBLING_LG_MAX + 1];
    let mut lg = 0usize;
    // `n = 2^lg` ごとに、位数 `MOD - 1` を `2n` で割った指数で原始根を累乗し、
    // 1 の原始 `2n` 乗根 `ζ` を求める。
    while lg <= NTT_DOUBLING_LG_MAX {
        let n = 1usize << lg;
        let exp = (modulo::M as usize - 1) / (2 * n);
        table[lg] = modulo::pow(convolution::PRIMITIVE_ROOT, exp);
        lg += 1;
    }
    table
}

/// NTT doubling で用いる回転因子列 (Montgomery 表現) をキャッシュする。
struct NttDoublingPowersMont {
    /// `zeta_mont_by_lg[lg]` は `n = 2^lg` に対する `2n` の原始根 `ζ` (Montgomery 表現) である。
    zeta_mont_by_lg: [u32; NTT_DOUBLING_LG_MAX + 1],
    /// `pows_mont_by_lg[lg]` は `ζ^i` (i = 0..n-1、n = 2^lg) の列を遅延初期化で保持する。
    pows_mont_by_lg: [sync::OnceLock<Box<[u32]>>; NTT_DOUBLING_LG_MAX + 1],
}

impl NttDoublingPowersMont {
    /// `NttDoublingPowersMont` を初期化する。
    ///
    /// # Args
    /// 引数はない。
    ///
    /// # Returns
    /// `Self`: `ζ` テーブルのみを Montgomery 表現へ変換済みで、冪乗列は
    /// 未計算 (遅延初期化前) の状態の `Self` を返す。
    ///
    /// # Constraints
    /// 制約はない。
    ///
    /// # Panics
    /// パニックしない。
    ///
    /// # Complexity
    /// - Time complexity: O(NTT_DOUBLING_LG_MAX)。
    /// - Space complexity: O(NTT_DOUBLING_LG_MAX)。
    ///
    /// # Examples
    /// ```rust,ignore
    /// // 内部専用の構築関数のため、直接の使用例は省略する。
    /// ```
    fn new() -> Self {
        let zeta_mont_by_lg =
            std::array::from_fn(|lg| standard_to_mont_scalar(NTT_DOUBLING_ZETA[lg]));
        let pows_mont_by_lg = std::array::from_fn(|_| sync::OnceLock::new());
        Self {
            zeta_mont_by_lg,
            pows_mont_by_lg,
        }
    }

    /// `n = 2^lg` に対して `ζ^i` (i = 0..n-1) の列を返す。
    ///
    /// # Args
    /// - `n`: `2^lg` となる NTT 長
    ///
    /// # Returns
    /// `&[u32]`: `ζ^i` (Montgomery 表現) の列を返す。
    ///
    /// # Constraints
    /// - `n` は 0 ではない 2 の冪である。
    /// - `2 * n <= convolution::MAX_NTT_LEN` を満たす。
    ///
    /// # Panics
    /// - この関数はパニックし得る (デバッグアサート)。
    ///
    /// # Complexity
    /// - Time complexity: 初回のみ O(n)、2 回目以降は O(1)。
    /// - Space complexity: O(n)。
    ///
    /// # Examples
    /// ```rust,ignore
    /// // 内部専用のアクセサ関数のため、直接の使用例は省略する。
    /// ```
    fn powers(&self, n: usize) -> &[u32] {
        debug_assert!(n.is_power_of_two());
        debug_assert!(n > 0);
        debug_assert!(2 * n <= convolution::MAX_NTT_LEN);

        let lg = n.trailing_zeros() as usize;
        debug_assert!(lg <= NTT_DOUBLING_LG_MAX);

        // `lg` ごとに一度だけ計算し、以後は使い回す。
        self.pows_mont_by_lg[lg]
            .get_or_init(|| {
                let zeta_mont = self.zeta_mont_by_lg[lg];
                // `ζ^0, ζ^1, ..., ζ^{n-1}` を、直前の値に `ζ` を掛けながら順に埋める。
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

/// NTT doubling で用いる回転因子列のキャッシュである。
static NTT_DOUBLING_POWERS_MONT: sync::OnceLock<NttDoublingPowersMont> = sync::OnceLock::new();

/// 初期化済みの NTT 前計算を保持する。
static NTT: sync::OnceLock<Ntt> = sync::OnceLock::new();

/// NTT doubling で用いる `ζ^i` (i = 0..n-1) の列 (Montgomery 表現) を返す。
///
/// # Args
/// - `n`: `2^lg` となる NTT 長
///
/// # Returns
/// `&'static [u32]`: `ζ^i` の列 (Montgomery 表現) を返す。
///
/// # Constraints
/// - `n` は 0 ではない 2 の冪である。
/// - `2 * n <= convolution::MAX_NTT_LEN` を満たす。
///
/// # Panics
/// - この関数はパニックし得る (デバッグアサート)。
///
/// # Complexity
/// - Time complexity: 初回のみ O(n)、2 回目以降は O(1)。
/// - Space complexity: O(n)。
///
/// # Examples
/// ```rust,ignore
/// // AVX2 依存の FPS 実装の内部で利用する。
/// ```
pub fn ntt_doubling_powers_mont(n: usize) -> &'static [u32] {
    debug_assert!(n.is_power_of_two());
    debug_assert!(n > 0);
    debug_assert!(2 * n <= convolution::MAX_NTT_LEN);

    NTT_DOUBLING_POWERS_MONT
        .get_or_init(|| NttDoublingPowersMont::new())
        .powers(n)
}

/// AVX2 を用いた Montgomery 演算の補助テーブル。
///
/// `_mm256_mul_epu32` は 32-bit lane のうち偶数番目 (0、2、4、6) のみを
/// 32×32→64-bit 乗算するため、各定数を 8 lane 全てへブロードキャストした
/// ベクターとして保持しておく。
#[derive(Clone, Copy)]
struct MontgomerySimd {
    /// `convolution::MOD` を全 lane にブロードキャストしたベクターである。
    mod_v: x86_64::__m256i,
    /// `N_INV` (`-MOD^{-1} mod 2^32`) を全 lane にブロードキャストしたベクターである。
    n_inv_v: x86_64::__m256i,
    /// `R2` (`R^2 mod MOD`) を全 lane にブロードキャストしたベクターである。
    r2_v: x86_64::__m256i,
}

impl MontgomerySimd {
    /// `MontgomerySimd` を初期化する。
    ///
    /// # Args
    /// 引数はない。
    ///
    /// # Returns
    /// `Self`: `MOD`、`N_INV`、`R2` をそれぞれ全 lane へブロードキャストした
    /// 状態の `Self` を返す。
    ///
    /// # Constraints
    /// - 呼び出し側は AVX2 が利用可能であることを保証する。
    ///
    /// # Panics
    /// パニックしない。
    ///
    /// # Complexity
    /// - Time complexity: O(1)。
    /// - Space complexity: O(1)。
    ///
    /// # Examples
    /// ```rust,ignore
    /// // 内部専用の構築関数のため、直接の使用例は省略する。
    /// ```
    #[target_feature(enable = "avx2")]
    unsafe fn new() -> Self {
        Self {
            mod_v: x86_64::_mm256_set1_epi32(convolution::MOD as i32),
            n_inv_v: x86_64::_mm256_set1_epi32(N_INV as i32),
            r2_v: x86_64::_mm256_set1_epi32(R2 as i32),
        }
    }

    /// 値を `0..MOD` に正規化する。
    ///
    /// # Args
    /// - `vec`: `0..2*MOD` の範囲に収まる 8 個の `u32` 値
    ///
    /// # Returns
    /// `__m256i`: 各 lane を `0..MOD` に正規化した値を返す。
    ///
    /// # Constraints
    /// - `vec` の各 lane は `0..2*MOD` の範囲に収まる必要がある。
    ///
    /// # Panics
    /// パニックしない。
    ///
    /// # Complexity
    /// - Time complexity: O(1)。
    /// - Space complexity: O(1)。
    ///
    /// # Examples
    /// ```rust,ignore
    /// // 内部専用の補助関数のため、直接の使用例は省略する。
    /// ```
    #[inline(always)]
    fn shrink(self, vec: x86_64::__m256i) -> x86_64::__m256i {
        // `vec - MOD` を符号なし比較で `vec` と比較し、小さい方 (= 減算後に
        // 負にラップアラウンドしていなければ減算後の値、そうでなければ元の値)
        // を採用することで、分岐なしに `0..MOD` へ畳み込む。
        unsafe { x86_64::_mm256_min_epu32(vec, x86_64::_mm256_sub_epi32(vec, self.mod_v)) }
    }

    /// `__m256i` 上で Montgomery reduction を行う。
    ///
    /// # Args
    /// - `x0246`: 偶数番目の lane (0、2、4、6) に 64-bit 積を保持するベクター
    /// - `x1357`: 奇数番目の lane (1、3、5、7) に 64-bit 積を保持するベクター
    ///
    /// # Returns
    /// `__m256i`: 各 lane に `reduce_mont` 相当の結果を格納したベクターを返す。
    ///
    /// # Constraints
    /// - `x0246`、`x1357` は `_mm256_mul_epu32` の出力形式 (64-bit 積を
    ///   32-bit 境界に配置した形式) に従う必要がある。
    ///
    /// # Panics
    /// パニックしない。
    ///
    /// # Complexity
    /// - Time complexity: O(1)。
    /// - Space complexity: O(1)。
    ///
    /// # Examples
    /// ```rust,ignore
    /// // 内部専用の補助関数のため、直接の使用例は省略する。
    /// ```
    #[inline(always)]
    unsafe fn reduce(self, x0246: x86_64::__m256i, x1357: x86_64::__m256i) -> x86_64::__m256i {
        unsafe {
            // スカラー版 `reduce_mont` の `t = (val as u32).wrapping_mul(N_INV)` に
            // 相当する部分を、偶数 lane と奇数 lane それぞれについて計算する。
            let x0246_ninv = x86_64::_mm256_mul_epu32(x0246, self.n_inv_v);
            let x1357_ninv = x86_64::_mm256_mul_epu32(x1357, self.n_inv_v);

            // `(val + t * MOD) >> 32` に相当する加算を行う。この時点で各 64-bit
            // 結果の上位 32 bit に reduction 後の値が現れる。
            let x0246_res =
                x86_64::_mm256_add_epi64(x0246, x86_64::_mm256_mul_epu32(x0246_ninv, self.mod_v));
            let x1357_res =
                x86_64::_mm256_add_epi64(x1357, x86_64::_mm256_mul_epu32(x1357_ninv, self.mod_v));

            // 各 64-bit レーンの上位 32 bit (reduction 結果) だけを 4 byte
            // シフトで取り出し、偶数・奇数 lane の結果を 1 本の u32x8 へまとめる。
            let mut res =
                x86_64::_mm256_or_si256(x86_64::_mm256_bsrli_epi128(x0246_res, 4), x1357_res);
            // reduction 結果は高々 `2*MOD` 未満なので、`0..MOD` へ畳み込む。
            res = self.shrink(res);
            res
        }
    }

    /// `u32x8` に対する Montgomery 乗算を行う。
    ///
    /// # Args
    /// - `a`: 乗数となる 8 個の `u32` 値 (Montgomery 表現)
    /// - `b`: 被乗数となる 8 個の `u32` 値 (Montgomery 表現)
    /// - `B_USE_ONLY_EVEN`: `b` が偶数 lane のみに有効な値を持ち、
    ///   奇数 lane も同じ値を複製済みであることを示すフラグである。
    ///   `true` の場合、`b` の 4 byte シフトを省略して命令数を削減する。
    ///
    /// # Returns
    /// `__m256i`: 各 lane で `a[i] * b[i] * R^{-1} mod MOD` を計算した結果を返す。
    ///
    /// # Constraints
    /// - `a`、`b` の各 lane は `MOD` 未満である必要がある。
    /// - `B_USE_ONLY_EVEN = true` の場合、`b` の奇数 lane はあらかじめ
    ///   偶数 lane と同じ値に複製されている必要がある。
    ///
    /// # Panics
    /// パニックしない。
    ///
    /// # Complexity
    /// - Time complexity: O(1)。
    /// - Space complexity: O(1)。
    ///
    /// # Examples
    /// ```rust,ignore
    /// // 内部専用の補助関数のため、直接の使用例は省略する。
    /// ```
    #[inline(always)]
    unsafe fn mul_u32x8<const B_USE_ONLY_EVEN: bool>(
        self,
        a: x86_64::__m256i,
        b: x86_64::__m256i,
    ) -> x86_64::__m256i {
        unsafe {
            // `_mm256_mul_epu32` は偶数 lane しか乗算できないため、奇数 lane の
            // 値を偶数位置へ寄せたベクターを別途用意する。
            let a_sh = x86_64::_mm256_bsrli_epi128(a, 4);
            let b_sh = if B_USE_ONLY_EVEN {
                b
            } else {
                x86_64::_mm256_bsrli_epi128(b, 4)
            };
            // 偶数 lane 同士、奇数 lane 同士でそれぞれ 32×32→64-bit 乗算する。
            let x0246 = x86_64::_mm256_mul_epu32(a, b);
            let x1357 = x86_64::_mm256_mul_epu32(a_sh, b_sh);
            self.reduce(x0246, x1357)
        }
    }
}

/// AVX2 用 NTT の前計算テーブル。
struct Ntt {
    /// AVX2 + Montgomery 演算の補助テーブルである。
    mts: MontgomerySimd,
    /// NTT の各段・各チャンクで用いる回転因子列 (Montgomery 表現) である。
    ntt_rots_by_chunks_lg_max: Box<[u32]>,
    /// 逆 NTT の各段・各チャンクで用いる回転因子列 (Montgomery 表現) である。
    intt_rots_by_chunks_lg_max: Box<[u32]>,
}

impl Ntt {
    /// NTT/INTT の回転因子列を構築する。
    ///
    /// # Args
    /// - `rate`: `trailing_ones` に応じて回転因子を更新するためのレート表
    ///   (Montgomery 表現) である。
    /// - `chunks_lg`: 生成する回転因子列の長さ `chunks = 2^chunks_lg` を表す。
    ///
    /// # Returns
    /// `Box<[u32]>`: 各 `s` に対する回転因子 `rot[s]` の列を返す。
    ///
    /// # Constraints
    /// - `chunks_lg < 22` を満たす必要がある。
    ///
    /// # Panics
    /// - `chunks_lg` が大きすぎて `1usize << chunks_lg` がオーバーフローする場合、
    ///   パニックし得る。
    ///
    /// # Complexity
    /// - Time complexity: O(2^chunks_lg)、ここで `chunks_lg` は引数である。
    /// - Space complexity: O(2^chunks_lg)、ここで `chunks_lg` は引数である。
    ///
    /// # Examples
    /// ```rust,ignore
    /// // 内部前計算用関数のため、直接の使用例は省略する。
    /// ```
    fn build_rot_pows(rate: &[u32; 22], chunks_lg: usize) -> Box<[u32]> {
        // NTT/INTT の呼び出しごとの mul_mont を削減するため、回転因子列を事前計算する。
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

    /// NTT の前計算テーブルを構築する。
    ///
    /// # Args
    /// 引数はない。
    ///
    /// # Returns
    /// `Self`: `MontgomerySimd` および、サポートする最大長に対応する
    /// NTT/INTT の回転因子列を保持した `Self` を返す。
    ///
    /// # Constraints
    /// - 呼び出し側は AVX2 が利用可能であることを保証する。
    ///
    /// # Panics
    /// パニックしない。
    ///
    /// # Complexity
    /// - Time complexity: O(MAX_NTT_LEN)。
    /// - Space complexity: O(MAX_NTT_LEN)。
    ///
    /// # Examples
    /// ```rust,ignore
    /// // 内部専用の構築関数のため、直接の使用例は省略する。
    /// ```
    #[target_feature(enable = "avx2")]
    unsafe fn new() -> Self {
        unsafe {
            let mts = MontgomerySimd::new();
            // 最大長に対応する回転因子列さえ持てば、それより短い NTT でも
            // 先頭部分を再利用できるため、チャンク数の上限は 1 度だけ計算する。
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

    /// 通常表現を Montgomery 表現へ変換する。
    ///
    /// # Args
    /// - `data`: 変換対象の先頭ポインタ
    /// - `len`: 変換する要素数
    ///
    /// # Returns
    /// `()`: `data[0..len]` を in-place で更新する。
    ///
    /// # Constraints
    /// - `len` は `AVX2_U32_LANES` 以上の 2 の冪である。
    /// - `data` は `len` 要素分の書き込み可能な領域を指す。
    ///
    /// # Panics
    /// - この関数はパニックし得る (デバッグアサート)。
    ///
    /// # Complexity
    /// - Time complexity: O(len)。
    /// - Space complexity: O(1)。
    ///
    /// # Examples
    /// ```rust,ignore
    /// // 内部専用の補助関数のため、直接の使用例は省略する。
    /// ```
    #[target_feature(enable = "avx2")]
    unsafe fn to_mont(&self, data: *mut u32, len: usize) {
        unsafe {
            debug_assert!(len.is_power_of_two());
            debug_assert!(len >= AVX2_U32_LANES);
            debug_assert_eq!(0, len % AVX2_U32_LANES);

            // `x * R^2 * R^{-1} = x * R (mod MOD)` が Montgomery 表現の定義その
            // ものなので、`R2` との Montgomery 乗算だけで変換できる。
            let mts = self.mts;
            for i in (0..len).step_by(AVX2_U32_LANES) {
                let v = x86_64::_mm256_loadu_si256(data.add(i).cast());
                let v = mts.mul_u32x8::<true>(v, mts.r2_v);
                x86_64::_mm256_storeu_si256(data.add(i).cast(), v);
            }
        }
    }

    /// Montgomery 表現を通常表現へ変換する。
    ///
    /// # Args
    /// - `data`: 変換対象の先頭ポインタ
    /// - `len`: 変換する要素数
    ///
    /// # Returns
    /// `()`: `data[0..len]` を in-place で更新する。
    ///
    /// # Constraints
    /// - `len` は `AVX2_U32_LANES` 以上の 2 の冪である。
    /// - `data` は `len` 要素分の書き込み可能な領域を指す。
    ///
    /// # Panics
    /// - この関数はパニックし得る (デバッグアサート)。
    ///
    /// # Complexity
    /// - Time complexity: O(len)。
    /// - Space complexity: O(1)。
    ///
    /// # Examples
    /// ```rust,ignore
    /// // 内部専用の補助関数のため、直接の使用例は省略する。
    /// ```
    #[target_feature(enable = "avx2")]
    unsafe fn to_standard(&self, data: *mut u32, len: usize) {
        unsafe {
            debug_assert!(len.is_power_of_two());
            debug_assert!(len >= AVX2_U32_LANES);
            debug_assert_eq!(0, len % AVX2_U32_LANES);

            // `x_mont * 1 * R^{-1} = x_mont * R^{-1} (mod MOD)` が Montgomery
            // reduction そのものなので、`1` との Montgomery 乗算で変換できる。
            let mts = self.mts;
            let one = x86_64::_mm256_set1_epi32(1);
            for i in (0..len).step_by(AVX2_U32_LANES) {
                let v = x86_64::_mm256_loadu_si256(data.add(i).cast());
                let v = mts.mul_u32x8::<true>(v, one);
                x86_64::_mm256_storeu_si256(data.add(i).cast(), v);
            }
        }
    }

    /// Montgomery 表現上で点ごとの積を計算する。
    ///
    /// # Args
    /// - `a`: 結果を格納する配列の先頭ポインタ
    /// - `b`: 乗じる配列の先頭ポインタ
    /// - `len`: 要素数
    ///
    /// # Returns
    /// `()`: `a[i] *= b[i]` (Montgomery 乗算) を in-place で計算する。
    ///
    /// # Constraints
    /// - `len` は `AVX2_U32_LANES` 以上の 2 の冪である。
    /// - `a`、`b` はそれぞれ `len` 要素分の有効な領域を指す。
    ///
    /// # Panics
    /// - この関数はパニックし得る (デバッグアサート)。
    ///
    /// # Complexity
    /// - Time complexity: O(len)。
    /// - Space complexity: O(1)。
    ///
    /// # Examples
    /// ```rust,ignore
    /// // 内部専用の補助関数のため、直接の使用例は省略する。
    /// ```
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

    /// Montgomery 表現上でスカラー倍を行う。
    ///
    /// # Args
    /// - `data`: 対象配列の先頭ポインタ
    /// - `len`: 要素数
    /// - `sc_mont`: 乗じるスカラー値 (Montgomery 表現)
    ///
    /// # Returns
    /// `()`: `data[i] *= sc_mont` (Montgomery 乗算) を in-place で計算する。
    ///
    /// # Constraints
    /// - `len` は `AVX2_U32_LANES` 以上の 2 の冪である。
    /// - `data` は `len` 要素分の有効な領域を指す。
    ///
    /// # Panics
    /// - この関数はパニックし得る (デバッグアサート)。
    ///
    /// # Complexity
    /// - Time complexity: O(len)。
    /// - Space complexity: O(1)。
    ///
    /// # Examples
    /// ```rust,ignore
    /// // 内部専用の補助関数のため、直接の使用例は省略する。
    /// ```
    #[target_feature(enable = "avx2")]
    unsafe fn mul_scalar_mont(&self, data: *mut u32, len: usize, sc_mont: u32) {
        unsafe {
            debug_assert!(len.is_power_of_two());
            debug_assert!(len >= AVX2_U32_LANES);
            debug_assert_eq!(0, len % AVX2_U32_LANES);

            // スカラー値を全 lane にブロードキャストしてから、一括で
            // Montgomery 乗算する。
            let mts = self.mts;
            let sc_v = x86_64::_mm256_set1_epi32(sc_mont as i32);
            for i in (0..len).step_by(AVX2_U32_LANES) {
                let v = x86_64::_mm256_loadu_si256(data.add(i).cast());
                let v = mts.mul_u32x8::<true>(v, sc_v);
                x86_64::_mm256_storeu_si256(data.add(i).cast(), v);
            }
        }
    }

    /// Montgomery 表現上で、数論変換を実行する。
    ///
    /// このメソッドは `convolution::ntt_butterfly` と同じ decimation-in-frequency
    /// バタフライを、AVX2 の `u32x8` レーンを使って一括処理する。半幅 `p` が
    /// 8 以上の間は 1 チャンクを複数の SIMD レーンで処理できるが、`p` が 8 未満に
    /// なると 1 回のロードに複数チャンク分のデータが混在するため、`p = 4, 2, 1`
    /// それぞれに専用のシャッフル・ブレンド命令列を用意している。
    ///
    /// # Args
    /// - `data`: 係数列 (Montgomery 表現) の先頭ポインタ
    /// - `len`: 係数列の長さ
    ///
    /// # Returns
    /// `()`: `data[0..len]` を in-place で更新する。
    ///
    /// # Constraints
    /// - `len` は `AVX2_U32_LANES` 以上の 2 の冪である。
    /// - `data` は `len` 要素分の書き込み可能な領域を指す。
    ///
    /// # Panics
    /// - この関数はパニックし得る (デバッグアサート)。
    ///
    /// # Complexity
    /// - Time complexity: O(len log len)。
    /// - Space complexity: O(1)。
    ///
    /// # Examples
    /// ```rust,ignore
    /// // `pub fn ntt_mont` から呼び出される。
    /// ```
    #[target_feature(enable = "avx2")]
    unsafe fn ntt_mont(&self, data: *mut u32, len: usize) {
        unsafe {
            debug_assert!(len.is_power_of_two());
            debug_assert!(len >= AVX2_U32_LANES);
            let lg = len.trailing_zeros() as usize;
            let n = len;
            let mts = self.mts;

            // ブロック幅を `len` から 1 まで半分ずつ縮めながら、段階的に
            // バタフライ演算を適用する (decimation-in-frequency)。
            for stage in 0..lg {
                // この段で扱うブロックの半幅 `p` と全幅 `step = 2p` である。
                let p = 1usize << (lg - stage - 1);
                let step = 1usize << (lg - stage);

                let chunks = n / step;
                let rots = self.ntt_rots_by_chunks_lg_max.as_ref();
                debug_assert!(rots.len() >= chunks);
                if p >= 8 {
                    // 半幅が 1 SIMD レーン (8 要素) 以上あるので、1 チャンクを
                    // 複数回の u32x8 ロードで素直に処理できる。
                    for s in 0..chunks {
                        let ptr = data.add(s * step);

                        // このチャンクの回転因子を全 lane にブロードキャストする。
                        let rot = rots[s];
                        let rot_v = x86_64::_mm256_set1_epi32(rot as i32);
                        for i in (0..p).step_by(8) {
                            let l = x86_64::_mm256_loadu_si256(ptr.add(i).cast());
                            let r = x86_64::_mm256_loadu_si256(ptr.add(i + p).cast());
                            // 後半 `r` に回転因子を掛けてから、`(l+r, l-r)` の
                            // バタフライを計算する。
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
                        // 半幅が 4 なので、前半 4 要素・後半 4 要素で合わせて
                        // ちょうど 1 レーン (8 要素) に収まる。SIMD 化のオーバー
                        // ヘッドの方が大きいため、ここではスカラー演算で処理する。
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
                        // 半幅が 2 なので、1 レーンに 2 チャンク分 (前半 2 +
                        // 後半 2 を 2 組) が同時に載る。2 チャンクをまとめて
                        // 1 回の Montgomery 乗算で処理するため、前半には `R`
                        // (乗算しても値が変わらない単位元) を、後半には各チャンク
                        // の回転因子を割り当てた乗数ベクターを組み立てる。
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

                            // `permutevar8x32` で前半 `l` と後半 `r` をそれぞれ
                            // 各チャンク内で複製し、バタフライの和・差を計算する。
                            let l = x86_64::_mm256_permutevar8x32_epi32(v, idx_l);
                            let r = x86_64::_mm256_permutevar8x32_epi32(v, idx_r);

                            let sum = mts.shrink(x86_64::_mm256_add_epi32(l, r));
                            let diff = mts.shrink(x86_64::_mm256_sub_epi32(
                                x86_64::_mm256_add_epi32(l, mts.mod_v),
                                r,
                            ));
                            // `sum`、`diff` を交互に元の位置へ書き戻すため、
                            // ブレンドマスク `0xCC` (= 0b11001100) で選択する。
                            let out = x86_64::_mm256_blend_epi32(sum, diff, 0xCC);
                            x86_64::_mm256_storeu_si256(ptr.cast(), out);
                        }
                    } else if p == 1 {
                        // 半幅が 1 なので、1 レーンに 4 チャンク分 (前後各 1 要素
                        // を 4 組) が同時に載る。p == 2 の場合と同様に、1 回の
                        // Montgomery 乗算で 4 チャンクをまとめて処理する。
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
                            // 前後の要素が 1 個ずつ交互に並ぶため、ブレンド
                            // マスクは `0xAA` (= 0b10101010) になる。
                            let out = x86_64::_mm256_blend_epi32(sum, diff, 0xAA);
                            x86_64::_mm256_storeu_si256(ptr.cast(), out);
                        }
                    } else {
                        // `p` は 2 の冪であり、`p >= 8` は上の分岐で処理済みなので、
                        // ここに到達するのは 4、2、1 のいずれかのみである。
                        unreachable!();
                    }
                }
            }
        }
    }

    /// Montgomery 表現上で、逆数論変換を実行する。
    ///
    /// このメソッドは `convolution::intt_butterfly` と同じ decimation-in-time
    /// バタフライを、AVX2 の `u32x8` レーンを使って一括処理する。段の進み方や
    /// 半幅 `p` に応じた分岐の考え方は `ntt_mont` と対称であり、相違点は
    /// 「先に和・差を取ってから、差の側にだけ回転因子を掛ける」順序のみである。
    ///
    /// # Args
    /// - `data`: NTT 値 (Montgomery 表現) の先頭ポインタ
    /// - `len`: 係数列の長さ
    ///
    /// # Returns
    /// `()`: `data[0..len]` を in-place で更新する (正規化なし)。
    ///
    /// # Constraints
    /// - `len` は `AVX2_U32_LANES` 以上の 2 の冪である。
    /// - `data` は `len` 要素分の書き込み可能な領域を指す。
    ///
    /// # Panics
    /// - この関数はパニックし得る (デバッグアサート)。
    ///
    /// # Complexity
    /// - Time complexity: O(len log len)。
    /// - Space complexity: O(1)。
    ///
    /// # Examples
    /// ```rust,ignore
    /// // `pub fn intt_mont` から呼び出される。
    /// ```
    #[target_feature(enable = "avx2")]
    unsafe fn intt_mont(&self, data: *mut u32, len: usize) {
        unsafe {
            debug_assert!(len.is_power_of_two());
            debug_assert!(len >= AVX2_U32_LANES);
            let lg = len.trailing_zeros() as usize;
            let n = len;
            let mts = self.mts;

            // ブロック幅を 1 段目 (幅 2) から `len` まで倍々に広げながら、
            // `ntt_mont` の各段を逆順にたどる (decimation-in-time)。
            for stage in (1..=lg).rev() {
                let p = 1usize << (lg - stage);
                let step = 1usize << (lg - stage + 1);

                let chunks = n / step;
                let irots = self.intt_rots_by_chunks_lg_max.as_ref();
                debug_assert!(irots.len() >= chunks);
                if p >= 8 {
                    // 半幅が 1 SIMD レーン以上あるので、素直に u32x8 単位で処理する。
                    for s in 0..chunks {
                        let ptr = data.add(s * step);

                        let irot = irots[s];
                        let irot_v = x86_64::_mm256_set1_epi32(irot as i32);
                        for i in (0..p).step_by(8) {
                            let l = x86_64::_mm256_loadu_si256(ptr.add(i).cast());
                            let r = x86_64::_mm256_loadu_si256(ptr.add(i + p).cast());
                            // 先に `(l+r, l-r)` を計算し、逆回転因子は差の側にのみ掛ける。
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
                        // p == 4 は前半 4 要素・後半 4 要素で 1 レーンに収まるため、
                        // SIMD 化のオーバーヘッドを避けてスカラー演算で処理する。
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
                        // p == 2 では 1 レーンに 2 チャンク分が同時に載るため、
                        // `permutevar8x32` で前半・後半をチャンクごとに複製してから
                        // バタフライし、最後に逆回転因子をまとめて掛ける。
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
                            // 前半 (`sum`) は単位元 `R` を、後半 (`diff`) は
                            // 逆回転因子を掛けるベクターを、ブレンドで組み立ててから
                            // 1 回の Montgomery 乗算で処理する。
                            let out = x86_64::_mm256_blend_epi32(sum, diff, 0xCC);
                            let out = mts.mul_u32x8::<true>(out, mul_v);
                            x86_64::_mm256_storeu_si256(ptr.cast(), out);
                        }
                    } else if p == 1 {
                        // p == 1 では 1 レーンに 4 チャンク分が同時に載るため、
                        // p == 2 と同様の考え方で 4 チャンクを一括処理する。
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
                        // `p` は 2 の冪であり、`p >= 8` は上の分岐で処理済みなので、
                        // ここに到達するのは 4、2、1 のいずれかのみである。
                        unreachable!();
                    }
                }
            }
        }
    }
}

/// AVX2 を用いて、Montgomery 表現へ変換する。
///
/// # Args
/// - `a`: 変換対象の配列 (通常表現)。この関数は `a` を in-place で更新する。
///
/// # Returns
/// `()`: `a` を Montgomery 表現へ変換して in-place で更新する。
///
/// # Constraints
/// - `a` は空であってはならず、長さは 2 の冪である必要がある。
/// - `a` の各要素は `convolution::MOD` 未満である。
///
/// # Panics
/// - この関数はパニックし得る (デバッグアサート)。
///
/// # Complexity
/// - Time complexity: O(a.len())。
/// - Space complexity: O(1)。
///
/// # Examples
/// ```rust
/// use anmitsu::modulo998244353::convolution_mont;
///
/// let mut a = vec![1_u32, 2, 3, 4, 5, 6, 7, 8];
/// unsafe { convolution_mont::standard_to_mont(&mut a); }
/// assert_eq!(1, convolution_mont::mont_to_standard_scalar(a[0]));
/// ```
#[target_feature(enable = "avx2")]
pub unsafe fn standard_to_mont(a: &mut [u32]) {
    unsafe {
        debug_assert!(!a.is_empty());
        debug_assert!(a.len().is_power_of_two());

        // 長さが 1 SIMD レーン (8 要素) 未満の場合は、SIMD 化のオーバーヘッドを
        // 避けてスカラー演算で処理する。
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
        // 前計算テーブルはプロセス全体で使い回すため、初回のみ構築する。
        let ntt = NTT.get_or_init(|| Ntt::new());
        ntt.to_mont(a.as_mut_ptr(), a.len());
    }
}

/// AVX2 を用いて、Montgomery 表現から通常表現へ戻す。
///
/// # Args
/// - `a`: 変換対象の配列 (Montgomery 表現)。この関数は `a` を in-place で更新する。
///
/// # Returns
/// `()`: `a` を通常表現へ変換して in-place で更新する。
///
/// # Constraints
/// - `a` は空であってはならず、長さは 2 の冪である必要がある。
/// - `a` の各要素は `convolution::MOD` 未満である。
///
/// # Panics
/// - この関数はパニックし得る (デバッグアサート)。
///
/// # Complexity
/// - Time complexity: O(a.len())。
/// - Space complexity: O(1)。
///
/// # Examples
/// ```rust
/// use anmitsu::modulo998244353::convolution_mont;
///
/// let mut a = vec![convolution_mont::standard_to_mont_scalar(1); 8];
/// unsafe { convolution_mont::mont_to_standard(&mut a); }
/// assert_eq!(1, a[0]);
/// ```
#[target_feature(enable = "avx2")]
pub unsafe fn mont_to_standard(a: &mut [u32]) {
    unsafe {
        debug_assert!(!a.is_empty());
        debug_assert!(a.len().is_power_of_two());

        // standard_to_mont と同様、短い配列はスカラー演算で処理する。
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

/// AVX2 を用いて、Montgomery 表現上の点ごとの積を計算する。
///
/// # Args
/// - `a`: 結果を格納する配列 (Montgomery 表現)。この関数は `a` を in-place で更新する。
/// - `b`: 乗じる配列 (Montgomery 表現)。
///
/// # Returns
/// `()`: `a[i] *= b[i]` (Montgomery 乗算) を in-place で計算する。
///
/// # Constraints
/// - `a.len() == b.len()` を満たし、空であってはならない。
/// - `a.len()` は 2 の冪である必要がある。
/// - `a`、`b` の各要素は `convolution::MOD` 未満である。
///
/// # Panics
/// - この関数はパニックし得る (デバッグアサート)。
///
/// # Complexity
/// - Time complexity: O(a.len())。
/// - Space complexity: O(1)。
///
/// # Examples
/// ```rust
/// use anmitsu::modulo998244353::convolution_mont;
///
/// let mut a = vec![convolution_mont::standard_to_mont_scalar(2); 8];
/// let b = vec![convolution_mont::standard_to_mont_scalar(3); 8];
/// unsafe { convolution_mont::mul_pointwise_mont(&mut a, &b); }
/// assert_eq!(6, convolution_mont::mont_to_standard_scalar(a[0]));
/// ```
#[target_feature(enable = "avx2")]
pub unsafe fn mul_pointwise_mont(a: &mut [u32], b: &[u32]) {
    unsafe {
        debug_assert_eq!(a.len(), b.len());
        debug_assert!(!a.is_empty());
        debug_assert!(a.len().is_power_of_two());

        // 短い配列はスカラー演算で処理する。
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

/// AVX2 を用いて、Montgomery 表現上でスカラー倍を行う。
///
/// # Args
/// - `a`: 対象の配列 (Montgomery 表現)。この関数は `a` を in-place で更新する。
/// - `sc_mont`: 乗じるスカラー値 (Montgomery 表現)。
///
/// # Returns
/// `()`: `a[i] *= sc_mont` (Montgomery 乗算) を in-place で計算する。
///
/// # Constraints
/// - `a` は空であってはならず、長さは 2 の冪である必要がある。
/// - `a` の各要素は `convolution::MOD` 未満である。
///
/// # Panics
/// - この関数はパニックし得る (デバッグアサート)。
///
/// # Complexity
/// - Time complexity: O(a.len())。
/// - Space complexity: O(1)。
///
/// # Examples
/// ```rust
/// use anmitsu::modulo998244353::convolution_mont;
///
/// let mut a = vec![convolution_mont::standard_to_mont_scalar(2); 8];
/// let sc_mont = convolution_mont::standard_to_mont_scalar(3);
/// unsafe { convolution_mont::mul_scalar_mont(&mut a, sc_mont); }
/// assert_eq!(6, convolution_mont::mont_to_standard_scalar(a[0]));
/// ```
#[target_feature(enable = "avx2")]
pub unsafe fn mul_scalar_mont(a: &mut [u32], sc_mont: u32) {
    unsafe {
        let n = a.len();
        debug_assert!(n.is_power_of_two());
        debug_assert!(n > 0);

        // 短い配列はスカラー演算で処理する。
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

/// AVX2 を用いて、Montgomery 表現上のベクター同士の加算を行う (`a += b`)。
///
/// # Args
/// - `a`: 加算対象の配列 (Montgomery 表現)。`a[i]` は `MOD` 未満である必要がある。
/// - `b`: 加算する配列 (Montgomery 表現)。`b[i]` は `MOD` 未満である必要がある。
///
/// # Returns
/// `()`: `a` を in-place に更新する。
///
/// # Constraints
/// - `a.len() == b.len()` を満たす。
/// - `a` と `b` の各要素は `convolution::MOD` 未満である。
///
/// # Panics
/// - この関数はパニックし得る (デバッグアサート)。
///
/// # Complexity
/// - Time complexity: O(n)、ここで n は配列長である。
/// - Space complexity: O(1)。
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

        // 長さが 8 の倍数とは限らないため、8 要素単位で処理できる先頭部分
        // `[0, end)` と、端数として残る `[end, n)` とに分けて処理する。
        let end = n / AVX2_U32_LANES * AVX2_U32_LANES;
        for i in (0..end).step_by(AVX2_U32_LANES) {
            let va = x86_64::_mm256_loadu_si256(a.as_ptr().add(i).cast());
            let vb = x86_64::_mm256_loadu_si256(b.as_ptr().add(i).cast());
            let sum = mts.shrink(x86_64::_mm256_add_epi32(va, vb));
            x86_64::_mm256_storeu_si256(a.as_mut_ptr().add(i).cast(), sum);
        }

        // 端数はスカラー演算で処理する。
        for i in end..n {
            a[i] = modulo::add(a[i], b[i]);
        }
    }
}

/// AVX2 を用いて、Montgomery 表現上のベクター同士の減算を行う (`a -= b`)。
///
/// # Args
/// - `a`: 減算対象の配列 (Montgomery 表現)。`a[i]` は `MOD` 未満である必要がある。
/// - `b`: 減算する配列 (Montgomery 表現)。`b[i]` は `MOD` 未満である必要がある。
///
/// # Returns
/// `()`: `a` を in-place に更新する。
///
/// # Constraints
/// - `a.len() == b.len()` を満たす。
/// - `a` と `b` の各要素は `convolution::MOD` 未満である。
///
/// # Panics
/// - この関数はパニックし得る (デバッグアサート)。
///
/// # Complexity
/// - Time complexity: O(n)、ここで n は配列長である。
/// - Space complexity: O(1)。
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
            // `u32` の減算は負の値を表現できないため、先に `MOD` を足してから
            // 引くことで、結果が符号なしのまま非負に収まるようにする。
            let diff = x86_64::_mm256_sub_epi32(x86_64::_mm256_add_epi32(va, mts.mod_v), vb);
            let diff = mts.shrink(diff);
            x86_64::_mm256_storeu_si256(a.as_mut_ptr().add(i).cast(), diff);
        }

        // 端数はスカラー演算で処理する。
        for i in end..n {
            a[i] = modulo::sub(a[i], b[i]);
        }
    }
}

/// AVX2 を用いて、Montgomery 表現上のベクターを 2 倍する (`a *= 2`)。
///
/// # Args
/// - `a`: 倍化対象の配列 (Montgomery 表現)。`a[i]` は `MOD` 未満である必要がある。
///
/// # Returns
/// `()`: `a` を in-place に更新する。
///
/// # Constraints
/// - `a` の各要素は `convolution::MOD` 未満である。
///
/// # Panics
/// - この関数はパニックしない。
///
/// # Complexity
/// - Time complexity: O(n)、ここで n は配列長である。
/// - Space complexity: O(1)。
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

        // 2 倍は自身との加算として計算する。
        let end = n / AVX2_U32_LANES * AVX2_U32_LANES;
        for i in (0..end).step_by(AVX2_U32_LANES) {
            let va = x86_64::_mm256_loadu_si256(a.as_ptr().add(i).cast());
            let sum = mts.shrink(x86_64::_mm256_add_epi32(va, va));
            x86_64::_mm256_storeu_si256(a.as_mut_ptr().add(i).cast(), sum);
        }

        // 端数はスカラー演算で処理する。
        for i in end..n {
            a[i] = modulo::add(a[i], a[i]);
        }
    }
}

/// AVX2 を用いて、Montgomery 表現上で `a = b - a` を計算する。
///
/// # Args
/// - `a`: 結果を格納する配列 (Montgomery 表現)。`a[i]` は `MOD` 未満である必要がある。
/// - `b`: 左辺となる配列 (Montgomery 表現)。`b[i]` は `MOD` 未満である必要がある。
///
/// # Returns
/// `()`: `a` を in-place に更新する。
///
/// # Constraints
/// - `a.len() == b.len()` を満たす。
/// - `a` と `b` の各要素は `convolution::MOD` 未満である。
///
/// # Panics
/// - この関数はパニックし得る (デバッグアサート)。
///
/// # Complexity
/// - Time complexity: O(n)、ここで n は配列長である。
/// - Space complexity: O(1)。
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
            // `sub_assign_mont` と同様、`b - a` が符号なしのまま非負に収まる
            // よう、先に `MOD` を足してから引く。
            let diff = x86_64::_mm256_sub_epi32(x86_64::_mm256_add_epi32(vb, mts.mod_v), va);
            let diff = mts.shrink(diff);
            x86_64::_mm256_storeu_si256(a.as_mut_ptr().add(i).cast(), diff);
        }

        // 端数はスカラー演算で処理する。
        for i in end..n {
            a[i] = modulo::sub(b[i], a[i]);
        }
    }
}

/// AVX2 を用いて、Montgomery 表現上で NTT を実行する。
///
/// # Args
/// - `a`: 係数列 (Montgomery 表現)。この関数は `a` を in-place で更新する。
///
/// # Returns
/// `()`: `a` を NTT 変換後の値 (Montgomery 表現) で in-place に更新する。
///
/// # Constraints
/// - `a` は空であってはならず、長さは 2 の冪である必要がある。
///
/// # Panics
/// - この関数はパニックし得る (デバッグアサート)。
///
/// # Complexity
/// - Time complexity: O(a.len() log a.len())。
/// - Space complexity: O(1)。
///
/// # Examples
/// ```rust
/// use anmitsu::modulo998244353::convolution_mont;
///
/// let mut a = (1..=8).map(convolution_mont::standard_to_mont_scalar).collect::<Vec<u32>>();
/// unsafe {
///     convolution_mont::ntt_mont(&mut a);
///     convolution_mont::intt_mont(&mut a);
///     convolution_mont::mont_to_standard(&mut a);
/// }
/// let inv_len = 873463809_u64; // 998244353 における 8 の逆元 (通常表現)
/// let v0 = (a[0] as u64 * inv_len % 998244353) as u32;
/// assert_eq!(1, v0);
/// ```
#[target_feature(enable = "avx2")]
pub unsafe fn ntt_mont(a: &mut [u32]) {
    unsafe {
        let n = a.len();
        debug_assert!(n.is_power_of_two());
        debug_assert!(n > 0);

        // 長さが 1 SIMD レーン未満の場合は、汎用のバタフライループではなく
        // 打ち切り展開した専用の計算式で処理する。
        if n < AVX2_U32_LANES {
            if n == 1 {
                // n = 1 の DFT は恒等変換である。
                return;
            } else if n == 2 {
                // 長さ 2 の DFT は単純な和・差のみで、回転因子は不要である。
                let t0 = modulo::add(a[0], a[1]);
                let t1 = modulo::sub(a[0], a[1]);
                a[0] = t0;
                a[1] = t1;
            } else if n == 4 {
                // 長さ 4 の DFT を 2 段のバタフライへ手動展開したものである。
                // `OMEGA_1_4_MONT` は 4 乗根 `ω` の 1 乗 (Montgomery 表現) を表す。
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
                // n は 2 の冪かつ `AVX2_U32_LANES = 8` 未満なので、1、2、4 以外はあり得ない。
                unreachable!()
            }
        } else {
            // 前計算テーブルはプロセス全体で使い回すため、初回のみ構築する。
            let ntt = NTT.get_or_init(|| Ntt::new());
            ntt.ntt_mont(a.as_mut_ptr(), n);
        }
    }
}

/// AVX2 を用いて、Montgomery 表現上で INTT を実行する。
///
/// # Args
/// - `a`: NTT 値 (Montgomery 表現)。この関数は `a` を in-place で更新する。
///
/// # Returns
/// `()`: `a` を逆 NTT 変換後の値 (Montgomery 表現、正規化なし) で in-place に更新する。
///
/// # Constraints
/// - `a` は空であってはならず、長さは 2 の冪である必要がある。
///
/// # Panics
/// - この関数はパニックし得る (デバッグアサート)。
///
/// # Complexity
/// - Time complexity: O(a.len() log a.len())。
/// - Space complexity: O(1)。
///
/// # Examples
/// ```rust,ignore
/// // `pub fn ntt_mont` の doctest を参照。
/// ```
#[target_feature(enable = "avx2")]
pub unsafe fn intt_mont(a: &mut [u32]) {
    unsafe {
        let n = a.len();
        debug_assert!(n.is_power_of_two());
        debug_assert!(n > 0);

        // small-n: ntt_mont と対称に、打ち切り展開した専用の計算式で処理する。
        if n < AVX2_U32_LANES {
            if n == 1 {
                // n = 1 の逆 DFT は恒等変換である。
                return;
            } else if n == 2 {
                // 入力: [A0、A1] (n=2 は bitrev が恒等)
                // 出力: [2*a0、2*a1] (正規化なし)
                let t0 = modulo::add(a[0], a[1]);
                let t1 = modulo::sub(a[0], a[1]);
                a[0] = t0;
                a[1] = t1;
                return;
            } else if n == 4 {
                // `OMEGA_1_4_INV_MONT` は `ω^1` の逆元 (Montgomery 表現) であり、
                // `MOD - OMEGA_1_4_MONT` として求まる (加法逆元 = 乗法逆元ではなく、
                // ここでは `ω^{-1} = ω^{n-1}` の関係から `MOD - ω` の形で表せる)。
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
                // n は 2 の冪かつ `AVX2_U32_LANES = 8` 未満なので、1、2、4 以外はあり得ない。
                unreachable!()
            }
        } else {
            let ntt = NTT.get_or_init(|| Ntt::new());
            ntt.intt_mont(a.as_mut_ptr(), n);
        }
    }
}

#[target_feature(enable = "avx2")]
/// NTT 結果を in-place で doubling する。
///
/// # Args
/// - `a_ntt`: 長さ `n` の NTT 結果 (通常表現)。この関数は `a_ntt` を in-place で更新する。
///
/// # Returns
/// `()`: `a_ntt` の長さを `2n` に拡張し、後半 `n` 要素を計算して格納する。
///
/// # Constraints
/// - `a_ntt.len()` は 0 ではない 2 の冪である。
/// - `2 * a_ntt.len()` は `convolution::MAX_NTT_LEN` を超えてはならない。
/// - `a_ntt` の全要素は `convolution::MOD` 未満である。
/// - 呼び出し側は AVX2 が利用可能であることを保証する。
///
/// # Panics
/// - この関数はパニックし得る (デバッグアサート)。
///
/// # Complexity
/// - Time complexity: O(n log n)、ここで n は入力長である。
/// - Space complexity: O(n)。
///
/// # Examples
/// ```rust,ignore
/// // AVX2 依存のため、直接の使用例は省略する。
/// ```
pub unsafe fn ntt_doubling(a_ntt: &mut Vec<u32>) {
    unsafe {
        let n = a_ntt.len();
        debug_assert!(n.is_power_of_two());
        debug_assert!(2 * n <= convolution::MAX_NTT_LEN);
        // 実体は Montgomery 表現上の doubling なので、変換して委譲するだけである。
        standard_to_mont(a_ntt);
        ntt_doubling_mont(a_ntt);
        mont_to_standard(a_ntt);
    }
}

#[target_feature(enable = "avx2")]
/// Montgomery 表現上で NTT 結果を in-place で doubling する。
///
/// # Args
/// - `a_ntt_mont`: 長さ `n` の NTT 結果 (Montgomery 表現)。この関数は `a_ntt_mont` を
///   in-place で更新する。
///
/// # Returns
/// `()`: `a_ntt_mont` の長さを `2n` に拡張し、後半 `n` 要素を計算して格納する。
///
/// # Constraints
/// - `a_ntt_mont.len()` は 0 ではない 2 の冪である。
/// - `2 * a_ntt_mont.len()` は `convolution::MAX_NTT_LEN` を超えてはならない。
/// - 呼び出し側は AVX2 が利用可能であることを保証する。
///
/// # Panics
/// - この関数はパニックし得る (デバッグアサート)。
///
/// # Complexity
/// - Time complexity: O(n log n)、ここで n は入力長である。
/// - Space complexity: O(n)。
///
/// # Examples
/// ```rust,ignore
/// // AVX2 依存のため、直接の使用例は省略する。
/// ```
pub unsafe fn ntt_doubling_mont(a_ntt_mont: &mut Vec<u32>) {
    unsafe {
        let n = a_ntt_mont.len();
        debug_assert!(n.is_power_of_two());
        debug_assert!(2 * n <= convolution::MAX_NTT_LEN);
        // 長さ `2n` の NTT のうち、前半 `n` 個 (`n` 乗根での評価値) は入力
        // `a_ntt_mont` そのものなので、後半 `n` 個 (奇数番目の `2n` 乗根での
        // 評価値) だけを新たに求めればよい。
        //
        // まず係数列を復元する。`intt_mont` は正規化を行わないため `n` 倍
        // された状態になっており、`1/n` を掛けて元の係数に戻す。
        let mut coeffs = a_ntt_mont.clone();
        intt_mont(&mut coeffs);
        let lg = n.trailing_zeros() as usize;
        let inv_n_mont = inv_len_mont(lg);
        mul_scalar_mont(&mut coeffs, inv_n_mont);

        // 各係数 `coeffs[j]` に `ζ^j` (`ζ` は `2n` 乗根) を掛けて多項式を
        // "ねじる" と、ねじった多項式の `n` 乗根での評価値が、元の多項式の
        // 奇数番目の `2n` 乗根での評価値に一致する。
        let powers = NTT_DOUBLING_POWERS_MONT
            .get_or_init(|| NttDoublingPowersMont::new())
            .powers(n);
        let mut twisted = coeffs;
        mul_pointwise_mont(&mut twisted, powers);
        ntt_mont(&mut twisted);

        // 前半は元の評価値のまま、後半にねじった多項式の評価値を追加する。
        a_ntt_mont.resize(2 * n, 0);
        a_ntt_mont[n..].copy_from_slice(&twisted);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Background: この環境で AVX2 が利用可能かどうかを判定する。
    ///
    /// 本モジュールの多くの関数は AVX2 命令を直接発行するため、 AVX2 非対応の
    /// 環境で呼び出すと未定義動作になる。CI 環境によっては AVX2 が利用できない
    /// 場合もあるため、 各テストの冒頭でこの関数を確認し、 利用不可能なら
    /// 早期リターンでスキップする。
    fn avx2_available() -> bool {
        std::is_x86_feature_detected!("avx2")
    }

    // reduce_mont のテスト: 戻り値を検証する。
    mod reduce_mont {
        use super::*;

        /// Scenario: `0` の reduction は `0` になる (境界値)。
        /// - Given: 引数として `0` がある。
        /// - When: `reduce_mont` を呼ぶ。
        /// - Then: `0` が返る。
        #[test]
        fn returns_zero_for_zero() {
            // Given, When
            let result = reduce_mont(0);
            // Then
            assert_eq!(0, result);
        }
    }

    // mul_mont のテスト: 戻り値を検証する。
    mod mul_mont {
        use super::*;

        /// Scenario: 典型的な値同士の Montgomery 乗算は、 通常表現での積と一致する。
        /// - Given: `2`, `3` を Montgomery 表現へ変換した値がある。
        /// - When: `mul_mont` を呼ぶ。
        /// - Then: 通常表現へ戻すと `6` になる。
        #[test]
        fn matches_product_for_typical_values() {
            // Given
            let a_mont = standard_to_mont_scalar(2);
            let b_mont = standard_to_mont_scalar(3);
            // When
            let result = mul_mont(a_mont, b_mont);
            // Then
            assert_eq!(6, mont_to_standard_scalar(result));
        }

        /// Scenario: `0` との乗算は `0` になる (境界値)。
        /// - Given: `0` を Montgomery 表現へ変換した値と、 `5` を変換した値がある。
        /// - When: `mul_mont` を呼ぶ。
        /// - Then: 通常表現へ戻すと `0` になる。
        #[test]
        fn returns_zero_when_multiplying_by_zero() {
            // Given
            let a_mont = standard_to_mont_scalar(0);
            let b_mont = standard_to_mont_scalar(5);
            // When
            let result = mul_mont(a_mont, b_mont);
            // Then
            assert_eq!(0, mont_to_standard_scalar(result));
        }
    }

    // standard_to_mont / mont_to_standard のテスト: 往復変換で元に戻ることを検証する。
    mod standard_to_mont {
        use super::*;

        /// Scenario: SIMD レーン未満の短い配列でも、 往復変換で元の値に戻る (境界値)。
        /// - Given: AVX2 が利用可能な環境で、 長さ 4 の配列がある。
        /// - When: `standard_to_mont` を適用してから `mont_to_standard` を適用する。
        /// - Then: 元の配列と一致する。
        #[test]
        fn round_trip_with_mont_to_standard_for_short_array() {
            if !avx2_available() {
                return;
            }
            // Given
            let input = vec![1_u32, 2, 3, 4];
            let mut actual = input.clone();
            // When
            unsafe {
                standard_to_mont(&mut actual);
                mont_to_standard(&mut actual);
            }
            // Then
            assert_eq!(input, actual);
        }

        /// Scenario: SIMD レーン以上の長い配列でも、 往復変換で元の値に戻る。
        /// - Given: AVX2 が利用可能な環境で、 長さ 64 の配列がある。
        /// - When: `standard_to_mont` を適用してから `mont_to_standard` を適用する。
        /// - Then: 元の配列と一致する。
        #[test]
        fn round_trip_with_mont_to_standard_for_long_array() {
            if !avx2_available() {
                return;
            }
            // Given
            let input = (0..64_u32).collect::<Vec<u32>>();
            let mut actual = input.clone();
            // When
            unsafe {
                standard_to_mont(&mut actual);
                mont_to_standard(&mut actual);
            }
            // Then
            assert_eq!(input, actual);
        }
    }

    // mul_pointwise_mont のテスト: 戻り値を検証する。
    mod mul_pointwise_mont {
        use super::*;

        /// Scenario: SIMD レーン未満の短い配列でも、 各要素の積を計算する (境界値)。
        /// - Given: AVX2 が利用可能な環境で、 `2` と `3` (Montgomery 表現) を
        ///   要素とする長さ 2 の配列がある。
        /// - When: `mul_pointwise_mont` を適用する。
        /// - Then: 通常表現へ戻すと、 各要素が `6` になる。
        #[test]
        fn computes_elementwise_product_for_short_array() {
            if !avx2_available() {
                return;
            }
            // Given
            let mut a = vec![standard_to_mont_scalar(2); 2];
            let b = vec![standard_to_mont_scalar(3); 2];
            // When
            unsafe {
                mul_pointwise_mont(&mut a, &b);
                mont_to_standard(&mut a);
            }
            // Then
            assert_eq!(vec![6_u32; 2], a);
        }

        /// Scenario: SIMD レーン以上の長い配列でも、 各要素の積を計算する。
        /// - Given: AVX2 が利用可能な環境で、 `2` と `3` (Montgomery 表現) を
        ///   要素とする長さ 16 の配列がある。
        /// - When: `mul_pointwise_mont` を適用する。
        /// - Then: 通常表現へ戻すと、 各要素が `6` になる。
        #[test]
        fn computes_elementwise_product_for_long_array() {
            if !avx2_available() {
                return;
            }
            // Given
            let mut a = vec![standard_to_mont_scalar(2); 16];
            let b = vec![standard_to_mont_scalar(3); 16];
            // When
            unsafe {
                mul_pointwise_mont(&mut a, &b);
                mont_to_standard(&mut a);
            }
            // Then
            assert_eq!(vec![6_u32; 16], a);
        }
    }

    // mul_scalar_mont のテスト: 戻り値を検証する。
    mod mul_scalar_mont {
        use super::*;

        /// Scenario: SIMD レーン未満の短い配列でも、 スカラー倍を計算する (境界値)。
        /// - Given: AVX2 が利用可能な環境で、 `2` (Montgomery 表現) を要素とする
        ///   長さ 2 の配列と、 `3` (Montgomery 表現) のスカラー値がある。
        /// - When: `mul_scalar_mont` を適用する。
        /// - Then: 通常表現へ戻すと、 各要素が `6` になる。
        #[test]
        fn scales_short_array() {
            if !avx2_available() {
                return;
            }
            // Given
            let mut a = vec![standard_to_mont_scalar(2); 2];
            let sc_mont = standard_to_mont_scalar(3);
            // When
            unsafe {
                mul_scalar_mont(&mut a, sc_mont);
                mont_to_standard(&mut a);
            }
            // Then
            assert_eq!(vec![6_u32; 2], a);
        }

        /// Scenario: SIMD レーン以上の長い配列でも、 スカラー倍を計算する。
        /// - Given: AVX2 が利用可能な環境で、 `2` (Montgomery 表現) を要素とする
        ///   長さ 16 の配列と、 `3` (Montgomery 表現) のスカラー値がある。
        /// - When: `mul_scalar_mont` を適用する。
        /// - Then: 通常表現へ戻すと、 各要素が `6` になる。
        #[test]
        fn scales_long_array() {
            if !avx2_available() {
                return;
            }
            // Given
            let mut a = vec![standard_to_mont_scalar(2); 16];
            let sc_mont = standard_to_mont_scalar(3);
            // When
            unsafe {
                mul_scalar_mont(&mut a, sc_mont);
                mont_to_standard(&mut a);
            }
            // Then
            assert_eq!(vec![6_u32; 16], a);
        }
    }

    // ntt_mont / intt_mont のテスト: 往復変換で元に戻ることを検証する。
    mod ntt_mont {
        use super::*;

        /// Scenario: 様々な長さ (小さい方の打ち切り展開分岐を含む) で、
        /// NTT に続けて逆 NTT を適用すると、 元の値の `n` 倍 (正規化なし) に戻る。
        /// - Given: AVX2 が利用可能な環境で、 長さ `2^lg` (lg = 0..=6) の係数列がある。
        /// - When: `ntt_mont` を適用してから `intt_mont` を適用し、 `1/n` で正規化する。
        /// - Then: 元の入力と一致する。
        #[test]
        fn round_trip_with_intt_mont_matches_original_after_scaling() {
            if !avx2_available() {
                return;
            }
            for lg in 0..=6 {
                // Given
                let n = 1usize << lg;
                let input = (0..n as u32)
                    .map(|x| x % convolution::MOD)
                    .collect::<Vec<u32>>();
                let mut actual = input.clone();

                // When
                unsafe {
                    standard_to_mont(&mut actual);
                    ntt_mont(&mut actual);
                    intt_mont(&mut actual);
                    let inv_n_mont = inv_len_mont(lg);
                    mul_scalar_mont(&mut actual, inv_n_mont);
                    mont_to_standard(&mut actual);
                }

                // Then
                assert_eq!(input, actual, "lg mismatch: {lg}");
            }
        }
    }

    // ntt_doubling のテスト: 戻り値を検証する。
    mod ntt_doubling {
        use super::*;

        /// Scenario: 長さ `n` の NTT を doubling して得た長さ `2n` の NTT は、
        /// 長さ `2n` で直接計算した NTT と一致する。
        /// - Given: AVX2 が利用可能な環境で、 次数が `n` 未満の多項式の係数列がある。
        /// - When: 長さ `n` で NTT を計算してから `ntt_doubling` で長さ `2n` へ拡張する。
        /// - Then: 係数列を長さ `2n` へゼロ埋めしてから直接計算した NTT と一致する。
        #[test]
        fn matches_direct_ntt_of_doubled_length() {
            if !avx2_available() {
                return;
            }
            // Given
            let n = 8usize;
            let original = (0..n as u32).collect::<Vec<u32>>();

            // 長さ 2n で直接 NTT を計算した結果を期待値とする。
            let mut expected = original.clone();
            expected.resize(2 * n, 0);
            unsafe {
                standard_to_mont(&mut expected);
                ntt_mont(&mut expected);
                mont_to_standard(&mut expected);
            }

            // When
            let mut actual = original.clone();
            unsafe {
                standard_to_mont(&mut actual);
                ntt_mont(&mut actual);
                mont_to_standard(&mut actual);
                ntt_doubling(&mut actual);
            }

            // Then
            assert_eq!(expected, actual);
        }
    }
}
