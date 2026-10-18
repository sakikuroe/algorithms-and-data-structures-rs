//! AtCoder ABC330 E - Mex and Update を値の有無を表すビット列で解きます。
//!
//! 問題: https://atcoder.jp/contests/abc330/tasks/abc330_e

use anmitsu::{ds::interval_bit_set, io::fastio};

/// 出現回数が 0 と 1 の間で変わる値を更新し、各操作後の mex を出力します。
fn main() {
    let mut io = fastio::Fastio::new();
    let n = io.u32() as usize;
    let q = io.u32() as usize;
    let mut values = Vec::with_capacity(n);
    let mut frequency = vec![0_usize; n + 1];
    let mut present = interval_bit_set::IntervalBitSet::new();

    // N 個の値の mex は N 以下なので、それより大きい値は管理不要です。
    for _ in 0..n {
        let value = io.u32() as usize;
        values.push(value);
        if value <= n {
            frequency[value] += 1;
            if frequency[value] == 1 {
                present.set(value..=value, true);
            }
        }
    }

    for _ in 0..q {
        let index = io.usize1();
        let value = io.u32() as usize;
        let previous = values[index];

        // 重複する値を一つ除いても、残りがあれば登録状態を維持します。
        if previous <= n {
            frequency[previous] -= 1;
            if frequency[previous] == 0 {
                present.set(previous..=previous, false);
            }
        }
        if value <= n {
            frequency[value] += 1;
            if frequency[value] == 1 {
                present.set(value..=value, true);
            }
        }
        values[index] = value;

        // 未登録の最小値を探す操作が、そのまま mex の問い合わせになります。
        let mex = present.find(.., false).expect("mex must exist");
        io.writeln(mex as u64);
    }
    io.flush();
}
