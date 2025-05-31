use anmitsu::math::number_theory::gcd;

// 基本的なテスト
#[test]
fn test_gcd_basic() {
    assert_eq!(5, gcd(10, 5));
    assert_eq!(9, gcd(27, 18));
    assert_eq!(25, gcd(100, 75));
}

// 片方が 0 の場合、もう片方の数が最大公約数となる
#[test]
fn test_gcd_one_is_zero() {
    assert_eq!(5, gcd(0, 5));
    assert_eq!(10, gcd(10, 0));
}

// 両方が 0 の場合、定義により 0 となる
#[test]
fn test_gcd_both_are_zero() {
    assert_eq!(0, gcd(0, 0));
}

// 互いに素な数の場合、最大公約数は 1
#[test]
fn test_gcd_prime_numbers() {
    assert_eq!(1, gcd(7, 5));
    assert_eq!(1, gcd(13, 17));
}

// 片方がもう片方の倍数である場合
#[test]
fn test_gcd_one_is_multiple_of_other() {
    assert_eq!(2, gcd(10, 2));
    assert_eq!(5, gcd(5, 20));
}

// 大きい数でのテスト
#[test]
fn test_gcd_large_numbers() {
    // u128::MAX と 1 の最大公約数は 1
    assert_eq!(1, gcd(u128::MAX, 1));
    assert_eq!(1, gcd(1, u128::MAX));

    // 3 * 2^{125} と 5 * 2^{125}
    assert_eq!(1 << 125, gcd(3 * 1 << 125, 5 * 1 << 125));
}
