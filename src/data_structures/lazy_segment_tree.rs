use crate::algebraic_structures::monoid::Monoid;

pub struct LazySegmentTree<X, M>
where
    X: Monoid,
    M: Monoid,
{
    size: usize,
    data: Vec<X::S>,
    lazy: Vec<M::S>,
    action: Box<dyn Fn(X::S, M::S) -> X::S>,
}

impl<X, M> LazySegmentTree<X, M>
where
    X: Monoid,
    X::S: Clone + Copy,
    M: Monoid,
    M::S: Clone + Copy,
{
    pub fn new(size: usize, f: Box<dyn Fn(X::S, M::S) -> X::S>) -> Self {
        let size = size.next_power_of_two();
        LazySegmentTree::<X, M> {
            size,
            data: vec![X::id(); 2 * size - 1],
            lazy: vec![M::id(); 2 * size - 1],
            action: f,
        }
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn propagate(&mut self, idx: usize) {
        if idx < self.size - 1 {
            self.lazy[2 * idx + 1] = M::op(&self.lazy[2 * idx + 1], &self.lazy[idx]);
            self.lazy[2 * idx + 2] = M::op(&self.lazy[2 * idx + 2], &self.lazy[idx]);
        }
        self.data[idx] = (self.action)(self.data[idx].clone(), self.lazy[idx].clone());
        self.lazy[idx] = M::id();
    }

    fn get_index(&self, mut l: usize, mut r: usize) -> Vec<usize> {
        let mut res = vec![];
        while l % 2 == 1 {
            l /= 2;
        }
        while l > 0 {
            l = (l - 1) / 2;
            res.push(l);
        }
        while r % 2 == 0 && r > 0 {
            r = (r - 1) / 2;
        }
        while r > 0 {
            r = (r - 1) / 2;
            res.push(r);
        }
        res.reverse();
        res
    }

    pub fn update(&mut self, mut l: usize, mut r: usize, x: M::S) {
        l += self.size - 1;
        r += self.size - 1;

        for idx in self.get_index(l, r - 1) {
            self.propagate(idx);
        }

        while l < r {
            if l % 2 == 0 {
                self.lazy[l] = M::op(&self.lazy[l], &x);
            }
            if r % 2 == 0 {
                self.lazy[r - 1] = M::op(&self.lazy[r - 1], &x);
            }
            l = l / 2;
            r = (r - 1) / 2;
        }

        for idx in self.get_index(l, r - 1).into_iter().rev() {
            self.data[idx] = X::op(
                &(self.action)(self.data[2 * idx + 1], self.lazy[2 * idx + 1]),
                &(self.action)(self.data[2 * idx + 2], self.lazy[2 * idx + 2]),
            );
        }
    }

    pub fn fold(&mut self, mut l: usize, mut r: usize) -> X::S {
        l += self.size - 1;
        r += self.size - 1;
        for idx in self.get_index(l, r - 1) {
            self.propagate(idx);
        }

        let mut sum_l = X::id();
        let mut sum_r = X::id();
        while l < r {
            if l % 2 == 0 {
                self.propagate(l);
                sum_l = X::op(&sum_l, &self.data[l]);
            }
            if r % 2 == 0 {
                self.propagate(r - 1);
                sum_r = X::op(&self.data[r - 1], &sum_r);
            }
            l = l / 2;
            r = (r - 1) / 2;
        }

        X::op(&sum_l, &sum_r)
    }
}
