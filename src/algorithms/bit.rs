pub fn is_subset_of(a: usize, b: usize) -> bool {
    a | b == b
}

/// verified by
/// - AtCoder | [UNICORNプログラミングコンテスト2022(AtCoder Beginner Contest 269)](https://atcoder.jp/contests/abc269/submissions/35021872))

// gen subset sequence of bit
pub fn gen_subset(bit: usize) -> Vec<usize> {
    let mut res = vec![];
    let mut sub = bit;

    loop {
        res.push(sub);
        if sub >= 1 {
            sub = (sub - 1) & bit;
        } else {
            break;
        }
    }

    res.reverse();
    res
}

// gen subset sequence of {1, 2, .. ,n} consisting of k elements
pub fn gen_subset_of_k_elements(n: usize, k: usize) -> Vec<usize> {
    let mut res = vec![];
    let mut comb = (1 << k) - 1;
    while comb < 1 << n {
        res.push(comb);
        let x = comb & (!comb + 1);
        let y = comb + x;
        comb = ((comb & !y) / x >> 1) | y;
    }

    res
}
