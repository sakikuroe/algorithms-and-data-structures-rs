/// verified by
/// - AtCoder | [AtCoder Beginner Contest 134 C - Exception Handling](https://atcoder.jp/contests/abc134/tasks/abc134_c), ([submittion](https://atcoder.jp/contests/abc134/submissions/34363132))
/// - Library Checker | [Point Add Range Sum](https://judge.yosupo.jp/problem/point_add_range_sum), ([submittion](https://judge.yosupo.jp/submission/121100))

use crate::algebraic_structures::monoid::Monoid;

#[cfg_attr(doc, katexit::katexit)]
/// Let $\\{a_{i}\\}_{i=1}^{N}$ be a sequence of type Monoid::S.
pub struct SegmentTree<M>
where
    M: Monoid,
{
    size: usize,
    data: Vec<M::S>,
}

impl<M> SegmentTree<M>
where
    M: Monoid,
    M::S: Clone,
{
    /// Creates a segment tree with $\\{a_{i}\\}_{i=1}^{N}$ inside.
    /// n: lenght of $\\{a_{i}\\}_{i=1}^{N}$ (i.e. N)
    pub fn new(n: usize) -> Self {
        let size = n.next_power_of_two();
        SegmentTree::<M> {
            size,
            data: vec![M::id(); 2 * size - 1],
        }
    }

    /// Returns lenght of $\\{a_{i}\\}_{i=1}^{N}$ (i.e. Return N)
    pub fn len(&self) -> usize {
        self.size
    }

    /// Sets x to $a_{idx}$.
    pub fn set(&mut self, mut idx: usize, x: M::S) {
        idx += self.size - 1;
        self.data[idx] = x;
    }

    /// Builds segment tree.
    pub fn build(&mut self) {
        for idx in (0..self.size - 1).rev() {
            self.data[idx] = M::op(&self.data[2 * idx + 1], &self.data[2 * idx + 2]);
        }
    }

    /// Updates $a_{idx}$ to x.
    pub fn update(&mut self, mut idx: usize, x: M::S) {
        idx += self.size - 1;
        self.data[idx] = x;
        while idx > 0 {
            idx = (idx - 1) / 2;
            self.data[idx] = M::op(&self.data[2 * idx + 1], &self.data[2 * idx + 2]);
        }
    }

    /// Returns $a_{idx}$.
    pub fn get(&self, mut idx: usize) -> M::S {
        idx += self.size - 1;
        self.data[idx].clone()
    }

    /// Returns the result (fold op $\left[a_{l}, ... ,a_{r}\right)).$
    /// (i.e. Return $a_{l} (op) a_{l + 1} (op) \cdots (op) a_{r-1})$
    /// Notice that this is a half-opened section.
    pub fn fold(&self, mut l: usize, mut r: usize) -> M::S {
        l += self.size - 1;
        r += self.size - 1;

        let mut sum_l = M::id();
        let mut sum_r = M::id();

        while l < r {
            if l % 2 == 0 {
                sum_l = M::op(&sum_l, &self.data[l]);
            }
            if r % 2 == 0 {
                sum_r = M::op(&self.data[r - 1], &sum_r);
            }
            l = l / 2;
            r = (r - 1) / 2;
        }

        M::op(&sum_l, &sum_r)
    }
}

impl<M> SegmentTree<M>
where
    M: Monoid,
    M::S: Clone + Ord,
{
    pub fn lower_bound(&mut self, x: M::S) -> usize {
        let mut ng = -1 as isize;
        let mut ok = self.size as isize;
        while ok - ng > 1 {
            let mid = (ng + ok) / 2;
            if x <= self.fold(0, mid as usize + 1) {
                ok = mid;
            } else {
                ng = mid;
            }
        }
        ok as usize
    }
}
