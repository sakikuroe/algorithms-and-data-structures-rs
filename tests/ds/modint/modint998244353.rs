use anmitsu::ds::modint::modint998244353::ModInt998244353;

const MOD: u32 = 998244353;

// --- Tests for Constructors (`new`, `new_raw`, `from`) ---

// Tests that `new` correctly handles values within the modulus.
#[test]
fn new_with_values_less_than_mod_creates_correctly() {
    // Arrange
    let cases = [0, 1, 123, MOD - 1];

    for &val in cases.iter() {
        // Act
        let m = ModInt998244353::new(val as u64);

        // Assert
        assert_eq!(val, m.val(), "Failed for input {}", val);
    }
}

// Tests that `new` correctly wraps values greater than or equal to the modulus.
#[test]
fn new_with_values_ge_mod_wraps_around() {
    // Arrange
    let cases = [
        (MOD as u64, 0),
        (MOD as u64 + 1, 1),
        (2 * MOD as u64, 0),
        (u64::MAX, (u64::MAX % MOD as u64) as u32),
    ];

    for &(input, expected) in cases.iter() {
        // Act
        let m = ModInt998244353::new(input);

        // Assert
        assert_eq!(expected, m.val(), "Failed for input {}", input);
    }
}

// Tests that `new_raw` creates an instance for a valid raw value.
#[test]
fn new_raw_with_valid_value_creates_correct_instance() {
    // Arrange
    let val = MOD - 1;

    // Act
    let m = ModInt998244353::new_raw(val);

    // Assert
    assert_eq!(val, m.val());
}

// Tests that `new_raw` panics when the input value is equal to MOD.
#[test]
#[should_panic(expected = "Raw value 998244353 must be less than MOD 998244353")]
fn new_raw_with_mod_value_panics() {
    // Arrange, Act, Assert (in panic)
    ModInt998244353::new_raw(MOD);
}

// Tests that `new_raw` panics when the input value is greater than MOD.
#[test]
#[should_panic(expected = "Raw value 998244354 must be less than MOD 998244353")]
fn new_raw_with_value_gt_mod_panics() {
    // Arrange, Act, Assert (in panic)
    ModInt998244353::new_raw(MOD + 1);
}

// Tests `From<u32>` implementation.
#[test]
fn from_u32_converts_correctly() {
    // Arrange
    let cases = [
        (0u32, 0u32),
        (123u32, 123u32),
        (MOD, 0u32),
        (MOD + 10, 10u32),
        (u32::MAX, u32::MAX % MOD),
    ];

    for &(input, expected) in cases.iter() {
        // Act
        let m: ModInt998244353 = input.into();

        // Assert
        assert_eq!(expected, m.val(), "Failed for input {}", input);
    }
}

// Tests `From<i32>` implementation for positive, negative, and zero values.
#[test]
fn from_i32_converts_correctly() {
    // Arrange
    let cases = [
        (0i32, 0u32),
        (123i32, 123u32),
        (-1i32, MOD - 1),
        (-123i32, MOD - 123),
        (i32::MAX, (i32::MAX as u64 % MOD as u64) as u32),
        // (-2147483648 % 998244353 + 998244353) % 998244353 = 847249411
        (i32::MIN, 847249411),
    ];

    for &(input, expected) in cases.iter() {
        // Act
        let m: ModInt998244353 = input.into();

        // Assert
        assert_eq!(expected, m.val(), "Failed for input {}", input);
    }
}

// --- Tests for Arithmetic Operations ---

// Tests addition and addition assignment with various cases.
#[test]
fn add_and_add_assign_work_correctly() {
    // Arrange
    let cases = [
        (1, 1, 2),
        (MOD - 1, 1, 0),
        (MOD - 1, 2, 1),
        (123, 456, 579),
        (0, 0, 0),
        (MOD - 1, MOD - 1, MOD - 2),
    ];

    for &(a_val, b_val, expected) in cases.iter() {
        // Act (Add)
        let a = ModInt998244353::new(a_val as u64);
        let b = ModInt998244353::new(b_val as u64);
        let result_add = a + b;

        // Assert (Add)
        assert_eq!(
            expected,
            result_add.val(),
            "Failed for {} + {}",
            a_val,
            b_val
        );

        // Act (AddAssign)
        let mut a_assign = ModInt998244353::new(a_val as u64);
        a_assign += b;

        // Assert (AddAssign)
        assert_eq!(
            expected,
            a_assign.val(),
            "Failed for {} += {}",
            a_val,
            b_val
        );
    }
}

