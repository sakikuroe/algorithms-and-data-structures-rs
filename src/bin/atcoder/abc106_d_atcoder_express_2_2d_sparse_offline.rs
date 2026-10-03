// AtCoder: ABC106 D - AtCoder Express 2
// https://atcoder.jp/contests/abc106/tasks/abc106_d

use anmitsu::{algebra::monoid, ds::segment_tree::segment_tree_2d_sparse_offline, io::fastio};

/// 始点・終点を事前登録し、区間内で完結する列車を数える。
fn main() {
    let mut io = fastio::Fastio::new();
    let n = io.u32() as usize;
    let m = io.u32() as usize;
    let q = io.u32() as usize;
    let mut counts = vec![vec![0_i64; n]; n];
    let mut points = Vec::with_capacity(m);
    for _ in 0..m {
        let left = io.usize1();
        let right = io.usize1();
        counts[left][right] += 1;
        points.push((left, right));
    }

    let mut seg =
        segment_tree_2d_sparse_offline::SegmentTree2dSparseOffline::<monoid::AddMonoid>::new(
            n, n, points,
        );
    // 重複した座標をまとめ、登録された葉だけ設定して一度だけ構築する。
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
