// AtCoder: FPS 24 N - Coin 2
// https://atcoder.jp/contests/fps-24/tasks/fps_24_n
//
// i 円硬貨を A_i 枚まで使って N 円をちょうど支払う方法の数を求める。i 円硬貨の枚数
// 選択は Σ_{k=0}^{A_i} x^{ik} = (1 - x^{i(A_i+1)}) / (1 - x^i) と表せるため、全体の
// 母関数は Π (1 - x^{i(A_i+1)}) / Π (1 - x^i) となる。分子・分母をそれぞれ partition
// モジュールの関数で計算してから掛け合わせる。

use anmitsu::io::fastio::Fastio;
use anmitsu::modulo998244353::fps::partition;

fn main() {
    let mut io = Fastio::new();

    let n = io.u32() as usize;

    let mut numerator_exponents = Vec::with_capacity(n);
    let mut denominator_exponents = Vec::with_capacity(n);
    for i in 1..=n {
        let a_i = io.u32() as u64;
        let exponent = (i as u64) * (a_i + 1);
        // 次数 n を超える指数は「制約なし (1 倍)」として扱われるため、
        // u32 に収まる範囲へ飽和させても結果は変わらない。
        let capped = exponent.min((n + 1) as u64) as u32;
        numerator_exponents.push(capped);
        denominator_exponents.push(i as u32);
    }

    let numerator = partition::product_one_minus_x_powers(&numerator_exponents, n).unwrap();
    let denominator = partition::product_inv_one_minus_x_powers(&denominator_exponents, n).unwrap();

    let result = numerator * denominator;

    io.writeln(result.get(n));

    io.flush();
}
