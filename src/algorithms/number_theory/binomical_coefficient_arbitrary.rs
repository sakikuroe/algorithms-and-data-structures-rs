//! verified by
//! - Library Checker | [Binomial Coefficient](https://judge.yosupo.jp/problem/binomial_coefficient) ([submittion](https://judge.yosupo.jp/submission/121318))
//! - AtCoder | [AtCoder Regular Contest 012 D - Don't worry. Be Together](https://atcoder.jp/contests/arc012/tasks/arc012_4) ([submittion](https://atcoder.jp/contests/arc012/submissions/38040263))

use crate::data_structures::modint_arbitrary::ModIntArbitrary;

pub fn safe_mod(mut x: i64, m: i64) -> i64 {
    x %= m;
    if x < 0 {
        x += m;
    }
    x
}

pub fn inv_gcd(a: i64, b: i64) -> (i64, i64) {
    let a = safe_mod(a, b);
    if a == 0 {
        return (b, 0);
    }
    let mut s = b;
    let mut t = a;
    let mut m0 = 0;
    let mut m1 = 1;

    while t != 0 {
        let u = s / t;
        s -= t * u;
        m0 -= m1 * u;
        std::mem::swap(&mut s, &mut t);
        std::mem::swap(&mut m0, &mut m1);
    }
    if m0 < 0 {
        m0 += b / s;
    }
    (s, m0)
}

pub fn crt(r: &[i64], m: &[i64]) -> (i64, i64) {
    let (mut r0, mut m0) = (0, 1);
    for (&(mut ri), &(mut mi)) in r.iter().zip(m.iter()) {
        ri = safe_mod(ri, mi);
        if m0 < mi {
            std::mem::swap(&mut r0, &mut ri);
            std::mem::swap(&mut m0, &mut mi);
        }
        if m0 % mi == 0 {
            if r0 % mi != ri {
                return (0, 0);
            }
            continue;
        }
        let (g, im) = inv_gcd(m0, mi);
        let u1 = mi / g;
        if (ri - r0) % g != 0 {
            return (0, 0);
        }
        let x = (ri - r0) / g % u1 * im % u1;
        r0 += x * m0;
        m0 *= u1;
        if r0 < 0 {
            r0 += m0
        };
    }

    (r0, m0)
}

const N_MAX: usize = 2000000;
const MODULO_MAX: usize = 2000000000;

// const N_MAX: usize = 1000000000000000000;
// const MODULO_MAX: usize = 20000000;

pub struct PrimePowerBinomial {
    p: usize,
    q: usize,
    m: usize,
    fact: Vec<ModIntArbitrary>,
    fact_inv: Vec<ModIntArbitrary>,
    delta: ModIntArbitrary,
}

impl PrimePowerBinomial {
    fn enu(m: usize, p: usize) -> (Vec<ModIntArbitrary>, Vec<ModIntArbitrary>) {
        let mx = std::cmp::min(m, N_MAX + 1);
        let mut fact = vec![ModIntArbitrary::new(0, m); mx + 1];
        let mut fact_inv = vec![ModIntArbitrary::new(0, m); mx + 1];
        fact[0] = ModIntArbitrary::new(1, m);
        fact_inv[0] = ModIntArbitrary::new(1, m);
        fact[1] = ModIntArbitrary::new(1, m);
        fact_inv[1] = ModIntArbitrary::new(1, m);
        let mut i = 2;
        while i < mx {
            if i % p == 0 {
                fact[i] = fact[i - 1];
                fact[i + 1] = fact[i - 1] * ModIntArbitrary::new(i + 1, m);
                i += 1;
            } else {
                fact[i] = fact[i - 1] * ModIntArbitrary::new(i, m);
            }
            i += 1;
        }
        fact_inv[mx - 1] = fact[mx - 1].inverse();
        let mut i = mx - 2;
        while i > 1 {
            if i % p == 0 {
                fact_inv[i] = fact_inv[i + 1] * ModIntArbitrary::new(i + 1, m);
                fact_inv[i - 1] = fact_inv[i];
                i -= 1;
            } else {
                fact_inv[i] = fact_inv[i + 1] * ModIntArbitrary::new(i + 1, m);
            }
            i -= 1;
        }
        (fact, fact_inv)
    }

    pub fn new(p: usize, q: usize) -> Self {
        let mut m = 1;
        for _ in 0..q {
            m *= p;
        }
        let (fact, fact_inv) = Self::enu(m, p);
        let delta = ModIntArbitrary::new(if p == 2 && q >= 3 { 1 } else { m - 1 }, m);
        PrimePowerBinomial {
            p,
            q,
            m,
            fact,
            fact_inv,
            delta,
        }
    }

    fn c(&self, mut n: usize, mut m: usize) -> ModIntArbitrary {
        if n < m {
            return ModIntArbitrary::new(0, self.m);
        }
        let mut r = n - m;
        let mut e0 = 0;
        let mut eq = 0;
        let mut i = 0;
        let mut res = ModIntArbitrary::new(1, self.m);
        while n > 0 {
            res *= self.fact[n % self.m];
            res *= self.fact_inv[m % self.m];
            res *= self.fact_inv[r % self.m];
            n /= self.p;
            m /= self.p;
            r /= self.p;
            let eps = n - m - r;
            e0 += eps;
            if e0 >= self.q {
                return ModIntArbitrary::new(0, self.m);
            }
            i += 1;
            if i >= self.q {
                eq += eps;
            }
        }
        if eq % 2 == 1 {
            res = res * self.delta;
        }
        res = res * ModIntArbitrary::new(self.p, self.m).pow(e0);
        res
    }
}

/// Returns:
///     nCk mod m
/// Note:
///     - (n <= 2 * 10^{6} and m <= 2 * 10^{9}) or (n <= 10^{18} and m <= 2 * 10^{7})
pub struct ArbitraryModBinomial {
    modulo: usize,
    modulo_org: usize,
    m: Vec<usize>,
    cs: Vec<PrimePowerBinomial>,
}

impl ArbitraryModBinomial {
    pub fn new(mut modulo: usize) -> Self {
        assert!(modulo <= MODULO_MAX);
        let modulo_org = modulo;
        let mut m = vec![];
        let mut cs = vec![];
        let mut i = 2;
        while i * i <= modulo {
            if modulo % i == 0 {
                let mut j = 0;
                let mut k = 1;
                while modulo % i == 0 {
                    modulo /= i;
                    j += 1;
                    k *= i;
                }
                m.push(k);
                cs.push(PrimePowerBinomial::new(i, j));
            }
            i += 1;
        }
        if modulo != 1 {
            m.push(modulo);
            cs.push(PrimePowerBinomial::new(modulo, 1));
        }
        ArbitraryModBinomial {
            modulo,
            modulo_org,
            m,
            cs,
        }
    }

    pub fn comb(&self, n: usize, k: usize) -> ModIntArbitrary {
        assert!(n <= N_MAX);
        if self.modulo_org == 1 {
            return ModIntArbitrary::new(0, self.modulo);
        }

        let mut rem = vec![];
        let mut d = vec![];
        for i in 0..self.cs.len() {
            rem.push(self.cs[i].c(n, k).value() as i64);
            d.push(self.m[i] as i64);
        }

        ModIntArbitrary::new(crt(&rem, &d).0 as usize, self.modulo_org)
    }
}