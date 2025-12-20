pub mod add;
pub mod bostan_mori;
pub mod div;
pub mod exp;
pub mod inv;
pub mod log;
pub mod mul;
pub mod partition;
pub mod pow;
pub mod sub;

use anmitsu::modulo998244353::{convolution, fps};

#[test]
fn new_trims_trailing_zeros_and_set_extends() {
    // Arrange
    let coeffs = vec![1, 2, 0];

    // Act
    let mut fps = fps::FPS::new(coeffs);
    fps.set(4, 3);

    // Assert
    assert_eq!(1, fps.get(0));
    assert_eq!(2, fps.get(1));
    assert_eq!(3, fps.get(4));
    assert_eq!(5, fps.len());
}

#[test]
fn zero_series_is_always_stored_trimmed() {
    // Arrange
    let fps = fps::FPS::new(vec![0, 0]);

    // Act
    let shifted = fps.mul_xk(2);

    // Assert
    assert!(fps.is_zero());
    assert_eq!(0, fps.len());
    assert!(shifted.is_zero());
    assert_eq!(None, shifted.degree());
}

#[test]
fn derivative_and_integral_round_trip_for_zero_constant() {
    // Arrange
    let fps = fps::FPS::new(vec![0, 2, 3]);

    // Act
    let mut restored = fps.clone();
    restored.derivative();
    restored.integral();

    // Assert
    assert_eq!(fps, restored);
}

#[test]
fn setting_zero_to_high_indices_does_not_expand() {
    // Arrange
    let mut fps = fps::FPS::new(Vec::new());

    // Act
    for i in 0..16 {
        fps.set(i, 0);
    }

    // Assert
    assert!(fps.is_zero());
    assert_eq!(0, fps.len());
}

#[test]
fn error_is_raised_when_constraints_violate() {
    // Arrange
    let not_one = fps::FPS::new(vec![2, 1]);
    let not_zero = fps::FPS::new(vec![1]);

    // Act
    let log_result = not_one.log(3);
    let exp_result = not_zero.exp(2);
    let inverse_result = not_zero.inverse(0);

    // Assert
    assert!(log_result.is_none(), "log should reject constant != 1");
    assert!(exp_result.is_none(), "exp should reject constant != 0");
    assert!(
        inverse_result.is_some(),
        "inverse should allow degree 0 when constant is non-zero"
    );
}

#[test]
fn set_panics_when_exceeding_max_ntt_len() {
    // Arrange
    let index = convolution::MAX_NTT_LEN;

    // Act
    let result = std::panic::catch_unwind(|| {
        let mut fps = fps::FPS::new(vec![1]);
        fps.set(index, 1);
    });

    // Assert
    assert!(
        result.is_err(),
        "set should panic when exceeding MAX_NTT_LEN"
    );
}
