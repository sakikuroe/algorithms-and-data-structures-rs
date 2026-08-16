// AtCoder: ABC177 E - Coprime
// https://atcoder.jp/contests/abc177/tasks/abc177_e

use anmitsu::io::fastio::Fastio;
use anmitsu::math::number_theory;
use anmitsu::math::sieve;

/// 文字列トークンを、改行を付けて出力バッファへ書き込む。
fn write_line(io: &mut Fastio, token: &str) {
    for c in token.chars() {
        io.write(c);
    }
    io.write('\n');
}

fn main() {
    let mut io = Fastio::new();

    let n = io.u32() as usize;
    let a = (0..n).map(|_| io.u64()).collect::<Vec<u64>>();

    // 全体の gcd が 1 でなければ、どの 2 つを取っても互いに素にはなり得ない。
    let setwise_gcd = a
        .iter()
        .fold(0_u128, |acc, &x| number_theory::gcd(acc, x as u128));
    if setwise_gcd != 1 {
        write_line(&mut io, "not coprime");
        io.flush();
        return;
    }

    // pairwise coprime であるかどうかは、各素数が 2 つ以上の A_i の素因数として
    // 現れないことと同値である。線形篩で前計算した最小素因数を使い、各 A_i を
    // 高速に素因数分解しながら、既に出現した素数と重複しないかを調べる。
    let max_a = *a.iter().max().unwrap() as usize;
    let spf = sieve::smallest_prime_factors(max_a);

    let mut seen = vec![false; max_a + 1];
    let mut pairwise = true;
    'outer: for &x in &a {
        let mut x = x as usize;
        while x > 1 {
            let p = spf[x];
            if seen[p] {
                pairwise = false;
                break 'outer;
            }
            seen[p] = true;
            while x % p == 0 {
                x /= p;
            }
        }
    }

    write_line(
        &mut io,
        if pairwise {
            "pairwise coprime"
        } else {
            "setwise coprime"
        },
    );
    io.flush();
}
