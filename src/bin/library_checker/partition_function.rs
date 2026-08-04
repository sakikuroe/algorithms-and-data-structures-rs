// Library Checker: Partition Function
// https://judge.yosupo.jp/problem/partition_function

use anmitsu::io::fastio::Fastio;
use anmitsu::modulo998244353::fps::partition;

fn main() {
    let mut io = Fastio::new();

    let n = io.u32() as usize;

    // 分割数の母関数 Π_{k=1}^{n} 1/(1 - x^k) を計算する。
    let exponents = (1..=n as u32).collect::<Vec<u32>>();
    let p = partition::product_inv_one_minus_x_powers(&exponents, n).unwrap();

    for i in 0..=n {
        io.writeln(p.get(i));
    }

    io.flush();
}
