// AtCoder: FPS 24 C - Sequence
// https://atcoder.jp/contests/fps-24/tasks/fps_24_c
//
// 長さ N で各要素が 0 以上 M 以下の整数からなる数列のうち、総和がちょうど S になる
// ものの個数を数える。これは (1 + x + ... + x^M)^N の x^S 係数に等しい。

use anmitsu::io::fastio::Fastio;
use anmitsu::modulo998244353::fps;

fn main() {
    let mut io = Fastio::new();

    let n = io.u32() as usize;
    let m = io.u32() as usize;
    let s = io.u32() as usize;

    // 1 + x + ... + x^M を構築し、N 乗した結果の x^S 係数を求める。
    let base = fps::FPS::new(vec![1_u32; m + 1]);
    let result = base.pow(n, s);

    io.writeln(result.get(s));

    io.flush();
}
