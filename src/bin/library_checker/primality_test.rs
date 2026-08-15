// Library Checker: Primality Test
// https://judge.yosupo.jp/problem/primality_test

use anmitsu::io::fastio::Fastio;
use anmitsu::math::primality;

/// 文字列トークンを、空白を挟まずにそのまま出力バッファへ書き込む。
fn write_token(io: &mut Fastio, token: &str) {
    for c in token.chars() {
        io.write(c);
    }
}

fn main() {
    let mut io = Fastio::new();

    let q = io.u32();
    for _ in 0..q {
        let n = io.u64();
        write_token(&mut io, if primality::is_prime(n) { "Yes" } else { "No" });
        io.write('\n');
    }

    io.flush();
}
