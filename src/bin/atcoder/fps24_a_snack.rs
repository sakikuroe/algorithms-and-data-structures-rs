// AtCoder: FPS 24 A - Snack
// https://atcoder.jp/contests/fps-24/tasks/fps_24_a
//
// D 日間毎日 1, 3, 4, 6 円のいずれかを支払う行動を選び、合計がちょうど N 円になる
// 行動パターン数を数える。これは (x + x^3 + x^4 + x^6)^D の x^N 係数に等しい。

use anmitsu::io::fastio::Fastio;
use anmitsu::modulo998244353::fps;

fn main() {
    let mut io = Fastio::new();

    let d = io.u32() as usize;
    let n = io.u32() as usize;

    let mut coeffs = vec![0_u32; 7];
    for &e in &[1, 3, 4, 6] {
        coeffs[e] = 1;
    }
    let base = fps::FPS::new(coeffs);
    let result = base.pow(d, n);

    io.writeln(result.get(n));

    io.flush();
}
