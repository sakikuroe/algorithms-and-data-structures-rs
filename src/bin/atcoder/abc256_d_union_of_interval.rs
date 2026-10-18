//! AtCoder ABC256 D - Union of Interval を 1 の区間列挙で解きます。
//!
//! 問題: https://atcoder.jp/contests/abc256/tasks/abc256_d

use anmitsu::{ds::interval_bit_set, io::fastio};
use std::io::{self, Write};

/// 入力区間の和集合を、最小個数の半開区間として出力します。
fn main() {
    let mut io = fastio::Fastio::new();
    let n = io.u32() as usize;
    let mut covered = interval_bit_set::IntervalBitSet::new();
    for _ in 0..n {
        let left = io.u32() as usize;
        let right = io.u32() as usize;
        covered.set(left..right, true);
    }

    // 半開区間の両端を同じ行に出力するため、標準のバッファーを使います。
    let stdout = io::stdout();
    let mut output = io::BufWriter::new(stdout.lock());
    // 右端は問題の制約で 20 万以下なので、閉区間から半開区間へ安全に戻せます。
    for run in covered.ranges(.., true) {
        writeln!(output, "{} {}", run.start(), run.end() + 1).expect("failed to write interval");
    }
    output.flush().expect("failed to flush output");
}
