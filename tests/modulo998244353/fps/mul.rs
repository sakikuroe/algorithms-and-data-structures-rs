use anmitsu::modulo998244353::fps;

#[test]
fn mul_and_mul_assign_work_correctly() {
    // Arrange
    let a = fps::FPS::new(vec![1, 1]);
    let b = fps::FPS::new(vec![1, 2]);

    // Act
    let product = a.clone() * b.clone();
    let mut product_assign = a;
    product_assign *= b;

    // Assert
    assert_eq!(1, product.get(0));
    assert_eq!(3, product.get(1));
    assert_eq!(2, product.get(2));
    assert_eq!(product, product_assign);
}

#[test]
fn product_with_empty_sequence_returns_one() {
    // Arrange
    let polynomials = Vec::new();

    // Act
    let product = fps::FPS::product(polynomials, 10);

    // Assert
    assert_eq!(1, product.get(0));
    assert_eq!(1, product.len());
}

#[test]
fn product_with_zero_polynomial_returns_zero() {
    // Arrange
    let polynomials = vec![fps::FPS::new(vec![1, 2]), fps::FPS::new(Vec::new())];

    // Act
    let product = fps::FPS::product(polynomials, 10);

    // Assert
    assert!(product.is_zero());
}

#[test]
fn product_with_three_polynomials_matches_sequential_multiplication() {
    // Arrange
    let polynomials = vec![
        fps::FPS::new(vec![1, 1]),
        fps::FPS::new(vec![1, 2]),
        fps::FPS::new(vec![3, 0, 4]),
    ];
    let expected = polynomials
        .iter()
        .cloned()
        .fold(fps::FPS::new(vec![1]), |acc, x| acc * x);

    // Act
    let actual = fps::FPS::product(polynomials, 100);

    // Assert
    assert_eq!(expected, actual);
}

#[test]
fn product_respects_degree_truncation() {
    // Arrange
    let polynomials = vec![fps::FPS::new(vec![1, 1]), fps::FPS::new(vec![1, 1])];

    // Act
    let actual_degree_0 = fps::FPS::product(polynomials.clone(), 0);
    let actual_degree_1 = fps::FPS::product(polynomials, 1);

    // Assert
    assert_eq!(fps::FPS::new(vec![1]), actual_degree_0);
    assert_eq!(fps::FPS::new(vec![1, 2]), actual_degree_1);
}
