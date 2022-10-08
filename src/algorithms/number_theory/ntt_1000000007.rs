//! verified by
//! - Library Checker | [Convolution (Mod 1,000,000,007)](https://judge.yosupo.jp/problem/convolution_mod_1000000007) ([submittion](https://judge.yosupo.jp/submission/106041))

use std::ops;

const MOD: usize = 1000000007;
const NTTMOD1: usize = 167772161; // 5 * (2 ** 25) + 1, primitive root: 3
const NTTMOD2: usize = 469762049; // 7 * (2 ** 26) + 1, primitive root: 3
const NTTMOD3: usize = 754974721; // 45 * (2 ** 24) + 1, primitive root: 11

#[derive(Copy, Clone)]
pub struct NTTModInt1 {
    value: usize,
}

impl NTTModInt1 {
    pub fn new(value: usize) -> NTTModInt1 {
        NTTModInt1 {
            value: value % NTTMOD1,
        }
    }

    pub fn value(&self) -> usize {
        self.value
    }

    pub fn inverse(&self) -> NTTModInt1 {
        // (a, b) -> (x, y) s.t. a * x + b * y = gcd(a, b)
        fn extended_gcd(a: isize, b: isize) -> (isize, isize) {
            if (a, b) == (1, 0) {
                (1, 0)
            } else {
                let (x, y) = extended_gcd(b, a % b);
                (y, x - (a / b) * y)
            }
        }

        let (x, _y) = extended_gcd(self.value() as isize, NTTMOD1 as isize);
        NTTModInt1::new((NTTMOD1 as isize + x) as usize)
    }

