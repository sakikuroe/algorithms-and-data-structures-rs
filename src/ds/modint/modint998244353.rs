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

    /// Computes `self` raised to the power of `n`.
    /// `self` の `n` 乗を計算する.
    ///
    /// # Args
    /// - `n`: The non-negative exponent.
    ///        冪指数.
    ///
    /// # Returns
    /// `Self`: The result of `self` raised to the power of `n`.
    ///         `self` を `n` 乗した結果.
    ///
    /// # Complexity
    /// - Time complexity: O(log n), where n is the exponent.
    ///                          ここで n は冪指数である.
    /// - Space complexity: O(1).
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::ds::modint::modint998244353::ModInt998244353;
    /// let base = ModInt998244353::new(3);
    /// let result = base.pow(4);
    /// assert_eq!(81, result.val());
    /// ```
    pub fn pow(&self, mut n: usize) -> Self {
        let mut res = ModInt998244353::new_raw(1);
        let mut base = *self;

        // This is a standard binary exponentiation (exponentiation by squaring).
        while n > 0 {
            // If the current bit of n is 1, multiply the result by the base.
            if n % 2 == 1 {
                res *= base;
            }
            // Square the base for the next bit.
            base *= base;
            n /= 2;
        }
        res
    }
}

/// Allows conversion from `u32` to `ModInt998244353`.
/// `u32` から `ModInt998244353` への変換を可能にする.
impl From<u32> for ModInt998244353 {
    /// Creates a `ModInt998244353` instance from a `u32` value.
    /// `u32` の値から `ModInt998244353` インスタンスを生成する.
    ///
    /// # Args
    /// - `num`: The `u32` value to convert.
    ///          変換する `u32` の値.
    ///
    /// # Returns
    /// A new `ModInt998244353` instance equivalent to `num` modulo `998244353`.
    /// `num` を `998244353` で割った余りと等価な, 新しい `ModInt998244353` インスタンス.
    ///
    /// # Constraints
    /// There are no constraints on the input value.
    /// 入力値に関する制約はない.
    ///
    /// # Complexity
    /// - Time complexity: O(1).
    /// - Space complexity: O(1).
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::ds::modint::modint998244353::ModInt998244353;
    /// let val: u32 = 1_000_000_007;
    /// let m: ModInt998244353 = val.into();
    /// assert_eq!(1755654, m.val()); // 1_000_000_007 % 998244353
    /// ```
    fn from(num: u32) -> Self {
        ModInt998244353::new(num as u64)
    }
}

/// Allows conversion from `i32` to `ModInt998244353`.
/// `i32` から `ModInt998244353` への変換を可能にする.
impl From<i32> for ModInt998244353 {
    /// Creates a `ModInt998244353` instance from an `i32` value, handling negative numbers correctly.
    /// `i32` の値から `ModInt998244353` インスタンスを生成する. 負数も正しく扱う.
    ///
    /// # Args
    /// - `num`: The `i32` value to convert.
    ///          変換する `i32` の値.
    ///
    /// # Returns
    /// A new `ModInt998244353` instance. Negative inputs are converted to a positive equivalent in modular arithmetic.
    /// 新しい `ModInt998244353` インスタンス. 負の入力は, 法演算における正の等価値に変換される.
    ///
    /// # Constraints
    /// There are no constraints on the input value.
    /// 入力値に関する制約はない.
    ///
    /// # Complexity
    /// - Time complexity: O(1).
    /// - Space complexity: O(1).
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::ds::modint::modint998244353::ModInt998244353;
    /// let m_pos: ModInt998244353 = 10i32.into();
    /// assert_eq!(10, m_pos.val());
    ///
    /// let m_neg: ModInt998244353 = (-10i32).into();
    /// assert_eq!(998244353 - 10, m_neg.val());
    /// ```
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

