// Library Checker: Enumerate Primes
// https://judge.yosupo.jp/problem/enumerate_primes

use anmitsu::io::fastio::Fastio;
use anmitsu::math::sieve;

/// 数値を、空白を挟まずにそのまま出力バッファへ書き込む。
fn write_value(io: &mut Fastio, value: usize) {
    for c in value.to_string().chars() {
        io.write(c);
    }
}

fn main() {
    let mut io = Fastio::new();

    let n = io.u64() as usize;
    let a = io.u64() as usize;
    let b = io.u64() as usize;

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
