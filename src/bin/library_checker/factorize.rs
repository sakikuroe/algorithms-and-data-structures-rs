// Library Checker: Factorize
// https://judge.yosupo.jp/problem/factorize

use anmitsu::io::fastio::Fastio;
use anmitsu::math::primality;

/// 数値を、空白を挟まずにそのまま出力バッファへ書き込む。
///
/// # Args
/// - `io` - 数値を追記する出力バッファ。
/// - `value` - 出力する非負整数。
///
/// # Returns
/// 返り値はない。数値の文字列表現を `io` に追記する。
fn write_value(io: &mut Fastio, value: u64) {
    for c in value.to_string().chars() {
        io.write(c);
    }
}

/// 各入力値の素因数分解を昇順の素因数列に展開して出力する。
fn main() {
    let mut io = Fastio::new();

    let q = io.u32();
    for _ in 0..q {
        let a = io.u64();

        // 素因数分解の結果 (素因数, 指数) から、指数の個数だけ素因数を並べた
        // 昇順の列を作る。
        let mut factors = Vec::new();
        for (p, e) in primality::factorize(a) {
            factors.extend(std::iter::repeat_n(p, e));
        }
        factors.sort_unstable();

        write_value(&mut io, factors.len() as u64);
        for f in factors {
            io.write(' ');
            write_value(&mut io, f);
        }
        io.write('\n');
    }

    io.flush();
}
