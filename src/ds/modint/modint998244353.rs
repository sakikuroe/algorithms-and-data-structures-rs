//! This module provides a modular integer implementation for the prime modulus 998244353.
//! このモジュールは, 素数 998244353 を法とするモジュラー整数実装を提供する.

use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

/// The modulus for modular arithmetic operations.
/// モジュラー演算の法.
const MOD: u32 = 998244353; // 119 * (1 << 23) + 1

/// Represents a modular integer for the prime modulus 998244353.
/// 素数 998244353 を法とするモジュラー整数を表現する.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ModInt998244353 {
    val: u32,
}

impl ModInt998244353 {
    /// Creates a new `ModInt998244353` instance from a `u64` value.
    /// `u64` 値から新しい `ModInt998244353` インスタンスを生成する.
    ///
    /// # Args
    /// - `n`: The `u64` value to be converted to a modular integer.
    ///        モジュラー整数に変換する `u64` 値.
    ///
    /// # Returns
    /// `Self`: A new `ModInt998244353` instance with its value reduced modulo `MOD`.
    ///         値が `MOD` で還元された新しい `ModInt998244353` インスタンス.
    ///
    /// # Complexity
    /// - Time complexity: O(1).
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::ds::modint::modint998244353::ModInt998244353;
    /// let m = ModInt998244353::new(1_000_000_000);
    /// assert_eq!(1755647, m.val());
    /// ```
    pub fn new(n: u64) -> Self {
        ModInt998244353 {
            val: (n % MOD as u64) as u32,
        }
    }

    /// Creates a new `ModInt998244353` instance from a raw `u32` value.
    /// 生の `u32` 値から新しい `ModInt998244353` インスタンスを生成する.
    ///
    /// # Args
    /// - `n`: The `u32` value, which must be less than `MOD`.
    ///        `MOD` 未満でなければならない `u32` 値.
    ///
    /// # Returns
    /// `Self`: A new `ModInt998244353` instance.
    ///         新しい `ModInt998244353` インスタンス.
    ///
    /// # Panics
    /// Panics if `n` is greater than or equal to `MOD`.
    /// `n` が `MOD` 以上の場合にパニックする.
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::ds::modint::modint998244353::ModInt998244353;
    /// let m = ModInt998244353::new_raw(100);
    /// assert_eq!(100, m.val());
    /// // ModInt998244353::new_raw(998244353); // This would panic
    /// ```
    pub fn new_raw(n: u32) -> Self {
        assert!(n < MOD, "Raw value {} must be less than MOD {}", n, MOD);
        ModInt998244353 { val: n }
    }

    /// Returns the underlying `u32` value of the modular integer.
    /// モジュラー整数の基となる `u32` 値を返す.
    ///
    /// # Returns
    /// `u32`: The raw `u32` value of the modular integer.
    ///         モジュラー整数の生の `u32` 値.
    ///
    /// # Complexity
    /// - Time complexity: O(1).
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::ds::modint::modint998244353::ModInt998244353;
    /// let m = ModInt998244353::new(123);
    /// assert_eq!(123, m.val());
    /// ```
    pub fn val(&self) -> u32 {
        self.val
    }

    /// Computes the modular multiplicative inverse of the modular integer.
    /// モジュラー整数のモジュラー乗法逆元を計算する.
    ///
    /// This method uses Fermat's Little Theorem, which requires the modulus `MOD` to be prime.
    /// Since `MOD` is prime (998244353), the inverse `a^(MOD-2)` is computed.
    /// このメソッドはフェルマーの小定理を使用し, 法 `MOD` が素数であることを前提とする.
    /// `MOD` は素数 (998244353) であるため, 逆元 `a^(MOD-2)` が計算される.
    ///
    /// # Returns
    /// `Option<Self>`: Returns `Some(inverse)` if `self.val` is non-zero,
    ///                 `None` if `self.val` is zero (as inverse does not exist).
    ///                 `self.val` が非ゼロの場合に `Some(inverse)` を返し,
    ///                 `self.val` がゼロの場合に `None` を返す (逆元が存在しないため).
    ///
    /// # Constraints
    /// For an inverse to exist, `self.val` must not be zero when `MOD` is prime.
    /// 逆元が存在するには, `MOD` が素数の場合, `self.val` がゼロであってはならない.
    ///
    /// # Complexity
    /// - Time complexity: O(log MOD).
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::ds::modint::modint998244353::ModInt998244353;
    /// let m = ModInt998244353::new(2);
    /// let inv_m = m.inv().unwrap();
    /// assert_eq!(1, (m * inv_m).val());
    ///
    /// let zero = ModInt998244353::new(0);
    /// assert!(zero.inv().is_none());
    /// ```
    pub fn inv(&self) -> Option<Self> {
        if self.val == 0 {
            None
        } else {
            // By Fermat's Little Theorem: a^(MOD-2) mod MOD is the inverse.
            Some(self.pow((MOD - 2) as usize))
        }
    }

