//! A trait for run-length encoding a sequence.
//! Run-length encode と decode に関する trait。

/// A trait for run-length encode.
/// Run-length encode の trait.
pub trait RunLengthEncode<T>
where
    T: Eq + Clone,
{
    /// Performs run-length encoding on a inputed sequence.
    /// 入力列を run-length encode する。
    ///
    /// # Args
    /// - `self`: A reference to the sequence to be encoded.
    ///           Encode 対象の列への参照。
    ///
    /// # Returns
    ///
    /// `Vec<(T, usize)>`: Returns a vector of tuples, where each tuple contains an element and its frequency.
    ///                    要素とその出現回数のタプルのベクタを返す。
    ///
    /// # Complexity
    ///
    /// - Time complexity: O(N),
    ///                    O(N)、ここで N は入力された列の長さ。
    /// - Space complexity: O(K), where K is
    ///                     O(K)、ここで K は入力列に現れる要素の種類。
    ///
    /// # Examples
    /// ```
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
        let mut res: Vec<(T, usize)> = Vec::new();

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

pub trait RunLengthDecode<T>
where
    T: Eq + Clone,
{
    /// ```
    /// use anmitsu::string::runlength::RunLengthDecode;
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
        let mut res: Vec<T> = Vec::new();

        for (x, cnt) in self.iter() {
            res.append(&mut vec![x.clone(); *cnt]);
        }

        res
    }
}
