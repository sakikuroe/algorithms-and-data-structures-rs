use anmitsu::ds::modint::convolution998244353;
use rand::{Rng, SeedableRng, rngs::StdRng};

#[test]
fn convolution_with_empty_input_returns_empty() {
    // Arrange
    let a = Vec::<u32>::new();
    let b = vec![1, 2, 3];

    // Act
    let result = convolution998244353::convolution(&a, &b);

    // Assert
    assert!(result.is_empty());
}

#[test]
fn convolution_with_small_inputs_matches_expected() {
    // Arrange
    let a = vec![1, 2, 3];
    let b = vec![4, 5, 6];
    let expected = vec![4, 13, 28, 27, 18];

    // Act
    let result = convolution998244353::convolution(&a, &b);

    // Assert
    assert_eq!(expected.len(), result.len(), "length mismatch");
    assert_eq!(expected, result);
}

#[test]
fn convolution_with_large_inputs_matches_naive_result() {
    // Arrange
    let a = vec![1u32; 64];
    let b = vec![1u32; 64];
    let expected_len = a.len() + b.len() - 1;
    let mut expected = vec![0u32; expected_len];
    for (i, &x) in a.iter().enumerate() {
        for (j, &y) in b.iter().enumerate() {
            expected[i + j] += x * y;
        }
    }

    // Act
    let result = convolution998244353::convolution(&a, &b);

    // Assert
    assert_eq!(expected.len(), result.len(), "length mismatch");
    assert_eq!(expected, result);
}

#[test]
fn convolution_with_max_length_sparse_random_matches_naive() {
    // Arrange
    let len_a = convolution998244353::MAX_NTT_LEN / 2;
    let len_b = convolution998244353::MAX_NTT_LEN / 2;
    let mut rng = StdRng::seed_from_u64(123_456_789);
    const NON_ZERO: usize = 128;

    let mut a = vec![0u32; len_a];
    let mut b = vec![0u32; len_b];

    let mut used_a = std::collections::HashSet::with_capacity(NON_ZERO);
    while used_a.len() < NON_ZERO {
        let idx = rng.random_range(0..len_a);
        if used_a.insert(idx) {
            a[idx] = rng.random_range(0..convolution998244353::MOD);
        }
    }

    let mut used_b = std::collections::HashSet::with_capacity(NON_ZERO);
    while used_b.len() < NON_ZERO {
        let idx = rng.random_range(0..len_b);
        if used_b.insert(idx) {
            b[idx] = rng.random_range(0..convolution998244353::MOD);
        }
    }

    let mut expected = vec![0u32; len_a + len_b - 1];
    for (i, &x) in a.iter().enumerate().filter(|(_, v)| **v != 0) {
        for (j, &y) in b.iter().enumerate().filter(|(_, v)| **v != 0) {
            let idx = i + j;
            let prod = (x as u64 * y as u64) % convolution998244353::MOD as u64;
            let next = (expected[idx] as u64 + prod) % convolution998244353::MOD as u64;
            expected[idx] = next as u32;
        }
    }

    // Act
    let result = convolution998244353::convolution(&a, &b);

    // Assert
    assert_eq!(expected, result);
}

#[test]
fn convolution_handles_values_near_modulus() {
    // Arrange
    let m = convolution998244353::MOD;
    let a = vec![m - 1, m - 2];
    let b = vec![2, 3];
    let mut expected = vec![0u32; a.len() + b.len() - 1];
    for (i, &x) in a.iter().enumerate() {
        for (j, &y) in b.iter().enumerate() {
            let prod = ((x as u64 * y as u64) % m as u64) as u32;
            expected[i + j] = ((expected[i + j] as u64 + prod as u64) % m as u64) as u32;
        }
    }

    // Act
    let result = convolution998244353::convolution(&a, &b);

    // Assert
    assert_eq!(expected, result);
}

#[test]
#[should_panic(expected = "Convolution length")]
fn convolution_panics_when_length_exceeds_limit() {
    // Arrange
    let len_a = convolution998244353::MAX_NTT_LEN;
    let len_b = 64;
    let a = vec![1u32; len_a];
    let b = vec![1u32; len_b];

    // Act, Assert (panic)
    let _ = convolution998244353::convolution(&a, &b);
}
