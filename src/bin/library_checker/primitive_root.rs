// Library Checker: Primitive Root
// https://judge.yosupo.jp/problem/primitive_root

use anmitsu::io::fastio::Fastio;
use anmitsu::math::primality;

fn main() {
    let mut io = Fastio::new();

    let q = io.u32();
    for _ in 0..q {
        let p = io.u64();
        io.writeln(primality::find_primitive_root(p));
    }

    io.flush();
}
