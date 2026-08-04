// AtCoder: FPS 24 I - Score
// https://atcoder.jp/contests/fps-24/tasks/fps_24_i
//
// N 個の相異なる整数から K 個選ぶ全ての選び方について、選んだ整数の積の総和を求める。
// これは Π_{i=1}^{N} (1 + a_i x) の x^K 係数 (K 次基本対称式) に等しい。

use anmitsu::io::fastio::Fastio;
use anmitsu::modulo998244353::fps;

fn main() {
    let mut io = Fastio::new();

    let n = io.u32() as usize;
    let k = io.u32() as usize;

    let mut polynomials = Vec::with_capacity(n);
    for _ in 0..n {
        let a_i = io.u32();
        polynomials.push(fps::FPS::new(vec![1, a_i]));
    }

    let product = fps::FPS::product(polynomials, k);

    io.writeln(product.get(k));

    io.flush();
}