    /// Adds two `ModInt998244353` instances.
    /// 2つの `ModInt998244353` インスタンスを加算する.
    ///
    /// # Args
    /// - `self`: The left-hand side operand.
    ///           左辺のオペランド.
    /// - `rhs`: The right-hand side operand.
    ///          右辺のオペランド.
    ///
    /// # Returns
    /// A new `ModInt998244353` instance representing the sum.
    /// 和を表す新しい `ModInt998244353` インスタンス.
    ///
    /// # Constraints
    /// There are no constraints on the input values.
    /// 入力値に関する制約はない.
    ///
    /// # Complexity
    /// - Time complexity: O(1).
    /// - Space complexity: O(1).
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::ds::modint::modint998244353::ModInt998244353;
    /// let a = ModInt998244353::new(998244350);
    /// let b = ModInt998244353::new(10);
    /// assert_eq!(ModInt998244353::new(7), a + b);
    /// ```
    fn add(mut self, rhs: Self) -> Self::Output {
        self += rhs;
        self
    }
}

/// Implements the addition operation (`+`) for `ModInt998244353` and `u32`.
/// `ModInt998244353` と `u32` に対する加算演算 (`+`) を実装する.
impl Add<u32> for ModInt998244353 {
    type Output = Self;

    /// Adds a `u32` value to a `ModInt998244353` instance.
    /// `ModInt998244353` インスタンスに `u32` の値を加算する.
    ///
    /// # Args
    /// - `self`: The `ModInt998244353` instance.
    ///           `ModInt998244353` インスタンス.
    /// - `rhs`: The `u32` value to add.
    ///          加算する `u32` の値.
    ///
    /// # Returns
    /// A new `ModInt998244353` instance representing the sum.
    /// 和を表す新しい `ModInt998244353` インスタンス.
    ///
    /// # Constraints
    /// There are no constraints on the input values.
    /// 入力値に関する制約はない.
    ///
    /// # Complexity
    /// - Time complexity: O(1).
    /// - Space complexity: O(1).
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::ds::modint::modint998244353::ModInt998244353;
    /// let a = ModInt998244353::new(998244350);
    /// let b: u32 = 10;
    /// assert_eq!(ModInt998244353::new(7), a + b);
    /// ```
    fn add(mut self, rhs: u32) -> Self::Output {
        self += rhs;
        self
    }
}

/// Implements the addition assignment operation (`+=`) for `ModInt998244353`.
/// `ModInt998244353` に対する加算代入演算 (`+=`) を実装する.
impl AddAssign for ModInt998244353 {
    /// Adds another `ModInt998244353` instance to `self`.
    /// 別の `ModInt998244353` インスタンスを `self` に加算する.
    ///
    /// # Args
    /// - `self`: The `ModInt998244353` instance to be modified.
    ///           変更される `ModInt998244353` インスタンス.
    /// - `rhs`: The right-hand side operand.
    ///          右辺のオペランド.
    ///
    /// # Constraints
    /// There are no constraints on the input values.
    /// 入力値に関する制約はない.
    ///
    /// # Complexity
    /// - Time complexity: O(1).
    /// - Space complexity: O(1).
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::ds::modint::modint998244353::ModInt998244353;
    /// let mut a = ModInt998244353::new(998244350);
    /// let b = ModInt998244353::new(10);
    /// a += b;
    /// assert_eq!(ModInt998244353::new(7), a);
    /// ```
    fn add_assign(&mut self, rhs: Self) {
        self.val += rhs.val;
        if self.val >= MOD {
            self.val -= MOD;
        }
    }
}

/// Implements the addition assignment operation (`+=`) for `ModInt998244353` and `u32`.
/// `ModInt998244353` と `u32` に対する加算代入演算 (`+=`) を実装する.
impl AddAssign<u32> for ModInt998244353 {
    /// Adds a `u32` value to `self`.
    /// `u32` の値を `self` に加算する.
    ///
    /// # Args
    /// - `self`: The `ModInt998244353` instance to be modified.
    ///           変更される `ModInt998244353` インスタンス.
    /// - `rhs`: The `u32` value to add.
    ///          加算する `u32` の値.
    ///
    /// # Constraints
    /// There are no constraints on the input values.
    /// 入力値に関する制約はない.
    ///
    /// # Complexity
    /// - Time complexity: O(1).
    /// - Space complexity: O(1).
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::ds::modint::modint998244353::ModInt998244353;
    /// let mut a = ModInt998244353::new(998244350);
    /// a += 10u32;
    /// assert_eq!(ModInt998244353::new(7), a);
    /// ```
    fn add_assign(&mut self, rhs: u32) {
        let rhs_mod = rhs % MOD;
        self.val += rhs_mod;
        if self.val >= MOD {
            self.val -= MOD;
        }
    }
}

