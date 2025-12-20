use anmitsu::modulo998244353::modulo as modulo998244353;

#[test]
fn add_and_subtract_wrap_modulus() {
    // Arrange
    let small_a = 5;
    let small_b = 7;
    let large = modulo998244353::M - 1;

    // Act
    let sum = modulo998244353::add(small_a, small_b);
    let wrapped = modulo998244353::add(large, 2);
    let diff = modulo998244353::sub(1, 2);

    // Assert
    assert_eq!(12, sum);
    assert_eq!(1, wrapped);
    assert_eq!(modulo998244353::M - 1, diff);
}

#[test]
fn multiply_and_pow_match_expected_results() {
    // Arrange
    let base = 3;
    let exponent = 5usize;

    // Act
    let product = modulo998244353::mul(base, base);
    let power = modulo998244353::pow(base, exponent);

    // Assert
    assert_eq!(9, product);
    assert_eq!(243, power);
}

#[test]
fn inverse_multiplies_to_one() {
    // Arrange
    let value = 7;

    // Act
    let inv = modulo998244353::inv(value);
    let product = modulo998244353::mul(value, inv);

    // Assert
    assert_eq!(1, product);
}
