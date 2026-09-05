//! run-length encoding および decoding を行うための trait である.

/// run-length encoding を実装するための trait である.
pub trait RunLengthEncode<T>
where
    T: Eq + Clone,
{
    /// 入力列に対して run-length encoding を行う.
    ///
    /// # Args
    /// - `self` - encoding 対象となる列への参照である.
    ///
    /// # Returns
    /// 各要素とその出現回数のタプルからなる `Vec<(T, usize)>` を返す.
    ///
    /// # Constraints
    /// 制約は, 本関数に対して指定されていない.
    ///
    /// # Panics
    /// 本関数はパニックしない.
    ///
    /// # Complexity
    /// - 時間計算量: $O(N)$
    ///   - ここで $N$ は入力された列の長さである.
    /// - 空間計算量: $O(K)$
    ///   - ここで $K$ は入力列に現れる異なる要素の種類数である.
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::string::run_length::RunLengthEncode;
    /// let data = vec!['a', 'a', 'b', 'b', 'b', 'a'];
    /// let encoded = data.run_length_encode();
    /// assert_eq!(encoded, vec![('a', 2), ('b', 3), ('a', 1)]);
    /// ```
    fn run_length_encode(&self) -> Vec<(T, usize)>;
}

impl<T> RunLengthEncode<T> for Vec<T>
where
    T: Eq + Clone,
{
    fn run_length_encode(&self) -> Vec<(T, usize)> {
        let mut res = Vec::<(T, usize)>::new();

        for x in self.iter() {
            // If the result vector is empty or the last element is different,
            // add a new tuple with the current element and a count of 1.
            if res.is_empty() || res.iter().last().unwrap().0 != *x {
                res.push((x.clone(), 1));
            } else {
                // Otherwise, increment the count of the last tuple.
                res.iter_mut().last().unwrap().1 += 1;
            }
        }

        res
    }
}

/// run-length decoding を行うための trait である.
pub trait RunLengthDecode<T>
where
    T: Eq + Clone,
{
    /// run-length encoded された列を decoding する.
    ///
    /// # Args
    /// - `self` - run-length encoded された列への参照である.
    ///
    /// # Returns
    /// decoded された列を要素のベクターとして格納した `Vec<T>` を返す.
    ///
    /// # Constraints
    /// 制約は, 本関数に対して指定されていない.
    ///
    /// # Panics
    /// 本関数はパニックしない.
    ///
    /// # Complexity
    /// - 時間計算量: $O(N_{decoded})$
    ///   - ここで $N_{decoded}$ は decoded 後の列の総要素数である.
    /// - 空間計算量: $O(N_{decoded})$
    ///   - ここで $N_{decoded}$ は decoded 後の列の総要素数である.
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::string::run_length::RunLengthDecode;
    /// let encoded_data = vec![('a', 2), ('b', 3), ('a', 1)];
    /// let decoded = encoded_data.run_length_decode();
    /// assert_eq!(decoded, vec!['a', 'a', 'b', 'b', 'b', 'a']);
    /// ```
    fn run_length_decode(&self) -> Vec<T>;
}

