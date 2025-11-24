//! Provides convolution modulo 998244353 using the number theoretic transform.
//! 998244353 を法として number theoretic transform を用いた畳み込みを提供する.

/// The modulus used by this convolution implementation.
/// この畳み込み実装で用いる法.
pub const MOD: u32 = 998244353;

/// The maximum supported length for the transform.
/// 変換がサポートする最大長.
pub const MAX_NTT_LEN: usize = 1 << 22;

const NTT_RATE: [u32; 22] = [
    0x3656d65b, 0x1e5ea9e6, 0x16038782, 0x13caac90, 0x3a9a4cfa, 0x761af21, 0xe372007, 0x3a2be7d4,
    0x23fe18b2, 0x330f5b68, 0x7d37cf9, 0x3239edef, 0x2b8ea5c3, 0x382d2452, 0x300e9be2, 0x908b3f5,
    0x1e726cd9, 0x1e02c2f0, 0x2c49629c, 0x2c2b7c93, 0x35a5081, 0x33b69d8b,
];

const INTT_RATE: [u32; 22] = [
    0x52929a6, 0x163456b8, 0x16400573, 0x267c5b5f, 0x6b059a5, 0x294c15f1, 0x94415d9, 0x2f83389c,
    0x569c0ec, 0x3346ebba, 0x37473ab0, 0x1524e16f, 0x68442e3, 0x117ab9d0, 0x1fe52df0, 0x1263f553,
    0x7392943, 0x24433aa8, 0x1a2993eb, 0x156d2fbf, 0x311e570f, 0x6294a13,
];

const INVS: [u32; 23] = [
    1, 499122177, 748683265, 873463809, 935854081, 967049217, 982646785, 990445569, 994344961,
    996294657, 997269505, 997756929, 998000641, 998122497, 998183425, 998213889, 998229121,
    998236737, 998240545, 998242449, 998243401, 998243877, 998244115,
];

#[inline]
fn add_mod(lhs: u32, rhs: u32) -> u32 {
    let sum = lhs + rhs;
    if sum >= MOD { sum - MOD } else { sum }
}

#[inline]
fn sub_mod(lhs: u32, rhs: u32) -> u32 {
    if lhs >= rhs {
        lhs - rhs
    } else {
        lhs + MOD - rhs
    }
}

#[inline]
fn mul_mod(lhs: u32, rhs: u32) -> u32 {
    ((lhs as u64 * rhs as u64) % MOD as u64) as u32
}

/// Performs an in-place number theoretic transform (NTT) on the given buffer.
/// 与えられた列に対してインプレースで number theoretic transform を実行する.
///
/// # Args
/// - `a`: A slice of coefficients whose length is a power of two.
///        長さが 2 の冪となる係数列.
///
/// # Returns
/// `()`: This function mutates `a` in place.
///       この関数は `a` をインプレースで更新する.
///
/// # Constraints
/// - `a.len()` must be a non-zero power of two.
/// - `a.len()` must not exceed `MAX_NTT_LEN`.
/// - All elements must be less than `MOD`.
/// - `a.len()` は 0 ではない 2 の冪でなければならない.
/// - `a.len()` は `MAX_NTT_LEN` を超えてはならない.
/// - 全ての要素は `MOD` 未満でなければならない.
///
/// # Panics
/// - Panics if the length constraint is violated.
///   制約に違反した場合はパニックする.
///
/// # Complexity
/// - Time complexity: O(N log N), where N is `a.len()`.
///                    ここで N は `a.len()` である.
/// - Space complexity: O(1).
///                     追加の領域は O(1).
///
/// # Examples
/// ```rust
/// use anmitsu::ds::modint::convolution998244353::{intt, ntt};
///
/// let mut values = vec![1, 2, 3, 4];
/// ntt(&mut values);
/// intt(&mut values);
/// let inv_len = 748683265; // modular inverse of 4 under 998244353
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
                    let r = mul_mod(*ptr.add(i + p), rot);
                    *ptr.add(i) = add_mod(l, r);
                    *ptr.add(i + p) = sub_mod(l, r);
                }
            }
            rot = mul_mod(rot, NTT_RATE[s.trailing_ones() as usize]);
        }
    }
}

