use std::ops;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ModIntArbitrary {
    value: usize,
    m: usize,
}

impl ModIntArbitrary {
    pub fn new(value: usize, m: usize) -> ModIntArbitrary {
        ModIntArbitrary {
            value: value % m,
            m,
        }
    }

    pub fn value(&self) -> usize {
        self.value
    }

    pub fn inverse(&self) -> ModIntArbitrary {
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
        ModIntArbitrary::new((self.m as isize + x) as usize, self.m)
    }

    // a^n
    pub fn pow(&self, mut n: usize) -> ModIntArbitrary {
        let mut res = ModIntArbitrary::new(1, self.m);
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

impl ops::Add for ModIntArbitrary {
    type Output = ModIntArbitrary;
    fn add(self, other: Self) -> Self {
        ModIntArbitrary::new(self.value + other.value, self.m)
    }
}

impl ops::Sub for ModIntArbitrary {
    type Output = ModIntArbitrary;
    fn sub(self, other: Self) -> Self {
        ModIntArbitrary::new(self.m + self.value - other.value, self.m)
    }
}

impl ops::Mul for ModIntArbitrary {
    type Output = ModIntArbitrary;
    fn mul(self, other: Self) -> Self {
        ModIntArbitrary::new(self.value * other.value, self.m)
    }
}

impl ops::Div for ModIntArbitrary {
    type Output = ModIntArbitrary;
    fn div(self, other: Self) -> Self {
        self * other.inverse()
    }
}

impl ops::AddAssign for ModIntArbitrary {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl ops::SubAssign for ModIntArbitrary {
    fn sub_assign(&mut self, other: Self) {
        *self = *self - other;
    }
}

impl ops::MulAssign for ModIntArbitrary {
    fn mul_assign(&mut self, other: Self) {
        *self = *self * other;
    }
}

impl ops::DivAssign for ModIntArbitrary {
    fn div_assign(&mut self, other: Self) {
        *self = *self / other;
    }
}

// n!
pub fn factorial(n: usize, m: usize) -> ModIntArbitrary {
    (1..=n).fold(ModIntArbitrary::new(1, m), |x, y| x * ModIntArbitrary::new(y, m))
}

// nPr
pub fn permutation(n: usize, r: usize, m: usize) -> ModIntArbitrary {
    if n < r {
        ModIntArbitrary::new(0, m)
    } else {
        (n - r + 1..=n).fold(ModIntArbitrary::new(1, m), |x, y| x * ModIntArbitrary::new(y, m))
    }
}

// nCr
pub fn combination(n: usize, r: usize, m: usize) -> ModIntArbitrary {
    if n < r {
        ModIntArbitrary::new(0, m)
    } else {
        permutation(n, r, m) / factorial(r, m)
    }
}

// nHr
pub fn homogeneous(n: usize, r: usize, m: usize) -> ModIntArbitrary {
    combination(n + r - 1, r, m)
}
