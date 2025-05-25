/// Calculates the greatest common divisor (GCD) of two non-negative integers.
/// 2 つの非負整数の最大公約数（GCD）を計算する。
///
/// This function implements the Euclidean algorithm to find the largest positive
/// integer that divides both `a` and `b` without a remainder.
/// この関数はユークリッドの互除法を実装し、`a` と `b` の両方を余りなく割り切る
/// 最大の正の整数を見つける。
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
/// `a` と `b` の最大公約数。
///
/// # Complexity
///
/// Time: O(log(min(a, b)))
/// 時間計算量: O(log(min(a, b)))
///
/// Space: O(1)
/// 空間計算量: O(1)
pub fn gcd(mut a: u128, mut b: u128) -> u128 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

pub fn lcm(a: u128, b: u128) -> Option<u128> {
    if a == 0 && b == 0 {
        return Some(0);
    }

    (a / gcd(a, b)).checked_mul(b)
}
