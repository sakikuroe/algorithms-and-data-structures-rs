// Library Checker: Log of Formal Power Series
// https://judge.yosupo.jp/problem/log_of_formal_power_series

use anmitsu::io::fastio::Fastio;
use anmitsu::modulo998244353::fps;

fn main() {
    let mut io = Fastio::new();

    let n = io.u32() as usize;

    let mut a = Vec::with_capacity(n);
    for _ in 0..n {
        a.push(io.u32());
    }

    // 制約より a_0 = 1 が保証されるため、形式的対数は必ず存在する。
    let b = fps::FPS::new(a).log(n - 1).unwrap();

    for i in 0..n {
        io.writeln(b.get(i));
    }

    io.flush();
}
