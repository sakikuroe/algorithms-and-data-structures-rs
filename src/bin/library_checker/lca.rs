// Library Checker: Lowest Common Ancestor
// https://judge.yosupo.jp/problem/lca

use anmitsu::algebra::monoid::MinMonoid;
use anmitsu::ds::segment_tree::segment_tree_dense::SegmentTreeDense;
use anmitsu::graph::graph::Graph;
use anmitsu::io::fastio::Fastio;

fn main() {
    let mut io = Fastio::new();

    let n = io.u32() as usize;
    let q = io.u32() as usize;

    let mut g = Graph::new(n);
    for i in 1..n {
        let p = io.u32() as usize;
        g.add_edge(p, i, ());
    }

    let forest = g.dfs_forest();
    let tour = forest.euler_tour();
    let m = tour.len();

    // オイラーツアー上の各位置を (その位置の頂点の深さ, 位置そのもの) の組として
    // エンコードし、区間最小値の segment tree に載せる。深さが最小となる位置を
    // 求めれば、その位置の頂点が LCA になる。位置は m 未満なので、
    // 深さを m 倍したものに足し込んでも情報が失われない。
    let mut seg = SegmentTreeDense::<MinMonoid>::new(m);
    for (i, &v) in tour.iter().enumerate() {
        seg.set(i, forest.depth(v) as i64 * m as i64 + i as i64);
    }
    seg.build();

    for _ in 0..q {
        let u = io.u32() as usize;
        let v = io.u32() as usize;
        let lo = forest.first_occurrence(u).min(forest.first_occurrence(v));
        let hi = forest.first_occurrence(u).max(forest.first_occurrence(v));
        let key = seg.fold(lo, hi + 1);
        let idx = (key % m as i64) as usize;
        io.writeln(tour[idx] as u32);
    }

    io.flush();
}
