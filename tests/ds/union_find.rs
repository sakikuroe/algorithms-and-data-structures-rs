use anmitsu::ds::union_find::UnionFind;

// Test: new_len_and_roots
// Description: Verifies initial length, roots, and singleton sizes.
#[test]
fn test_new_len_and_roots() {
    let mut uf = UnionFind::new(4);
    assert_eq!(uf.len(), 4);

    for i in 0..4 {
        assert!(uf.is_root(i));
        assert_eq!(uf.get_size(i), 1);
    }

    assert!(!uf.is_same(0, 1));
    assert!(!uf.is_same(2, 3));
}

// Test: new_zero_elements
// Description: Confirms an empty UnionFind reports length zero.
#[test]
fn test_new_zero_elements() {
    let uf = UnionFind::new(0);
    assert_eq!(uf.len(), 0);
}

// Test: union_and_is_same
// Description: Tests merging sets and membership checks.
#[test]
fn test_union_and_is_same() {
    let mut uf = UnionFind::new(4);
    uf.union(0, 1);
    uf.union(2, 3);

    assert!(uf.is_same(0, 1));
    assert!(uf.is_same(2, 3));
    assert!(!uf.is_same(1, 2));

    uf.union(1, 2);
    assert!(uf.is_same(0, 2));
    assert!(uf.is_same(0, 3));
}

// Test: self_union_no_effect
// Description: Ensures unioning an element with itself is a no-op.
#[test]
fn test_self_union_no_effect() {
    let mut uf = UnionFind::new(1);
    uf.union(0, 0);
    assert!(uf.is_same(0, 0));
    assert_eq!(uf.get_size(0), 1);
    assert_eq!(uf.get_group(0), vec![0]);
}

// Test: redundant_union_does_not_change_size
// Description: Repeated unions of the same pair keep size stable.
#[test]
fn test_redundant_union_does_not_change_size() {
    let mut uf = UnionFind::new(3);
    uf.union(0, 1);
    uf.union(0, 1); // Same union again // 同じ union を再度実行

    assert!(uf.is_same(0, 1));
    assert_eq!(uf.get_size(0), 2);
    assert_eq!(uf.get_group(1), vec![0, 1]);
}

// Test: get_size_after_unions
// Description: Ensures set sizes are updated after unions.
#[test]
fn test_get_size_after_unions() {
    let mut uf = UnionFind::new(5);
    uf.union(0, 1);
    uf.union(0, 2);
    uf.union(3, 4);

    assert_eq!(uf.get_size(0), 3);
    assert_eq!(uf.get_size(2), 3); // Non-root query // 根以外からのクエリ
    assert_eq!(uf.get_size(3), 2);
    assert_eq!(uf.get_size(4), 2);
}

// Test: get_group_sorted_members
// Description: Checks that get_group returns all members sorted.
#[test]
fn test_get_group_sorted_members() {
    let mut uf = UnionFind::new(6);
    uf.union(0, 1);
    uf.union(2, 3);
    uf.union(1, 2); // Merge {0,1} and {2,3} // {0,1} と {2,3} を統合
    uf.union(4, 5);

    assert_eq!(uf.get_group(0), vec![0, 1, 2, 3]);
    assert_eq!(uf.get_group(3), vec![0, 1, 2, 3]); // Non-root entry point // 根以外から
    assert_eq!(uf.get_group(5), vec![4, 5]);
}

// Test: panic_find_out_of_bounds
// Description: Ensures find() panics on an invalid index.
#[test]
#[should_panic(expected = "Index 5 is out of bounds for UnionFind with size 5")]
fn test_panic_find_out_of_bounds() {
    let mut uf = UnionFind::new(5);
    uf.find(5); // len is 5, index 5 is out of bounds // len は 5, index 5 は範囲外
}

// Test: panic_find_empty
// Description: Ensures find() panics when called on an empty structure.
#[test]
#[should_panic(expected = "Index 0 is out of bounds for UnionFind with size 0")]
fn test_panic_find_empty() {
    let mut uf = UnionFind::new(0);
    uf.find(0); // len is 0, index 0 is out of bounds // len は 0, index 0 は範囲外
}

// Test: panic_union_out_of_bounds
// Description: Ensures union() panics when either index is invalid.
#[test]
#[should_panic(expected = "Index out of bounds for union: x=4, y=5, len=5")]
fn test_panic_union_out_of_bounds() {
    let mut uf = UnionFind::new(5);
    uf.union(4, 5); // y=5 is out of bounds // y=5 は範囲外
}

// Test: panic_is_same_out_of_bounds
// Description: Ensures is_same() panics when given an invalid index.
#[test]
#[should_panic(expected = "Index out of bounds for is_same: x=0, y=3, len=3")]
fn test_panic_is_same_out_of_bounds() {
    let mut uf = UnionFind::new(3);
    uf.is_same(0, 3); // y=3 is out of bounds // y=3 は範囲外
}

// Test: panic_is_same_empty
// Description: Ensures is_same() panics on an empty structure.
#[test]
#[should_panic(expected = "Index out of bounds for is_same: x=0, y=0, len=0")]
fn test_panic_is_same_empty() {
    let mut uf = UnionFind::new(0);
    uf.is_same(0, 0); // len is 0, both indices are out of bounds // len は 0, どちらの index も範囲外
}

// Test: panic_get_size_out_of_bounds
// Description: Ensures get_size() panics on an invalid index.
#[test]
#[should_panic(expected = "Index 3 is out of bounds for UnionFind with size 3")]
fn test_panic_get_size_out_of_bounds() {
    let mut uf = UnionFind::new(3);
    uf.get_size(3); // index 3 is out of bounds // index 3 は範囲外
}

// Test: panic_get_group_out_of_bounds
// Description: Ensures get_group() panics on an invalid index.
#[test]
#[should_panic(expected = "Index 2 is out of bounds for UnionFind with size 2")]
fn test_panic_get_group_out_of_bounds() {
    let mut uf = UnionFind::new(2);
    uf.get_group(2); // index 2 is out of bounds // index 2 は範囲外
}

// Test: panic_union_empty
// Description: Ensures union() panics when called on an empty structure.
#[test]
#[should_panic(expected = "Index out of bounds for union: x=0, y=0, len=0")]
fn test_panic_union_empty() {
    let mut uf = UnionFind::new(0);
    uf.union(0, 0); // len is 0, both indices are out of bounds // len は 0, どちらの index も範囲外
}

// Test: panic_get_size_empty
// Description: Ensures get_size() panics when called on an empty structure.
#[test]
#[should_panic(expected = "Index 0 is out of bounds for UnionFind with size 0")]
fn test_panic_get_size_empty() {
    let mut uf = UnionFind::new(0);
    uf.get_size(0); // len is 0, index 0 is out of bounds // len は 0, index 0 は範囲外
}

// Test: panic_get_group_empty
// Description: Ensures get_group() panics when called on an empty structure.
#[test]
#[should_panic(expected = "Index 0 is out of bounds for UnionFind with size 0")]
fn test_panic_get_group_empty() {
    let mut uf = UnionFind::new(0);
    uf.get_group(0); // len is 0, index 0 is out of bounds // len は 0, index 0 は範囲外
}
