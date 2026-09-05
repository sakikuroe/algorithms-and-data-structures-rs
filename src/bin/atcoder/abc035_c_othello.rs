// AtCoder: ABC035 C - オセロ
// https://atcoder.jp/contests/abc035/tasks/abc035_c

use anmitsu::algebra::monoid;
use anmitsu::ds::segment_tree::lazy_segment_tree;
use anmitsu::io::fastio;

/// 反転操作を表す作用。`true` のとき値を反転する。
#[derive(Clone)]
struct FlipEffect(bool);

impl lazy_segment_tree::Hom<i64> for FlipEffect {
    /// 反転フラグが `true` であれば `1 - x` を返し、
    /// `false` であればそのまま返す。
    fn f(&self, x: &i64) -> i64 {
        if self.0 { 1 - x } else { *x }
    }

    /// 2 つの反転操作を合成する。反転の偶奇が結果を決める
    /// ため、排他的論理和で合成する。
    fn composition(&self, other: &Self) -> Self {
        FlipEffect(self.0 ^ other.0)
    }
}

fn main() {
    let mut io = fastio::Fastio::new();

    // 盤面のサイズとクエリ数を読み込む。
    let n = io.u32() as usize;
    let q = io.u32() as usize;

    // 初期状態はすべて 0 (白) の盤面である。
    // 加法モノイド上の遅延セグメント木として構築する。
    let mut seg = lazy_segment_tree::SegmentTreeLazyDense::<monoid::AddMonoid, FlipEffect>::new(n);

    // 各クエリで指定された区間を反転する。
    // 入力は 1-indexed であるため、左端を 0-indexed に変換する。
    for _ in 0..q {
        let l = io.u32() as usize;
        let r = io.u32() as usize;
        seg.effect(l - 1, r, FlipEffect(true));
    }

    // 各マスの最終状態を出力する。
    for i in 0..n {
        let v = seg.fold(i, i + 1);
        io.write(if v == 1 { '1' } else { '0' });
    }
    io.write('\n');

    io.flush();
}
