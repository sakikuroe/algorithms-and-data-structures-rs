//! AtCoder ABC435 E - Cover query を区間で圧縮したビット列で解きます。
//!
//! 問題: https://atcoder.jp/contests/abc435/tasks/abc435_e

use anmitsu::{ds::interval_bit_set, io::fastio};

/// 各塗りつぶし後に残る白いマスの個数を出力します。
fn main() {
    let mut io = fastio::Fastio::new();
    let n = io.u64() as usize;
    let q = io.u32() as usize;
    let mut bits = interval_bit_set::IntervalBitSet::new();
    let mut white = n as u128;

    // 既に黒いマスを二重に減らさないよう、値が変化した個数を使います。
    for _ in 0..q {
        let left = io.u64() as usize;
        let right = io.u64() as usize;
        white -= bits.set(left..=right, true);
        // 白マス数は問題の制約で 10 億以下なので、出力用の u64 に収まります。
        io.writeln(white as u64);
    }
    io.flush();
}
