// AtCoder: ABC241 D - Sequence Query
// https://atcoder.jp/contests/abc241/tasks/abc241_d

use anmitsu::{algebra::monoid, ds::segment_tree::segment_tree_sparse_online, io::fastio};

/// 問題の最大値を含めるための半開区間の右端。
const DOMAIN_END: usize = 1_000_000_000_000_000_001;

/// 入力を順に読み、未知の挿入座標をその場で追加して順位を求める。
fn main() {
    let mut io = fastio::Fastio::new();
    let q = io.u32() as usize;
    let mut seg =
        segment_tree_sparse_online::SegmentTreeSparseOnline::<monoid::AddMonoid>::new(DOMAIN_END);

    for _ in 0..q {
        let kind = io.u32();
        let x = io.u64() as usize;
        match kind {
            1 => seg.update(x, seg.get(x) + 1),
            2 => {
                let k = io.u32() as i64;
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
                let k = io.u32() as i64;
                // 前方へ k 個含めたときに述語が初めて偽になる位置が答えになる。
                let boundary = seg.max_right(x, |&count| count < k);
                let answer = if boundary == DOMAIN_END {
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
