use std::mem;

pub fn gcd(mut a: usize, mut b: usize) -> usize {
    if a < b {
        mem::swap(&mut a, &mut b);
    }
    while b != 0 {
        let temp = a % b;
        a = b;
        b = temp;
    }
    a
}

pub fn lcm(a: usize, b: usize) -> usize {
    a * b / gcd(a, b)
}
