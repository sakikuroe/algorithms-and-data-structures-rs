// Library Checker: Pow of Formal Power Series
// https://judge.yosupo.jp/problem/pow_of_formal_power_series

use anmitsu::io::fastio::Fastio;
use anmitsu::modulo998244353::fps;

fn main() {
    let mut io = Fastio::new();

    let n = io.u32() as usize;
    let m = io.u64() as usize;

    let mut a = Vec::with_capacity(n);
    for _ in 0..n {
        a.push(io.u32());
    }

    let b = fps::FPS::new(a).pow(m, n - 1);

    for i in 0..n {
        io.writeln(b.get(i));
    }

    io.flush();
}
