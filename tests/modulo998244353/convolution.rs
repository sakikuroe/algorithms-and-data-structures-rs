use anmitsu::modulo998244353::convolution as convolution998244353;

const MOD: u32 = convolution998244353::MOD;

const NTT_RATE: [u32; 22] = [
    0x3656d65b, 0x1e5ea9e6, 0x16038782, 0x13caac90, 0x3a9a4cfa, 0x761af21, 0xe372007, 0x3a2be7d4,
    0x23fe18b2, 0x330f5b68, 0x7d37cf9, 0x3239edef, 0x2b8ea5c3, 0x382d2452, 0x300e9be2, 0x908b3f5,
    0x1e726cd9, 0x1e02c2f0, 0x2c49629c, 0x2c2b7c93, 0x35a5081, 0x33b69d8b,
];

const INTT_RATE: [u32; 22] = [
    0x52929a6, 0x163456b8, 0x16400573, 0x267c5b5f, 0x6b059a5, 0x294c15f1, 0x94415d9, 0x2f83389c,
    0x569c0ec, 0x3346ebba, 0x37473ab0, 0x1524e16f, 0x68442e3, 0x117ab9d0, 0x1fe52df0, 0x1263f553,
    0x7392943, 0x24433aa8, 0x1a2993eb, 0x156d2fbf, 0x311e570f, 0x6294a13,
];

const INVS: [u32; 23] = [
    1, 499122177, 748683265, 873463809, 935854081, 967049217, 982646785, 990445569, 994344961,
    996294657, 997269505, 997756929, 998000641, 998122497, 998183425, 998213889, 998229121,
    998236737, 998240545, 998242449, 998243401, 998243877, 998244115,
];

#[inline]
fn add_mod(lhs: u32, rhs: u32) -> u32 {
    let sum = lhs + rhs;
    if sum >= MOD { sum - MOD } else { sum }
}

#[inline]
fn sub_mod(lhs: u32, rhs: u32) -> u32 {
    if lhs >= rhs {
        lhs - rhs
    } else {
        lhs + MOD - rhs
    }
}

#[inline]
fn mul_mod(lhs: u32, rhs: u32) -> u32 {
    ((lhs as u64 * rhs as u64) % MOD as u64) as u32
}

fn ntt_ref(a: &mut [u32]) {
    if a.is_empty() {
        return;
    }

    let n = a.len();
    assert!(n.is_power_of_two(), "NTT length must be a power of two");
    assert!(
        n <= convolution998244353::MAX_NTT_LEN,
        "NTT length exceeds MAX_NTT_LEN"
    );

    let h = n.trailing_zeros();

    for len in 0..h {
        let p = 1 << (h - len - 1);
        let mut rot = 1;
        let step = 1 << (h - len);
        for (s, chunk) in a.chunks_mut(step).enumerate() {
            let ptr = chunk.as_mut_ptr();
            for i in 0..p {
                unsafe {
                    let l = *ptr.add(i);
                    let r = mul_mod(*ptr.add(i + p), rot);
                    *ptr.add(i) = add_mod(l, r);
                    *ptr.add(i + p) = sub_mod(l, r);
                }
            }
            rot = mul_mod(rot, NTT_RATE[s.trailing_ones() as usize]);
        }
    }
}

fn intt_ref(a: &mut [u32]) {
    if a.is_empty() {
        return;
    }

    let n = a.len();
    assert!(n.is_power_of_two(), "NTT length must be a power of two");
    assert!(
        n <= convolution998244353::MAX_NTT_LEN,
        "NTT length exceeds MAX_NTT_LEN"
    );

    let h = n.trailing_zeros();

    for len in (1..=h).rev() {
        let mut irot = 1;
        let p = 1 << (h - len);
        let step = 1 << (h - len + 1);
        for (s, chunk) in a.chunks_mut(step).enumerate() {
            let ptr = chunk.as_mut_ptr();
            for i in 0..p {
                unsafe {
                    let l = *ptr.add(i);
                    let r = *ptr.add(i + p);
                    *ptr.add(i) = add_mod(l, r);
                    *ptr.add(i + p) = mul_mod(sub_mod(l, r), irot);
                }
            }
            irot = mul_mod(irot, INTT_RATE[s.trailing_ones() as usize]);
        }
    }
}

