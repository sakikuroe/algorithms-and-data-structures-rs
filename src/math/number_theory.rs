/// Calculates the greatest common divisor (GCD) of two non-negative integers.
/// 2 つの非負整数の最大公約数（GCD）を計算する。
///
/// This function implements the Euclidean algorithm to find the largest positive
/// integer that divides both `a` and `b` without a remainder.
/// The GCD of 0 and any non-zero number `x` is `x`. The GCD of 0 and 0 is defined as 0.
/// この関数はユークリッドの互除法を実装し、`a` と `b` の両方を余りなく割り切る
/// 最大の正の整数を見つける。
/// 0 と任意の正の数 `x` の GCD は `x` である。また、0 と 0 の GCD は 0 と定義される。
///
/// # Args
///
/// a: The first non-negative integer.
///     最初の非負整数
/// b: The second non-negative integer.
///     ２つ目の非負整数
///
/// # Returns
///
/// The greatest common divisor of `a` and `b`.
/// Specifically, for non-negative integers `a` and `b`:
/// - If `a = 0` and `b = 0`, returns `0`.
/// - If `a = 0` and `b > 0`, returns `b`.
/// - If `a > 0` and `b = 0`, returns `a`.
/// - If `a > 0` and `b > 0`, returns the largest positive integer that divides both `a` and `b`.
/// `a` と `b` の最大公約数。
/// 具体的には、非負整数 `a` と `b` に対して:
/// - `a = 0` かつ `b = 0` の場合、`0` を返す。
/// - `a = 0` かつ `b > 0` の場合、`b` を返す。
/// - `a > 0` かつ `b = 0` の場合、`a` を返す。
/// - `a > 0` かつ `b > 0` の場合、`a` と `b` の両方を割り切る最大の正の整数を返す。
///
/// # Complexity
///
/// Time: O(log(min(a, b)))
/// 時間計算量: O(log(min(a, b)))
///
/// Space: O(1)
/// 空間計算量: O(1)
///
/// # Examples
///
/// ```
/// use anmitsu::math::number_theory;
///
/// assert_eq!(6, number_theory::gcd(12, 18));
/// assert_eq!(12, number_theory::gcd(12, 12));
/// assert_eq!(5, number_theory::gcd(0, 5));
/// assert_eq!(0, number_theory::gcd(0, 0));
/// ```
pub fn gcd(mut a: u128, mut b: u128) -> u128 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

/// Calculates the least common multiple (LCM) of two non-negative integers.
/// 2 つの非負整数の最小公倍数（LCM）を計算する。
///
/// This function calculates the LCM using the formula: LCM(a, b) = |a * b| / GCD(a, b).
/// It returns `None` if the multiplication `a * b` would overflow `u128`.
/// この関数は、LCM(a, b) = |a * b| / GCD(a, b) の公式を使用して LCM を計算する。
/// `a * b` の乗算が `u128` の範囲を超過する場合、`None` を返す。
///
/// # Args
///
/// a: The first non-negative integer.
///     最初の非負整数
/// b: The second non-negative integer.
///     2 つ目の非負整数
///
/// # Returns
///
/// The least common multiple of `a` and `b` wrapped in an `Option`, or `None` if
/// the calculation overflows.
/// `a` と `b` の最小公倍数を `Option` でラップして返す。計算がオーバーフローした場合は `None`。
///
/// # Complexity
///
/// Time: O(log(min(a, b)))
/// 時間計算量: O(log(min(a, b)))
///
/// Space: O(1)
/// 空間計算量: O(1)
///
/// # Examples
///
/// ```
/// use anmitsu::math::number_theory;
///
/// assert_eq!(Some(36), number_theory::lcm(12, 18));
/// assert_eq!(Some(12), number_theory::lcm(12, 12));
/// assert_eq!(Some(0), number_theory::lcm(0, 5));
/// assert_eq!(Some(0), number_theory::lcm(0, 0));
/// assert_eq!(Some(72), number_theory::lcm(8, 9));
/// assert_eq!(Some(100), number_theory::lcm(20, 25));
/// ```
pub fn lcm(a: u128, b: u128) -> Option<u128> {
    if a == 0 && b == 0 {
        return Some(0);
    }

    (a / gcd(a, b)).checked_mul(b)
}
