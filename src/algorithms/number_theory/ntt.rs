//! verified by
//! - Library Checker | [Convolution](https://judge.yosupo.jp/problem/convolution_mod) ([submittion](https://judge.yosupo.jp/submission/103577))

use std::ops;

const MOD: usize = 998244353; // 119 * (1 << 23) + 1

#[derive(Copy, Clone)]
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
                res *= x;
            }
            x *= x;
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

pub fn ntt(a: &Vec<ModInt>, root: &Vec<ModInt>) -> Vec<ModInt> {
    let n = a.len();
    let d = n.trailing_zeros();
    let mut a = {
        let mut idx = vec![0_usize];
        for i in 0..d {
            let mut add = vec![];
            for x in idx.iter() {
                add.push(x + n / (1 << (i + 1)));
            }
            idx.append(&mut add);
        }
        let mut res = vec![ModInt::new(0); n];
        for i in 0..n {
            res[i] = a[idx[i]];
        }

        res
    };

    for i in 0..d {
        let b = 1 << i;
        for j in (0..n).step_by(2 * b) {
            for k in 0..b {
                let w = root[k * (root.len() / (2 * b))];
                let x = a[j + k];
                let y = a[j + k + b] * w;
                a[j + k] = x + y;
                a[j + k + b] = x - y;
            }
        }
    }

    a
}

pub fn conv(a: &Vec<usize>, b: &Vec<usize>) -> Vec<ModInt> {
    let s = a.len() + b.len() - 1;
    let t = s.next_power_of_two();

    let root = {
        let mut root = vec![ModInt::new(1); t];
        let z_t_n = ModInt::new(3).pow(119).pow((1 << 23) / t);
        for i in 0..t - 1 {
            root[i + 1] = root[i] * z_t_n;
        }
        root
    };

    let root_inv = {
        let mut root_inv = root.clone();
        root_inv[1..].reverse();
        root_inv
    };

    let mut a = a.iter().copied().map(ModInt::new).collect::<Vec<_>>();
    a.resize(t, ModInt::new(0));
    let a_inv = ntt(&a, &root);

    let mut b = b.iter().copied().map(ModInt::new).collect::<Vec<_>>();
    b.resize(t, ModInt::new(0));
    let b_inv = ntt(&b, &root);

    let c_inv = a_inv
        .iter()
        .zip(b_inv.iter())
        .map(|(a, b)| *a * *b)
        .collect();
    let c = ntt(&c_inv, &root_inv);

    c.into_iter().take(s).map(|x| x / ModInt::new(t)).collect()
}
