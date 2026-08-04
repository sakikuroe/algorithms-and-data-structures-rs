// Library Checker: Pow of Formal Power Series (Sparse)
// https://judge.yosupo.jp/problem/pow_of_formal_power_series_sparse

use anmitsu::io::fastio::Fastio;
use anmitsu::modulo998244353::fps;

fn main() {
    let mut io = Fastio::new();

    let n = io.u32() as usize;
    let k = io.u32() as usize;
    let m = io.u64() as usize;

    // 非零項のみが与えられるため、まず全項をゼロで初期化してから埋める。
    let mut a = vec![0; n];
    for _ in 0..k {
        let i = io.u32() as usize;
        a[i] = io.u32();
    }

    let b = fps::FPS::new(a).pow(m, n - 1);

    for i in 0..n {
        io.writeln(b.get(i));
    }

    io.flush();
}
