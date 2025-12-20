use anmitsu::modulo998244353::fps;

#[test]
fn log_and_exp_are_inverses_for_small_polynomials() {
    // Arrange
    let input = fps::FPS::new(vec![0, 1]);
    let degree = 6;

    // Act
    let exp = input.exp(degree).expect("exp should succeed");
    let log = exp.log(degree).expect("log should succeed");

    // Assert
    assert_eq!(0, log.get(0));
    assert_eq!(1, log.get(1));
    for i in 2..=degree {
        assert_eq!(0, log.get(i), "Coefficient {} should be zero", i);
    }
}

#[test]
fn log_sparse_matches_dense_log_for_sparse_series() {
    // Arrange
    let fps = fps::FPS::new(vec![1, 2, 0, 3, 0, 0, 4]);
    let degree = 6;

    // Act
    let log_dense = fps.log_dense(degree).expect("dense log should succeed");
    let log_sparse = fps.log_sparse(degree).expect("sparse log should succeed");

    // Assert
    assert_eq!(log_dense, log_sparse);
}
