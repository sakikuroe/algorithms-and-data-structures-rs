// Library Checker: Convolution (mod 998244353)
// https://judge.yosupo.jp/problem/convolution_mod

use anmitsu::io::fastio::Fastio;
use anmitsu::modulo998244353::convolution;

fn main() {
    let mut io = Fastio::new();

    let n = io.u32() as usize;
    let m = io.u32() as usize;

    let mut a = Vec::with_capacity(n);
    for _ in 0..n {
        a.push(io.u32());
    }
    let mut b = Vec::with_capacity(m);
    for _ in 0..m {
        b.push(io.u32());
    }

    let c = convolution::convolution(a, b);

    for c in c {
        io.writeln(c);
    }

    io.flush();
}
