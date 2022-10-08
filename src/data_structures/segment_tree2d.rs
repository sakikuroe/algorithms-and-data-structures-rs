//! verified by
//! - AtCoder | [トヨタシステムズプログラミングコンテスト2021(AtCoder Beginner Contest 228)](https://atcoder.jp/contests/abc228) ([submittion](https://atcoder.jp/contests/abc228/submissions/35240127))

use crate::algebraic_structures::monoid::Monoid;

pub struct SegmentTree2D<M>
where
    M: Monoid,
{
    h: usize,
    w: usize,
    data: Vec<M::S>,
}

impl<M> SegmentTree2D<M>
where
    M: Monoid,
    M::S: Clone,
{
    pub fn new(h: usize, w: usize) -> Self {
        SegmentTree2D::<M> {
            h,
            w,
            data: vec![M::id(); 4 * h * w],
        }
    }

    pub fn set(&mut self, (mut i, mut j): (usize, usize), x: M::S) {
        let self_w = self.w;
        let idx = |i: usize, j: usize| 2 * i * self_w + j;
        i += self.h;
        j += self.w;
        self.data[idx(i, j)] = x;
    }

    pub fn build(&mut self) {
        let self_w = self.w;
        let idx = |i: usize, j: usize| 2 * i * self_w + j;
        for j in self.w..2 * self.w {
            for i in (0..self.h).rev() {
                self.data[idx(i, j)] =
                    M::op(&self.data[idx(2 * i, j)], &self.data[idx(2 * i + 1, j)]);
            }
        }
        for i in 0..2 * self.h {
            for j in (0..self.w).rev() {
                self.data[idx(i, j)] =
                    M::op(&self.data[idx(i, 2 * j)], &self.data[idx(i, 2 * j + 1)]);
            }
        }
    }

    pub fn update(&mut self, (mut i, mut j): (usize, usize), x: M::S) {
        let self_w = self.w;
        let idx = |i: usize, j: usize| 2 * i * self_w + j;
        i += self.h;
        j += self.w;
        self.data[idx(i, j)] = x;
        {
            let mut i = i;
            while i > 0 {
                i /= 2;
                self.data[idx(i, j)] =
                    M::op(&self.data[idx(2 * i, j)], &self.data[idx(2 * i + 1, j)]);
            }
        }
        {
            while i > 0 {
                let mut j = j;
                while j > 0 {
                    j /= 2;
                    self.data[idx(i, j)] =
                        M::op(&self.data[idx(i, 2 * j)], &self.data[idx(i, 2 * j + 1)]);
                }
                i /= 2;
            }
        }
    }

    pub fn get(&self, (mut i, mut j): (usize, usize)) -> M::S {
        let self_w = self.w;
        let idx = |i: usize, j: usize| 2 * i * self_w + j;
        i += self.h;
        j += self.w;
        self.data[idx(i, j)].clone()
    }

    pub fn fold(&self, (mut i1, mut j1): (usize, usize), (mut i2, mut j2): (usize, usize)) -> M::S {
        let self_w = self.w;
        let idx = |i: usize, j: usize| 2 * i * self_w + j;

        let sub = |h: usize, mut w1: usize, mut w2: usize| {
            let mut sum_l = M::id();
            let mut sum_r = M::id();
            while w1 < w2 {
                if w1 % 2 == 1 {
                    sum_l = M::op(&sum_l, &self.data[idx(h, w1)]);
                    w1 += 1;
                }
                if w2 % 2 == 1 {
                    w2 -= 1;
                    sum_r = M::op(&self.data[idx(h, w2)], &sum_r);
                }
                w1 /= 2;
                w2 /= 2;
            }
            return M::op(&sum_l, &sum_r);
        };

        i1 += self.h;
        j1 += self.w;
        i2 += self.h;
        j2 += self.w;

        let mut sum_l = M::id();
        let mut sum_r = M::id();

        while i1 < i2 {
            if i1 % 2 == 1 {
                sum_l = M::op(&sum_l, &sub(i1, j1, j2));
                i1 += 1;
            }
            if i2 % 2 == 1 {
                i2 -= 1;
                sum_r = M::op(&sub(i2, j1, j2), &sum_r);
            }
            i1 /= 2;
            i2 /= 2;
        }

        M::op(&sum_l, &sum_r)
    }
}