    pub fn pow(&self, mut n: usize) -> Self {
        let mut res = ModInt998244353::new_raw(1);
        let mut base = *self;

        while n > 0 {
            if n % 2 == 1 {
                res *= base;
            }
            base *= base;
            n /= 2;
        }
        res
    }
}

/// Allows conversion from `u32` to `ModInt998244353`.
/// `u32` から `ModInt998244353` への変換を可能にする.
impl From<u32> for ModInt998244353 {
    fn from(num: u32) -> Self {
        ModInt998244353::new(num as u64)
    }
}

/// Allows conversion from `i32` to `ModInt998244353`.
/// `i32` から `ModInt998244353` への変換を可能にする.
impl From<i32> for ModInt998244353 {
    fn from(num: i32) -> Self {
        // Ensure non-negative value for modular arithmetic.
        let val = if num >= 0 {
            num as u64
        } else {
            (num % MOD as i32 + MOD as i32) as u64
        };
        ModInt998244353::new(val)
    }
}

/// Implements the addition operation (`+`) for two `ModInt998244353` instances.
/// 2つの `ModInt998244353` インスタンスに対する加算演算 (`+`) を実装する.
impl Add for ModInt998244353 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        ModInt998244353::new_raw((self.val + rhs.val) % MOD)
    }
}

/// Implements the addition operation (`+`) for `ModInt998244353` and `u32`.
/// `ModInt998244353` と `u32` に対する加算演算 (`+`) を実装する.
impl Add<u32> for ModInt998244353 {
    type Output = Self;

    fn add(self, rhs: u32) -> Self::Output {
        ModInt998244353::new((self.val as u64 + rhs as u64) % MOD as u64)
    }
}

/// Implements the addition assignment operation (`+=`) for `ModInt998244353`.
/// `ModInt998244353` に対する加算代入演算 (`+=`) を実装する.
impl AddAssign for ModInt998244353 {
    fn add_assign(&mut self, rhs: Self) {
        self.val = (self.val + rhs.val) % MOD;
    }
}

/// Implements the addition assignment operation (`+=`) for `ModInt998244353` and `u32`.
/// `ModInt998244353` と `u32` に対する加算代入演算 (`+=`) を実装する.
impl AddAssign<u32> for ModInt998244353 {
    fn add_assign(&mut self, rhs: u32) {
        self.val = (self.val as u64 + rhs as u64 % MOD as u64) as u32 % MOD;
    }
}

/// Implements the subtraction operation (`-`) for two `ModInt998244353` instances.
/// 2つの `ModInt998244353` インスタンスに対する減算演算 (`-`) を実装する.
impl Sub for ModInt998244353 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        // Ensure the result is non-negative.
        ModInt998244353::new_raw((self.val + MOD - rhs.val) % MOD)
    }
}

/// Implements the subtraction operation (`-`) for `ModInt998244353` and `u32`.
/// `ModInt998244353` と `u32` に対する減算演算 (`-`) を実装する.
impl Sub<u32> for ModInt998244353 {
    type Output = Self;

    fn sub(self, rhs: u32) -> Self::Output {
        // Ensure the result is non-negative.
        ModInt998244353::new_raw((self.val + MOD - (rhs % MOD)) % MOD)
    }
}

/// Implements the subtraction assignment operation (`-=`) for `ModInt998244353`.
/// `ModInt998244353` に対する減算代入演算 (`-=`) を実装する.
impl SubAssign for ModInt998244353 {
    fn sub_assign(&mut self, rhs: Self) {
        self.val = (self.val + MOD - rhs.val) % MOD;
    }
}

/// Implements the subtraction assignment operation (`-=`) for `ModInt998244353` and `u32`.
/// `ModInt998244353` と `u32` に対する減算代入演算 (`-=`) を実装する.
impl SubAssign<u32> for ModInt998244353 {
    fn sub_assign(&mut self, rhs: u32) {
        self.val = (self.val + MOD - (rhs % MOD)) % MOD;
    }
}

