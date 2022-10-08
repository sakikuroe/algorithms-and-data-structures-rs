use std::{fmt, ops};

const MOD: usize = 1000000007;
// const MOD: usize = 998244353; // 119 * (1 << 23) + 1

#[derive(Copy, Clone, Debug)]
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

// n!
pub fn factorial(n: usize) -> ModInt {
    (1..=n).fold(ModInt::new(1), |x, y| x * ModInt::new(y))
}

// nPr
pub fn permutation(n: usize, r: usize) -> ModInt {
    if n < r {
        ModInt::new(0)
    } else {
        (n - r + 1..=n).fold(ModInt::new(1), |x, y| x * ModInt::new(y))
    }
}

// nCr
pub fn combination(n: usize, r: usize) -> ModInt {
    if n < r {
        ModInt::new(0)
    } else {
        permutation(n, r) / factorial(r)
    }
}

// nHr
pub fn homogeneous(n: usize, r: usize) -> ModInt {
    combination(n + r - 1, r)
}

// Catalan number
pub fn catalan(n: usize) -> ModInt {
    combination(2 * n, n) / ModInt::new(n + 1)
}

impl fmt::Display for ModInt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalan_test() {
        assert_eq!(catalan(0).value(), 1);
        assert_eq!(catalan(1).value(), 1);
        assert_eq!(catalan(2).value(), 2);
        assert_eq!(catalan(3).value(), 5);
        assert_eq!(catalan(4).value(), 14);
        assert_eq!(catalan(5).value(), 42);
    }
}
