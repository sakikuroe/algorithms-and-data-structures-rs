use super::compress::compress;
use crate::{algebraic_structures::monoid::AddMonoid, data_structures::segment_tree::SegmentTree};
use std::{collections::VecDeque, fmt::Debug};

pub fn get_inversion_number<T>(v: &[T]) -> usize
where
    T: Ord + Clone,
{
    let v = compress(v).0;
    let mut segtree = SegmentTree::<AddMonoid>::new(v.len());
    let mut ans = 0;
    for &x in v.iter() {
        ans += segtree.fold(x + 1, segtree.len());
        segtree.update(x, segtree.get(x) + 1);
    }

    ans
}

pub fn minimum_times_of_adjacent_swaps_to_match<T>(s: &Vec<T>, t: &Vec<T>) -> usize
where
    T: Ord + Clone + Debug,
{
    {
        let mut s = s.clone();
        let mut t = t.clone();
        s.sort();
        t.sort();
        assert_eq!(s, t);
    }
    {
        let (compressed_s, _xs_s) = compress(&s);
        let mut cnt = vec![VecDeque::new(); s.len()];
        for i in 0..s.len() {
            cnt[compressed_s[i]].push_back(i);
        }
        let (compressed_t, _xs_t) = compress(&t);
        let mut v = vec![];
        for i in 0..t.len() {
            v.push(cnt[compressed_t[i]].pop_front().unwrap());
        }
        get_inversion_number(&v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_inversion_number_test() {
        let v: Vec<i32> = vec![3, 1, 4, 1, 5, 9, 2, 6];
        // 0: [3, 1, 4, 1, 5, 9, 2, 6]
        // 1: [1, 3, 4, 1, 5, 9, 2, 6]
        // 2: [1, 3, 1, 4, 5, 9, 2, 6]
        // 3: [1, 1, 3, 4, 5, 9, 2, 6]
        // 4: [1, 1, 3, 4, 5, 2, 9, 6]
        // 5: [1, 1, 3, 4, 2, 5, 9, 6]
        // 6: [1, 1, 3, 2, 4, 5, 9, 6]
        // 7: [1, 1, 2, 3, 4, 5, 9, 6]
        // 8: [1, 1, 2, 3, 4, 5, 6, 9]
        assert_eq!(8, get_inversion_number(&v));
    }
}

