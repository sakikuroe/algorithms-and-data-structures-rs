use anmitsu::modulo998244353::fps;

#[test]
fn add_and_add_assign_work_correctly() {
    // Arrange
    let a = fps::FPS::new(vec![1, 2]);
    let b = fps::FPS::new(vec![3, 4, 5]);

    // Act
    let sum = a.clone() + b.clone();
    let mut sum_assign = a;
    sum_assign += b;

    // Assert
    assert_eq!(4, sum.get(0));
    assert_eq!(6, sum.get(1));
    assert_eq!(5, sum.get(2));
    assert_eq!(sum, sum_assign);
}
