//! AtCoder ABC480 C - Self-Stopping Shower を補集合の区間列挙で解きます。
//!
//! 問題: https://atcoder.jp/contests/abc480/tasks/abc480_c

use anmitsu::{ds::interval_bit_set, io::fastio};

/// 給湯されない各時間帯を埋めるために必要なボタン操作数を出力します。
fn main() {
    let mut io = fastio::Fastio::new();
    let n = io.u32() as usize;
    let k = io.u32() as usize;
    let t = io.u32() as usize;
    let mut hot = interval_bit_set::IntervalBitSet::new();

    // 添字 x は時間帯 [x, x + 1) を表し、時刻 T で列挙範囲を切り取ります。
    for _ in 0..n {
        let a = io.u32() as usize;
        hot.set(a..a + k, true);
    }

    // 無料の給湯区間はそれぞれ K 秒以上あり、異なる隙間は独立に埋められます。
    let answer = hot
        .ranges(0..t, false)
        .map(|gap| (gap.end() - gap.start() + 1).div_ceil(k) as u64)
        .sum::<u64>();
    io.writeln(answer);
    io.flush();
}