/// Performs an in-place inverse number theoretic transform (INTT).
/// 与えられた列に対してインプレースで逆 number theoretic transform を実行する.
///
/// # Args
/// - `a`: A slice of coefficients whose length is a power of two.
///        長さが 2 の冪となる係数列.
///
/// # Returns
/// `()`: This function mutates `a` in place.
///       この関数は `a` をインプレースで更新する.
///
/// # Constraints
/// - `a.len()` must be a non-zero power of two.
/// - `a.len()` must not exceed `MAX_NTT_LEN`.
/// - All elements must be less than `MOD`.
/// - `a.len()` は 0 ではない 2 の冪でなければならない.
/// - `a.len()` は `MAX_NTT_LEN` を超えてはならない.
/// - 全ての要素は `MOD` 未満でなければならない.
///
/// # Panics
/// - Panics if the length constraint is violated.
///   制約に違反した場合はパニックする.
///
/// # Complexity
/// - Time complexity: O(N log N), where N is `a.len()`.
///                    ここで N は `a.len()` である.
/// - Space complexity: O(1).
///                     追加の領域は O(1).
///
/// # Examples
/// ```rust
/// use anmitsu::ds::modint::convolution998244353::{intt, ntt};
///
/// let mut values = vec![5, 6, 7, 8];
/// ntt(&mut values);
/// intt(&mut values);
/// let inv_len = 748683265; // modular inverse of 4 under 998244353
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
                    *ptr.add(i) = add_mod(l, r);
                    *ptr.add(i + p) = mul_mod(sub_mod(l, r), irot);
                }
            }
            irot = mul_mod(irot, INTT_RATE[s.trailing_ones() as usize]);
        }
    }
}

/// Computes the convolution of two sequences modulo 998244353.
/// 998244353 を法として 2 つの列の畳み込みを計算する.
///
/// # Args
/// - `a`: The first input sequence with coefficients reduced modulo `MOD`.
///        法 `MOD` で還元された最初の入力列.
/// - `b`: The second input sequence with coefficients reduced modulo `MOD`.
///        法 `MOD` で還元された 2 番目の入力列.
///
/// # Returns
/// `Vec<u32>`: Convolution result modulo `MOD`.
///             法 `MOD` での畳み込み結果.
///
/// # Constraints
/// - If either slice is empty, an empty vector is returned.
/// - The total length `a.len() + b.len() - 1` must not exceed `MAX_NTT_LEN`.
/// - All coefficients must be less than `MOD`.
/// - いずれかのスライスが空の場合は空のベクターを返す.
/// - `a.len() + b.len() - 1` は `MAX_NTT_LEN` を超えてはならない.
/// - すべての係数は `MOD` 未満でなければならない.
///
/// # Panics
/// - Panics if the length constraint is violated.
///   長さ制約に違反した場合にパニックする.
///
/// # Complexity
/// - Time complexity: O((N + M) log K) where N and M are input lengths and
///                    K is the next power of two of `N + M - 1`.
///                    時間計算量は O((N + M) log K) で, K は `N + M - 1`
///                    を超えない最小の 2 の冪.
/// - Space complexity: O(K).
///                     追加領域は O(K).
///
/// # Examples
/// ```rust
/// use anmitsu::ds::modint::convolution998244353::convolution;
///
/// let a = vec![1, 2, 3];
/// let b = vec![4, 5, 6];
/// let result = convolution(&a, &b);
/// assert_eq!(vec![4, 13, 28, 27, 18], result);
/// ```
pub fn convolution(a: &[u32], b: &[u32]) -> Vec<u32> {
    if a.is_empty() || b.is_empty() {
        return Vec::new();
    }

    debug_assert!(a.iter().all(|&x| x < MOD));
    debug_assert!(b.iter().all(|&x| x < MOD));

    let s = a.len() + b.len() - 1;
    if a.len().min(b.len()) <= 32 {
        let mut res = vec![0; s];
        for i in 0..a.len() {
            let ai = a[i];
            for j in 0..b.len() {
                res[i + j] = add_mod(res[i + j], mul_mod(ai, b[j]));
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

    let mut fa = Vec::with_capacity(t);
    fa.extend_from_slice(a);
    fa.resize(t, 0);
    let mut fb = Vec::with_capacity(t);
    fb.extend_from_slice(b);
    fb.resize(t, 0);

    ntt(&mut fa);
    ntt(&mut fb);
    fa.iter_mut()
        .zip(fb.iter())
        .for_each(|(x, y)| *x = mul_mod(*x, *y));
    intt(&mut fa);
    let t_inv = INVS[t.trailing_zeros() as usize];
    fa.iter_mut().take(s).for_each(|x| *x = mul_mod(*x, t_inv));
    fa.truncate(s);
    fa
}