/// Implements the subtraction operation (`-`) for two `ModInt998244353` instances.
/// 2つの `ModInt998244353` インスタンスに対する減算演算 (`-`) を実装する.
impl Sub for ModInt998244353 {
    type Output = Self;

    /// Subtracts one `ModInt998244353` instance from another.
    /// ある `ModInt998244353` インスタンスから別のインスタンスを減算する.
    ///
    /// # Args
    /// - `self`: The left-hand side operand (minuend).
    ///           左辺のオペランド (被減数).
    /// - `rhs`: The right-hand side operand (subtrahend).
    ///          右辺のオペランド (減数).
    ///
    /// # Returns
    /// A new `ModInt998244353` instance representing the difference.
    /// 差を表す新しい `ModInt998244353` インスタンス.
    ///
    /// # Constraints
    /// There are no constraints on the input values.
    /// 入力値に関する制約はない.
    ///
    /// # Complexity
    /// - Time complexity: O(1).
    /// - Space complexity: O(1).
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::ds::modint::modint998244353::ModInt998244353;
    /// let a = ModInt998244353::new(10);
    /// let b = ModInt998244353::new(20);
    /// assert_eq!(ModInt998244353::new(998244343), a - b);
    /// ```
    fn sub(mut self, rhs: Self) -> Self::Output {
        self -= rhs;
        self
    }
}

/// Implements the subtraction operation (`-`) for `ModInt998244353` and `u32`.
/// `ModInt998244353` と `u32` に対する減算演算 (`-`) を実装する.
impl Sub<u32> for ModInt998244353 {
    type Output = Self;

    /// Subtracts a `u32` value from a `ModInt998244353` instance.
    /// `ModInt998244353` インスタンスから `u32` の値を減算する.
    ///
    /// # Args
    /// - `self`: The `ModInt998244353` instance.
    ///           `ModInt998244353` インスタンス.
    /// - `rhs`: The `u32` value to subtract.
    ///          減算する `u32` の値.
    ///
    /// # Returns
    /// A new `ModInt998244353` instance representing the difference.
    /// 差を表す新しい `ModInt998244353` インスタンス.
    ///
    /// # Constraints
    /// There are no constraints on the input values.
    /// 入力値に関する制約はない.
    ///
    /// # Complexity
    /// - Time complexity: O(1).
    /// - Space complexity: O(1).
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::ds::modint::modint998244353::ModInt998244353;
    /// let a = ModInt998244353::new(10);
    /// let b: u32 = 20;
    /// assert_eq!(ModInt998244353::new(998244343), a - b);
    /// ```
    fn sub(mut self, rhs: u32) -> Self::Output {
        self -= rhs;
        self
    }
}

/// Implements the subtraction assignment operation (`-=`) for `ModInt998244353`.
/// `ModInt998244353` に対する減算代入演算 (`-=`) を実装する.
impl SubAssign for ModInt998244353 {
    /// Subtracts another `ModInt998244353` instance from `self`.
    /// 別の `ModInt998244353` インスタンスを `self` から減算する.
    ///
    /// # Args
    /// - `self`: The `ModInt998244353` instance to be modified.
    ///           変更される `ModInt998244353` インスタンス.
    /// - `rhs`: The right-hand side operand.
    ///          右辺のオペランド.
    ///
    /// # Constraints
    /// There are no constraints on the input values.
    /// 入力値に関する制約はない.
    ///
    /// # Complexity
    /// - Time complexity: O(1).
    /// - Space complexity: O(1).
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::ds::modint::modint998244353::ModInt998244353;
    /// let mut a = ModInt998244353::new(10);
    /// let b = ModInt998244353::new(20);
    /// a -= b;
    /// assert_eq!(ModInt998244353::new(998244343), a);
    /// ```
    fn sub_assign(&mut self, rhs: Self) {
        if self.val >= rhs.val {
            self.val -= rhs.val;
        } else {
            self.val += MOD - rhs.val;
        }
    }
}

