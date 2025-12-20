use anmitsu::modulo998244353;

#[test]
fn fact_and_inv_fact_with_small_values() {
    // Arrange
    let mut comb = modulo998244353::combinatorics::Combinatorics::new();

    // Act
    let fact_5 = comb.fact(5);
    let inv_fact_5 = comb.inv_fact(5);

    // Assert
    assert_eq!(120, fact_5.val());
    assert_eq!(1, (fact_5 * inv_fact_5).val());
}

#[test]
fn inv_fact_can_be_queried_sequentially_without_pow_per_call() {
    // Arrange
    let mut comb = modulo998244353::combinatorics::Combinatorics::new();

    // Act
    for n in 0..=2000 {
        let fact_n = comb.fact(n);
        let inv_fact_n = comb.inv_fact(n);

        // Assert
        assert_eq!(1, (fact_n * inv_fact_n).val());
    }
}

#[test]
fn inv_returns_none_for_zero_and_multiple_of_mod() {
    // Arrange
    let mut comb = modulo998244353::combinatorics::Combinatorics::new();

    // Act
    let inv_0 = comb.inv(0);
    let inv_mod = comb.inv(modulo998244353::modulo::M as usize);

    // Assert
    assert!(inv_0.is_none());
    assert!(inv_mod.is_none());
}

#[test]
fn comb_with_typical_values() {
    // Arrange
    let mut comb = modulo998244353::combinatorics::Combinatorics::new();

    // Act
    let c_5_2 = comb.comb(5, 2);
    let c_10_0 = comb.comb(10, 0);
    let c_10_10 = comb.comb(10, 10);
    let c_10_3 = comb.comb(10, 3);
    let c_10_7 = comb.comb(10, 7);

    // Assert
    assert_eq!(10, c_5_2.val());
    assert_eq!(1, c_10_0.val());
    assert_eq!(1, c_10_10.val());
    assert_eq!(120, c_10_3.val());
    assert_eq!(120, c_10_7.val());
}

#[test]
fn comb_with_large_n_and_small_k() {
    // Arrange
    let mut comb = modulo998244353::combinatorics::Combinatorics::new();
    let n = modulo998244353::modulo::M as u64 + 5;
    let k = 3_usize;

    // Act
    let actual = comb.comb(n, k);

    let mut num = modulo998244353::modint::ModInt998244353::new_raw(1);
    for i in 0..k {
        num *= modulo998244353::modint::ModInt998244353::new(n - i as u64);
    }
    let mut den = modulo998244353::modint::ModInt998244353::new_raw(1);
    for i in 1..=k {
        den *= i as u32;
    }
    let expected = num * den.inv().unwrap();

    // Assert
    assert_eq!(expected, actual);
}

#[test]
fn perm_with_typical_values() {
    // Arrange
    let mut comb = modulo998244353::combinatorics::Combinatorics::new();

    // Act
    let p_5_3 = comb.perm(5, 3);
    let p_5_0 = comb.perm(5, 0);
    let p_5_6 = comb.perm(5, 6);

    // Assert
    assert_eq!(60, p_5_3.val());
    assert_eq!(1, p_5_0.val());
    assert_eq!(0, p_5_6.val());
}

#[test]
fn tables_extend_by_block_and_keep_consistency() {
    // Arrange
    let mut comb = modulo998244353::combinatorics::Combinatorics::new();

    // Act
    let fact_1023 = comb.fact(1023);
    let inv_fact_1023 = comb.inv_fact(1023);
    let fact_1024 = comb.fact(1024);
    let inv_fact_1024 = comb.inv_fact(1024);
    let inv_1024 = comb.inv(1024).unwrap();

    // Assert
    assert_eq!(1, (fact_1023 * inv_fact_1023).val());
    assert_eq!(fact_1023 * 1024_u32, fact_1024);
    assert_eq!(1, (fact_1024 * inv_fact_1024).val());
    assert_eq!(1, (inv_1024 * 1024_u32).val());
}