impl<T> RunLengthDecode<T> for Vec<(T, usize)>
where
    T: Eq + Clone,
{
    fn run_length_decode(&self) -> Vec<T> {
        let mut res = Vec::new();

        for (x, cnt) in self.iter() {
            // For each element and its count in the encoded sequence,
            // create a vector with 'cnt' copies of the element.
            for _ in 0..*cnt {
                res.push(x.clone());
            }
        }

        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // run_length_encode のテスト: 戻り値を検証する。
    mod run_length_encode {
        use super::*;

        /// Scenario: 典型的な文字列パターン (空, 単一要素, 全同一要素, 交互要素,
        /// 混合パターン) に対して、 期待通りに run-length encoding される
        /// (正常系 + 境界値)。
        /// - Given: 要素数や並び方が異なる複数の `Vec<char>` がある。
        /// - When: 各列に対して `run_length_encode` を呼ぶ。
        /// - Then: 各ケースで、 要素とその出現回数のタプル列が期待通りに返る。
        #[test]
        fn encodes_various_char_sequences() {
            // Given
            let cases = [
                (vec![], vec![]),
                (vec!['x'], vec![('x', 1)]),
                (vec!['z', 'z', 'z', 'z'], vec![('z', 4)]),
                (
                    vec!['a', 'b', 'a', 'b'],
                    vec![('a', 1), ('b', 1), ('a', 1), ('b', 1)],
                ),
                (
                    vec!['a', 'a', 'b', 'b', 'b', 'a'],
                    vec![('a', 2), ('b', 3), ('a', 1)],
                ),
                (
                    vec!['x', 'y', 'y', 'x', 'x', 'x', 'y'],
                    vec![('x', 1), ('y', 2), ('x', 3), ('y', 1)],
                ),
            ];
            // When, Then
            for (input, expected) in cases {
                let sut = input;
                let result = sut.run_length_encode();
                assert_eq!(expected, result);
            }
        }

        /// Scenario: 要素の型が整数であっても同様に encoding される (正常系)。
        /// - Given: 連続する整数の並びを持つ `Vec<i32>` がある。
        /// - When: `run_length_encode` を呼ぶ。
        /// - Then: 要素とその出現回数のタプル列が期待通りに返る。
        #[test]
        fn encodes_int_sequence() {
            // Given
            let sut = vec![1, 1, 1, 2, 2, 2, 2, 3];
            // When
            let result = sut.run_length_encode();
            // Then
            assert_eq!(vec![(1, 3), (2, 4), (3, 1)], result);
        }
    }

    // run_length_decode のテスト: 戻り値を検証する。
    mod run_length_decode {
        use super::*;

        /// Scenario: 典型的なタプル列パターン (空, 単一要素, 全同一要素, 交互要素,
        /// 混合パターン) に対して、 期待通りに run-length decoding される
        /// (正常系 + 境界値)。
        /// - Given: 要素数や並び方が異なる複数の `Vec<(char, usize)>` がある。
        /// - When: 各列に対して `run_length_decode` を呼ぶ。
        /// - Then: 各ケースで、 復元された要素列が期待通りに返る。
        #[test]
        fn decodes_various_char_sequences() {
            // Given
            let cases = [
                (vec![], vec![]),
                (vec![('x', 1)], vec!['x']),
                (vec![('z', 3)], vec!['z', 'z', 'z']),
                (
                    vec![('a', 1), ('b', 1), ('a', 1), ('b', 1)],
                    vec!['a', 'b', 'a', 'b'],
                ),
                (
                    vec![('a', 2), ('b', 3), ('a', 1)],
                    vec!['a', 'a', 'b', 'b', 'b', 'a'],
                ),
                (
                    vec![('x', 1), ('y', 2), ('x', 3), ('y', 1)],
                    vec!['x', 'y', 'y', 'x', 'x', 'x', 'y'],
                ),
            ];
            // When, Then
            for (input, expected) in cases {
                let sut = input;
                let result = sut.run_length_decode();
                assert_eq!(expected, result);
            }
        }

        /// Scenario: カウントが `0` のタプルは、 その区間を出力しない (境界値)。
        /// - Given: カウント `0` のタプルを含む `Vec<(char, usize)>` がある。
        /// - When: `run_length_decode` を呼ぶ。
        /// - Then: カウント `0` の要素が含まれない、 復元された要素列が返る。
        #[test]
        fn skips_segment_with_zero_count() {
            // Given
            let sut = vec![('a', 2), ('b', 0), ('c', 1)];
            // When
            let result = sut.run_length_decode();
            // Then
            assert_eq!(vec!['a', 'a', 'c'], result);
        }

        /// Scenario: 要素の型が整数であっても同様に decoding される
        /// (正常系 + 境界値)。
        /// - Given: 要素数が異なる複数の `Vec<(i32, usize)>` がある。
        /// - When: 各列に対して `run_length_decode` を呼ぶ。
        /// - Then: 各ケースで、 復元された要素列が期待通りに返る。
        #[test]
        fn decodes_int_sequences() {
            // Given
            let cases = [
                (vec![(1, 3), (2, 4), (3, 1)], vec![1, 1, 1, 2, 2, 2, 2, 3]),
                (vec![], vec![]),
                (vec![(99, 1)], vec![99]),
                (vec![(10, 5)], vec![10, 10, 10, 10, 10]),
            ];
            // When, Then
            for (input, expected) in cases {
                let sut = input;
                let result = sut.run_length_decode();
                assert_eq!(expected, result);
            }
        }
    }
}
