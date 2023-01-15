//! verified by
//! - Library Checker | [Discrete Logarithm](https://judge.yosupo.jp/problem/discrete_logarithm_mod) ([submittion](https://judge.yosupo.jp/submission/107411))

use super::{
    baby_step_giant_step::baby_step_giant_step, gcd_lcm::gcd,
    integer_factorization::integer_factorization,
};
use crate::data_structures::modint_arbitrary::ModIntArbitrary;

/// return k s.t. x^{k} = y mod m
pub fn discrete_logarithm_mod(x: usize, y: usize, m: usize) -> Option<usize> {
    let sqrt = |n| -> usize {
        let mut ng = n + 1;
        let mut ok = 0;
        while ng - ok > 1 {
            let mid = (ng + ok) / 2;
            if mid * mid <= n {
                ok = mid;
            } else {
                ng = mid;
            }
        }
        ok
    };

    if gcd(x, m) == 1 {
        let bs = sqrt(m);
        let fff = |t: ModIntArbitrary| t * ModIntArbitrary::new(x, m).pow(bs);
        let f_inv = |t: ModIntArbitrary| t * ModIntArbitrary::new(x, m).inverse();
        baby_step_giant_step(
            ModIntArbitrary::new(1, m),
            ModIntArbitrary::new(y, m),
            bs,
            fff,
            f_inv,
        )
    } else {
        let mut q = 1;
        for (p, e) in integer_factorization(m) {
            if x % p == 0 {
                for _ in 0..e {
                    q *= p;
                }
            }
        }
        let mut c = 0;
        let mut x_c = 1;
        while x_c % q != 0 {
            if x_c == y {
                return Some(c);
            }
            c += 1;
            x_c *= x;
            x_c %= m;
        }

        if y % q != 0 {
            None
        } else {
            let bs = sqrt(m / q);
            let fff = |t: ModIntArbitrary| t * ModIntArbitrary::new(x, m / q).pow(bs);
            let f_inv = |t: ModIntArbitrary| t * ModIntArbitrary::new(x, m / q).inverse();

            match baby_step_giant_step(
                ModIntArbitrary::new(x_c / q, m / q),
                ModIntArbitrary::new(y / q, m / q),
                bs,
                fff,
                f_inv,
            ) {
                Some(v) => Some(v + c),
                None => None,
            }
        }
    }
}
