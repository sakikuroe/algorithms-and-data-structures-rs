// Library Checker: Vertex Add Path Sum
// https://judge.yosupo.jp/problem/vertex_add_path_sum

use anmitsu::algebra::monoid;
use anmitsu::ds::segment_tree::segment_tree_dense;
use anmitsu::graph::graph::Graph;
use anmitsu::io::fastio::Fastio;

fn main() {
    let mut io = Fastio::new();

    let n = io.u32() as usize;
    let q = io.u32() as usize;

    let a = (0..n).map(|_| io.i64()).collect::<Vec<i64>>();

    let mut g = Graph::new(n);
    for _ in 0..n - 1 {
        let u = io.u32() as usize;
        let v = io.u32() as usize;
        g.add_undirected_edge(u, v, ());
    }

    // HLD で頂点を並べ替え、各頂点の値をその番号の位置に置いたセグメント木で
    // 管理する。パス上の頂点の値の総和は、HLD が返す区間ごとに fold した
    // 結果を足し合わせれば求まる (総和は可換であるため、区間の順序は問わない)。
    let hld = g.hld(0);
    let mut seg = segment_tree_dense::SegmentTreeDense::<monoid::AddMonoid>::new(n);
    for (v, &value) in a.iter().enumerate() {
        seg.set(hld.vertex_id(v), value);
    }
    seg.build();

    for _ in 0..q {
        let kind = io.u32();
        if kind == 0 {
            let p = io.u32() as usize;
            let x = io.i64();
            let id = hld.vertex_id(p);
            seg.update(id, seg.get(id) + x);
        } else {
            let u = io.u32() as usize;
            let v = io.u32() as usize;
            let sum = hld
                .vertex_path_ranges(u, v)
                .into_iter()
                .fold(0_i64, |acc, (l, r)| acc + seg.fold(l, r));
            io.writeln(sum);
        }
    }

    io.flush();
}
