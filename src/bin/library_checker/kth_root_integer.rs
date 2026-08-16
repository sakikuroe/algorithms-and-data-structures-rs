// Library Checker: Kth Root (Integer)
// https://judge.yosupo.jp/problem/kth_root_integer

use anmitsu::io::fastio::Fastio;
use anmitsu::math::kth_root;

fn main() {
    let mut io = Fastio::new();

    let t = io.u32();
    for _ in 0..t {
        let a = io.u64();
        let k = io.u64();
        io.writeln(kth_root::kth_root(a, k));
    }

    io.flush();
}
