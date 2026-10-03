// AtCoder: ABC106 D - AtCoder Express 2
// https://atcoder.jp/contests/abc106/tasks/abc106_d

use anmitsu::{algebra::monoid, ds::segment_tree::segment_tree_2d_dense, io::fastio};

/// 始点と終点を 2 次元の点とみなし、区間内で完結する列車を数える。
fn main() {
    let mut io = fastio::Fastio::new();
    let n = io.u32() as usize;
    let m = io.u32() as usize;
    let q = io.u32() as usize;
    let mut counts = vec![vec![0_i64; n]; n];
    for _ in 0..m {
        let left = io.usize1();
        let right = io.usize1();
        counts[left][right] += 1;
    }

    let mut seg = segment_tree_2d_dense::SegmentTree2dDense::<monoid::AddMonoid>::new(n, n);
    // 同じ始点・終点の列車をまとめ、葉を設定してから一度だけ構築する。
    for (row, columns) in counts.iter().enumerate() {
        for (col, &count) in columns.iter().enumerate() {
            if count != 0 {
                seg.set((row, col), count);
            }
        }
    }
    seg.build();

    for _ in 0..q {
        let start = io.usize1();
        let end = io.usize1();
        // 始点が start 以上で終点が end 以下の点は正方形内にある。
        io.writeln(seg.fold(start..(end + 1), start..(end + 1)));
    }
    io.flush();
}
