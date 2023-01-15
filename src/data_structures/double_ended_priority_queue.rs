/// verified by
/// - Library Checker | [Double-Ended Priority Queue](https://judge.yosupo.jp/problem/double_ended_priority_queue), ([submittion](https://judge.yosupo.jp/submission/121074))

use std::{cmp::Reverse, collections::BinaryHeap};

pub struct BinaryHeapRev<T> {
    pq: BinaryHeap<Reverse<T>>,
}

impl<T> BinaryHeapRev<T>
where
    T: Ord,
{
    pub fn new() -> Self {
        BinaryHeapRev {
            pq: BinaryHeap::new(),
        }
    }

    pub fn push(&mut self, x: T) {
        self.pq.push(Reverse(x));
    }

    pub fn pop(&mut self) -> Option<T> {
        if let Some(Reverse(x)) = self.pq.pop() {
            Some(x)
        } else {
            None
        }
    }

    pub fn len(&self) -> usize {
        self.pq.len()
    }

    pub fn is_empty(&self) -> bool {
        self.pq.is_empty()
    }
}

pub struct DoubleEndedPriorityQueue {
    pq_min: BinaryHeapRev<isize>,
    pq_max: BinaryHeap<isize>,
    pq_min_max: isize,
}

impl DoubleEndedPriorityQueue {
    pub fn new() -> Self {
        DoubleEndedPriorityQueue {
            pq_min: BinaryHeapRev::new(),
            pq_max: BinaryHeap::new(),
            pq_min_max: 0,
        }
    }

    pub fn push(&mut self, x: isize) {
        if self.pq_min.len() == 0 {
            self.pq_max.push(x);
        } else if self.pq_max.len() == 0 {
            self.pq_min.push(x);
        } else {
            if x <= self.pq_min_max {
                self.pq_min.push(x);
            } else {
                self.pq_max.push(x);
            }
        }
    }

    pub fn pop_min(&mut self) -> Option<isize> {
        if self.pq_min.is_empty() {
            let mut temp_max = vec![];
            for _ in 0..self.pq_max.len() / 2 {
                temp_max.push(self.pq_max.pop().unwrap());
            }
            for _ in 0..self.pq_max.len() {
                self.pq_min.push(self.pq_max.pop().unwrap());
            }
            for x in temp_max {
                self.pq_max.push(x);
            }

            if !self.pq_min.is_empty() {
                let x = self.pq_min.pq.iter().map(|Reverse(x)| *x).max().unwrap();
                self.pq_min_max = x;
            }
        }

        self.pq_min.pop()
    }

    pub fn pop_max(&mut self) -> Option<isize> {
        if self.pq_max.is_empty() {
            let mut temp_min = vec![];
            for _ in 0..self.pq_min.len() / 2 {
                temp_min.push(self.pq_min.pop().unwrap());
            }
            for _ in 0..self.pq_min.len() {
                self.pq_max.push(self.pq_min.pop().unwrap());
            }
            for x in temp_min {
                self.pq_min.push(x);
            }

            if !self.pq_min.is_empty() {
                let x = self.pq_min.pq.iter().map(|Reverse(x)| *x).max().unwrap();
                self.pq_min_max = x;
            }
        }

        self.pq_max.pop()
    }

    pub fn len(&self) -> usize {
        self.pq_min.len() + self.pq_max.len()
    }
}
