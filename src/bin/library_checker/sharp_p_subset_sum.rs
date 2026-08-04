// Library Checker: #P Subset Sum
// https://judge.yosupo.jp/problem/sharp_p_subset_sum

use anmitsu::io::fastio::Fastio;
use anmitsu::modulo998244353::fps::partition;

fn main() {
    let mut io = Fastio::new();

    let n = io.u32() as usize;
    let t = io.u32() as usize;

    let s = (0..n).map(|_| io.u32()).collect::<Vec<u32>>();

    // 部分集合の総和の母関数 Π(1 + x^{s_i}) を計算する。
    let p = partition::product_one_plus_x_powers(&s, t).unwrap();

    for i in 1..=t {
        io.writeln(p.get(i));
    }

    io.flush();
}
