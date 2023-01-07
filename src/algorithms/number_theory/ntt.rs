//! verified by
//! - Library Checker | [Convolution](https://judge.yosupo.jp/problem/convolution_mod) ([submittion](https://judge.yosupo.jp/submission/119138))

use std::{fmt, ops};

const MOD: usize = 998244353; // 119 * (1 << 23) + 1
const RANK: usize = 23;
const PRIMITIVE_ROOT: usize = 3;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ModInt {
    value: usize,
}

impl ModInt {
    pub fn new(value: usize) -> ModInt {
        ModInt { value: value % MOD }
    }

    pub fn value(&self) -> usize {
        self.value
    }

    pub fn inverse(&self) -> ModInt {
        // (a, b) -> (x, y) s.t. a * x + b * y = gcd(a, b)
        fn extended_gcd(a: isize, b: isize) -> (isize, isize) {
            if (a, b) == (1, 0) {
                (1, 0)
            } else {
                let (x, y) = extended_gcd(b, a % b);
                (y, x - (a / b) * y)
            }
        }

        let (x, _y) = extended_gcd(self.value() as isize, MOD as isize);
        ModInt::new((MOD as isize + x) as usize)
    }

    // a^n
    pub fn pow(&self, mut n: usize) -> ModInt {
        let mut res = ModInt::new(1);
        let mut x = *self;
        while n > 0 {
            if n % 2 == 1 {
                res = res * x;
            }
            x = x * x;
            n /= 2;
        }

        res
    }
}

impl ops::Add for ModInt {
    type Output = ModInt;
    fn add(self, other: Self) -> Self {
        ModInt::new(self.value + other.value)
    }
}

impl ops::Sub for ModInt {
    type Output = ModInt;
    fn sub(self, other: Self) -> Self {
        ModInt::new(MOD + self.value - other.value)
    }
}

impl ops::Mul for ModInt {
    type Output = ModInt;
    fn mul(self, other: Self) -> Self {
        ModInt::new(self.value * other.value)
    }
}

impl ops::Div for ModInt {
    type Output = ModInt;
    fn div(self, other: Self) -> Self {
        self * other.inverse()
    }
}

impl ops::AddAssign for ModInt {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl ops::SubAssign for ModInt {
    fn sub_assign(&mut self, other: Self) {
        *self = *self - other;
    }
}

impl ops::MulAssign for ModInt {
    fn mul_assign(&mut self, other: Self) {
        *self = *self * other;
    }
}

impl ops::DivAssign for ModInt {
    fn div_assign(&mut self, other: Self) {
        *self = *self / other;
    }
}

impl fmt::Display for ModInt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value())
    }
}

pub struct FftCache {
    rate: Vec<ModInt>,
    irate: Vec<ModInt>,
}

impl FftCache {
    pub fn new() -> Self {
        let mut root = vec![ModInt::new(0); RANK + 1];
        let mut iroot = vec![ModInt::new(0); RANK + 1];
        let mut rate = vec![ModInt::new(0); RANK - 1];
        let mut irate = vec![ModInt::new(0); RANK - 1];

        root[RANK] = ModInt::new(PRIMITIVE_ROOT).pow((MOD - 1) >> RANK);
        iroot[RANK] = root[RANK].inverse();
        for i in (0..RANK).rev() {
            root[i] = root[i + 1] * root[i + 1];
            iroot[i] = iroot[i + 1] * iroot[i + 1];
        }

        {
            let mut prod = ModInt::new(1);
            let mut iprod = ModInt::new(1);
            for i in 0..RANK - 1 {
                rate[i] = root[i + 2] * prod;
                irate[i] = iroot[i + 2] * iprod;
                prod *= iroot[i + 2];
                iprod *= root[i + 2];
            }
        }

        FftCache { rate, irate }
    }
}

pub fn conv(a: &Vec<ModInt>, b: &Vec<ModInt>, cache: &FftCache) -> Vec<ModInt> {
    let ntt = |a: &mut Vec<ModInt>| {
        let n = a.len();
        let h = n.trailing_zeros();

        for len in 0..h {
            let p = 1 << (h - len - 1);
            let mut rot = ModInt::new(1);
            for (s, offset) in (0..1 << len).map(|s| s << (h - len)).enumerate() {
                for i in 0..p {
                    let l = a[i + offset];
                    let r = a[i + offset + p] * rot;
                    a[i + offset] = l + r;
                    a[i + offset + p] = l - r;
                }
                rot *= cache.rate[(!s).trailing_zeros() as usize];
            }
        }
    };

    let intt = |a: &mut Vec<ModInt>| {
        let n = a.len();
        let h = n.trailing_zeros();

        for len in (1..=h).rev() {
            let p = 1 << (h - len);
            let mut irot = ModInt::new(1);
            for (s, offset) in (0..1 << (len - 1)).map(|s| s << (h - len + 1)).enumerate() {
                for i in 0..p {
                    let l = a[i + offset];
                    let r = a[i + offset + p];
                    a[i + offset] = l + r;
                    a[i + offset + p] = (l - r) * irot;
                }
                irot *= cache.irate[(!s).trailing_zeros() as usize];
            }
        }
    };

    let s = a.len() + b.len() - 1;
    let t = s.next_power_of_two();

    let mut a = a.clone();
    a.resize(t, ModInt::new(0));
    ntt(&mut a);

    let mut b = b.clone();
    b.resize(t, ModInt::new(0));
    ntt(&mut b);

    a.iter_mut().zip(b.iter()).for_each(|(x, y)| *x = *x * *y);
    intt(&mut a);
    a.resize(s, ModInt::new(0));
    let t_inv = ModInt::new(t).inverse();
    a.iter_mut().for_each(|x| *x = *x * t_inv);

    a
}