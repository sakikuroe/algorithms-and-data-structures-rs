use std::{cmp::Ordering, ops};

#[cfg_attr(doc, katexit::katexit)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ComplexNumber<T> {
    pub re: T,
    pub im: T,
}

impl<T> ComplexNumber<T>
where
    T: Clone + Copy,
{
    pub fn new(re: T, im: T) -> Self {
        ComplexNumber { re, im }
    }

    pub fn re(&self) -> T {
        self.re
    }

    pub fn im(&self) -> T {
        self.im
    }
}

impl ops::Add for ComplexNumber<isize> {
    type Output = ComplexNumber<isize>;
    fn add(self, other: Self) -> Self {
        ComplexNumber::new(self.re + other.re, self.im + other.im)
    }
}

impl ops::Sub for ComplexNumber<isize> {
    type Output = ComplexNumber<isize>;
    fn sub(self, other: Self) -> Self {
        ComplexNumber::new(self.re - other.re, self.im - other.im)
    }
}

impl ops::Mul for ComplexNumber<isize> {
    type Output = ComplexNumber<isize>;
    fn mul(self, other: Self) -> Self {
        ComplexNumber::new(
            self.re * other.re - self.im * other.im,
            self.re * other.im + self.im * other.re,
        )
    }
}

impl ops::AddAssign for ComplexNumber<isize> {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl ops::SubAssign for ComplexNumber<isize> {
    fn sub_assign(&mut self, other: Self) {
        *self = *self - other;
    }
}

impl ops::MulAssign for ComplexNumber<isize> {
    fn mul_assign(&mut self, other: Self) {
        *self = *self * other;
    }
}

impl ComplexNumber<isize> {
    pub fn conj(&self) -> Self {
        ComplexNumber::new(self.re, -self.im)
    }

    pub fn pow(&self, mut n: usize) -> Self {
        let mut res = ComplexNumber::new(1, 0);
        let mut x = *self;
        while n > 0 {
            if n & 1 == 1 {
                res *= x;
            }
            x *= x;
            n >>= 1;
        }
        res
    }

    /// Chooses the branch of $z \in \mathbb{C}$ that makes $\mathrm{Arg}(z) \in (−\pi, \pi]$.
    ///
    /// verified by
    /// - Library Checker | [Sort Points by Argument](https://judge.yosupo.jp/problem/sort_points_by_argument) ([submittion](https://judge.yosupo.jp/submission/100649))
    /// - AtCoder | [AtCoder Beginner Contest 033 D - 三角形の分類](https://atcoder.jp/contests/abc033/tasks/abc033_d) ([submittion](https://atcoder.jp/contests/abc033/submissions/34133404))
    /// - AtCoder | [AtCoder Beginner Contest 139 F - Engines](https://atcoder.jp/contests/abc139/tasks/abc139_f) ([submittion](https://atcoder.jp/contests/abc139/submissions/34133418))
    pub fn argument_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.re == 0 && self.im == 0 || other.re == 0 && other.im == 0 {
            None
        } else if ((self.im == 0 && self.re > 0) || self.im < 0)
            && ((other.im == 0 && other.re < 0) || other.im > 0)
        {
            Some(Ordering::Less)
        } else if ((self.im == 0 && self.re < 0) || self.im > 0)
            && ((other.im == 0 && other.re > 0) || other.im < 0)
        {
            Some(Ordering::Greater)
        } else {
            let cross = self.re * other.im - other.re * self.im;
            Some(0.cmp(&cross))
        }
    }

    pub fn abs_cmp(&self, other: &Self) -> Ordering {
        (self.re * self.re + self.im * self.im).cmp(&(other.re * other.re + other.im * other.im))
    }
}

pub fn cross_product(ca: ComplexNumber<isize>, cb: ComplexNumber<isize>) -> isize {
    ca.re() * cb.im() - ca.im() * cb.re()
}

pub fn inner_product(ca: ComplexNumber<isize>, cb: ComplexNumber<isize>) -> isize {
    ca.re() * cb.re() + ca.im() * cb.im()
}

pub fn distance(ca: ComplexNumber<f64>, cb: ComplexNumber<f64>) -> f64 {
    ((ca.re() - cb.re()).powf(2.0) + (ca.im() - cb.im()).powf(2.0)).powf(0.5)
}

pub fn on_the_same_line(
    c1: ComplexNumber<isize>,
    c2: ComplexNumber<isize>,
    c3: ComplexNumber<isize>,
) -> bool {
    cross_product(c2 - c1, c3 - c1) == 0
}
