// AtCoder: FPS 24 M - Connected Graph
// https://atcoder.jp/contests/fps-24/tasks/fps_24_m
//
// N 頂点のラベル付き単純連結無向グラフの個数を数える。全グラフの個数列 g_n =
// 2^C(n,2) の指数型母関数を G(x) とすると、連結グラフの個数列 c_n の指数型母関数
// C(x) は G(x) = exp(C(x))、すなわち C(x) = log(G(x)) を満たす。

use anmitsu::io::fastio::Fastio;
use anmitsu::modulo998244353::{fps, modulo};

fn main() {
    let mut io = Fastio::new();

    let n = io.u32() as usize;

    // 階乗と階乗の逆元のテーブルを、指数型母関数の係数変換に用いるため構築する。
    let mut fact = vec![1_u32; n + 1];
    for i in 1..=n {
        fact[i] = modulo::mul(fact[i - 1], i as u32);
    }
    let mut inv_fact = vec![1_u32; n + 1];
    inv_fact[n] = modulo::inv(fact[n]);
    for i in (1..=n).rev() {
        inv_fact[i - 1] = modulo::mul(inv_fact[i], i as u32);
    }

    // G(x) の係数 a_i = g_i / i! を、g_i = 2^C(i,2) から直接構築する。
    let mut a = vec![0_u32; n + 1];
    for (i, a_i) in a.iter_mut().enumerate() {
        let g_i = modulo::pow(2, i * (i.saturating_sub(1)) / 2);
        *a_i = modulo::mul(g_i, inv_fact[i]);
    }

    // G(0) = 1 であるため、形式的対数は必ず存在する。
    let l = fps::FPS::new(a).log(n).unwrap();

    // 指数型母関数の x^n 係数に n! を掛けて、連結グラフの個数へ戻す。
    io.writeln(modulo::mul(l.get(n), fact[n]));

    io.flush();
}
