//! AtCoder ABC294 E - 2xN Grid を、値ごとの位置集合の AND で解きます。
//!
//! 問題: https://atcoder.jp/contests/abc294/tasks/abc294_e

use anmitsu::{ds::interval_bit_set, io::fastio};
use std::collections;

/// 同じ値を持つ位置集合の共通部分を数え、上下が一致する列数を出力します。
fn main() {
    let mut io = fastio::Fastio::new();
    let length = io.u64() as usize;
    let row_sizes = [io.u32() as usize, io.u32() as usize];
    let mut rows = [collections::BTreeMap::new(), collections::BTreeMap::new()];

    // 連長を座標として扱い、列数が大きくても実際のマスには展開しません。
    for (row, size) in rows.iter_mut().zip(row_sizes) {
        let mut offset = 0;
        for _ in 0..size {
            let value = io.u32();
            let run_length = io.u64() as usize;
            row.entry(value)
                .or_insert_with(interval_bit_set::IntervalBitSet::new)
                .set(offset..offset + run_length, true);
            offset += run_length;
        }
        debug_assert_eq!(length, offset);
    }

    // 値ごとの共通位置は重ならないため、それぞれの個数を足せます。
    let [upper, lower] = rows;
    let mut answer = 0;
    for (value, upper_positions) in upper {
        if let Some(lower_positions) = lower.get(&value) {
            answer += (&upper_positions & lower_positions).count(.., true);
        }
    }
    // 一致列数は問題の制約で 10^12 以下なので、出力用の u64 に収まります。
    io.writeln(answer as u64);
    io.flush();
}
