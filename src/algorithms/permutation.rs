pub trait Permutation<T> {
    fn next_permutation(&mut self);
    fn prev_permutation(&mut self);
}

impl<T> Permutation<T> for [T]
where
    T: Ord,
{
    fn next_permutation(&mut self) {
        if self.len() == 0 {
            return;
        }

        if let Some(i) = (0..self.len() - 1).rev().find(|&i| self[i] < self[i + 1]) {
            let j = (0..self.len()).rev().find(|&j| self[i] < self[j]).unwrap();
            self.swap(i, j);
            self[i + 1..].reverse();
        } else {
            self.reverse();
        }
    }

    fn prev_permutation(&mut self) {
        if self.len() == 0 {
            return;
        }

        if let Some(i) = (0..self.len() - 1).rev().find(|&i| self[i] > self[i + 1]) {
            let j = (0..self.len()).rev().find(|&j| self[i] > self[j]).unwrap();
            self.swap(i, j);
            self[i + 1..].reverse();
        } else {
            self.reverse();
        }
    }
}
