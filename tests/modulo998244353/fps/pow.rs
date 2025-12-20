use anmitsu::modulo998244353::fps;

#[test]
fn pow_sparse_matches_sample_problem() {
    // Arrange
    // f(x) = x + x^2, M = 3
    let f = fps::FPS::new(vec![0, 1, 1]);
    let degree = 4;

    // Act
    let result = f.pow_sparse(3, degree);

    // Assert
    assert_eq!(0, result.get(0));
    assert_eq!(0, result.get(1));
    assert_eq!(0, result.get(2));
    assert_eq!(1, result.get(3));
    assert_eq!(3, result.get(4));
}

#[test]
fn pow_matches_pow_sparse_for_sparse_series() {
    // Arrange
    let f = fps::FPS::new(vec![0, 1, 0, 2, 0, 0, 3]);
    let exponent = 5;
    let degree = 12;

    // Act
    let sparse = f.pow_sparse(exponent, degree);
    let auto = f.pow(exponent, degree);

    // Assert
    assert_eq!(sparse, auto);
}

#[test]
fn pow_sparse_handles_zero_exponent() {
    // Arrange
    let f = fps::FPS::new(vec![2, 3, 4]);
    let degree = 5;

    // Act
    let result = f.pow_sparse(0, degree);

    // Assert
    assert_eq!(1, result.get(0));
    for i in 1..=degree {
        assert_eq!(0, result.get(i), "Coefficient {} should be zero", i);
    }
}

#[test]
fn pow_dense_matches_pow_sparse_for_dense_series() {
    // Arrange
    let f = fps::FPS::new(vec![1, 2, 3, 4, 5, 6, 7, 8]);
    let exponent = 3;
    let degree = 16;

    // Act
    let dense = f.pow_dense(exponent, degree);
    let sparse = f.pow_sparse(exponent, degree);

    // Assert
    assert_eq!(sparse, dense);
}

#[test]
fn pow_dense_handles_zero_exponent() {
    // Arrange
    let f = fps::FPS::new(vec![2, 3, 4]);
    let degree = 5;

    // Act
    let result = f.pow_dense(0, degree);

    // Assert
    assert_eq!(1, result.get(0));
    for i in 1..=degree {
        assert_eq!(0, result.get(i), "Coefficient {} should be zero", i);
    }
}

#[test]
fn pow_handles_zero_exponent() {
    // Arrange
    let f = fps::FPS::new(vec![2, 3, 4]);
    let degree = 5;

    // Act
    let result = f.pow(0, degree);

    // Assert
    assert_eq!(1, result.get(0));
    for i in 1..=degree {
        assert_eq!(0, result.get(i), "Coefficient {} should be zero", i);
    }
}

#[test]
fn pow_sparse_handles_shift_and_scaling() {
    // Arrange
    // f(x) = 2x^2 + 3x^3
    let f = fps::FPS::new(vec![0, 0, 2, 3]);
    let degree = 8;

    // Act
    let result = f.pow_sparse(2, degree);

    // Assert
    // (2x^2 + 3x^3)^2 = 4x^4 + 12x^5 + 9x^6
    for i in 0..4 {
        assert_eq!(0, result.get(i), "Coefficient {} should be zero", i);
    }
    assert_eq!(4, result.get(4));
    assert_eq!(12 % 998244353, result.get(5));
    assert_eq!(9, result.get(6));
    for i in 7..=degree {
        assert_eq!(0, result.get(i), "Coefficient {} should be zero", i);
    }
}

#[test]
fn pow_sparse_returns_zero_when_shift_exceeds_degree() {
    // Arrange
    // f(x) = x^3, M = 2, N = 5 -> Mt = 6 >= 5 so all zero.
    let f = fps::FPS::new(vec![0, 0, 0, 1]);
    let degree = 4;

    // Act
    let result = f.pow_sparse(2, degree);

    // Assert
    for i in 0..=degree {
        assert_eq!(0, result.get(i), "Coefficient {} should be zero", i);
    }
}