// Tests subtraction and subtraction assignment with various cases.
#[test]
fn sub_and_sub_assign_work_correctly() {
    // Arrange
    let cases = [
        (2, 1, 1),
        (1, 2, MOD - 1),
        (0, 1, MOD - 1),
        (0, 0, 0),
        (123, 23, 100),
        (123, 123, 0),
    ];

    for &(a_val, b_val, expected) in cases.iter() {
        // Act (Sub)
        let a = ModInt998244353::new(a_val);
        let b = ModInt998244353::new(b_val);
        let result_sub = a - b;

        // Assert (Sub)
        assert_eq!(
            expected,
            result_sub.val(),
            "Failed for {} - {}",
            a_val,
            b_val
        );

        // Act (SubAssign)
        let mut a_assign = ModInt998244353::new(a_val);
        a_assign -= b;

        // Assert (SubAssign)
        assert_eq!(
            expected,
            a_assign.val(),
            "Failed for {} -= {}",
            a_val,
            b_val
        );
    }
}

// Tests multiplication and multiplication assignment with various cases.
#[test]
fn mul_and_mul_assign_work_correctly() {
    // Arrange
    let cases = [
        (2, 3, 6),
        (0, 123, 0),
        (1, 123, 123),
        (MOD - 1, 2, MOD - 2),        // (-1) * 2 = -2
        (MOD - 1, MOD - 1, 1),        // (-1) * (-1) = 1
        (100_000, 100_000, 17556470), // 10_000_000_000 % MOD
    ];

    for &(a_val, b_val, expected) in cases.iter() {
        // Act (Mul)
        let a = ModInt998244353::new(a_val as u64);
        let b = ModInt998244353::new(b_val as u64);
        let result_mul = a * b;

        // Assert (Mul)
        assert_eq!(
            expected,
            result_mul.val(),
            "Failed for {} * {}",
            a_val,
            b_val
        );

        // Act (MulAssign)
        let mut a_assign = ModInt998244353::new(a_val as u64);
        a_assign *= b;

        // Assert (MulAssign)
        assert_eq!(
            expected,
            a_assign.val(),
            "Failed for {} *= {}",
            a_val,
            b_val
        );
    }
}

// Tests division and division assignment with various cases.
#[test]
fn div_and_div_assign_work_correctly() {
    // Arrange
    let cases = [
        (6, 2, 3),
        (7, 2, 499122180), // 7 * 2^-1 mod MOD
        (0, 123, 0),
        (123, 123, 1),
        (MOD - 6, 2, MOD - 3), // -6 / 2 = -3
    ];

    for &(a_val, b_val, expected) in cases.iter() {
        // Act (Div)
        let a = ModInt998244353::new(a_val as u64);
        let b = ModInt998244353::new(b_val);
        let result_div = a / b;

        // Assert (Div)
        assert_eq!(
            expected,
            result_div.val(),
            "Failed for {} / {}",
            a_val,
            b_val
        );

        // Act (DivAssign)
        let mut a_assign = ModInt998244353::new(a_val as u64);
        a_assign /= b;

        // Assert (DivAssign)
        assert_eq!(
            expected,
            a_assign.val(),
            "Failed for {} /= {}",
            a_val,
            b_val
        );
    }
}

// Tests that division by zero panics as expected.
#[test]
#[should_panic(expected = "Division by zero is not allowed for ModInt998244353")]
fn div_by_zero_panics() {
    // Arrange
    let a = ModInt998244353::new(10);
    let b = ModInt998244353::new(0);

    // Act
    let _ = a / b;
}

// Tests that division assignment by zero panics as expected.
#[test]
#[should_panic(expected = "Division by zero is not allowed for ModInt998244353")]
fn div_assign_by_zero_panics() {
    // Arrange
    let mut a = ModInt998244353::new(10);
    let b = ModInt998244353::new(0);

    // Act
    a /= b;
}

