// AtCoder: FPS 24 D - Sequence 2
// https://atcoder.jp/contests/fps-24/tasks/fps_24_d
//
// 長さ N で要素が 0 以上 M 以下の整数列のうち、昇順に並べ替えた列で隣接要素の偶奇が
// 常に異なるものの個数を求める。値が重複すると昇順で隣接する偶奇が一致してしまうため、
// これは [0, M] から相異なる N 個の値を選び、隣接する差が全て奇数になるように並べた
// 場合の個数 (集合の選び方) を N! 倍したものに等しい。差の内訳を数える母関数は
// 1/(1-x)^2 * (x/(1-x^2))^{N-1} で与えられる (先頭・末尾の余白 2 つと、内部 N-1 個の
// 奇数の隙間)。

use anmitsu::io::fastio::Fastio;
use anmitsu::modulo998244353::{fps, modulo};

fn main() {
    let mut io = Fastio::new();

    let n = io.u32() as usize;
    let m = io.u32() as usize;

    // 1/(1-x) の平方 (先頭・末尾の余白 2 つ分)。
    let inv_one_minus_x = fps::FPS::new(vec![1, modulo::neg(1)]).inverse(m).unwrap();
    let squared = inv_one_minus_x.clone() * inv_one_minus_x;

    // (x/(1-x^2))^{N-1} = x^{N-1} * (1-x^2)^{-(N-1)} (内部 N-1 個の奇数の隙間)。
    let inv_one_minus_x2 = fps::FPS::new(vec![1, 0, modulo::neg(1)])
        .inverse(m)
        .unwrap();
    let gap = inv_one_minus_x2.pow(n - 1, m).mul_xk(n - 1);

    let combined = squared * gap;

    let mut fact_n = 1_u32;
    for i in 1..=n {
        fact_n = modulo::mul(fact_n, i as u32);
    }

    io.writeln(modulo::mul(combined.get(m), fact_n));

    io.flush();
}