    // a^n
    pub fn pow(&self, mut n: usize) -> NTTModInt1 {
        let mut res = NTTModInt1::new(1);
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

impl ops::Add for NTTModInt1 {
    type Output = NTTModInt1;
    fn add(self, other: Self) -> Self {
        NTTModInt1::new(self.value + other.value)
    }
}

impl ops::Sub for NTTModInt1 {
    type Output = NTTModInt1;
    fn sub(self, other: Self) -> Self {
        NTTModInt1::new(NTTMOD1 + self.value - other.value)
    }
}

impl ops::Mul for NTTModInt1 {
    type Output = NTTModInt1;
    fn mul(self, other: Self) -> Self {
        NTTModInt1::new(self.value * other.value)
    }
}

impl ops::Div for NTTModInt1 {
    type Output = NTTModInt1;
    fn div(self, other: Self) -> Self {
        self * other.inverse()
    }
}

impl ops::AddAssign for NTTModInt1 {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl ops::SubAssign for NTTModInt1 {
    fn sub_assign(&mut self, other: Self) {
        *self = *self - other;
    }
}

impl ops::MulAssign for NTTModInt1 {
    fn mul_assign(&mut self, other: Self) {
        *self = *self * other;
    }
}

impl ops::DivAssign for NTTModInt1 {
    fn div_assign(&mut self, other: Self) {
        *self = *self / other;
    }
}

fn ntt1(a: &Vec<NTTModInt1>, root: &Vec<NTTModInt1>) -> Vec<NTTModInt1> {
    let n = a.len();
    let d = (0_usize..).find(|i| (1 << *i) == n).unwrap();
    let mut a = {
        let mut idx = vec![0_usize];
        for i in 0..d {
            let mut add = vec![];
            for x in idx.iter() {
                add.push(x + n / (1 << (i + 1)));
            }
            idx.append(&mut add);
        }
        let mut res = vec![NTTModInt1::new(0); n];
        for i in 0..n {
            res[i] = a[idx[i]];
        }

        res
    };
    let mut b = 1;
    loop {
        if b == n {
            break;
        }
        for j in 0..b {
            let w = root[j * (root.len() / (2 * b))];
            let mut k = 0;
            loop {
                if k == n {
                    break;
                }
                let s = a[j + k];
                let t = a[j + k + b] * w;
                a[j + k] = s + t;
                a[j + k + b] = s - t;

                k += 2 * b;
            }
        }

        b *= 2;
    }
    a
}

fn conv1(a: &Vec<usize>, b: &Vec<usize>) -> Vec<NTTModInt1> {
    let s = a.len() + b.len() - 1;
    let t = s.next_power_of_two();

    let root = {
        let mut root = vec![NTTModInt1::new(1); t];
        let z_t_n = NTTModInt1::new(3).pow(5).pow((1 << 25) / t);
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

    let mut a = a.iter().copied().map(NTTModInt1::new).collect::<Vec<_>>();
    a.resize(t, NTTModInt1::new(0));
    let a_inv = ntt1(&a, &root);

    let mut b = b.iter().copied().map(NTTModInt1::new).collect::<Vec<_>>();
    b.resize(t, NTTModInt1::new(0));
    let b_inv = ntt1(&b, &root);

    let c_inv = a_inv
        .iter()
        .zip(b_inv.iter())
        .map(|(a, b)| *a * *b)
        .collect();
    let c = ntt1(&c_inv, &root_inv);

    c.into_iter()
        .take(s)
        .map(|x| x / NTTModInt1::new(t))
        .collect()
}

#[derive(Copy, Clone)]
pub struct NTTModInt2 {
    value: usize,
}

impl NTTModInt2 {
    pub fn new(value: usize) -> NTTModInt2 {
        NTTModInt2 {
            value: value % NTTMOD2,
        }
    }

    pub fn value(&self) -> usize {
        self.value
    }

    pub fn inverse(&self) -> NTTModInt2 {
        // (a, b) -> (x, y) s.t. a * x + b * y = gcd(a, b)
        fn extended_gcd(a: isize, b: isize) -> (isize, isize) {
            if (a, b) == (1, 0) {
                (1, 0)
            } else {
                let (x, y) = extended_gcd(b, a % b);
                (y, x - (a / b) * y)
            }
        }

        let (x, _y) = extended_gcd(self.value() as isize, NTTMOD2 as isize);
        NTTModInt2::new((NTTMOD2 as isize + x) as usize)
    }

    // a^n
    pub fn pow(&self, mut n: usize) -> NTTModInt2 {
        let mut res = NTTModInt2::new(1);
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

impl ops::Add for NTTModInt2 {
    type Output = NTTModInt2;
    fn add(self, other: Self) -> Self {
        NTTModInt2::new(self.value + other.value)
    }
}

impl ops::Sub for NTTModInt2 {
    type Output = NTTModInt2;
    fn sub(self, other: Self) -> Self {
        NTTModInt2::new(NTTMOD2 + self.value - other.value)
    }
}

impl ops::Mul for NTTModInt2 {
    type Output = NTTModInt2;
    fn mul(self, other: Self) -> Self {
        NTTModInt2::new(self.value * other.value)
    }
}

impl ops::Div for NTTModInt2 {
    type Output = NTTModInt2;
    fn div(self, other: Self) -> Self {
        self * other.inverse()
    }
}

impl ops::AddAssign for NTTModInt2 {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl ops::SubAssign for NTTModInt2 {
    fn sub_assign(&mut self, other: Self) {
        *self = *self - other;
    }
}

impl ops::MulAssign for NTTModInt2 {
    fn mul_assign(&mut self, other: Self) {
        *self = *self * other;
    }
}

impl ops::DivAssign for NTTModInt2 {
    fn div_assign(&mut self, other: Self) {
        *self = *self / other;
    }
}

fn ntt2(a: &Vec<NTTModInt2>, root: &Vec<NTTModInt2>) -> Vec<NTTModInt2> {
    let n = a.len();
    let d = (0_usize..).find(|i| (1 << *i) == n).unwrap();
    let mut a = {
        let mut idx = vec![0_usize];
        for i in 0..d {
            let mut add = vec![];
            for x in idx.iter() {
                add.push(x + n / (1 << (i + 1)));
            }
            idx.append(&mut add);
        }
        let mut res = vec![NTTModInt2::new(0); n];
        for i in 0..n {
            res[i] = a[idx[i]];
        }

        res
    };
    let mut b = 1;
    loop {
        if b == n {
            break;
        }
        for j in 0..b {
            let w = root[j * (root.len() / (2 * b))];
            let mut k = 0;
            loop {
                if k == n {
                    break;
                }
                let s = a[j + k];
                let t = a[j + k + b] * w;
                a[j + k] = s + t;
                a[j + k + b] = s - t;

                k += 2 * b;
            }
        }

        b *= 2;
    }
    a
}

fn conv2(a: &Vec<usize>, b: &Vec<usize>) -> Vec<NTTModInt2> {
    let s = a.len() + b.len() - 1;
    let t = s.next_power_of_two();

    let root = {
        let mut root = vec![NTTModInt2::new(1); t];
        let z_t_n = NTTModInt2::new(3).pow(7).pow((1 << 26) / t);
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

    let mut a = a.iter().copied().map(NTTModInt2::new).collect::<Vec<_>>();
    a.resize(t, NTTModInt2::new(0));
    let a_inv = ntt2(&a, &root);

    let mut b = b.iter().copied().map(NTTModInt2::new).collect::<Vec<_>>();
    b.resize(t, NTTModInt2::new(0));
    let b_inv = ntt2(&b, &root);

    let c_inv = a_inv
        .iter()
        .zip(b_inv.iter())
        .map(|(a, b)| *a * *b)
        .collect();
    let c = ntt2(&c_inv, &root_inv);

    c.into_iter()
        .take(s)
        .map(|x| x / NTTModInt2::new(t))
        .collect()
}

#[derive(Copy, Clone)]
pub struct NTTModInt3 {
    value: usize,
}

impl NTTModInt3 {
    pub fn new(value: usize) -> NTTModInt3 {
        NTTModInt3 {
            value: value % NTTMOD3,
        }
    }

    pub fn value(&self) -> usize {
        self.value
    }

    pub fn inverse(&self) -> NTTModInt3 {
        // (a, b) -> (x, y) s.t. a * x + b * y = gcd(a, b)
        fn extended_gcd(a: isize, b: isize) -> (isize, isize) {
            if (a, b) == (1, 0) {
                (1, 0)
            } else {
                let (x, y) = extended_gcd(b, a % b);
                (y, x - (a / b) * y)
            }
        }

        let (x, _y) = extended_gcd(self.value() as isize, NTTMOD3 as isize);
        NTTModInt3::new((NTTMOD3 as isize + x) as usize)
    }

    // a^n
    pub fn pow(&self, mut n: usize) -> NTTModInt3 {
        let mut res = NTTModInt3::new(1);
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

impl ops::Add for NTTModInt3 {
    type Output = NTTModInt3;
    fn add(self, other: Self) -> Self {
        NTTModInt3::new(self.value + other.value)
    }
}

impl ops::Sub for NTTModInt3 {
    type Output = NTTModInt3;
    fn sub(self, other: Self) -> Self {
        NTTModInt3::new(NTTMOD3 + self.value - other.value)
    }
}

impl ops::Mul for NTTModInt3 {
    type Output = NTTModInt3;
    fn mul(self, other: Self) -> Self {
        NTTModInt3::new(self.value * other.value)
    }
}

impl ops::Div for NTTModInt3 {
    type Output = NTTModInt3;
    fn div(self, other: Self) -> Self {
        self * other.inverse()
    }
}

impl ops::AddAssign for NTTModInt3 {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl ops::SubAssign for NTTModInt3 {
    fn sub_assign(&mut self, other: Self) {
        *self = *self - other;
    }
}

impl ops::MulAssign for NTTModInt3 {
    fn mul_assign(&mut self, other: Self) {
        *self = *self * other;
    }
}

impl ops::DivAssign for NTTModInt3 {
    fn div_assign(&mut self, other: Self) {
        *self = *self / other;
    }
}

fn ntt3(a: &Vec<NTTModInt3>, root: &Vec<NTTModInt3>) -> Vec<NTTModInt3> {
    let n = a.len();
    let d = (0_usize..).find(|i| (1 << *i) == n).unwrap();
    let mut a = {
        let mut idx = vec![0_usize];
        for i in 0..d {
            let mut add = vec![];
            for x in idx.iter() {
                add.push(x + n / (1 << (i + 1)));
            }
            idx.append(&mut add);
        }
        let mut res = vec![NTTModInt3::new(0); n];
        for i in 0..n {
            res[i] = a[idx[i]];
        }

        res
    };
    let mut b = 1;
    loop {
        if b == n {
            break;
        }
        for j in 0..b {
            let w = root[j * (root.len() / (2 * b))];
            let mut k = 0;
            loop {
                if k == n {
                    break;
                }
                let s = a[j + k];
                let t = a[j + k + b] * w;
                a[j + k] = s + t;
                a[j + k + b] = s - t;

                k += 2 * b;
            }
        }

        b *= 2;
    }
    a
}

fn conv3(a: &Vec<usize>, b: &Vec<usize>) -> Vec<NTTModInt3> {
    let s = a.len() + b.len() - 1;
    let t = s.next_power_of_two();

    let root = {
        let mut root = vec![NTTModInt3::new(1); t];
        let z_t_n = NTTModInt3::new(11).pow(45).pow((1 << 24) / t);
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

    let mut a = a.iter().copied().map(NTTModInt3::new).collect::<Vec<_>>();
    a.resize(t, NTTModInt3::new(0));
    let a_inv = ntt3(&a, &root);

    let mut b = b.iter().copied().map(NTTModInt3::new).collect::<Vec<_>>();
    b.resize(t, NTTModInt3::new(0));
    let b_inv = ntt3(&b, &root);

    let c_inv = a_inv
        .iter()
        .zip(b_inv.iter())
        .map(|(a, b)| *a * *b)
        .collect();
    let c = ntt3(&c_inv, &root_inv);

    c.into_iter()
        .take(s)
        .map(|x| x / NTTModInt3::new(t))
        .collect()
}

pub fn garner(nms: Vec<(usize, usize)>, m: usize) -> usize {
    let mut v = nms.clone();
    v.push((0, m));

    let mut coeffs = vec![1; v.len()];
    let mut constants = vec![0; v.len()];

    let inv = |a, m| {
        // (a, b) -> (x, y) s.t. a * x + b * y = gcd(a, b)
        fn extended_gcd(a: isize, b: isize) -> (isize, isize) {
            if (a, b) == (1, 0) {
                (1, 0)
            } else {
                let (x, y) = extended_gcd(b, a % b);
                (y, x - (a / b) * y)
            }
        }

        let (x, _y) = extended_gcd(a as isize, m as isize);
        (m as isize + x) as usize
    };

    for i in 0..v.len() - 1 {
        let mut x = (v[i].1 + v[i].0 - constants[i]) * inv(coeffs[i], v[i].1);
        x %= v[i].1;

        for j in i + 1..v.len() {
            constants[j] += coeffs[j] * x;
            constants[j] %= v[j].1;
            coeffs[j] *= v[i].1;
            coeffs[j] %= v[j].1;
        }
    }

    constants.pop().unwrap()
}

pub fn conv(a: &Vec<usize>, b: &Vec<usize>) -> Vec<usize> {
    let c1 = conv1(&a, &b);
    let c2 = conv2(&a, &b);
    let c3 = conv3(&a, &b);

    (0..a.len() + b.len() - 1)
        .map(|i| {
            garner(
                vec![
                    (c1[i].value(), NTTMOD1),
                    (c2[i].value(), NTTMOD2),
                    (c3[i].value(), NTTMOD3),
                ],
                MOD,
            )
        })
        .collect()
}
