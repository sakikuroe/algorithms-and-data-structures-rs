// Library Checker: Kth term of Linearly Recurrent Sequence
// https://judge.yosupo.jp/problem/kth_term_of_linearly_recurrent_sequence

use anmitsu::io::fastio::Fastio;
use anmitsu::modulo998244353::fps::bostan_mori;

fn main() {
    let mut io = Fastio::new();

    let d = io.u32() as usize;
    let k = io.u64() as usize;

    let mut a = Vec::with_capacity(d);
    for _ in 0..d {
        a.push(io.u32());
    }
    let mut c = Vec::with_capacity(d);
    for _ in 0..d {
        c.push(io.u32());
    }

    let ans = bostan_mori::linear_recurrence_kth_term(&a, &c, k);

    io.writeln(ans);

    io.flush();
}
