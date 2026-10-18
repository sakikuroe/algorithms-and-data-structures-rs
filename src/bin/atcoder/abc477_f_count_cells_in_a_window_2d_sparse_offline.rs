// AtCoder: ABC477 F - Count Cells in a Window
// https://atcoder.jp/contests/abc477/tasks/abc477_f

use anmitsu::{
    algebra::{monoid, semi_group},
    ds::segment_tree::segment_tree_2d_sparse_offline,
    io::fastio,
};

/// 列の差分の個数と、差分に列座標を掛けた値の和を保持する可換モノイド。
struct DifferenceMonoid;

impl semi_group::SemiGroup for DifferenceMonoid {
    type S = (i64, i64);

    /// 二つの矩形にある差分の個数と加重和をそれぞれ加える。
    fn op(left: &Self::S, right: &Self::S) -> Self::S {
        (left.0 + right.0, left.1 + right.1)
    }
}

impl monoid::Monoid for DifferenceMonoid {
    /// 差分がない矩形を表す。
    fn id() -> Self::S {
        (0, 0)
    }
}

impl monoid::CommutativeMonoid for DifferenceMonoid {}

/// 各行の黒区間を二つの差分点で表し、矩形内の黒マスを数える。
fn main() {
    let mut io = fastio::Fastio::new();
    let n = io.u32() as usize;
    let _m = io.u32();
    let q = io.u32() as usize;
    let mut intervals = Vec::with_capacity(n);
    let mut points = Vec::with_capacity(n * 2);
    for row in 0..n {
        let left = io.usize1();
        let right = io.u32() as usize;
        intervals.push((left, right));
        points.push((row, left));
        points.push((row, right));
    }

    // 黒区間 [left, right) は left で +1、right で -1 の差分になる。
    // 列 M の差分も更新候補点として登録する。
    let mut seg =
        segment_tree_2d_sparse_offline::SegmentTree2dSparseOffline::<DifferenceMonoid>::new(points);
    for (row, &(left, right)) in intervals.iter().enumerate() {
        seg.set((row, left), (1, left as i64));
        seg.set((row, right), (-1, -(right as i64)));
    }
    seg.build();
    seg.prepare_prefix_folds();

    for _ in 0..q {
        let top = io.usize1();
        let bottom = io.u32() as usize;
        let left = io.usize1();
        let right = io.u32() as usize;

        // 差分の接頭辞 (個数, 加重和) を (count, weight) とすると、
        // 先頭 x 列の黒マス数は x * count - weight になる。
        let ((right_count, right_weight), (left_count, left_weight)) =
            seg.fold_prefix_pair(top..bottom, right, left);
        let answer =
            right as i64 * right_count - right_weight - (left as i64 * left_count - left_weight);
        io.writeln(answer);
    }
    io.flush();
}
