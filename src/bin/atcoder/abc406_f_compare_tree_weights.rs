// AtCoder: ABC406 F - Compare Tree Weights
// https://atcoder.jp/contests/abc406/tasks/abc406_f

use anmitsu::algebra::monoid;
use anmitsu::ds::segment_tree::segment_tree_dense::SegmentTreeDense;
use anmitsu::graph::graph::Graph;
use anmitsu::io::fastio::Fastio;

fn main() {
    let mut io = Fastio::new();

    let n = io.u32() as usize;

    let mut g = Graph::new(n);
    let mut edges = Vec::with_capacity(n - 1);
    for _ in 0..n - 1 {
        let u = io.u32() as usize - 1;
        let v = io.u32() as usize - 1;
        g.add_undirected_edge(u, v, ());
        edges.push((u, v));
    }

    // 頂点0を根に HLD で番号を振り、その番号上でセグメント木を構築する。
    // 辺を1本取り除いたときの2つの部分木は、深い方の端点の部分木区間
    // (subtree_range) と、それ以外 (全体 - その区間) に対応する。
    let hld = g.try_hld(0).unwrap();

    let mut seg = SegmentTreeDense::<monoid::AddMonoid>::new(n);
    for v in 0..n {
        seg.set(hld.vertex_id(v), 1);
    }
    seg.build();

    let q = io.u32() as usize;
    for _ in 0..q {
        let kind = io.u32();
        if kind == 1 {
            let x = io.u32() as usize - 1;
            let w = io.i64();
            let id = hld.vertex_id(x);
            seg.update(id, seg.get(id) + w);
        } else {
            let y = io.u32() as usize - 1;
            let (u, v) = edges[y];
            // 辺の両端点のうち、深い方が子であり、その部分木がこの辺を
            // 取り除いたときの片側にちょうど対応する。
            let child = if hld.depth(u) > hld.depth(v) { u } else { v };
            let (l, r) = hld.subtree_range(child);
            let child_side = seg.fold(l, r);
            let other_side = seg.fold(0, n) - child_side;
            io.writeln((child_side - other_side).abs());
        }
    }

    io.flush();
}
