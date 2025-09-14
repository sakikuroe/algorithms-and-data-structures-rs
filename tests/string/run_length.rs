use anmitsu::string::run_length::{RunLengthDecode as _, RunLengthEncode as _};

// === RunLengthEncode Tests ===

// Tests run-length encoding with an empty vector.
#[test]
fn run_length_encode_empty_vector_returns_empty() {
    // Arrange
    let data: Vec<char> = Vec::new();

    // Act
    let encoded = data.run_length_encode();

    // Assert
    assert_eq!(Vec::<(char, usize)>::new(), encoded);
}

// Tests run-length encoding with a single element vector.
#[test]
fn run_length_encode_single_element_vector_returns_single_tuple() {
    // Arrange
    let data = vec!['x'];

    // Act
    let encoded = data.run_length_encode();

    // Assert
    assert_eq!(vec![('x', 1)], encoded);
}

// Tests run-length encoding with all identical elements.
#[test]
fn run_length_encode_all_identical_elements_returns_single_tuple_with_total_count() {
    // Arrange
    let data = vec!['z', 'z', 'z', 'z'];

    // Act
    let encoded = data.run_length_encode();

    // Assert
    assert_eq!(vec![('z', 4)], encoded);
}

// Tests run-length encoding with alternating elements.
#[test]
fn run_length_encode_alternating_elements_returns_alternating_tuples() {
    // Arrange
    let data = vec!['a', 'b', 'a', 'b'];

    // Act
    let encoded = data.run_length_encode();

    // Assert
    assert_eq!(vec![('a', 1), ('b', 1), ('a', 1), ('b', 1)], encoded);
}

// Tests run-length encoding with character sequences containing mixed elements.
#[test]
fn run_length_encode_char_mixed_elements_returns_correct_encoding() {
    // Arrange
    let cases = vec![
        // Verify encoding of a sequence with alternating characters
        // and varying run lengths.
        (
            vec!['a', 'a', 'b', 'b', 'b', 'a'],
            vec![('a', 2), ('b', 3), ('a', 1)],
        ),
        // Confirm encoding of a sequence with more complex character patterns.
        (
            vec!['x', 'y', 'y', 'x', 'x', 'x', 'y'],
            vec![('x', 1), ('y', 2), ('x', 3), ('y', 1)],
        ),
    ];

    for (input_data, expected_output) in cases {
        // Act
        let encoded = input_data.run_length_encode();

        // Assert
        assert_eq!(
            expected_output, encoded,
            "Failed encoding characters: {:?}",
            input_data
        );
    }
}

// Tests run-length encoding with integer sequences containing mixed elements.
#[test]
fn run_length_encode_int_mixed_elements_returns_correct_encoding() {
    // Arrange
    let cases = vec![
        // Verify encoding of a sequence with varying integer runs.
        (vec![1, 1, 1, 2, 2, 2, 2, 3], vec![(1, 3), (2, 4), (3, 1)]),
    ];

    for (input_data, expected_output) in cases {
        // Act
        let encoded = input_data.run_length_encode();

        // Assert
        assert_eq!(
            expected_output, encoded,
            "Failed encoding integers: {:?}",
            input_data
        );
    }
}

// === RunLengthDecode Tests ===

// Tests run-length decoding with an empty encoded vector.
#[test]
fn run_length_decode_empty_encoded_vector_returns_empty() {
    // Arrange
    let encoded_data: Vec<(char, usize)> = Vec::new();

    // Act
    let decoded = encoded_data.run_length_decode();

    // Assert
    assert_eq!(Vec::<char>::new(), decoded);
}

// Tests run-length decoding with a single tuple (element, count 1).
#[test]
fn run_length_decode_single_tuple_count_one_returns_single_element() {
    // Arrange
    let encoded_data = vec![('x', 1)];

    // Act
    let decoded = encoded_data.run_length_decode();

    // Assert
    assert_eq!(vec!['x'], decoded);
}

// Tests run-length decoding with a single tuple (element, count > 1).
#[test]
fn run_length_decode_single_tuple_multiple_counts_returns_repeated_elements() {
    // Arrange
    let encoded_data = vec![('z', 3)];

    // Act
    let decoded = encoded_data.run_length_decode();

    // Assert
    assert_eq!(vec!['z', 'z', 'z'], decoded);
}

// Tests run-length decoding with alternating tuples.
#[test]
fn run_length_decode_alternating_tuples_returns_alternating_elements() {
    // Arrange
    let encoded_data = vec![('a', 1), ('b', 1), ('a', 1), ('b', 1)];

    // Act
    let decoded = encoded_data.run_length_decode();

    // Assert
    assert_eq!(vec!['a', 'b', 'a', 'b'], decoded);
}

// Tests run-length decoding with a zero count tuple.
// This is an edge case that `encode` wouldn't produce, but `decode` should handle.
#[test]
fn run_length_decode_tuple_with_zero_count_returns_empty_for_that_segment() {
    // Arrange
    let encoded_data = vec![('a', 2), ('b', 0), ('c', 1)];

    // Act
    let decoded = encoded_data.run_length_decode();

    // Assert
    assert_eq!(vec!['a', 'a', 'c'], decoded);
}

// Tests run-length decoding with various character sequences using a table-driven approach.
#[test]
fn run_length_decode_char_elements_returns_correct_decoding() {
    // Arrange
    let cases = vec![
        // Case: Standard mixed sequence
        (
            vec![('a', 2), ('b', 3), ('a', 1)],
            vec!['a', 'a', 'b', 'b', 'b', 'a'],
        ),
        // Case: Complex sequence
        (
            vec![('x', 1), ('y', 2), ('x', 3), ('y', 1)],
            vec!['x', 'y', 'y', 'x', 'x', 'x', 'y'],
        ),
        // Boundary Case: Empty sequence
        (vec![], vec![]),
        // Boundary Case: Single element sequence
        (vec![('c', 1)], vec!['c']),
        // Boundary Case: Single element with multiple count
        (vec![('d', 4)], vec!['d', 'd', 'd', 'd']),
    ];

    for (input_encoded, expected_decoded) in cases {
        // Act
        let decoded = input_encoded.run_length_decode();

        // Assert
        assert_eq!(
            expected_decoded, decoded,
            "Failed decoding {:?} for char type",
            input_encoded
        );
    }
}

// Tests run-length decoding with various integer sequences using a table-driven approach.
#[test]
fn run_length_decode_int_elements_returns_correct_decoding() {
    // Arrange
    let cases = vec![
        // Case: Standard integer sequence with longer runs
        (vec![(1, 3), (2, 4), (3, 1)], vec![1, 1, 1, 2, 2, 2, 2, 3]),
        // Boundary Case: Empty sequence
        (vec![], vec![]),
        // Boundary Case: Single element sequence
        (vec![(99, 1)], vec![99]),
        // Boundary Case: Single element with multiple count
        (vec![(10, 5)], vec![10, 10, 10, 10, 10]),
    ];

    for (input_encoded, expected_decoded) in cases {
        // Act
        let decoded = input_encoded.run_length_decode();

        // Assert
        assert_eq!(
            expected_decoded, decoded,
            "Failed decoding {:?} for integer type",
            input_encoded
        );
    }
}
