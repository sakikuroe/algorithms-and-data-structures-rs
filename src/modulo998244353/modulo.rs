//! Provides primitive modular arithmetic for the modulus 998244353.
//! 法 998244353 向けの基本的なモジュラー演算を提供する.

/// The modulus for arithmetic.
/// 演算で用いる法.
pub const M: u32 = 998244353;

/// Reduces `a` into the range `[0, M)`.
/// `a` を `[0, M)` に還元する.
///
/// # Args
/// - `a`: Value to reduce.
///        還元する値.
///
/// # Returns
/// `u32`: Reduced value.
///        還元後の値.
///
/// # Constraints
/// None.
/// 制約はない.
///
/// # Panics
/// This function does not panic.
/// この関数はパニックしない.
///
/// # Complexity
/// - Time complexity: O(1).
///                    時間計算量は O(1).
///
/// # Examples
/// ```rust
/// use anmitsu::modulo998244353::modulo;
///
/// assert_eq!(42, modulo::modulo(42));
/// assert_eq!(1, modulo::modulo(998244354));
/// ```
pub const fn modulo(a: u64) -> u32 {
    (a % M as u64) as u32
}

/// Adds two residues under the modulus.
/// 法の下で 2 つの値を加算する.
///
/// # Args
/// - `a`: Left operand (< M).
///        左辺の被演算子 (M 未満).
/// - `b`: Right operand (< M).
///        右辺の被演算子 (M 未満).
///
/// # Returns
/// `u32`: `(a + b) mod M`.
///        `(a + b) mod M` の値.
///
/// # Constraints
/// - `a < M`.
/// - `b < M`.
/// - `a` と `b` はいずれも M 未満でなければならない.
///
/// # Panics
/// Panics if either operand is greater than or equal to `M`.
/// 被演算子が M 以上の場合にパニックする.
///
/// # Complexity
/// - Time complexity: O(1).
///                    時間計算量は O(1).
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
    let t = a + b;
    if t < M { t } else { t.wrapping_sub(M) }
}

/// Subtracts two residues under the modulus.
/// 法の下で 2 つの値を減算する.
///
/// # Args
/// - `a`: Left operand (< M).
///        左辺の被演算子 (M 未満).
/// - `b`: Right operand (< M).
///        右辺の被演算子 (M 未満).
///
/// # Returns
/// `u32`: `(a - b) mod M`.
///        `(a - b) mod M` の値.
///
/// # Constraints
/// - `a < M`.
/// - `b < M`.
/// - `a` と `b` はいずれも M 未満でなければならない.
///
/// # Panics
/// Panics if either operand is greater than or equal to `M`.
/// 被演算子が M 以上の場合にパニックする.
///
/// # Complexity
/// - Time complexity: O(1).
///                    時間計算量は O(1).
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
    let (t, f) = a.overflowing_sub(b);
    if !f { t } else { t.wrapping_add(M) }
}

/// Multiplies two residues under the modulus.
/// 法の下で 2 つの値を乗算する.
///
/// # Args
/// - `a`: Left operand (< M).
///        左辺の被演算子 (M 未満).
/// - `b`: Right operand (< M).
///        右辺の被演算子 (M 未満).
///
/// # Returns
/// `u32`: `(a * b) mod M`.
///        `(a * b) mod M` の値.
///
/// # Constraints
/// - `a < M`.
/// - `b < M`.
/// - `a` と `b` はいずれも M 未満でなければならない.
///
/// # Panics
/// Panics if either operand is greater than or equal to `M`.
/// 被演算子が M 以上の場合にパニックする.
///
/// # Complexity
/// - Time complexity: O(1).
///                    時間計算量は O(1).
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
    modulo(a as u64 * b as u64)
}

/// Computes the additive inverse of `a`.
/// `a` の加法逆元を計算する.
///
/// # Args
/// - `a`: Operand (< M).
///        被演算子 (M 未満).
///
/// # Returns
/// `u32`: `(-a) mod M`.
///        `(-a) mod M` の値.
///
/// # Constraints
/// - `a < M`.
/// - `a` は M 未満でなければならない.
///
/// # Panics
/// Panics if `a` is greater than or equal to `M`.
/// `a` が M 以上の場合にパニックする.
///
/// # Complexity
/// - Time complexity: O(1).
///                    時間計算量は O(1).
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
    if a == 0 { 0 } else { M - a }
}

/// Builds a table of modular inverses `inv[i] = 1/i (mod M)` for `i = 0..len-1`.
/// `inv[i] = 1/i (mod M)` を満たす逆元テーブルを `i = 0..len-1` について構築する.
///
/// # Args
/// - `len`: The desired length of the table.
///          テーブル長.
///
/// # Returns
/// `Vec<u32>`: A vector satisfying `inv[i] = 1/i (mod M)` for `i >= 1`.
///             `i >= 1` について `inv[i] = 1/i (mod M)` を満たすベクター.
///
/// # Constraints
/// - `len < M`.
/// - `len` は `M` 未満でなければならない.
///
/// # Panics
/// Panics if `len >= M`.
/// `len >= M` のときパニックする.
///
/// # Complexity
/// - Time complexity: O(len).
///                    時間計算量は O(len).
/// - Space complexity: O(len).
///                     追加領域は O(len).
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

    let mut inv = vec![0_u32; len];
    if len > 1 {
        inv[1] = 1;
        for i in 2..len {
            let iu = i as u32;
            let q = M / iu;
            let r = (M % iu) as usize;
            let t = mul(q, inv[r]);
            let val = M - t;
            inv[i] = if val == M { 0 } else { val };
        }
    }

    inv
}

/// Raises `a` to the power `n` under the modulus.
/// `a` の `n` 乗を法の下で計算する.
///
/// # Args
/// - `a`: Base (< M).
///        底 (M 未満).
/// - `n`: Exponent.
///        指数.
///
/// # Returns
/// `u32`: `a^n mod M`.
///        `a^n mod M` の値.
///
/// # Constraints
/// - `a < M`.
/// - `a` は M 未満でなければならない.
///
/// # Panics
/// Panics if `a` is greater than or equal to `M`.
/// `a` が M 以上の場合にパニックする.
///
/// # Complexity
/// - Time complexity: O(log n).
///                    時間計算量は O(log n).
///
/// # Examples
/// ```rust
/// use anmitsu::modulo998244353::modulo;
///
/// assert_eq!(8, modulo::pow(2, 3));
/// ```
pub const fn pow(a: u32, mut n: usize) -> u32 {
    debug_assert!(a < M);

    let mut res = 1;
    let mut x = a;

    while n > 0 {
        if n % 2 == 1 {
            res = mul(res, x);
        }
        x = mul(x, x);
        n /= 2;
    }

    res
}

/// Computes the multiplicative inverse of `a`.
/// `a` の乗法逆元を計算する.
///
/// # Args
/// - `a`: Operand (< M).
///        被演算子 (M 未満).
///
/// # Returns
/// `u32`: `a^-1 mod M`.
///        `a^-1 mod M` の値.
///
/// # Constraints
/// - `a < M`.
/// - `a` must not be zero.
/// - `a` は M 未満で, かつゼロであってはならない.
///
/// # Panics
/// Panics if `a` is zero or greater than or equal to `M`.
/// `a` が 0 または M 以上の場合にパニックする.
///
/// # Complexity
/// - Time complexity: O(log M).
///                    時間計算量は O(log M).
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
    pow(a, M as usize - 2)
}
