pub trait Permutation<T> {
    fn next_permutation(&mut self);
    fn prev_permutation(&mut self);
}

impl<T> Permutation<T> for [T]
where
    T: Ord,
{
    fn next_permutation(&mut self) {
        if self.len() <= 1 {
            return;
        }

        if let Some(i) = self.windows(2).rposition(|s| s[0] < s[1]) {
            let j = self.iter().rposition(|x| self[i] < *x).unwrap();
            self.swap(i, j);
            self[i + 1..].reverse();
        } else {
            self.reverse();
        }
    }

    fn prev_permutation(&mut self) {
        if self.len() <= 1 {
            return;
        }

        if let Some(i) = self.windows(2).rposition(|s| s[0] > s[1]) {
            let j = self.iter().rposition(|x| self[i] > *x).unwrap();
            self.swap(i, j);
            self[i + 1..].reverse();
        } else {
            self.reverse();
        }
    }
}
