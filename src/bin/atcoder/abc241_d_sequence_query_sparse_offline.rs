// AtCoder: ABC241 D - Sequence Query
// https://atcoder.jp/contests/abc241/tasks/abc241_d

use anmitsu::{algebra::monoid, ds::segment_tree::segment_tree_sparse_offline, io::fastio};

/// 入力で現れる挿入座標を圧縮し、順位クエリを処理する。
fn main() {
    let mut io = fastio::Fastio::new();
    let q = io.u32() as usize;
    let mut queries = Vec::with_capacity(q);
    let mut points = Vec::new();

    // 更新する座標だけを登録し、クエリの端点は登録しない。
    for _ in 0..q {
        let kind = io.u32();
        let x = io.u64() as usize;
        let k = if kind == 1 { 0_i64 } else { io.u32() as i64 };
        if kind == 1 {
            points.push(x);
        }
        queries.push((kind, x, k));
    }

    let mut seg =
        segment_tree_sparse_offline::SegmentTreeSparseOffline::<monoid::AddMonoid>::new(points);

    for (kind, x, k) in queries {
        match kind {
            1 => seg.update(x, seg.get(x) + 1),
            2 => {
                // 後方へ k 個含めたときの最初の失敗位置は答えの直後になる。
                let boundary = seg.min_left(x + 1, |&count| count < k);
                let answer = if boundary == 0 {
                    -1_i64
                } else {
                    (boundary - 1) as i64
                };
                io.writeln(answer);
            }
            3 => {
                // 前方へ k 個含めたときに述語が初めて偽になる位置が答えになる。
                let boundary = seg.max_right(x, |&count| count < k);
                let answer = if boundary == usize::MAX {
                    -1_i64
                } else {
                    boundary as i64
                };
                io.writeln(answer);
            }
            _ => unreachable!(),
        }
    }
    io.flush();
}
