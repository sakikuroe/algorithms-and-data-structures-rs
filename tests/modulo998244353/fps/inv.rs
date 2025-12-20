use anmitsu::modulo998244353::fps;

#[test]
fn inverse_multiplies_to_one_up_to_requested_degree() {
    // Arrange
    let fps = fps::FPS::new(vec![3, 4, 5]);
    let degree = 5;

    // Act
    let inverse = fps
        .inverse(degree)
        .expect("inverse should exist for non-zero constant");
    let product = fps.clone() * inverse;

    // Assert
    assert_eq!(1, product.get(0));
    for i in 1..=degree {
        assert_eq!(0, product.get(i), "Coefficient {} should be zero", i);
    }
}

#[test]
fn inverse_sparse_multiplies_to_one_up_to_requested_degree() {
    // Arrange
    let fps = fps::FPS::new(vec![3, 0, 5, 0, 7]);
    let degree = 6;

    // Act
    let inverse_sparse = fps
        .inverse_sparse(degree)
        .expect("inverse_sparse should exist for non-zero constant");
    let product = fps.clone() * inverse_sparse;

    // Assert
    assert_eq!(1, product.get(0));
    for i in 1..=degree {
        assert_eq!(0, product.get(i), "Coefficient {} should be zero", i);
    }
}
