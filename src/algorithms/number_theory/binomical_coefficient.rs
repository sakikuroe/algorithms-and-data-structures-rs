//! verified by
//! - Library Checker | [](), ([submittion]())
//! - Aizu Online Judge | [](), ([submittion]())
//! - AtCoder | [AtCoder Beginner Contest 132 D - Blue and Red Balls](https://atcoder.jp/contests/abc132/tasks/abc132_d), ([submittion](https://atcoder.jp/contests/abc132/submissions/34338970))

use crate::{
    algebraic_structures::monoid::Monoid,
    data_structures::{modint::ModInt, segment_tree::SegmentTree},
};

pub struct BinomicalCoefficient {
    factorial: Vec<ModInt>,
    factorial_inv: Vec<ModInt>,
}

impl BinomicalCoefficient {
    pub fn new(max_size: usize) -> BinomicalCoefficient {
        let mut factorial = vec![ModInt::new(1)];
        for i in 1..=max_size {
            factorial.push(factorial[i - 1] * ModInt::new(i));
        }

        let mut factorial_inv = vec![ModInt::new(0); max_size + 1];
        factorial_inv[max_size] = factorial[max_size].inverse();
        for i in (0..max_size).rev() {
            factorial_inv[i] = factorial_inv[i + 1] * ModInt::new(i + 1);
        }

        BinomicalCoefficient {
            factorial,
            factorial_inv,
        }
    }

    // n!
    pub fn factorial(&self, n: usize) -> ModInt {
        self.factorial[n]
    }

    // inverse of (n!)
    pub fn factorial_inv(&self, n: usize) -> ModInt {
        self.factorial_inv[n]
    }

    // nPr
    pub fn permutation(&self, n: usize, r: usize) -> ModInt {
        self.factorial[n] / self.factorial[n - r]
    }

    // nCr
    pub fn combination(&self, n: usize, r: usize) -> ModInt {
        if n < r {
            return ModInt::new(0);
        }
        self.factorial[n] * self.factorial_inv[r] * self.factorial_inv[n - r]
    }

    // nHr
    pub fn homogeneous(&self, n: usize, r: usize) -> ModInt {
        self.combination(n + r - 1, r)
    }
}

pub struct MulMonoid;

impl Monoid for MulMonoid {
    type S = ModInt;
    fn op(a: &Self::S, b: &Self::S) -> Self::S {
        *a * *b
    }
    fn id() -> Self::S {
        ModInt::new(1)
    }
}

/// verified by
/// - AtCoder | [square869120Contest #2 F - Range Sum Queries](https://atcoder.jp/contests/s8pc-2/tasks/s8pc_2_f), ([submittion](https://atcoder.jp/contests/s8pc-2/submissions/34842436))

/// find nCr when n: large, r: small
pub struct BinomicalCoefficient2 {
    pub factorial: Vec<ModInt>,
    pub factorial_inv: Vec<ModInt>,
    pub seg: SegmentTree<MulMonoid>,
    pub max_r: usize,
    pub min_n: usize,
    pub max_n: usize,
    pub offset: usize,
}

impl BinomicalCoefficient2 {
    pub fn new(max_r: usize, min_n: usize, max_n: usize) -> BinomicalCoefficient2 {
        assert!(min_n <= max_n);
        let mut factorial = vec![ModInt::new(1)];
        for i in 1..=max_r {
            factorial.push(factorial[i - 1] * ModInt::new(i));
        }

        let mut factorial_inv = vec![factorial[max_r].inverse()];
        for i in 1..=max_r {
            factorial_inv.push(factorial_inv[i - 1] * ModInt::new(max_r + 1 - i));
        }
        factorial_inv.reverse();

        let mut seg = SegmentTree::<MulMonoid>::new(max_n - min_n + max_r);
        let offset = if min_n + 1 >= max_r {
            min_n + 1 - max_r
        } else {
            0
        };
        for i in 0..max_n - min_n + max_r {
            seg.set(i, ModInt::new(i + offset));
        }
        seg.build();

        BinomicalCoefficient2 {
            factorial,
            factorial_inv,
            seg,
            max_r,
            min_n,
            max_n,
            offset,
        }
    }

    // nPr
    pub fn permutation(&self, n: usize, r: usize) -> ModInt {
        assert!(r <= self.max_r);
        assert!(self.min_n <= n);
        assert!(n <= self.max_n);
        if n < r {
            return ModInt::new(0);
        }
        self.seg.fold(n + 1 - r - self.offset, n + 1 - self.offset)
    }

    // nCr
    pub fn combination(&self, n: usize, r: usize) -> ModInt {
        assert!(r <= self.max_r);
        assert!(self.min_n <= n);
        assert!(n <= self.max_n);
        if n < r {
            return ModInt::new(0);
        }
        self.permutation(n, r) * self.factorial_inv[r]
    }

    // nHr
    pub fn homogeneous(&self, n: usize, r: usize) -> ModInt {
        assert!(r <= self.max_r);
        assert!(self.min_n <= n);
        assert!(n <= self.max_n);
        self.combination(n + r - 1, r)
    }
}
