//! A trait for run-length encoding a sequence.
//! run-length encoding および decoding を行うための trait である.

/// A trait for run-length encode.
/// run-length encoding を実装するための trait である.
pub trait RunLengthEncode<T>
where
    T: Eq + Clone,
{
    /// Performs run-length encoding on a inputed sequence.
    /// 入力列に対して run-length encoding を行う.
    ///
    /// # Args
    /// - `self`: A reference to the sequence to be encoded.
    ///           encoding 対象となる列への参照である.
    ///
    /// # Returns
    /// `Vec<(T, usize)>`: Returns a vector of tuples, where each tuple contains an element and its frequency.
    ///                    各要素とその出現回数のタプルからなるベクターを返す.
    ///
    /// # Constraints
    /// Constraints are not specified for this function.
    /// 制約は, 本関数に対して指定されていない.
    ///
    /// # Panics
    /// This function does not panic.
    /// 本関数はパニックしない.
    ///
    /// # Complexity
    /// - Time complexity: O(N), where N is the length of the input sequence.
    ///                          ここで N は入力された列の長さである.
    /// - Space complexity: O(K), where K is the number of distinct elements in the input.
    ///                           ここで K は入力列に現れる異なる要素の種類数である.
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

/// A trait for run-length decoding a sequence.
/// run-length decoding を行うための trait である.
pub trait RunLengthDecode<T>
where
    T: Eq + Clone,
{
    /// Decodes a run-length encoded sequence.
    /// run-length encoded された列を decoding する.
    ///
    /// # Args
    /// - `self`: A reference to the run-length encoded sequence.
    ///           run-length encoded された列への参照である.
    ///
    /// # Returns
    /// `Vec<T>`: Returns the decoded sequence as a vector of elements.
    ///           decoded された列を要素のベクターとして返す.
    ///
    /// # Constraints
    /// Constraints are not specified for this function.
    /// 制約は, 本関数に対して指定されていない.
    ///
    /// # Panics
    /// This function does not panic.
    /// 本関数はパニックしない.
    ///
    /// # Complexity
    /// - Time complexity: O(N_decoded), where N_decoded is the total number of elements in the decoded sequence.
    ///                    ここで N_decoded は decoded 後の列の総要素数である.
    /// - Space complexity: O(N_decoded), where N_decoded is the total number of elements in the decoded sequence.
    ///                     ここで N_decoded は decoded 後の列の総要素数である.
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
