// Library Checker: Enumerate Primes
// https://judge.yosupo.jp/problem/enumerate_primes

use anmitsu::io::fastio::Fastio;
use anmitsu::math::sieve;

/// 数値を、空白を挟まずにそのまま出力バッファへ書き込む。
///
/// # Args
/// - `io` - 数値を追記する出力バッファ。
/// - `value` - 出力する非負整数。
///
/// # Returns
/// 返り値はない。数値の文字列表現を `io` に追記する。
fn write_value(io: &mut Fastio, value: usize) {
    for c in value.to_string().chars() {
        io.write(c);
    }
}

/// `n` 以下の素数から、添字 `b` 以降の `a` 個おきの要素を列挙する。
fn main() {
    let mut io = Fastio::new();

    let n = io.u64() as usize;
    let a = io.u64() as usize;
    let b = io.u64() as usize;

    // 昇順の素数列から、0 始まりの添字 b, b+a, b+2a, ... にある
    // 素数だけを取り出す。
    let primes = sieve::primes_up_to(n);
    let picked = primes
        .iter()
        .copied()
        .skip(b)
        .step_by(a)
        .collect::<Vec<usize>>();

    write_value(&mut io, primes.len());
    io.write(' ');
    write_value(&mut io, picked.len());
    io.write('\n');
    for (i, &p) in picked.iter().enumerate() {
        if i > 0 {
            io.write(' ');
        }
        write_value(&mut io, p);
    }
    io.write('\n');

    io.flush();
}
