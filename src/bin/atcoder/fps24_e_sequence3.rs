// AtCoder: FPS 24 E - Sequence 3
// https://atcoder.jp/contests/fps-24/tasks/fps_24_e
//
// 長さ N の数列で、各値 m (1 <= m <= M) の出現回数が m 回以下であるものの個数を
// 数える。値 m ごとに「0 から m 回まで出現する」ことを表す指数型母関数
// Σ_{k=0}^{m} x^k / k! の総積を取り、x^N 係数に N! を掛けたものが答えになる。

use anmitsu::io::fastio::Fastio;
use anmitsu::modulo998244353::{fps, modulo};

fn main() {
    let mut io = Fastio::new();

    let n = io.u32() as usize;
    let m = io.u32() as usize;

    // n! までの階乗と、その逆元のテーブルを構築する。
    let mut fact = vec![1_u32; n + 1];
    for i in 1..=n {
        fact[i] = modulo::mul(fact[i - 1], i as u32);
    }
    let mut inv_fact = vec![1_u32; n + 1];
    inv_fact[n] = modulo::inv(fact[n]);
    for i in (1..=n).rev() {
        inv_fact[i - 1] = modulo::mul(inv_fact[i], i as u32);
    }

    // 値 m ごとの指数型母関数 Σ_{k=0}^{min(m,n)} x^k / k! を構築する。
    let mut polynomials = Vec::with_capacity(m);
    for value in 1..=m {
        let len = value.min(n) + 1;
        polynomials.push(fps::FPS::new(inv_fact[..len].to_vec()));
    }

    let product = fps::FPS::product(polynomials, n);

    io.writeln(modulo::mul(product.get(n), fact[n]));

    io.flush();
}
