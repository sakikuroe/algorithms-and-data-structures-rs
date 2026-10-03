// Library Checker: Point Add Rectangle Sum
// https://judge.yosupo.jp/problem/point_add_rectangle_sum

use anmitsu::{algebra::monoid, ds::segment_tree::segment_tree_2d_sparse_online, io::fastio};

/// 問題で取りうる点座標の上限 10^9 を含む、論理上の各軸の長さ。
const COORDINATE_LIMIT: usize = 1_000_000_001;

/// 入力を先読みせず、更新時に必要な節点だけを生成して矩形和を処理する。
fn main() {
    let mut io = fastio::Fastio::new();
    let n = io.u32() as usize;
    let q = io.u32() as usize;
    let mut seg =
        segment_tree_2d_sparse_online::SegmentTree2dSparseOnline::<monoid::AddMonoid>::new(
            COORDINATE_LIMIT,
            COORDINATE_LIMIT,
        );

    for _ in 0..n {
        let point = (io.u32() as usize, io.u32() as usize);
        let weight = io.i64();
        seg.update(point, seg.get(point) + weight);
    }
    for _ in 0..q {
        if io.u32() == 0 {
            let point = (io.u32() as usize, io.u32() as usize);
            let weight = io.i64();
            seg.update(point, seg.get(point) + weight);
        } else {
            let left = io.u32() as usize;
            let bottom = io.u32() as usize;
            let right = io.u32() as usize;
            let top = io.u32() as usize;
            io.writeln(seg.fold(left..right, bottom..top));
        }
    }
    io.flush();
}
