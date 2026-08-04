// AtCoder: ABC422 G - Balls and Boxes
// https://atcoder.jp/contests/abc422/tasks/abc422_g
//
// N 個のボールを 3 つの箱に、箱 1 の個数が A の倍数、箱 2 が B の倍数、箱 3 が C の
// 倍数となるように入れる方法の数を、区別できないボール (問題 1) と区別できる
// ボール (問題 2) のそれぞれについて求める。
//
// 問題 1: 各箱の個数選択は Σ_{k≡0 (mod s)} x^{k} = 1/(1-x^s) と表せるため、答えは
// 1 / ((1-x^A)(1-x^B)(1-x^C)) の x^N 係数に等しい。
//
// 問題 2: ボールが区別できるため指数型母関数を用いる。各箱の個数選択は
// Σ_{k≡0 (mod s)} x^k / k! と表せ、答えはこれら 3 つの総積の x^N 係数に N! を
// 掛けたものに等しい。

use anmitsu::io::fastio::Fastio;
use anmitsu::modulo998244353::fps::partition;
use anmitsu::modulo998244353::{fps, modulo};

fn main() {
    let mut io = Fastio::new();

    let n = io.u32() as usize;
    let a = io.u32() as usize;
    let b = io.u32() as usize;
    let c = io.u32() as usize;

    // 問題 1: 1 / ((1-x^A)(1-x^B)(1-x^C)) の x^N 係数。
    let denominators = [a as u32, b as u32, c as u32];
    let indistinguishable = partition::product_inv_one_minus_x_powers(&denominators, n).unwrap();
    io.writeln(indistinguishable.get(n));

    // 問題 2: 階乗と階乗の逆元のテーブルを、指数型母関数の係数として用いる。
    let mut fact = vec![1_u32; n + 1];
    for i in 1..=n {
        fact[i] = modulo::mul(fact[i - 1], i as u32);
    }
    let mut inv_fact = vec![1_u32; n + 1];
    inv_fact[n] = modulo::inv(fact[n]);
    for i in (1..=n).rev() {
        inv_fact[i - 1] = modulo::mul(inv_fact[i], i as u32);
    }

    // 箱の容量 s ごとに、Σ_{k≡0 (mod s)} x^k / k! を構築する。
    let egf_for_step = |step: usize| -> fps::FPS {
        let mut coeffs = vec![0_u32; n + 1];
        let mut k = 0;
        while step * k <= n {
            coeffs[step * k] = inv_fact[step * k];
            k += 1;
        }
        fps::FPS::new(coeffs)
    };

    let polynomials = vec![egf_for_step(a), egf_for_step(b), egf_for_step(c)];
    let distinguishable = fps::FPS::product(polynomials, n);
    io.writeln(modulo::mul(distinguishable.get(n), fact[n]));

    io.flush();
}