// --- Tests for Other Methods (`pow`, `inv`, `neg`) ---

// Tests `pow` method with various base and exponent values.
#[test]
fn pow_computes_correctly() {
    // Arrange
    let cases = [
        (2, 10, 1024),
        (3, 4, 81),
        (123, 0, 1),
        (123, 1, 123),
        (0, 5, 0),
        (0, 0, 1), // By convention in modular arithmetic
        (MOD - 1, 2, 1),
        (MOD - 1, 3, MOD - 1),
    ];

    for &(base_val, exp, expected) in cases.iter() {
        // Act
        let base = ModInt998244353::new(base_val as u64);
        let result = base.pow(exp);

        // Assert
        assert_eq!(expected, result.val(), "Failed for {}^{}", base_val, exp);
    }
}

// Tests `inv` method for invertible and non-invertible elements.
#[test]
fn inv_computes_correctly() {
    // Arrange
    let cases = [2, 123, MOD - 1];

    for &val in cases.iter() {
        // Arrange
        let m = ModInt998244353::new(val as u64);
        let one = ModInt998244353::new(1);

        // Act
        let inv_m_opt = m.inv();

        // Assert
        assert!(inv_m_opt.is_some(), "Inverse of {} should exist", val);
        let inv_m = inv_m_opt.unwrap();
        assert_eq!(one, m * inv_m, "m * m^-1 should be 1 for m = {}", val);
    }
}

// Tests `inv` method for zero, which has no inverse.
#[test]
fn inv_of_zero_is_none() {
    // Arrange
    let zero = ModInt998244353::new(0);

    // Act
    let result = zero.inv();

    // Assert
    assert!(result.is_none());
}

// Tests `neg` (unary negation) operation.
#[test]
fn neg_computes_correctly() {
    // Arrange
    let cases = [(0, 0), (1, MOD - 1), (123, MOD - 123), (MOD - 1, 1)];

    for &(val, expected) in cases.iter() {
        // Act
        let m = ModInt998244353::new(val as u64);
        let neg_m = -m;

        // Assert
        assert_eq!(expected, neg_m.val(), "Failed for -{}", val);

        // Double negation should be identity
        assert_eq!(m, -neg_m, "Failed for -(-{})", val);
    }
}

// Tests `Display` implementation outputs the underlying value.
#[test]
fn display_outputs_underlying_value() {
    // Arrange
    let cases = [(10_u64, "10"), (MOD as u64 + 5, "5")];

    for &(input, expected) in cases.iter() {
        let m = ModInt998244353::new(input);

        // Act
        let displayed = format!("{}", m);

        // Assert
        assert_eq!(expected, displayed, "Failed for input {}", input);
    }
}

// --- Tests for Derived Traits (`PartialEq`, `Eq`, `Copy`, `Clone`) ---

// Tests `PartialEq` and `Eq` implementations.
#[test]
fn equality_and_inequality_work_correctly() {
    // Arrange
    let a1 = ModInt998244353::new(100);
    let a2 = ModInt998244353::new(100);
    let b = ModInt998244353::new(200);
    let c = ModInt998244353::new(MOD as u64 + 100);

    // Act and Assert
    assert_eq!(a1, a2);
    assert_eq!(a1, c); // 100 == 100 + MOD
    assert_ne!(a1, b);
}

// Tests `Copy` and `Clone` implementations.
#[test]
fn copy_and_clone_work_correctly() {
    // Arrange
    let m1 = ModInt998244353::new(12345);

    // Act (Copy)
    let m2 = m1;
    let mut m3 = m1;
    m3 += 1;

    // Assert (Copy)
    assert_eq!(m1.val(), 12345); // Original should be unchanged
    assert_eq!(m2.val(), 12345);

    // Act (Clone)
    let m4 = m1.clone();
    let mut m5 = m1.clone();
    m5 -= 1;

    // Assert (Clone)
    assert_eq!(m1.val(), 12345); // Original should be unchanged
    assert_eq!(m4.val(), 12345);
}
