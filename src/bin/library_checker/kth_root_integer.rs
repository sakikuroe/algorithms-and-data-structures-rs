// Library Checker: Kth Root (Integer)
// https://judge.yosupo.jp/problem/kth_root_integer

use anmitsu::io::fastio::Fastio;
use anmitsu::math::kth_root;

/// 各入力の整数 `k` 乗根を求め、その整数部分を 1 行ずつ出力する。
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
