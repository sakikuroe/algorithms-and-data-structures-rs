use crate::data_structures::modint::ModInt;
use std::ops;

const MOD: usize = 1000000007;

#[allow(unused_macros)]
pub mod macros {
    macro_rules! min { ($x: expr) => { $x }; ($x: expr, $($xs: expr),+) => {{ let y = macros::min!($($xs),+); std::cmp::min($x, y) } }}
    macro_rules! max { ($x: expr) => { $x }; ($x: expr, $($xs: expr),+) => {{ let y = macros::max!($($xs),+); std::cmp::max($x, y) } }}
    macro_rules! chmin { ($x: expr, $($xs: expr),+) => {{ let y = macros::min!($($xs),+); if $x > y { $x = y; true } else { false } }}}
    macro_rules! chmax { ($x: expr, $($xs: expr),+) => {{ let y = macros::max!($($xs),+); if $x < y { $x = y; true } else { false } }}}
    macro_rules! multi_vec { ($element: expr; ($len: expr, $($lens: expr),*)) => ( vec![macros::multi_vec![$element; ($($lens),*)]; $len] ); ($element: expr; ($len: expr)) => ( vec![$element; $len] ); }
    macro_rules! multi_box_array { ($element: expr; ($len: expr, $($lens: expr),*)) => ( vec![macros::multi_box_array![$element; ($($lens),*)]; $len].into_boxed_slice() ); ($element: expr; ($len: expr)) => ( vec![$element; $len].into_boxed_slice() ); }
    #[allow(unused_imports)]
    pub(super) use {chmax, chmin, max, min, multi_box_array, multi_vec};
}

#[derive(Clone)]
pub struct Matrix {
    value: Box<[Box<[ModInt]>]>,
    shape: (usize, usize),
}

impl Matrix {
    pub fn new(value: Box<[Box<[usize]>]>) -> Matrix {
        let shape = (value.len(), value[0].len());
        let value = {
            let mut res = macros::multi_box_array!(ModInt::new(0); (value.len(), value[0].len()));
            for i in 0..value.len() {
                for j in 0..value[0].len() {
                    res[i][j] = ModInt::new(value[i][j]);
                }
            }
            res
        };
        Matrix { value, shape }
    }

    pub fn pow(&self, mut n: usize) -> Matrix {
        let mut res = eye(self.value.len());
        let mut x = self.clone();
        while n > 0 {
            if n % 2 == 1 {
                res *= x.clone();
            }
            x *= x.clone();
            n /= 2;
        }
        res
    }

    pub fn det(&self) -> ModInt {
        let mut cnt = 0;
        let mut u = self.clone();
        let n = u.value.len();
        for i in 0..n {
            if u.value[i][i].value() == 0 {
                for j in i + 1..n {
                    if u.value[j][i].value() != 0 {
                        unsafe {
                            (&mut u.value[i] as *mut Box<[ModInt]>)
                                .swap(&mut u.value[j] as *mut Box<[ModInt]>);
                        }
                        cnt += 1;
                        break;
                    }
                    if j == n - 1 {
                        return ModInt::new(0);
                    }
                }
            }

            for j in i + 1..n {
                let b = u.value[j][i] / u.value[i][i];
                for k in i..n {
                    u.value[j][k] -= b * u.value[i][k];
                }
            }
        }

        (0..n)
            .map(|i| u.value[i][i])
            .fold(ModInt::new(1), |x, y| x * y)
            * if cnt % 2 == 0 {
                ModInt::new(1)
            } else {
                ModInt::new(MOD - 1)
            }
    }
}

fn eye(n: usize) -> Matrix {
    let mut res = macros::multi_box_array!(ModInt::new(0); (n, n));
    for i in 0..n {
        res[i][i] = ModInt::new(1);
    }
    Matrix {
        value: res,
        shape: (n, n),
    }
}

pub fn zero(shape: (usize, usize)) -> Matrix {
    let res = macros::multi_box_array!(ModInt::new(0); (shape.0, shape.1));
    Matrix { value: res, shape }
}

pub fn inv(mut a: Matrix) -> Option<Matrix> {
    assert_eq!(a.shape.0, a.shape.1);
    let n = a.shape.0;
    let mut b = eye(n);

    for i in 0..n {
        for i1 in i.. {
            if i1 == n {
                return None;
            }
            if a.value[i1][i] != ModInt::new(0) {
                a.value.swap(i, i1);
                b.value.swap(i, i1);
                break;
            }
        }
        let temp = a.value[i][i];
        for j in 0..n {
            a.value[i][j] /= temp;
            b.value[i][j] /= temp;
        }
        for i1 in (0..n).filter(|i1| *i1 != i) {
            let temp = a.value[i1][i];
            for j in 0..n {
                a.value[i1][j] = a.value[i1][j] - temp * a.value[i][j];
                b.value[i1][j] = b.value[i1][j] - temp * b.value[i][j];
            }
        }
    }

    Some(b)
}

impl ops::Add for Matrix {
    type Output = Matrix;
    fn add(self, other: Self) -> Self {
        let a = &self.value;
        let b = &other.value;

        let mut c = macros::multi_box_array!(ModInt::new(0); (a.len(), a[0].len()));

        for i in 0..a.len() {
            for j in 0..a[0].len() {
                c[i][j] = a[i][j] + b[i][j];
            }
        }

        Matrix {
            value: c,
            shape: self.shape,
        }
    }
}

impl ops::Sub for Matrix {
    type Output = Matrix;
    fn sub(self, other: Self) -> Self {
        let a = &self.value;
        let b = &other.value;

        let mut c = macros::multi_box_array!(ModInt::new(0); (a.len(), a[0].len()));

        for i in 0..a.len() {
            for j in 0..a[0].len() {
                c[i][j] = a[i][j] - b[i][j];
            }
        }

        Matrix {
            value: c,
            shape: self.shape,
        }
    }
}

impl ops::Mul for Matrix {
    type Output = Matrix;
    fn mul(self, other: Self) -> Self {
        assert_eq!(self.shape.1, other.shape.0);
        let a = &self.value;
        let b = &other.value;
        let l = a.len();
        let n = b[0].len();

        let mut c = macros::multi_box_array!(ModInt::new(0); (l, n));

        for (ci, ai) in c.iter_mut().zip(a.iter()) {
            for (aij, bj) in ai.iter().zip(b.iter()) {
                ci.iter_mut()
                    .zip(bj.iter())
                    .for_each(|(x, y)| *x += *aij * *y);
            }
        }

        Matrix {
            value: c,
            shape: (l, n),
        }
    }
}

impl ops::AddAssign for Matrix {
    fn add_assign(&mut self, other: Self) {
        *self = self.clone() + other;
    }
}

impl ops::SubAssign for Matrix {
    fn sub_assign(&mut self, other: Self) {
        *self = self.clone() - other;
    }
}

impl ops::MulAssign for Matrix {
    fn mul_assign(&mut self, other: Self) {
        *self = self.clone() * other;
    }
}
