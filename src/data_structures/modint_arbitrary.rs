use std::ops;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ModInt {
    value: usize,
    m: usize,
}

impl ModInt {
    pub fn new(value: usize, m: usize) -> ModInt {
        ModInt {
            value: value % m,
            m,
        }
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

        let (x, _y) = extended_gcd(self.value() as isize, self.m as isize);
        ModInt::new((self.m as isize + x) as usize, self.m)
    }

    // a^n
    pub fn pow(&self, mut n: usize) -> ModInt {
        let mut res = ModInt::new(1, self.m);
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
        ModInt::new(self.value + other.value, self.m)
    }
}

impl ops::Sub for ModInt {
    type Output = ModInt;
    fn sub(self, other: Self) -> Self {
        ModInt::new(self.m + self.value - other.value, self.m)
    }
}

impl ops::Mul for ModInt {
    type Output = ModInt;
    fn mul(self, other: Self) -> Self {
        ModInt::new(self.value * other.value, self.m)
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

// n!
pub fn factorial(n: usize, m: usize) -> ModInt {
    (1..=n).fold(ModInt::new(1, m), |x, y| x * ModInt::new(y, m))
}

// nPr
pub fn permutation(n: usize, r: usize, m: usize) -> ModInt {
    if n < r {
        ModInt::new(0, m)
    } else {
        (n - r + 1..=n).fold(ModInt::new(1, m), |x, y| x * ModInt::new(y, m))
    }
}

// nCr
pub fn combination(n: usize, r: usize, m: usize) -> ModInt {
    if n < r {
        ModInt::new(0, m)
    } else {
        permutation(n, r, m) / factorial(r, m)
    }
}

// nHr
pub fn homogeneous(n: usize, r: usize, m: usize) -> ModInt {
    combination(n + r - 1, r, m)
}