/// Implements the subtraction assignment operation (`-=`) for `ModInt998244353` and `u32`.
/// `ModInt998244353` と `u32` に対する減算代入演算 (`-=`) を実装する.
impl SubAssign<u32> for ModInt998244353 {
    /// Subtracts a `u32` value from `self`.
    /// `u32` の値を `self` から減算する.
    ///
    /// # Args
    /// - `self`: The `ModInt998244353` instance to be modified.
    ///           変更される `ModInt998244353` インスタンス.
    /// - `rhs`: The `u32` value to subtract.
    ///          減算する `u32` の値.
    ///
    /// # Constraints
    /// There are no constraints on the input values.
    /// 入力値に関する制約はない.
    ///
    /// # Complexity
    /// - Time complexity: O(1).
    /// - Space complexity: O(1).
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::ds::modint::modint998244353::ModInt998244353;
    /// let mut a = ModInt998244353::new(10);
    /// a -= 20u32;
    /// assert_eq!(ModInt998244353::new(998244343), a);
    /// ```
    fn sub_assign(&mut self, rhs: u32) {
        let rhs_mod = rhs % MOD;
        if self.val >= rhs_mod {
            self.val -= rhs_mod;
        } else {
            self.val += MOD - rhs_mod;
        }
    }
}

/// Implements the multiplication operation (`*`) for two `ModInt998244353` instances.
/// 2つの `ModInt998244353` インスタンスに対する乗算演算 (`*`) を実装する.
impl Mul for ModInt998244353 {
    type Output = Self;

    /// Multiplies two `ModInt998244353` instances.
    /// 2つの `ModInt998244353` インスタンスを乗算する.
    ///
    /// # Args
    /// - `self`: The left-hand side operand.
    ///           左辺のオペランド.
    /// - `rhs`: The right-hand side operand.
    ///          右辺のオペランド.
    ///
    /// # Returns
    /// A new `ModInt998244353` instance representing the product.
    /// 積を表す新しい `ModInt998244353` インスタンス.
    ///
    /// # Constraints
    /// There are no constraints on the input values.
    /// 入力値に関する制約はない.
    ///
    /// # Complexity
    /// - Time complexity: O(1).
    /// - Space complexity: O(1).
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::ds::modint::modint998244353::ModInt998244353;
    /// let a = ModInt998244353::new(100_000);
    /// let b = ModInt998244353::new(100_000);
    /// assert_eq!(ModInt998244353::new(17556470), a * b);
    /// ```
    fn mul(self, rhs: Self) -> Self::Output {
        ModInt998244353::new((self.val as u64) * (rhs.val as u64))
    }
}

/// Implements the multiplication operation (`*`) for `ModInt998244353` and `u32`.
/// `ModInt998244353` と `u32` に対する乗算演算 (`*`) を実装する.
impl Mul<u32> for ModInt998244353 {
    type Output = Self;

    /// Multiplies a `ModInt998244353` instance by a `u32` value.
    /// `ModInt998244353` インスタンスに `u32` の値を乗算する.
    ///
    /// # Args
    /// - `self`: The `ModInt998244353` instance.
    ///           `ModInt998244353` インスタンス.
    /// - `rhs`: The `u32` value to multiply by.
    ///          乗算する `u32` の値.
    ///
    /// # Returns
    /// A new `ModInt998244353` instance representing the product.
    /// 積を表す新しい `ModInt998244353` インスタンス.
    ///
    /// # Constraints
    /// There are no constraints on the input values.
    /// 入力値に関する制約はない.
    ///
    /// # Complexity
    /// - Time complexity: O(1).
    /// - Space complexity: O(1).
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::ds::modint::modint998244353::ModInt998244353;
    /// let a = ModInt998244353::new(100_000);
    /// let b: u32 = 100_000;
    /// assert_eq!(ModInt998244353::new(17556470), a * b);
    /// ```
    fn mul(self, rhs: u32) -> Self::Output {
        ModInt998244353::new((self.val as u64) * (rhs as u64))
    }
}

