use anmitsu::modulo998244353::fps;

#[test]
fn sub_and_sub_assign_work_correctly() {
    // Arrange
    let a = fps::FPS::new(vec![5, 2]);
    let b = fps::FPS::new(vec![3, 1, 7]);

    // Act
    let diff = a.clone() - b.clone();
    let mut diff_assign = a;
    diff_assign -= b;

    // Assert
    assert_eq!(2, diff.get(0));
    assert_eq!(1, diff.get(1));
    assert_eq!(998244353 - 7, diff.get(2));
    assert_eq!(diff, diff_assign);
}

#[test]
fn neg_works_correctly() {
    // Arrange
    let a = fps::FPS::new(vec![1, 2]);

    // Act
    let neg = -a;

    // Assert
    assert_eq!(998244352, neg.get(0));
    assert_eq!(998244351, neg.get(1));
}
