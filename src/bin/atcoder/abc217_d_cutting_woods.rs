//! AtCoder ABC217 D - Cutting Woods を未切断の連続区間の取得で解きます。
//!
//! 問題: https://atcoder.jp/contests/abc217/tasks/abc217_d

use anmitsu::{ds::interval_bit_set, io::fastio};

/// 切断位置を 1 として登録し、問い合わせ位置を含む木材の長さを出力します。
fn main() {
    let mut io = fastio::Fastio::new();
    let length = io.u32() as usize;
    let q = io.u32() as usize;
    let mut cuts = interval_bit_set::IntervalBitSet::new();

    // 木材の外側も切断済みとすれば、両端の特別な場合分けを避けられます。
    cuts.set(..=0, true);
    cuts.set(length.., true);
    for _ in 0..q {
        let kind = io.u32();
        let x = io.u32() as usize;
        if kind == 1 {
            cuts.set(x..=x, true);
        } else {
            // 隣の切断点が a, b なら、未切断の目印は [a + 1, b - 1] です。
            // 目印の個数に 1 を加えると、木材の長さ b - a になります。
            let run = cuts.run_at(x);
            io.writeln((run.end() - run.start() + 2) as u64);
        }
    }
    io.flush();
}