/// Implements the multiplication assignment operation (`*=`) for `ModInt998244353`.
/// `ModInt998244353` に対する乗算代入演算 (`*=`) を実装する.
impl MulAssign for ModInt998244353 {
    /// Multiplies `self` by another `ModInt998244353` instance.
    /// `self` に別の `ModInt998244353` インスタンスを乗算する.
    ///
    /// # Args
    /// - `self`: The `ModInt998244353` instance to be modified.
    ///           変更される `ModInt998244353` インスタンス.
    /// - `rhs`: The right-hand side operand.
    ///          右辺のオペランド.
    ///
    /// # Constraints
    /// There are no constraints on the input values.
    /// 入力値に関する制約はない.
    ///
    /// # Complexity
    /// - Time complexity: O(1).
    /// - Space complexity: O(1).
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::ds::modint::modint998244353::ModInt998244353;
    /// let mut a = ModInt998244353::new(100_000);
    /// let b = ModInt998244353::new(100_000);
    /// a *= b;
    /// assert_eq!(ModInt998244353::new(17556470), a);
    /// ```
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

/// Implements the multiplication assignment operation (`*=`) for `ModInt998244353` and `u32`.
/// `ModInt998244353` と `u32` に対する乗算代入演算 (`*=`) を実装する.
impl MulAssign<u32> for ModInt998244353 {
    /// Multiplies `self` by a `u32` value.
    /// `self` に `u32` の値を乗算する.
    ///
    /// # Args
    /// - `self`: The `ModInt998244353` instance to be modified.
    ///           変更される `ModInt998244353` インスタンス.
    /// - `rhs`: The `u32` value to multiply by.
    ///          乗算する `u32` の値.
    ///
    /// # Constraints
    /// There are no constraints on the input values.
    /// 入力値に関する制約はない.
    ///
    /// # Complexity
    /// - Time complexity: O(1).
    /// - Space complexity: O(1).
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::ds::modint::modint998244353::ModInt998244353;
    /// let mut a = ModInt998244353::new(100_000);
    /// a *= 100_000u32;
    /// assert_eq!(ModInt998244353::new(17556470), a);
    /// ```
    fn mul_assign(&mut self, rhs: u32) {
        *self = *self * rhs;
    }
}

/// Implements the division operation (`/`) for two `ModInt998244353` instances.
/// 2つの `ModInt998244353` インスタンスに対する除算演算 (`/`) を実装する.
impl Div for ModInt998244353 {
    type Output = Self;

    /// Divides `self` by another `ModInt998244353` instance using modular inverse.
    /// モジュラ逆数を用いて, `self` を別の `ModInt998244353` インスタンスで除算する.
    ///
    /// # Args
    /// - `self`: The dividend.
    ///           被除数.
    /// - `rhs`: The divisor.
    ///          除数.
    ///
    /// # Returns
    /// A new `ModInt998244353` instance representing the quotient.
    /// 商を表す新しい `ModInt998244353` インスタンス.
    ///
    /// # Constraints
    /// There are no constraints on the input values, but the divisor must not be zero.
    /// 入力値に関する制約はないが, 除数がゼロであってはならない.
    ///
    /// # Panics
    /// Panics if the divisor `rhs` is zero.
    /// 除数 `rhs` がゼロの場合にパニックする.
    ///
    /// # Complexity
    /// - Time complexity: O(log M), where M is the modulus.
    ///                          ここで M は法である.
    /// - Space complexity: O(1).
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::ds::modint::modint998244353::ModInt998244353;
    /// let a = ModInt998244353::new(20);
    /// let b = ModInt998244353::new(4);
    /// assert_eq!(ModInt998244353::new(5), a / b);
    /// ```
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

