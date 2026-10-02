// AtCoder: ABC035 C - オセロ
// https://atcoder.jp/contests/abc035/tasks/abc035_c

use anmitsu::{algebra::monoid, ds::segment_tree::lazy_segment_tree_sparse_online, io::fastio};

/// 区間中の 0 と 1 を反転する作用。
#[derive(Clone)]
struct FlipEffect(bool);

impl lazy_segment_tree_sparse_online::RangeAction<i64> for FlipEffect {
    /// 区間長から反転前の 1 の個数を引き、反転後の個数を得る。
    fn apply(&self, value: &i64, len: usize) -> i64 {
        if self.0 { len as i64 - value } else { *value }
    }

    /// 反転を二度行うと元に戻るため、作用を排他的論理和で合成する。
    fn composition(&self, other: &Self) -> Self {
        Self(self.0 ^ other.0)
    }
}

/// 指定された各区間を反転し、最後の盤面を出力する。
fn main() {
    let mut io = fastio::Fastio::new();
    let n = io.u32() as usize;
    let q = io.u32() as usize;
    let mut seg = lazy_segment_tree_sparse_online::SegmentTreeLazySparseOnline::<
        monoid::AddMonoid,
        FlipEffect,
    >::new(n);

    for _ in 0..q {
        let left = io.usize1();
        let right = io.u32() as usize;
        seg.effect(left, right, FlipEffect(true));
    }

    for index in 0..n {
        io.write(if seg.get(index) == 0 { '0' } else { '1' });
    }
    io.write('\n');
    io.flush();
}
