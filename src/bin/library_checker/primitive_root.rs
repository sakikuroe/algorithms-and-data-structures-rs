// Library Checker: Primitive Root
// https://judge.yosupo.jp/problem/primitive_root

use anmitsu::io::fastio::Fastio;
use anmitsu::math::primality;

/// 各素数に対して原始根を 1 つ求め、問い合わせ順に出力する。
fn main() {
    let mut io = Fastio::new();

    let q = io.u32();
    for _ in 0..q {
        let p = io.u64();
        io.writeln(primality::find_primitive_root(p));
    }

    io.flush();
}
