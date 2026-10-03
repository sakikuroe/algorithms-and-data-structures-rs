// Library Checker: Point Add Rectangle Sum
// https://judge.yosupo.jp/problem/point_add_rectangle_sum

use anmitsu::{algebra::monoid, ds::segment_tree::segment_tree_2d_sparse_offline, io::fastio};

/// 問題で取りうる点座標の上限 10^9 を含む、論理上の各軸の長さ。
const COORDINATE_LIMIT: usize = 1_000_000_001;

/// 更新点の事前登録後に入力順で処理するクエリ。
enum Query {
    /// 指定した点へ重みを加える。
    Add {
        /// 点の座標。
        point: (usize, usize),
        /// 加える重み。
        weight: i64,
    },
    /// 半開矩形の重みの総和を求める。
    Fold {
        /// x 座標の下限。
        left: usize,
        /// y 座標の下限。
        bottom: usize,
        /// x 座標の上限。
        right: usize,
        /// y 座標の上限。
        top: usize,
    },
}

/// 更新する可能性がある点だけを登録して矩形和を処理する。
fn main() {
    let mut io = fastio::Fastio::new();
    let n = io.u32() as usize;
    let q = io.u32() as usize;

    let mut initial_points = Vec::with_capacity(n);
    let mut registered_points = Vec::with_capacity(n + q);
    for _ in 0..n {
        let point = (io.u32() as usize, io.u32() as usize);
        let weight = io.i64();
        initial_points.push((point, weight));
        registered_points.push(point);
    }

    // offline 版には将来の更新点も必要だが、矩形端点の登録は不要。
    let mut queries = Vec::with_capacity(q);
    for _ in 0..q {
        if io.u32() == 0 {
            let point = (io.u32() as usize, io.u32() as usize);
            let weight = io.i64();
            registered_points.push(point);
            queries.push(Query::Add { point, weight });
        } else {
            queries.push(Query::Fold {
                left: io.u32() as usize,
                bottom: io.u32() as usize,
                right: io.u32() as usize,
                top: io.u32() as usize,
            });
        }
    }

    let mut seg =
        segment_tree_2d_sparse_offline::SegmentTree2dSparseOffline::<monoid::AddMonoid>::new(
            COORDINATE_LIMIT,
            COORDINATE_LIMIT,
            registered_points,
        );
    for (point, weight) in initial_points {
        seg.set(point, seg.get(point) + weight);
    }
    seg.build();

    for query in queries {
        match query {
            Query::Add { point, weight } => {
                seg.update(point, seg.get(point) + weight);
            }
            Query::Fold {
                left,
                bottom,
                right,
                top,
            } => io.writeln(seg.fold(left..right, bottom..top)),
        }
    }
    io.flush();
}
