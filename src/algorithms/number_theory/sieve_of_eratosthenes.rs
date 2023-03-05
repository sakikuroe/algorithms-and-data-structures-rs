use std::collections::{BTreeMap, BTreeSet};

pub struct SieveOfEratosthenes {
    max_int: usize,
    sieve: Vec<Option<usize>>,
}

impl SieveOfEratosthenes {
    pub fn new(max_int: usize) -> SieveOfEratosthenes {
        let gen_sieve = |max_int| {
            let mut res = (0..=max_int)
                .map(|x| Some(x))
                .collect::<Vec<Option<usize>>>();
            res[0] = None;
            res[1] = None;
            for i in 2..=max_int {
                if res[i] == Some(i) {
                    for j in (2 * i..=max_int).step_by(i) {
                        res[j] = Some(i);
                    }
                }
            }
            res
        };
        let sieve = gen_sieve(max_int);
        SieveOfEratosthenes { max_int, sieve }
    }

    pub fn is_prime(&self, n: usize) -> bool {
        self.sieve[n] == Some(n)
    }

    pub fn get_max_int(&self) -> usize {
        self.max_int
    }

    pub fn integer_factorization(&self, mut n: usize) -> BTreeMap<usize, usize> {
        let mut res = BTreeMap::new();

        if !(n == 0 || n == 1) {
            while n != 1 {
                let prime_factor = self.sieve[n].unwrap();
                *res.entry(prime_factor).or_insert(0) += 1;
                n /= prime_factor;
            }
        }

        res
    }

    pub fn gen_divisors(&self, n: usize) -> BTreeSet<usize> {
        if n == 1 {
            return vec![1].into_iter().collect();
        }
        let factorization = self.integer_factorization(n);
        let mut res = vec![];

        for (a, n) in factorization {
            if res.len() == 0 {
                let mut p = 1;
                for _ in 0..=n {
                    res.push(p);
                    p *= a;
                }
            } else {
                res = {
                    let u = res;
                    let w = {
                        let mut res = vec![];
                        let mut p = 1;
                        for _ in 0..=n {
                            res.push(p);
                            p *= a;
                        }
                        res
                    };
                    let mut res = vec![];
                    for &x in &u {
                        for &y in &w {
                            res.push(x * y);
                        }
                    }
                    res
                };
            }
        }

        res.into_iter().collect::<BTreeSet<usize>>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let sieve = SieveOfEratosthenes::new(1000);
        assert_eq!(sieve.is_prime(2), true);
        assert_eq!(sieve.is_prime(3), true);
        assert_eq!(sieve.is_prime(5), true);
        assert_eq!(sieve.is_prime(7), true);
        assert_eq!(sieve.is_prime(47), true);
        assert_eq!(sieve.is_prime(53), true);
        assert_eq!(sieve.is_prime(71), true);
        assert_eq!(sieve.is_prime(97), true);

        assert_eq!(sieve.is_prime(0), false);
        assert_eq!(sieve.is_prime(1), false);
        assert_eq!(sieve.is_prime(4), false);
        assert_eq!(sieve.is_prime(8), false);
        assert_eq!(sieve.is_prime(24), false);
        assert_eq!(sieve.is_prime(33), false);
        assert_eq!(sieve.is_prime(91), false);

        assert_eq!(sieve.get_max_int(), 1000);

        assert_eq!(
            BTreeMap::from([(2, 3), (5, 2)]),
            sieve.integer_factorization(200)
        );
        assert_eq!(
            BTreeSet::from([1, 2, 4, 5, 8, 10, 20, 25, 40, 50, 100, 200]),
            sieve.gen_divisors(200)
        );
        assert_eq!(BTreeSet::from([1, 5, 11, 55]), sieve.gen_divisors(55));
    }
}
