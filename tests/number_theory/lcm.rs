use anmitsu::math::number_theory::lcm;

// 基本的なテスト
#[test]
fn test_lcm_basic() {
    assert_eq!(Some(36), lcm(12, 18));
    assert_eq!(Some(300), lcm(100, 75));
}

// いずれかの引数が0の場合
#[test]
fn test_lcm_one_zero() {
    assert_eq!(Some(0), lcm(0, 5));
    assert_eq!(Some(0), lcm(5, 0));
}

// 両方の引数が0の場合
#[test]
fn test_lcm_both_zero() {
    assert_eq!(Some(0), lcm(0, 0));
}

// 互いに素な数の場合
#[test]
fn test_lcm_coprime() {
    assert_eq!(Some(15), lcm(3, 5));
    assert_eq!(Some(77), lcm(7, 11));
}

// 一方がもう一方の倍数の場合
#[test]
fn test_lcm_multiple() {
    assert_eq!(Some(10), lcm(5, 10));
    assert_eq!(Some(8), lcm(8, 2));
    assert_eq!(Some(10), lcm(10, 1));
}

// 同じ数の場合
#[test]
fn test_lcm_same_numbers() {
    assert_eq!(Some(1), lcm(1, 1));
    assert_eq!(Some(7), lcm(7, 7));
    assert_eq!(Some(100), lcm(100, 100));
}

// 大きい数の場合
#[test]
fn test_large_numbers() {
    {
        // 2^{64} 未満の最大の整数
        let p = 18446744073709551557_u128;
        assert_eq!(Some((p - 1) * p), lcm(p - 1, p));
    }

    {
        // 2^{128} 未満の最大の整数
        let p = 340282366920938463463374607431768211297_u128;
        assert_eq!(Some(p), lcm(p, p));
    }
}

// オーバーフローする場合
#[test]
fn test_overflow() {
    // 2^{128} 未満の最大の整数
    let p = 340282366920938463463374607431768211297_u128;
    assert_eq!(None, lcm(p - 1, p));
}
