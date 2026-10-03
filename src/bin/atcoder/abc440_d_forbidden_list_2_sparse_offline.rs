// AtCoder: ABC440 D - Forbidden List 2
// https://atcoder.jp/contests/abc440/tasks/abc440_d

use anmitsu::{algebra::monoid, ds::segment_tree::segment_tree_sparse_offline, io::fastio};

/// 禁止数の間にある許可数の個数を管理し、各質問の答えを探す。
fn main() {
    let mut io = fastio::Fastio::new();
    let n = io.u32() as usize;
    let q = io.u32() as usize;
    let mut forbidden: Vec<_> = (0..n).map(|_| io.u32() as usize).collect();
    forbidden.sort_unstable();

    // 座標 a_i には、直前の禁止数と a_i の間にある許可数の個数を置く。
    let mut seg = segment_tree_sparse_offline::SegmentTreeSparseOffline::<monoid::AddMonoid>::new(
        forbidden.iter().copied(),
    );
    let mut previous = 0;
    for &value in &forbidden {
        seg.set(value, (value - previous - 1) as i64);
        previous = value;
    }
    seg.build();

    for _ in 0..q {
        let x = io.u32() as usize;
        let y = io.u32() as usize;
        let first = forbidden.partition_point(|&value| value < x);
        if first == n {
            io.writeln((x + y - 1) as u64);
            continue;
        }

        let before_first = forbidden[first] - x;
        if y <= before_first {
            io.writeln((x + y - 1) as u64);
            continue;
        }

        // 最初の禁止数を越えた後、答えを含む空き区間を探す。
        let remaining = y - before_first;
        let start = forbidden[first] + 1;
        let boundary = seg.max_right(start, |&count| count < remaining as i64);
        let passed = seg.fold(start..boundary) as usize;
        let left = if boundary == usize::MAX {
            forbidden[n - 1]
        } else {
            let rank = forbidden.binary_search(&boundary).unwrap();
            forbidden[rank - 1]
        };
        io.writeln((left + remaining - passed) as u64);
    }
    io.flush();
}
