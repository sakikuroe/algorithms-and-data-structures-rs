// AtCoder: FPS 24 B - Tuple of Integers
// https://atcoder.jp/contests/fps-24/tasks/fps_24_b
//
// a + b + c + d = N (a in {0,1}, b in {0,1,2}, c は偶数, d は 3 の倍数) を満たす
// 非負整数の組の個数を求める。母関数は (1+x)(1+x+x^2) / ((1-x^2)(1-x^3)) であり、
// N が大きいためこの有理関数の x^N 係数を Bostan-Mori 法で求める。

use anmitsu::io::fastio::Fastio;
use anmitsu::modulo998244353::{
    fps::{self, bostan_mori},
    modulo,
};

fn main() {
    let mut io = Fastio::new();

    let n = io.u32() as usize;

    // 分子 P(x) = (1 + x)(1 + x + x^2) = 1 + 2x + 2x^2 + x^3。
    let p = fps::FPS::new(vec![1, 2, 2, 1]);
    // 分母 Q(x) = (1 - x^2)(1 - x^3) = 1 - x^2 - x^3 + x^5。
    let q = fps::FPS::new(vec![1, 0, modulo::neg(1), modulo::neg(1), 0, 1]);

    io.writeln(bostan_mori::bostan_mori(&p, &q, n));

    io.flush();
}
