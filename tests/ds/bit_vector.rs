use anmitsu::ds::bit_vector;

// Test: new_empty
// Description: Tests creating an empty BitVector.
#[test]
fn test_new_empty() {
    let bv = bit_vector::BitVector::new(&[]);
    assert_eq!(bv.len(), 0,);
    assert!(bv.is_empty(),);
    assert_eq!(bv.rank(0), 0,);
}

// Test: len_is_empty_non_empty
// Description: Tests len() and is_empty() for non-empty BitVectors.
#[test]
fn test_len_is_empty_non_empty() {
    let bv1 = bit_vector::BitVector::new(&[0]);
    assert_eq!(bv1.len(), 1);
    assert!(!bv1.is_empty());

    let bv2 = bit_vector::BitVector::new(&[1, 0, 1, 1, 0]);
    assert_eq!(bv2.len(), 5);
    assert!(!bv2.is_empty());
}

// Test: new_all_zeros
// Description: Tests BitVector with all zero bits.
#[test]
fn test_new_all_zeros() {
    let v = vec![0; 100];
    let bv = bit_vector::BitVector::new(&v);
    assert_eq!(bv.len(), 100);
    for i in 0..=100 {
        assert_eq!(0, bv.rank(i));
    }
}

// Test: new_all_ones
// Description: Tests BitVector with all one bits.
#[test]
fn test_new_all_ones() {
    let v = vec![1; 100];
    let bv = bit_vector::BitVector::new(&v);
    assert_eq!(bv.len(), 100);
    for i in 0..=100 {
        assert_eq!(i, bv.rank(i));
    }
}

// Test: panic_rank_out_of_bounds
// Description: Tests that BitVector::rank panics if r > len.
#[test]
#[should_panic(expected = "cannot be greater than the length of the BitVector")]
fn test_panic_rank_out_of_bounds() {
    let bv = bit_vector::BitVector::new(&[1, 0, 1]);
    bv.rank(4); // len is 3, rank(4) is out of bounds / lenは3、rank(4)は範囲外
}

// Test: panic_rank_out_of_bounds_empty
// Description: Tests that BitVector::rank panics if r > len for an empty vector (e.g. r=1, len=0).
#[test]
#[should_panic(expected = "cannot be greater than the length of the BitVector")]
fn test_panic_rank_out_of_bounds_empty() {
    let bv = bit_vector::BitVector::new(&[]);
    bv.rank(1); // len is 0, rank(1) is out of bounds / lenは0、rank(1)は範囲外
}

// Test: rank_at_block_boundaries
// Description: Tests rank specifically at and around 64-bit block boundaries.
#[test]
fn test_rank_at_block_boundaries() {
    let mut v = vec![0; 192]; // Exactly 3 blocks // 正確に3ブロック
    v[0] = 1; // Start of block 0 // ブロック0の開始
    v[63] = 1; // End of block 0   // ブロック0の終了
    v[64] = 1; // Start of block 1 // ブロック1の開始
    v[127] = 1; // End of block 1   // ブロック1の終了
    v[128] = 1; // Start of block 2 // ブロック2の開始
    v[191] = 1; // End of block 2   // ブロック2の終了

    let bv = bit_vector::BitVector::new(&v);
    assert_eq!(192, bv.len());

    assert_eq!(0, bv.rank(0));
    assert_eq!(1, bv.rank(1)); // v[0] を含む rank(1)

    assert_eq!(1, bv.rank(63)); // v[63] の前の rank(63)
    assert_eq!(2, bv.rank(64)); // v[63] を含む rank(64)

    assert_eq!(3, bv.rank(65)); // v[64] を含む rank(65)

    assert_eq!(3, bv.rank(127)); // v[127] の前の rank(127)
    assert_eq!(4, bv.rank(128)); // v[127] を含む rank(128)

    assert_eq!(5, bv.rank(129)); // v[128] を含む rank(129)
    assert_eq!(5, bv.rank(191)); // v[191] の前の rank(191)
    assert_eq!(6, bv.rank(192)); // v[191] を含む rank(192), 全長の合計
}
