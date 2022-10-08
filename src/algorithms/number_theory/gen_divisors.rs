pub fn gen_divisors(n: usize) -> Vec<usize> {
    let mut res = vec![];

    for i in (1..).take_while(|&x| x * x <= n) {
        if n % i == 0 {
            res.push(i);
            if i * i < n {
                res.push(n / i);
            }
        }
    }

    res.sort();
    res
}
