pub fn pow(a: usize, mut n: usize) -> usize {
    let mut res = 1;
    let mut x = a;
    while n > 0 {
        if n % 2 == 1 {
            res *= x;
        }
        x *= x;
        n /= 2;
    }

    res
}
