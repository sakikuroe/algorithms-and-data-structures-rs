use anmitsu::modulo998244353::fps;

#[test]
fn mul_and_div_preserve_coefficients() {
    // Arrange
    let fps = fps::FPS::new(vec![5, 6]);

    // Act
    let shifted_left = fps.mul_xk(3);
    let shifted_right = shifted_left.div_xk(2);

    // Assert
    assert_eq!(0, shifted_left.get(0));
    assert_eq!(5, shifted_left.get(3));
    assert_eq!(5, shifted_right.get(1));
    assert_eq!(6, shifted_right.get(2));
}
