// AtCoder: FPS 24 L - Permutation 2
// https://atcoder.jp/contests/fps-24/tasks/fps_24_l
//
// 「全ての i について p[p[i]] != i」を満たす順列 (=長さ 1, 2 の巡回を持たない順列) の
// 個数を数える。長さ 1, 2 の巡回を除いた巡回のみからなる順列の指数型母関数は、
// exp(-x - x^2/2) / (1 - x) で与えられる。ここで -x - x^2/2 は、全ての巡回の指数型
// 母関数 -log(1 - x) から長さ 1, 2 の巡回の寄与 (x + x^2/2) を除いたものである。

use anmitsu::io::fastio::Fastio;
use anmitsu::modulo998244353::{fps, modulo};

fn main() {
    let mut io = Fastio::new();

    let n = io.u32() as usize;

    // -x - x^2/2 を構築する (長さ 3 以上の巡回のみを許す指数型母関数の被 exp 部分)。
    let mut s = vec![0_u32; n + 1];
    if n >= 1 {
        s[1] = modulo::neg(1);
    }
    if n >= 2 {
        s[2] = modulo::neg(modulo::inv(2));
    }

    // 定数項が 0 であるため、形式的指数は必ず存在する。
    let e = fps::FPS::new(s).exp(n).unwrap();

    // 1 / (1 - x) を掛けることは、係数の累積和を取ることに相当する。
    let mut cum = 0_u32;
    for i in 0..=n {
        cum = modulo::add(cum, e.get(i));
    }

    // 指数型母関数の x^n 係数に n! を掛けて、実際の個数へ戻す。
    let mut fact_n = 1_u32;
    for i in 1..=n {
        fact_n = modulo::mul(fact_n, i as u32);
    }

    io.writeln(modulo::mul(cum, fact_n));

    io.flush();
}
