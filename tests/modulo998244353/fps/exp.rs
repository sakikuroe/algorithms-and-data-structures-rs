use anmitsu::modulo998244353::fps;

#[test]
fn exp_sparse_matches_dense_exp_for_sparse_series() {
    // Arrange
    let fps = fps::FPS::new(vec![0, 1, 0, 2, 0, 0, 3]);
    let degree = 6;

    // Act
    let exp_dense = fps.exp_dense(degree).expect("dense exp should succeed");
    let exp_sparse = fps.exp_sparse(degree).expect("sparse exp should succeed");

    // Assert
    assert_eq!(exp_dense, exp_sparse);
}

#[test]
fn log_dense_of_exp_dense_recovers_input_for_dense_series() {
    // Arrange
    let h = fps::FPS::new(vec![0, 1, 2, 3, 4, 5, 6]);
    let degree = 32;

    // Act
    let exp = h.exp_dense(degree).expect("exp_dense should succeed");
    let log = exp.log_dense(degree).expect("log_dense should succeed");

    // Assert
    for i in 0..=degree {
        assert_eq!(h.get(i), log.get(i), "Coefficient {} should match", i);
    }
}
