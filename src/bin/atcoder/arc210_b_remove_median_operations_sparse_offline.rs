// AtCoder: ARC210 B - Remove Median Operations
// https://atcoder.jp/contests/arc210/tasks/arc210_b

use anmitsu::algebra::{monoid, semi_group};
use anmitsu::ds::segment_tree::segment_tree_sparse_offline;
use anmitsu::io::fastio;

/// 問題の最大値を含む半開区間の右端。
const DOMAIN_END: usize = 1_000_000_001;

/// 値ごとの個数と合計を同時に管理するモノイド。
struct CountSumMonoid;

impl semi_group::SemiGroup for CountSumMonoid {
    type S = (i64, i64);

    /// 隣接する値域の個数と合計を足す。
    fn op(a: &Self::S, b: &Self::S) -> Self::S {
        (a.0 + b.0, a.1 + b.1)
    }
}

impl monoid::Monoid for CountSumMonoid {
    /// 空の値域を表す。
    fn id() -> Self::S {
        (0, 0)
    }
}

/// 更新後の多重集合から小さい方と大きい方の各 N/2 個の和を求める。
fn main() {
    let mut io = fastio::Fastio::new();
    let n = io.u32() as usize;
    let m = io.u32() as usize;
    let q = io.u32() as usize;
    let mut a: Vec<_> = (0..n).map(|_| io.u32() as usize).collect();
    let mut b: Vec<_> = (0..m).map(|_| io.u32() as usize).collect();

    let mut initial: Vec<_> = a.iter().chain(&b).copied().collect();
    let mut points = initial.clone();
    let mut queries = Vec::with_capacity(q);
    for _ in 0..q {
        let kind = io.u32();
        let index = io.u32() as usize - 1;
        let value = io.u32() as usize;
        points.push(value);
        queries.push((kind, index, value));
    }

    let mut seg = segment_tree_sparse_offline::SegmentTreeSparseOffline::<CountSumMonoid>::new(
        DOMAIN_END, points,
    );
    initial.sort_unstable();
    let mut start = 0;
    while start < initial.len() {
        let value = initial[start];
        let mut end = start + 1;
        while end < initial.len() && initial[end] == value {
            end += 1;
        }
        let count = (end - start) as i64;
        seg.set(value, (count, count * value as i64));
        start = end;
    }
    seg.build();

    let half = (n / 2) as i64;
    for (kind, index, value) in queries {
        let old = match kind {
            1 => std::mem::replace(&mut a[index], value),
            2 => std::mem::replace(&mut b[index], value),
            _ => unreachable!(),
        };
        if old != value {
            let previous = seg.get(old);
            seg.update(old, (previous.0 - 1, previous.1 - old as i64));
            let next = seg.get(value);
            seg.update(value, (next.0 + 1, next.1 + value as i64));
        }

        // 中央値を順に除いた後には、全要素の両端 N/2 個ずつが残る。
        let small_boundary = seg.max_right(0, |&(count, _)| count < half);
        let small_before = seg.fold(0, small_boundary);
        let small_sum = small_before.1 + (half - small_before.0) * small_boundary as i64;

        // min_left の返す位置は、境界値の 1 つ右である。
        let large_boundary = seg.min_left(DOMAIN_END, |&(count, _)| count < half);
        let large_after = seg.fold(large_boundary, DOMAIN_END);
        let large_sum = large_after.1 + (half - large_after.0) * (large_boundary - 1) as i64;
        io.writeln(small_sum + large_sum);
    }
    io.flush();
}
