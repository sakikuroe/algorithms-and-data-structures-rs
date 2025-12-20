use anmitsu::modulo998244353::fps;
use anmitsu::modulo998244353::fps::partition;
use anmitsu::modulo998244353::modulo;

fn mul_and_truncate(mut acc: fps::FPS, rhs: fps::FPS, degree: usize) -> fps::FPS {
    acc *= rhs;
    acc.truncate(degree + 1);
    acc
}

fn brute_product_one_plus(exponents: &[u32], degree: usize) -> fps::FPS {
    let mut acc = fps::FPS::new(vec![1]);
    for &a in exponents {
        let a = a as usize;
        let factor = if a == 0 {
            fps::FPS::new(vec![2])
        } else if a <= degree {
            let mut coeffs = vec![0; a + 1];
            coeffs[0] = 1;
            coeffs[a] = 1;
            fps::FPS::new(coeffs)
        } else {
            fps::FPS::new(vec![1])
        };
        acc = mul_and_truncate(acc, factor, degree);
    }
    acc
}

fn brute_product_one_minus(exponents: &[u32], degree: usize) -> fps::FPS {
    let mut acc = fps::FPS::new(vec![1]);
    for &a in exponents {
        let a = a as usize;
        let factor = if a == 0 {
            fps::FPS::new(Vec::new())
        } else if a <= degree {
            let mut coeffs = vec![0; a + 1];
            coeffs[0] = 1;
            coeffs[a] = modulo::sub(0, 1);
            fps::FPS::new(coeffs)
        } else {
            fps::FPS::new(vec![1])
        };
        acc = mul_and_truncate(acc, factor, degree);
    }
    acc
}

fn brute_product_inv_one_plus(exponents: &[u32], degree: usize) -> Option<fps::FPS> {
    let mut acc = fps::FPS::new(vec![1]);
    for &a in exponents {
        let a = a as usize;
        let factor = if a == 0 {
            fps::FPS::new(vec![modulo::inv(2)])
        } else {
            let mut coeffs = vec![0; degree + 1];
            let mut t = 0usize;
            while a * t <= degree {
                coeffs[a * t] = if t % 2 == 0 { 1 } else { modulo::sub(0, 1) };
                t += 1;
            }
            fps::FPS::new(coeffs)
        };
        acc = mul_and_truncate(acc, factor, degree);
    }
    Some(acc)
}

fn brute_product_inv_one_minus(exponents: &[u32], degree: usize) -> Option<fps::FPS> {
    let mut acc = fps::FPS::new(vec![1]);
    for &a in exponents {
        let a = a as usize;
        if a == 0 {
            return None;
        }
        let mut coeffs = vec![0; degree + 1];
        let mut t = 0usize;
        while a * t <= degree {
            coeffs[a * t] = 1;
            t += 1;
        }
        let factor = fps::FPS::new(coeffs);
        acc = mul_and_truncate(acc, factor, degree);
    }
    Some(acc)
}

#[test]
fn product_one_plus_x_powers_matches_bruteforce() {
    // Arrange
    let exponents = vec![1_u32, 2, 2, 5];
    let degree = 10;
    let expected = brute_product_one_plus(&exponents, degree);

    // Act
    let actual = partition::product_one_plus_x_powers(&exponents, degree).unwrap();

    // Assert
    assert_eq!(expected, actual);
}

#[test]
fn product_one_minus_x_powers_matches_bruteforce() {
    // Arrange
    let exponents = vec![1_u32, 2, 2, 5];
    let degree = 10;
    let expected = brute_product_one_minus(&exponents, degree);

    // Act
    let actual = partition::product_one_minus_x_powers(&exponents, degree).unwrap();

    // Assert
    assert_eq!(expected, actual);
}

#[test]
fn product_inv_one_plus_x_powers_matches_bruteforce() {
    // Arrange
    let exponents = vec![1_u32, 2, 2, 5];
    let degree = 10;
    let expected = brute_product_inv_one_plus(&exponents, degree).unwrap();

    // Act
    let actual = partition::product_inv_one_plus_x_powers(&exponents, degree).unwrap();

    // Assert
    assert_eq!(expected, actual);
}

#[test]
fn product_inv_one_minus_x_powers_matches_bruteforce() {
    // Arrange
    let exponents = vec![1_u32, 2, 2, 5];
    let degree = 10;
    let expected = brute_product_inv_one_minus(&exponents, degree).unwrap();

    // Act
    let actual = partition::product_inv_one_minus_x_powers(&exponents, degree).unwrap();

    // Assert
    assert_eq!(expected, actual);
}

#[test]
fn product_functions_handle_zero_exponent_consistently() {
    // Arrange
    let exponents = vec![0_u32, 1];
    let degree = 3;

    // Act
    let plus = partition::product_one_plus_x_powers(&exponents, degree).unwrap();
    let minus = partition::product_one_minus_x_powers(&exponents, degree).unwrap();
    let inv_plus = partition::product_inv_one_plus_x_powers(&exponents, degree).unwrap();
    let inv_minus = partition::product_inv_one_minus_x_powers(&exponents, degree);

    // Assert
    assert_eq!(fps::FPS::new(vec![2, 2]), plus);
    assert!(minus.is_zero());

    let inv_2 = modulo::inv(2);
    assert_eq!(
        fps::FPS::new(vec![
            inv_2,
            modulo::sub(0, inv_2),
            inv_2,
            modulo::sub(0, inv_2)
        ]),
        inv_plus
    );
    assert!(inv_minus.is_none());
}
