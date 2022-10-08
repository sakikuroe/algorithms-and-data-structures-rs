//! verified by
//! - Library Checker | [](), ([submittion]())
//! - Aizu Online Judge | [](), ([submittion]())
//! - AtCoder | [](), ([submittion]())

use std::ops::Bound;

pub trait BinarySearch<T> {
    fn lower_bound(&self, key: &T) -> usize;
    fn upper_bound(&self, key: &T) -> usize;
    fn count(&self, l: Bound<T>, r: Bound<T>) -> usize;
}

impl<T> BinarySearch<T> for [T]
where
    T: Ord,
{
    fn lower_bound(&self, key: &T) -> usize {
        let mut ng = -1_isize;
        let mut ok = self.len() as isize;
        while ok - ng > 1 {
            let mid = (ok + ng) / 2;
            if *key <= self[mid as usize] {
                ok = mid;
            } else {
                ng = mid;
            }
        }
        ok as usize
    }

    fn upper_bound(&self, key: &T) -> usize {
        let mut ng = -1_isize;
        let mut ok = self.len() as isize;
        while ok - ng > 1 {
            let mid = (ok + ng) / 2;
            if *key < self[mid as usize] {
                ok = mid;
            } else {
                ng = mid;
            }
        }
        ok as usize
    }

    fn count(&self, l: Bound<T>, r: Bound<T>) -> usize {
        match (l, r) {
            (Bound::Included(l), Bound::Included(r)) => self.upper_bound(&r) - self.lower_bound(&l),
            (Bound::Included(l), Bound::Excluded(r)) => self.lower_bound(&r) - self.lower_bound(&l),
            (Bound::Included(l), Bound::Unbounded) => self.len() - self.lower_bound(&l),
            (Bound::Excluded(l), Bound::Included(r)) => self.upper_bound(&r) - self.upper_bound(&l),
            (Bound::Excluded(l), Bound::Excluded(r)) => self.lower_bound(&r) - self.upper_bound(&l),
            (Bound::Excluded(l), Bound::Unbounded) => self.len() - self.upper_bound(&l),
            (Bound::Unbounded, Bound::Included(r)) => self.upper_bound(&r),
            (Bound::Unbounded, Bound::Excluded(r)) => self.lower_bound(&r),
            (Bound::Unbounded, Bound::Unbounded) => self.len(),
        }
    }
}
