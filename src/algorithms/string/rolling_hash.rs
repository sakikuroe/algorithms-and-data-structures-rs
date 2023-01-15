use std::{fmt, ops};

/// verified by
/// - AtCoder | [AtCoder Beginner Contest 284 F - ABCBAC](https://atcoder.jp/contests/abc284/tasks/abc284_f), ([submittion](https://atcoder.jp/contests/abc284/submissions/37959608))

//    ξ
//    ll
//   _ll_
// /......\
// │* 衝 *│
// │* 突 *│
// │* 退 *│
// │* 散 *│
// │*    *│
// --------

const MODS3: (usize, usize, usize) = (4294966297, 4294966591, 4294967231);
const BASES3: (usize, usize, usize) = (3164574161, 2680895341, 4019940150);

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ModInts3 {
    value: (usize, usize, usize),
}

impl ModInts3 {
    pub fn new(value: (usize, usize, usize)) -> ModInts3 {
        ModInts3 {
            value: (value.0 % MODS3.0, value.1 % MODS3.1, value.2 % MODS3.2),
        }
    }

    pub fn value(&self) -> (usize, usize, usize) {
        self.value
    }

    pub fn inverse(&self) -> ModInts3 {
        // (a, b) -> (x, y) s.t. a * x + b * y = gcd(a, b)
        fn extended_gcd(a: isize, b: isize) -> (isize, isize) {
            if (a, b) == (1, 0) {
                (1, 0)
            } else {
                let (x, y) = extended_gcd(b, a % b);
                (y, x - (a / b) * y)
            }
        }

        let (x0, _y0) = extended_gcd(self.value().0 as isize, MODS3.0 as isize);
        let (x1, _y1) = extended_gcd(self.value().1 as isize, MODS3.1 as isize);
        let (x2, _y2) = extended_gcd(self.value().2 as isize, MODS3.2 as isize);
        ModInts3::new((
            (MODS3.0 as isize + x0) as usize,
            (MODS3.1 as isize + x1) as usize,
            (MODS3.2 as isize + x2) as usize,
        ))
    }

    // a^n
    pub fn pow(&self, mut n: usize) -> ModInts3 {
        let mut res = ModInts3::new((1, 1, 1));
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

impl ops::Add for ModInts3 {
    type Output = ModInts3;
    fn add(self, other: Self) -> Self {
        ModInts3::new((
            self.value.0 + other.value.0,
            self.value.1 + other.value.1,
            self.value.2 + other.value.2,
        ))
    }
}

impl ops::Sub for ModInts3 {
    type Output = ModInts3;
    fn sub(self, other: Self) -> Self {
        ModInts3::new((
            MODS3.0 + self.value.0 - other.value.0,
            MODS3.1 + self.value.1 - other.value.1,
            MODS3.2 + self.value.2 - other.value.2,
        ))
    }
}

impl ops::Mul for ModInts3 {
    type Output = ModInts3;
    fn mul(self, other: Self) -> Self {
        ModInts3::new((
            self.value.0 * other.value.0,
            self.value.1 * other.value.1,
            self.value.2 * other.value.2,
        ))
    }
}

impl ops::Div for ModInts3 {
    type Output = ModInts3;
    fn div(self, other: Self) -> Self {
        self * other.inverse()
    }
}

impl ops::AddAssign for ModInts3 {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl ops::SubAssign for ModInts3 {
    fn sub_assign(&mut self, other: Self) {
        *self = *self - other;
    }
}

impl ops::MulAssign for ModInts3 {
    fn mul_assign(&mut self, other: Self) {
        *self = *self * other;
    }
}

impl ops::DivAssign for ModInts3 {
    fn div_assign(&mut self, other: Self) {
        *self = *self / other;
    }
}

impl fmt::Display for ModInts3 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.value())
    }
}

pub struct RollingHashCache {
    pow: Vec<ModInts3>,
    pow_inv: Vec<ModInts3>,
}

impl RollingHashCache {
    pub fn new(max_len: usize) -> RollingHashCache {
        let mut pow = vec![ModInts3::new((1, 1, 1))];
        let mut pow_inv = vec![ModInts3::new((1, 1, 1))];
        let base_inv = ModInts3::new(BASES3).inverse();
        for i in 0..max_len {
            pow.push(pow[i] * ModInts3::new(BASES3));
            pow_inv.push(pow_inv[i] * base_inv);
        }
        RollingHashCache { pow, pow_inv }
    }
}

type RollingHash = (ModInts3, usize);

pub struct RollingHashGenerator {
    hash: Vec<ModInts3>,
}

impl RollingHashGenerator {
    pub fn new(s: &Vec<ModInts3>, rhc: &RollingHashCache) -> RollingHashGenerator {
        let n = s.len();
        let mut hash = vec![ModInts3::new((0, 0, 0))];
        for i in 0..n {
            hash.push(hash[i] + rhc.pow[i] * s[i]);
        }
        RollingHashGenerator { hash }
    }

    /// Returns:
    ///     hash of s[l..r]
    pub fn get_hash(&self, l: usize, r: usize, rhc: &RollingHashCache) -> RollingHash {
        assert!(l <= r);
        ((self.hash[r] - self.hash[l]) * rhc.pow_inv[l], r - l)
    }

    pub fn concat_hash(
        &self,
        h1: RollingHash,
        h2: RollingHash,
        rhc: &RollingHashCache,
    ) -> RollingHash {
        (h1.0 + h2.0 * rhc.pow[h1.1], h1.1 + h2.1)
    }
}
