use anmitsu::modulo998244353::fps;
use anmitsu::modulo998244353::fps::bostan_mori;
use anmitsu::modulo998244353::modulo;

#[test]
fn bostan_mori_extracts_geometric_series_coefficients() {
    // Arrange
    let p = fps::FPS::new(vec![1]);
    let q = fps::FPS::new(vec![1, modulo::neg(1)]);

    // Act
    let a0 = bostan_mori::bostan_mori(&p, &q, 0);
    let a1 = bostan_mori::bostan_mori(&p, &q, 1);
    let a63 = bostan_mori::bostan_mori(&p, &q, 63);

    // Assert
    assert_eq!(1, a0);
    assert_eq!(1, a1);
    assert_eq!(1, a63);
}

#[test]
fn linear_recurrence_kth_term_matches_sample() {
    // Arrange
    // d = 2, k = 5
    // a0 = 1, a1 = 1
    // c1 = 1, c2 = 1
    let initial = vec![1, 1];
    let coeffs = vec![1, 1];

    // Act
    let actual = bostan_mori::linear_recurrence_kth_term(&initial, &coeffs, 5);

    // Assert
    assert_eq!(8, actual);
}

#[test]
fn linear_recurrence_kth_term_returns_initial_when_k_is_small() {
    // Arrange
    let initial = vec![7, 11, 13];
    let coeffs = vec![2, 3, 5];

    // Act
    let a0 = bostan_mori::linear_recurrence_kth_term(&initial, &coeffs, 0);
    let a2 = bostan_mori::linear_recurrence_kth_term(&initial, &coeffs, 2);

    // Assert
    assert_eq!(7, a0);
    assert_eq!(13, a2);
}

#[test]
fn linear_recurrence_kth_term_matches_naive_for_small_cases() {
    // Arrange
    let degree = 5;
    let initial = vec![1, 2, 3, 4, 5];
    let coeffs = vec![6, 7, 8, 9, 10];
    let k = 40_usize;

    let mut naive = initial.clone();
    for i in degree..=k {
        let mut acc = 0_u32;
        for j in 1..=degree {
            let term = naive[i - j];
            let coef = coeffs[j - 1];
            acc = modulo::add(acc, modulo::mul(coef, term));
        }
        naive.push(acc);
    }

    // Act
    let actual = bostan_mori::linear_recurrence_kth_term(&initial, &coeffs, k);

    // Assert
    assert_eq!(naive[k], actual);
}
