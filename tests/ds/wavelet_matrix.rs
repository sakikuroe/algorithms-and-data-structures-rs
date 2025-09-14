use anmitsu::ds::wavelet_matrix;

// Sets up a common WaveletMatrix instance for general testing.
fn setup_general_wavelet_matrix() -> wavelet_matrix::WaveletMatrix {
    let data = vec![5, 4, 8, 6, 0, 7, 2, 5];
    wavelet_matrix::WaveletMatrix::new(&data)
}

// Tests the constructor with various boundary slices.
#[test]
fn new_with_boundary_slices_works() {
    // Arrange
    let cases = [
        // Case: Empty slice
        vec![],
        // Case: Single element
        vec![100],
        // Case: All identical elements
        vec![3, 3, 3, 3, 3],
        // Case: Contains type's max value
        vec![0, 1, usize::MAX, 5],
    ];

    for data in cases.iter() {
        // Act & Assert (The constructor should not panic and complete successfully)
        let _ = wavelet_matrix::WaveletMatrix::new(data);
    }
}

// Tests `count_less_than` with various ranges and values.
#[test]
fn count_less_than_returns_correct_count() {
    // Arrange
    let wm = setup_general_wavelet_matrix(); // data is [5, 4, 8, 6, 0, 7, 2, 5]
    let cases = [
        // (l, r, upper, expected)
        (0, 8, 5, 3), // Full range: [4, 0, 2] are less than 5.
        (2, 6, 7, 2), // v[2..6] is [8, 6, 0, 7]. Less than 7 are [6, 0].
        (3, 3, 5, 0), // Empty range.
    ];

    for &(l, r, upper, expected) in cases.iter() {
        // Act
        let result = wm.count_less_than(l, r, upper);
        // Assert
        assert_eq!(
            expected, result,
            "Failed on count_less_than({}, {}, {})",
            l, r, upper
        );
    }
}

// Tests `count` with various ranges and value bounds.
#[test]
fn count_in_range_returns_correct_count() {
    // Arrange
    let wm = setup_general_wavelet_matrix(); // data is [5, 4, 8, 6, 0, 7, 2, 5]
    let cases = [
        // (l, r, lower, upper, expected)
        (0, 8, 4, 7, 4), // Full range: [5, 4, 6, 5] are in [4, 7).
        (2, 7, 5, 9, 3), // v[2..7] is [8, 6, 0, 7, 2]. In [5, 9) are [8, 6, 7].
        (0, 8, 8, 7, 0), // Invalid value range (lower >= upper).
    ];

    for &(l, r, lower, upper, expected) in cases.iter() {
        // Act
        let result = wm.count(l, r, lower, upper);
        // Assert
        assert_eq!(
            expected, result,
            "Failed on count({}, {}, {}, {})",
            l, r, lower, upper
        );
    }
}

// Tests queries on a WaveletMatrix containing `usize::MAX`.
#[test]
fn queries_with_max_value_data_work_correctly() {
    // Arrange
    let data = vec![0, usize::MAX, 1, 100, usize::MAX];
    let wm = wavelet_matrix::WaveletMatrix::new(&data);

    // Act & Assert for `count_less_than`
    assert_eq!(
        2,
        wm.count_less_than(0, 5, 100),
        "Failed on count_less_than with MAX"
    );
    assert_eq!(
        3,
        wm.count_less_than(0, 5, usize::MAX),
        "Failed on count_less_than up to MAX"
    );

    // Act & Assert for `count_more_than`
    assert_eq!(
        3,
        wm.count_more_than(0, 5, 100),
        "Failed on count_more_than with MAX"
    );
    assert_eq!(
        2,
        wm.count_more_than(0, 5, usize::MAX),
        "Failed on count_more_than at MAX"
    );

    // Act & Assert for `count`
    assert_eq!(
        2,
        wm.count(0, 5, 1, usize::MAX),
        "Failed on count up to MAX"
    );
    assert_eq!(
        1,
        wm.count(0, 5, 100, usize::MAX),
        "Failed on count between 100 and MAX"
    );
    assert_eq!(
        2,
        wm.count(1, 4, 0, usize::MAX),
        "Failed on count in partial range with MAX"
    );
}