    /// Divides a `ModInt998244353` instance by a `u32` value using modular inverse.
    /// モジュラ逆数を用いて, `ModInt998244353` インスタンスを `u32` の値で除算する.
    ///
    /// # Args
    /// - `self`: The dividend.
    ///           被除数.
    /// - `rhs`: The `u32` divisor.
    ///          `u32` の除数.
    ///
    /// # Returns
    /// A new `ModInt998244353` instance representing the quotient.
    /// 商を表す新しい `ModInt998244353` インスタンス.
    ///
    /// # Constraints
    /// The divisor must not be zero.
    /// 除数がゼロであってはならない.
    ///
    /// # Panics
    /// Panics if the divisor `rhs` is zero.
    /// 除数 `rhs` がゼロの場合にパニックする.
    ///
    /// # Complexity
    /// - Time complexity: O(log M), where M is the modulus.
    ///                          ここで M は法である.
    /// - Space complexity: O(1).
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::ds::modint::modint998244353::ModInt998244353;
    /// let a = ModInt998244353::new(20);
    /// let b: u32 = 4;
    /// assert_eq!(ModInt998244353::new(5), a / b);
    /// ```
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
    /// Divides `self` by another `ModInt998244353` instance.
    /// `self` を別の `ModInt998244353` インスタンスで除算する.
    ///
    /// # Args
    /// - `self`: The `ModInt998244353` instance to be modified.
    ///           変更される `ModInt998244353` インスタンス.
    /// - `rhs`: The divisor.
    ///          除数.
    ///
    /// # Constraints
    /// The divisor must not be zero.
    /// 除数がゼロであってはならない.
    ///
    /// # Panics
    /// Panics if the divisor `rhs` is zero.
    /// 除数 `rhs` がゼロの場合にパニックする.
    ///
    /// # Complexity
    /// - Time complexity: O(log M), where M is the modulus.
    ///                          ここで M は法である.
    /// - Space complexity: O(1).
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::ds::modint::modint998244353::ModInt998244353;
    /// let mut a = ModInt998244353::new(20);
    /// let b = ModInt998244353::new(4);
    /// a /= b;
    /// assert_eq!(ModInt998244353::new(5), a);
    /// ```
    fn div_assign(&mut self, rhs: Self) {
        *self = *self / rhs;
    }
}

/// Implements the division assignment operation (`/=`) for `ModInt998244353` and `u32`.
/// `ModInt998244353` と `u32` に対する除算代入演算 (`/=`) を実装する.
impl DivAssign<u32> for ModInt998244353 {
    /// Divides `self` by a `u32` value.
    /// `self` を `u32` の値で除算する.
    ///
    /// # Args
    /// - `self`: The `ModInt998244353` instance to be modified.
    ///           変更される `ModInt998244353` インスタンス.
    /// - `rhs`: The `u32` divisor.
    ///          `u32` の除数.
    ///
    /// # Constraints
    /// The divisor must not be zero.
    /// 除数がゼロであってはならない.
    ///
    /// # Panics
    /// Panics if the divisor `rhs` is zero.
    /// 除数 `rhs` がゼロの場合にパニックする.
    ///
    /// # Complexity
    /// - Time complexity: O(log M), where M is the modulus.
    ///                          ここで M は法である.
    /// - Space complexity: O(1).
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::ds::modint::modint998244353::ModInt998244353;
    /// let mut a = ModInt998244353::new(20);
    /// a /= 4u32;
    /// assert_eq!(ModInt998244353::new(5), a);
    /// ```
    fn div_assign(&mut self, rhs: u32) {
        *self = *self / rhs;
    }
}

/// Implements the unary negation operation (`-`) for `ModInt998244353`.
/// `ModInt998244353` に対する単項否定演算 (`-`) を実装する.
impl Neg for ModInt998244353 {
    type Output = Self;

    /// Computes the unary negation of a `ModInt998244353` instance.
    /// `ModInt998244353` インスタンスの単項否定 (符号反転) を計算する.
    ///
    /// # Args
    /// - `self`: The value to negate.
    ///           符号反転する値.
    ///
    /// # Returns
    /// A new `ModInt998244353` instance representing the negated value.
    /// 符号反転された値を表す新しい `ModInt998244353` インスタンス.
    ///
    /// # Constraints
    /// There are no constraints on the input value.
    /// 入力値に関する制約はない.
    ///
    /// # Complexity
    /// - Time complexity: O(1).
    /// - Space complexity: O(1).
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::ds::modint::modint998244353::ModInt998244353;
    /// let a = ModInt998244353::new(10);
    /// assert_eq!(ModInt998244353::new(998244343), -a);
    ///
    /// let zero = ModInt998244353::new(0);
    /// assert_eq!(zero, -zero);
    /// ```
    fn neg(self) -> Self::Output {
        if self.val == 0 {
            Self::new_raw(0)
        } else {
            Self::new_raw(MOD - self.val)
        }
    }
}
