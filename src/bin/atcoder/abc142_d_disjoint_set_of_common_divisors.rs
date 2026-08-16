// AtCoder: ABC142 D - Disjoint Set of Common Divisors
// https://atcoder.jp/contests/abc142/tasks/abc142_d

use anmitsu::io::fastio::Fastio;
use anmitsu::math::number_theory;
use anmitsu::math::primality;

fn main() {
    let mut io = Fastio::new();

    let a = io.u64();
    let b = io.u64();

    // A, B の正の公約数から互いに素なものを選べるだけ選ぶ操作は、gcd(A, B) の
    // 相異なる素因数それぞれから高々 1 個ずつ選ぶことに帰着する。素因数を 1 個も
    // 含まない `1` も互いに素な集合に加えられるため、素因数の種類数に 1 を足した
    // ものが答えになる。
    let g = number_theory::gcd(a as u128, b as u128) as u64;
    let answer = primality::factorize(g).len() as u64 + 1;

    io.writeln(answer);
    io.flush();
}
