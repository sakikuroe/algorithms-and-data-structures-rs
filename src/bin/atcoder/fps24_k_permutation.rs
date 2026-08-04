// AtCoder: FPS 24 K - Permutation
// https://atcoder.jp/contests/fps-24/tasks/fps_24_k
//
// 「1 <= i <= N-1 の全ての i について max(p_1, ..., p_i) != i」を満たす順列 (=分解不能
// な順列) の個数を数える。分解不能順列の個数列 g_n の通常型母関数 G(x) は、
// 全順列の個数列 (n!)_n の母関数 F(x) = Σ n! x^n との間に F(x) = 1 / (1 - G(x))、
// すなわち G(x) = 1 - F(x)^{-1} という関係を持つ。

use anmitsu::io::fastio::Fastio;
use anmitsu::modulo998244353::{fps, modulo};

fn main() {
    let mut io = Fastio::new();

    let n = io.u32() as usize;

    // F(x) = Σ_{i=0}^{n} i! x^i を構築する。
    let mut fact = vec![0_u32; n + 1];
    fact[0] = 1;
    for i in 1..=n {
        fact[i] = modulo::mul(fact[i - 1], i as u32);
    }

    // F(0) = 0! = 1 != 0 であるため、逆元は必ず存在する。
    let f_inv = fps::FPS::new(fact).inverse(n).unwrap();

    // G(x) = 1 - F(x)^{-1} の x^n 係数を取り出す。
    let g_n = if n == 0 {
        modulo::sub(1, f_inv.get(0))
    } else {
        modulo::neg(f_inv.get(n))
    };

    io.writeln(g_n);

    io.flush();
}