/// Implements the multiplication operation (`*`) for two `ModInt998244353` instances.
/// 2つの `ModInt998244353` インスタンスに対する乗算演算 (`*`) を実装する.
impl Mul for ModInt998244353 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        ModInt998244353::new((self.val as u64) * (rhs.val as u64))
    }
}

/// Implements the multiplication operation (`*`) for `ModInt998244353` and `u32`.
/// `ModInt998244353` と `u32` に対する乗算演算 (`*`) を実装する.
impl Mul<u32> for ModInt998244353 {
    type Output = Self;

    fn mul(self, rhs: u32) -> Self::Output {
        ModInt998244353::new((self.val as u64) * (rhs as u64))
    }
}

/// Implements the multiplication assignment operation (`*=`) for `ModInt998244353`.
/// `ModInt998244353` に対する乗算代入演算 (`*=`) を実装する.
impl MulAssign for ModInt998244353 {
    fn mul_assign(&mut self, rhs: Self) {
        self.val = (self.val as u64 * rhs.val as u64 % MOD as u64) as u32;
    }
}

/// Implements the multiplication assignment operation (`*=`) for `ModInt998244353` and `u32`.
/// `ModInt998244353` と `u32` に対する乗算代入演算 (`*=`) を実装する.
impl MulAssign<u32> for ModInt998244353 {
    fn mul_assign(&mut self, rhs: u32) {
        self.val = (self.val as u64 * rhs as u64 % MOD as u64) as u32;
    }
}

/// Implements the division operation (`/`) for two `ModInt998244353` instances.
/// 2つの `ModInt998244353` インスタンスに対する除算演算 (`/`) を実装する.
impl Div for ModInt998244353 {
    type Output = Self;

    /// # Panics
    /// Panics if the divisor `rhs` is zero.
    /// 除数 `rhs` がゼロの場合にパニックする.
    fn div(self, rhs: Self) -> Self::Output {
        // Calculate modular inverse for division.
        let inv_rhs = rhs
            .inv()
            .unwrap_or_else(|| panic!("Division by zero is not allowed for ModInt998244353"));
        self * inv_rhs
    }
}

/// Implements the division operation (`/`) for `ModInt998244353` and `u32`.
/// `ModInt998244353` と `u32` に対する除算演算 (`/`) を実装する.
impl Div<u32> for ModInt998244353 {
    type Output = Self;

    /// # Panics
    /// Panics if the divisor `rhs` is zero.
    /// 除数 `rhs` がゼロの場合にパニックする.
    fn div(self, rhs: u32) -> Self::Output {
        let rhs_mod = ModInt998244353::new(rhs as u64);
        let inv_rhs = rhs_mod
            .inv()
            .unwrap_or_else(|| panic!("Division by zero is not allowed for ModInt998244353"));
        self * inv_rhs
    }
}

/// Implements the division assignment operation (`/=`) for `ModInt998244353`.
/// `ModInt998244353` に対する除算代入演算 (`/=`) を実装する.
impl DivAssign for ModInt998244353 {
    /// # Panics
    /// Panics if the divisor `rhs` is zero.
    /// 除数 `rhs` がゼロの場合にパニックする.
    fn div_assign(&mut self, rhs: Self) {
        // Calculate modular inverse for division.
        let inv_rhs = rhs
            .inv()
            .unwrap_or_else(|| panic!("Division by zero is not allowed for ModInt998244353"));
        *self *= inv_rhs;
    }
}

/// Implements the division assignment operation (`/=`) for `ModInt998244353` and `u32`.
/// `ModInt998244353` と `u32` に対する除算代入演算 (`/=`) を実装する.
impl DivAssign<u32> for ModInt998244353 {
    /// # Panics
    /// Panics if the divisor `rhs` is zero.
    /// 除数 `rhs` がゼロの場合にパニックする.
    fn div_assign(&mut self, rhs: u32) {
        let rhs_mod = ModInt998244353::new(rhs as u64);
        let inv_rhs = rhs_mod
            .inv()
            .unwrap_or_else(|| panic!("Division by zero is not allowed for ModInt998244353"));
        *self *= inv_rhs;
    }
}

/// Implements the unary negation operation (`-`) for `ModInt998244353`.
/// `ModInt998244353` に対する単項否定演算 (`-`) を実装する.
impl Neg for ModInt998244353 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        if self.val == 0 {
            Self::new_raw(0)
        } else {
            Self::new_raw(MOD - self.val)
        }
    }
}