fn convolution_ref(a: &[u32], b: &[u32]) -> Vec<u32> {
    if a.is_empty() || b.is_empty() {
        return Vec::new();
    }

    let s = a.len() + b.len() - 1;
    let t = s.next_power_of_two();
    assert!(
        t <= convolution998244353::MAX_NTT_LEN,
        "Convolution length exceeds MAX_NTT_LEN"
    );

    let mut fa = Vec::with_capacity(t);
    fa.extend_from_slice(a);
    fa.resize(t, 0);
    let mut fb = Vec::with_capacity(t);
    fb.extend_from_slice(b);
    fb.resize(t, 0);

    ntt_ref(&mut fa);
    ntt_ref(&mut fb);
    fa.iter_mut()
        .zip(fb.iter())
        .for_each(|(x, y)| *x = mul_mod(*x, *y));
    intt_ref(&mut fa);

    let inv_len = INVS[t.trailing_zeros() as usize];
    fa.iter_mut()
        .take(s)
        .for_each(|x| *x = mul_mod(*x, inv_len));
    fa.truncate(s);
    fa
}

fn gen_values(len: usize, mut seed: u64) -> Vec<u32> {
    let mut values = Vec::with_capacity(len);
    for _ in 0..len {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        values.push((seed % MOD as u64) as u32);
    }
    values
}

#[test]
fn convolution_with_empty_input_returns_empty() {
    // Arrange
    let a = Vec::<u32>::new();
    let b = vec![1, 2, 3];

    // Act
    let result = convolution998244353::convolution(a, b);

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
    let result = convolution998244353::convolution(a, b);

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
    let result = convolution998244353::convolution(a, b);

    // Assert
    assert_eq!(expected.len(), result.len(), "length mismatch");
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
    let result = convolution998244353::convolution(a, b);

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
    let _ = convolution998244353::convolution(a, b);
}

#[test]
fn ntt_matches_reference_for_various_lengths() {
    let test_lg = 0..=16;

    for lg in test_lg {
        let len = 1usize << lg;
        let input = gen_values(len, 1 + lg as u64);
        let mut expected = input.clone();
        let mut actual = input;

        ntt_ref(&mut expected);
        convolution998244353::ntt(&mut actual);

        assert_eq!(expected, actual, "lg mismatch: {}", lg);
    }
}

#[test]
fn intt_matches_reference_for_various_lengths() {
    let test_lg = 0..=16;

    for lg in test_lg {
        let len = 1usize << lg;
        let input = gen_values(len, 100 + lg as u64);
        let mut expected = input.clone();
        let mut actual = input;

        intt_ref(&mut expected);
        convolution998244353::intt(&mut actual);

        assert_eq!(expected, actual, "lg mismatch: {}", lg);
    }
}

#[test]
fn ntt_then_intt_roundtrip_matches_original_after_scaling() {
    let test_lg = 0..=16;

    for lg in test_lg {
        let len = 1usize << lg;
        let input = gen_values(len, 999 + lg as u64);
        let mut actual = input.clone();

        convolution998244353::ntt(&mut actual);
        convolution998244353::intt(&mut actual);
        let inv_len = INVS[lg];
        actual.iter_mut().for_each(|x| *x = mul_mod(*x, inv_len));

        assert_eq!(input, actual, "lg mismatch: {}", lg);
    }
}

#[test]
fn convolution_with_large_random_inputs_matches_reference() {
    let a = gen_values(100, 12345);
    let b = gen_values(120, 54321);
    let expected = convolution_ref(&a, &b);

    let result = convolution998244353::convolution(a, b);

    assert_eq!(expected, result);
}
