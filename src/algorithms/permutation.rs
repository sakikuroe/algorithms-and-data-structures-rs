use crate::{
    algebraic_structures::monoid::Monoid,
    data_structures::{modint::ModInt, segment_tree::SegmentTree, splay_bst::SplayBST},
};

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

/// verified by
/// - AtCoder | [Chokudai SpeedRun 001 K - 辞書順で何番目？](https://atcoder.jp/contests/chokudai_S001/tasks/chokudai_S001_k) ([submittion](https://atcoder.jp/contests/chokudai_S001/submissions/36289138))
pub trait Lexicographical {
    fn get_kth_in_lexicographical_order(&self) -> ModInt;
}

impl Lexicographical for [usize] {
    fn get_kth_in_lexicographical_order(&self) -> ModInt {
        pub struct AddMonoidMod;

        impl Monoid for AddMonoidMod {
            type S = ModInt;
            fn op(a: &Self::S, b: &Self::S) -> Self::S {
                *a + *b
            }
            fn id() -> Self::S {
                ModInt::new(0)
            }
        }

        let mut factorial = vec![ModInt::new(1)];
        for i in 1..self.len() {
            factorial.push(factorial[i - 1] * ModInt::new(i));
        }
        let mut seg = SegmentTree::<AddMonoidMod>::new(self.len());

        let mut res = ModInt::new(0);
        for i in 0..self.len() {
            res += (ModInt::new(self[i]) - seg.fold(0, self[i])) * factorial[self.len() - i - 1];
            seg.update(self[i], seg.get(self[i]) + ModInt::new(1));
        }

        res
    }
}

/// # Returns
/// k-th permutation of [0..n]
///
/// # Exsample
///
/// ```rust
/// use algorithms_and_data_structures_rs::algorithms::permutation::gen_kth_in_lexicographical_order;
/// assert_eq!(gen_kth_in_lexicographical_order(3, 0), vec![0, 1, 2]);
/// assert_eq!(gen_kth_in_lexicographical_order(3, 1), vec![0, 2, 1]);
/// assert_eq!(gen_kth_in_lexicographical_order(3, 2), vec![1, 0, 2]);
/// assert_eq!(gen_kth_in_lexicographical_order(3, 3), vec![1, 2, 0]);
/// assert_eq!(gen_kth_in_lexicographical_order(3, 4), vec![2, 0, 1]);
/// assert_eq!(gen_kth_in_lexicographical_order(3, 5), vec![2, 1, 0]);
///
/// assert_eq!(gen_kth_in_lexicographical_order(3, 6), vec![0, 1, 2]);
/// assert_eq!(gen_kth_in_lexicographical_order(3, 7), vec![0, 2, 1]);
/// assert_eq!(gen_kth_in_lexicographical_order(3, 8), vec![1, 0, 2]);
/// assert_eq!(gen_kth_in_lexicographical_order(3, 9), vec![1, 2, 0]);
/// assert_eq!(gen_kth_in_lexicographical_order(3, 10), vec![2, 0, 1]);
/// assert_eq!(gen_kth_in_lexicographical_order(3, 11), vec![2, 1, 0]);
/// ```
pub fn gen_kth_in_lexicographical_order(n: usize, mut k: usize) -> Vec<usize> {
    k %= (1..=n).product::<usize>();
    let mut bst = SplayBST::new();
    for i in 0..n {
        bst.insert(i);
    }

    let mut iter = vec![];
    for i in 1..=n {
        iter.push(k % i);
        k /= i;
    }
    iter.reverse();

    let mut res = vec![];
    for i in iter {
        let x = bst.get_nth(i).unwrap();
        res.push(x);
        bst.remove(x);
    }

    res
}
